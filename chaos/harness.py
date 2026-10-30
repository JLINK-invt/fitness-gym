"""Shared primitives for the chaos harness.

The harness drives the desktop app through its local HTTP API and records
every sample so the orchestrator can compute percentiles and compare them
against the acceptance thresholds defined in docs/runbooks/chaos-testing.md.
"""

from __future__ import annotations

import json
import os
import random
import time
from dataclasses import dataclass, field
from pathlib import Path
from typing import Any, Iterable

DEFAULT_DB_PATH = os.environ.get("CHAOS_DB_PATH", "/tmp/chaos_fitness.db")
DEFAULT_APP_BASE_URL = os.environ.get("CHAOS_APP_URL", "http://127.0.0.1:1420")
DEFAULT_RESULTS_DIR = Path(os.environ.get("CHAOS_RESULTS_DIR", "chaos_results"))


@dataclass
class Sample:
    name: str
    latency_ms: float
    at: float = field(default_factory=time.time)
    ok: bool = True
    detail: dict[str, Any] = field(default_factory=dict)

    def to_dict(self) -> dict[str, Any]:
        return {
            "name": self.name,
            "latency_ms": round(self.latency_ms, 3),
            "at": self.at,
            "ok": self.ok,
            "detail": self.detail,
        }


class EventLog:
    """Append-only JSONL log used by every loader."""

    def __init__(self, path: Path) -> None:
        self.path = path
        self.path.parent.mkdir(parents=True, exist_ok=True)

    def write(self, event: str, **payload: Any) -> None:
        record = {"event": event, "at": time.time(), **payload}
        with self.path.open("a", encoding="utf-8") as handle:
            handle.write(json.dumps(record, ensure_ascii=False) + "\n")


class Metrics:
    """Latency samples grouped by metric name."""

    def __init__(self) -> None:
        self._samples: dict[str, list[float]] = {}

    def record(self, name: str, latency_ms: float) -> None:
        self._samples.setdefault(name, []).append(latency_ms)

    def percentile(self, name: str, percentile: float = 95.0) -> float:
        values = sorted(self._samples.get(name, []))
        if not values:
            return 0.0
        index = min(len(values) - 1, int(round((percentile / 100.0) * len(values) + 0.5)) - 1)
        return values[max(index, 0)]

    def count(self, name: str) -> int:
        return len(self._samples.get(name, []))

    def as_dict(self) -> dict[str, dict[str, float]]:
        return {
            name: {
                "count": float(len(values)),
                "p50": self.percentile(name, 50),
                "p95": self.percentile(name, 95),
                "p99": self.percentile(name, 99),
                "max": max(values),
            }
            for name, values in self._samples.items()
        }


class Workload:
    """Base class for the load generators."""

    def __init__(self, name: str, duration_seconds: float, rate: float, seed: int | None = None) -> None:
        self.name = name
        self.duration_seconds = duration_seconds
        self.rate = max(rate, 0.0)
        self.random = random.Random(seed)
        self.metrics = Metrics()
        self.events = EventLog(DEFAULT_RESULTS_DIR / "events.jsonl")

    def run(self) -> Metrics:
        raise NotImplementedError

    def _pace(self, started_at: float) -> float:
        if self.rate <= 0:
            return 0.0
        return max(0.0, (1.0 / self.rate) - (time.time() - started_at))

    def _deadline(self, started_at: float) -> float:
        return started_at + self.duration_seconds

    def _expired(self, started_at: float) -> bool:
        return time.time() >= self._deadline(started_at)


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8") as handle:
        json.dump(payload, handle, ensure_ascii=False, indent=2)


def read_csv(path: Path) -> list[dict[str, str]]:
    if not path.exists():
        return []
    import csv

    with path.open("r", encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle))


def chunks(items: Iterable[Any], size: int) -> Iterable[list[Any]]:
    batch: list[Any] = []
    for item in items:
        batch.append(item)
        if len(batch) >= size:
            yield batch
            batch = []
    if batch:
        yield batch
