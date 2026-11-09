"""Sync chaos agent: injects network faults into the sync transport.

Faults are applied with Linux `tc` when the harness runs with privileges. In
unprivileged environments the agent degrades to simulation so the 72h plan can
still be rehearsed without root access.
"""

from __future__ import annotations

import argparse
import json
import shutil
import subprocess
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Any

from harness import DEFAULT_RESULTS_DIR, EventLog, write_json


@dataclass
class Fault:
    name: str
    delay_ms: int = 0
    loss_percent: int = 0
    partition_seconds: int = 0

    def tc_command(self, interface: str) -> list[str]:
        if self.partition_seconds:
            return ["tc", "qdisc", "change", "dev", interface, "root", "netem", "loss", "100%"]
        netem = []
        if self.delay_ms:
            netem += ["delay", f"{self.delay_ms}ms"]
        if self.loss_percent:
            netem += ["loss", f"{self.loss_percent}%"]
        return ["tc", "qdisc", "change", "dev", interface, "root", "netem", *netem]


class SyncChaosAgent:
    def __init__(self, interface: str = "eth0", dry_run: bool = False) -> None:
        self.interface = interface
        self.dry_run = dry_run
        self.events = EventLog(DEFAULT_RESULTS_DIR / "events.jsonl")
        self.applied: list[dict[str, Any]] = []
        self.has_tc = shutil.which("tc") is not None

    @property
    def effective_dry_run(self) -> bool:
        return self.dry_run or not self.has_tc

    def apply(self, fault: Fault) -> dict[str, Any]:
        record: dict[str, Any] = {"fault": fault.name, "at": time.time(), "mode": "simulated"}
        if not self.effective_dry_run:
            command = fault.tc_command(self.interface)
            try:
                subprocess.run(command, check=True, capture_output=True, timeout=10)
                record["mode"] = "applied"
                record["command"] = " ".join(command)
            except (subprocess.CalledProcessError, subprocess.TimeoutExpired) as error:
                record["mode"] = "failed"
                record["error"] = str(error)
        self.applied.append(record)
        self.events.write("sync.fault", **record)
        return record

    def clear(self) -> dict[str, Any]:
        record: dict[str, Any] = {"fault": "clear", "at": time.time(), "mode": "simulated"}
        if not self.effective_dry_run:
            command = ["tc", "qdisc", "del", "dev", self.interface, "root"]
            try:
                subprocess.run(command, check=True, capture_output=True, timeout=10)
                record["mode"] = "applied"
            except (subprocess.CalledProcessError, subprocess.TimeoutExpired) as error:
                record["mode"] = "failed"
                record["error"] = str(error)
        self.events.write("sync.fault", **record)
        return record

    def summary(self) -> dict[str, Any]:
        return {
            "interface": self.interface,
            "tc_available": self.has_tc,
            "dry_run": self.effective_dry_run,
            "faults_applied": self.applied,
        }


def main() -> None:
    parser = argparse.ArgumentParser(description="Agente de caos de sincronización")
    parser.add_argument("--interface", default="eth0")
    parser.add_argument("--fault", default="latency_100ms")
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()

    faults = {
        "latency_100ms": Fault(name="latency_100ms", delay_ms=100),
        "latency_500ms": Fault(name="latency_500ms", delay_ms=500),
        "packet_loss_10": Fault(name="packet_loss_10", loss_percent=10),
        "packet_loss_30": Fault(name="packet_loss_30", loss_percent=30),
        "offline": Fault(name="offline", partition_seconds=300),
    }

    agent = SyncChaosAgent(interface=args.interface, dry_run=args.dry_run)
    fault = faults.get(args.fault)
    if fault is None:
        parser.error(f"fallo desconocido: {args.fault}")

    agent.apply(fault)
    time.sleep(1)
    agent.clear()
    summary = agent.summary()
    write_json(DEFAULT_RESULTS_DIR / "sync_chaos_summary.json", summary)
    print(json.dumps(summary, ensure_ascii=False, indent=2))


if __name__ == "__main__":
    main()
