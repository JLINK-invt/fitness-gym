"""Acceptance gate for the chaos harness.

Reads summary.json produced by orchestrator.py and compares the observed
metrics against the objectives in docs/runbooks/chaos-testing.md. Exit code 0
means PASS, 1 means FAIL.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

THRESHOLDS = {
    "access_latency_p95_ms": 300,
    "pos_latency_p95_ms": 60_000,
    "duplicates_detected": 0,
    "events_lost": 0,
    "db_integrity_failures": 0,
}


def evaluate(summary: dict[str, Any]) -> tuple[bool, list[dict[str, Any]]]:
    results: list[dict[str, Any]] = []

    access_metrics = summary.get("access_metrics", {})
    pos_metrics = summary.get("pos_metrics", {})

    access_p95 = float(access_metrics.get("access_latency", 0.0))
    results.append(
        {
            "metric": "access_latency_p95_ms",
            "observed": access_p95,
            "threshold": THRESHOLDS["access_latency_p95_ms"],
            "pass": access_p95 <= THRESHOLDS["access_latency_p95_ms"],
        }
    )

    pos_p95 = float(pos_metrics.get("pos_latency", 0.0))
    results.append(
        {
            "metric": "pos_latency_p95_ms",
            "observed": pos_p95,
            "threshold": THRESHOLDS["pos_latency_p95_ms"],
            "pass": pos_p95 <= THRESHOLDS["pos_latency_p95_ms"],
        }
    )

    for metric in ("duplicates_detected", "events_lost", "db_integrity_failures"):
        observed = float(summary.get(metric, 0))
        results.append(
            {
                "metric": metric,
                "observed": observed,
                "threshold": THRESHOLDS[metric],
                "pass": observed <= THRESHOLDS[metric],
            }
        )

    verdict = all(item["pass"] for item in results)
    return verdict, results


def main() -> int:
    parser = argparse.ArgumentParser(description="Verifica los criterios de aceptación del caos")
    parser.add_argument("summary", help="ruta a summary.json")
    args = parser.parse_args()

    path = Path(args.summary)
    if not path.exists():
        print(f"ERROR: no existe {path}")
        return 1

    summary = json.loads(path.read_text(encoding="utf-8"))
    verdict, results = evaluate(summary)

    for item in results:
        state = "PASS" if item["pass"] else "FAIL"
        print(f"[{state}] {item['metric']}: {item['observed']} (objetivo {item['threshold']})")

    final = "PASS" if verdict else "FAIL"
    print(f"VEREDICTO: {final}")
    return 0 if verdict else 1


if __name__ == "__main__":
    sys.exit(main())
