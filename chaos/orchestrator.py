"""Chaos orchestrator: executes the 72h plan described in the runbook.

The plan is a list of phases. Each phase pairs a load profile with a chaos
fault. The orchestrator runs the phases sequentially, collects the summaries
produced by every loader and writes a consolidated summary.json plus
metrics.csv that verify_acceptance.py can evaluate.
"""

from __future__ import annotations

import argparse
import csv
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from access_loader import AccessLoader
from harness import DEFAULT_RESULTS_DIR, write_json
from pos_loader import PosLoader
from sync_chaos import Fault, SyncChaosAgent

PLAN_72H: list[dict[str, Any]] = [
    {"duration": "2h", "load": "baseline", "chaos": "none"},
    {"duration": "4h", "load": "high", "chaos": "latency_100ms"},
    {"duration": "8h", "load": "high", "chaos": "packet_loss_10"},
    {"duration": "24h", "load": "variable", "chaos": "offline_total"},
    {"duration": "8h", "load": "recovery", "chaos": "reconnection_storm"},
    {"duration": "26h", "load": "baseline", "chaos": "none"},
]

PLAN_QUICK: list[dict[str, Any]] = [
    {"duration": "10m", "load": "baseline", "chaos": "none"},
    {"duration": "10m", "load": "high", "chaos": "latency_100ms"},
    {"duration": "10m", "load": "high", "chaos": "packet_loss_10"},
    {"duration": "20m", "load": "variable", "chaos": "offline_total"},
    {"duration": "10m", "load": "recovery", "chaos": "reconnection_storm"},
]

DURATION_SECONDS = {"10m": 600, "20m": 1200, "2h": 7200, "4h": 14400, "8h": 28800, "24h": 86400, "26h": 93600}

LOAD_PROFILES = {
    "baseline": {"access_rate": 20.0, "pos_rate": 4.0},
    "high": {"access_rate": 50.0, "pos_rate": 10.0},
    "variable": {"access_rate": 35.0, "pos_rate": 7.0},
    "recovery": {"access_rate": 80.0, "pos_rate": 20.0},
}

FAULTS = {
    "none": None,
    "latency_100ms": Fault(name="latency_100ms", delay_ms=100),
    "packet_loss_10": Fault(name="packet_loss_10", loss_percent=10),
    "offline_total": Fault(name="offline_total", partition_seconds=300),
    "reconnection_storm": Fault(name="reconnection_storm", delay_ms=500, loss_percent=30),
}


@dataclass
class PhaseResult:
    name: str
    load: str
    chaos: str
    duration_seconds: int
    access: dict[str, Any]
    pos: dict[str, Any]
    chaos_agent: dict[str, Any]


def run_phase(phase: dict[str, Any], duration_override: int | None, dry_run: bool, interface: str) -> PhaseResult:
    duration = duration_override or DURATION_SECONDS.get(phase["duration"], 600)
    profile = LOAD_PROFILES.get(phase["load"], LOAD_PROFILES["baseline"])

    agent = SyncChaosAgent(interface=interface, dry_run=dry_run)
    fault = FAULTS.get(phase["chaos"])
    if fault is not None:
        agent.apply(fault)

    access = AccessLoader(
        duration_seconds=duration,
        rate=profile["access_rate"],
        duplicate_rate=0.05,
        network_failure_rate=0.2 if phase["chaos"] == "offline_total" else 0.0,
    ).run()

    pos = PosLoader(
        duration_seconds=duration,
        rate=profile["pos_rate"],
        return_rate=0.05,
        offline_rate=0.3 if phase["chaos"] == "offline_total" else 0.0,
    ).run()

    if fault is not None:
        agent.clear()

    return PhaseResult(
        name=phase["duration"],
        load=phase["load"],
        chaos=phase["chaos"],
        duration_seconds=duration,
        access=access,
        pos=pos,
        chaos_agent=agent.summary(),
    )


def write_metrics_csv(path: Path, phases: list[PhaseResult]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.writer(handle)
        writer.writerow(["phase", "load", "chaos", "metric", "count", "p50", "p95", "p99", "max"])
        for phase in phases:
            for metric, values in phase.access.get("metrics", {}).items():
                writer.writerow([phase.name, phase.load, phase.chaos, metric, *values.values()])
            for metric, values in phase.pos.get("metrics", {}).items():
                writer.writerow([phase.name, phase.load, phase.chaos, metric, *values.values()])


def main() -> None:
    parser = argparse.ArgumentParser(description="Orquestador de caos")
    parser.add_argument("--duration", type=int, default=72, help="duración total en horas")
    parser.add_argument("--profile", choices=["quick", "full"], default="full")
    parser.add_argument("--output", default=str(DEFAULT_RESULTS_DIR))
    parser.add_argument("--dry-run", action="store_true", help="no aplicar tc, solo simular")
    parser.add_argument("--interface", default="eth0")
    parser.add_argument("--scale", type=float, default=1.0, help="escala de las fases (1.0 = plan completo)")
    args = parser.parse_args()

    output = Path(args.output)
    output.mkdir(parents=True, exist_ok=True)

    plan = PLAN_QUICK if args.profile == "quick" else PLAN_72H
    if args.profile == "full" and args.duration != 72:
        scale = max(1, int(round(args.duration / 72)))
        plan = [
            {
                **phase,
                "duration": _scale_duration(phase["duration"], scale),
            }
            for phase in plan
        ]

    phases: list[PhaseResult] = []
    started_at = time.time()
    for phase in plan:
        duration_override = int(DURATION_SECONDS.get(phase["duration"], 600) * args.scale)
        print(f"=== fase {phase['duration']} load={phase['load']} chaos={phase['chaos']} ===")
        phases.append(run_phase(phase, duration_override, args.dry_run, args.interface))

    write_metrics_csv(output / "metrics.csv", phases)

    access_metrics: dict[str, float] = {}
    pos_metrics: dict[str, float] = {}
    for phase in phases:
        for metric, values in phase.access.get("metrics", {}).items():
            access_metrics[metric] = max(access_metrics.get(metric, 0.0), float(values.get("p95", 0.0)))
        for metric, values in phase.pos.get("metrics", {}).items():
            pos_metrics[metric] = max(pos_metrics.get(metric, 0.0), float(values.get("p95", 0.0)))

    summary = {
        "duration_hours": round((time.time() - started_at) / 3600, 2),
        "profile": args.profile,
        "dry_run": args.dry_run,
        "phases": [phase.__dict__ for phase in phases],
        "access_metrics": access_metrics,
        "pos_metrics": pos_metrics,
        "verdict": "PENDING",
    }
    write_json(output / "summary.json", summary)
    print(f"resumen escrito en {output / 'summary.json'}")


def _scale_duration(duration: str, scale: int) -> str:
    base = DURATION_SECONDS.get(duration, 600)
    scaled = base * scale
    if scaled % 3600 == 0:
        return f"{scaled // 3600}h"
    return f"{scaled // 60}m"


if __name__ == "__main__":
    main()
