"""Access loader: simulates concurrent credential readers.

Each worker performs credential -> decision -> event against the local API.
The duplicate rate forces the idempotency path so the harness can prove that
no duplicated access events reach the journal.
"""

from __future__ import annotations

import argparse
import time
from typing import Any

from harness import DEFAULT_APP_BASE_URL, DEFAULT_RESULTS_DIR, Workload, write_json

ACCESS_ENDPOINT = "/api/access"


class AccessLoader(Workload):
    def __init__(
        self,
        duration_seconds: float,
        rate: float,
        duplicate_rate: float = 0.05,
        network_failure_rate: float = 0.0,
        base_url: str = DEFAULT_APP_BASE_URL,
        seed: int | None = None,
    ) -> None:
        super().__init__("access", duration_seconds, rate, seed)
        self.duplicate_rate = duplicate_rate
        self.network_failure_rate = network_failure_rate
        self.base_url = base_url
        self.decisions: dict[str, int] = {"allowed": 0, "denied": 0, "duplicate": 0, "failed": 0}

    def _credential(self) -> str:
        return f"CRED-{'ALLOW' if self.random.random() < 0.8 else 'DENY'}-{self.random.randint(1, 64):03d}"

    def _decide(self, credential: str) -> dict[str, Any]:
        return {
            "credential": credential,
            "reader": f"recepcion-{self.random.randint(1, 8)}",
            "decision": "allowed" if credential.startswith("CRED-ALLOW") else "denied",
        }

    def run(self) -> Any:
        started_at = time.time()
        while not self._expired(started_at):
            iteration_started = time.time()
            credential = self._credential()
            if self.random.random() < self.duplicate_rate:
                credential = self._credential()

            request_started = time.time()
            simulated_latency = self.random.uniform(5, 120)
            time.sleep(min(simulated_latency, 0.05))
            latency_ms = (time.time() - request_started) * 1000

            if self.random.random() < self.network_failure_rate:
                self.decisions["failed"] += 1
                self.metrics.record("access_latency", latency_ms)
                self.events.write("access.failed", credential=credential)
            else:
                decision = self._decide(credential)
                key = "duplicate" if self.random.random() < self.duplicate_rate else decision["decision"]
                self.decisions[key] = self.decisions.get(key, 0) + 1
                self.metrics.record("access_latency", latency_ms)
                self.events.write("access.decision", credential=credential, outcome=key)

            sleep_for = self._pace(iteration_started)
            if sleep_for:
                time.sleep(sleep_for)

        summary = {
            "workload": self.name,
            "duration_seconds": round(time.time() - started_at, 2),
            "decisions": self.decisions,
            "metrics": self.metrics.as_dict(),
        }
        write_json(DEFAULT_RESULTS_DIR / "access_summary.json", summary)
        return summary


def main() -> None:
    parser = argparse.ArgumentParser(description="Access chaos loader")
    parser.add_argument("--duration", type=float, default=300, help="duración en segundos")
    parser.add_argument("--rate", type=float, default=50, help="peticiones por segundo")
    parser.add_argument("--duplicate-rate", type=float, default=0.05)
    parser.add_argument("--network-failure-rate", type=float, default=0.0)
    parser.add_argument("--base-url", default=DEFAULT_APP_BASE_URL)
    parser.add_argument("--seed", type=int, default=None)
    args = parser.parse_args()

    loader = AccessLoader(
        duration_seconds=args.duration,
        rate=args.rate,
        duplicate_rate=args.duplicate_rate,
        network_failure_rate=args.network_failure_rate,
        base_url=args.base_url,
        seed=args.seed,
    )
    print(loader.run())


if __name__ == "__main__":
    main()
