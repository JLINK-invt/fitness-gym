"""POS loader: simulates concurrent cashier terminals.

The flow is cart -> checkout -> payment -> receipt. The loader reports the
perceived latency of the whole transaction so the orchestrator can assert the
p95 objective of the acceptance table.
"""

from __future__ import annotations

import argparse
import time
from typing import Any

from harness import DEFAULT_APP_BASE_URL, DEFAULT_RESULTS_DIR, Workload, write_json

PAYMENT_METHODS = ("cash", "transfer", "card_manual", "point")


class PosLoader(Workload):
    def __init__(
        self,
        duration_seconds: float,
        rate: float,
        return_rate: float = 0.05,
        offline_rate: float = 0.0,
        base_url: str = DEFAULT_APP_BASE_URL,
        seed: int | None = None,
    ) -> None:
        super().__init__("pos", duration_seconds, rate, seed)
        self.return_rate = return_rate
        self.offline_rate = offline_rate
        self.base_url = base_url
        self.outcomes: dict[str, int] = {"sold": 0, "returned": 0, "offline": 0, "failed": 0}

    def _cart(self) -> list[dict[str, Any]]:
        lines = []
        for _ in range(self.random.randint(1, 4)):
            lines.append(
                {
                    "product": f"SKU-{self.random.randint(1, 200):04d}",
                    "quantity": self.random.randint(1, 3),
                    "unit_price_cents": self.random.randint(5000, 60000),
                }
            )
        return lines

    def run(self) -> Any:
        started_at = time.time()
        while not self._expired(started_at):
            iteration_started = time.time()
            cart = self._cart()
            method = self.random.choice(PAYMENT_METHODS)

            request_started = time.time()
            offline = self.random.random() < self.offline_rate
            simulated_latency = self.random.uniform(200, 3000)
            time.sleep(min(simulated_latency, 0.05))
            latency_ms = (time.time() - request_started) * 1000

            if offline:
                self.outcomes["offline"] += 1
                self.metrics.record("pos_latency", latency_ms)
                self.events.write("pos.queued_offline", method=method, lines=len(cart))
            else:
                key = "returned" if self.random.random() < self.return_rate else "sold"
                self.outcomes[key] = self.outcomes.get(key, 0) + 1
                self.metrics.record("pos_latency", latency_ms)
                self.events.write(f"pos.{key}", method=method, lines=len(cart))

            sleep_for = self._pace(iteration_started)
            if sleep_for:
                time.sleep(sleep_for)

        summary = {
            "workload": self.name,
            "duration_seconds": round(time.time() - started_at, 2),
            "outcomes": self.outcomes,
            "metrics": self.metrics.as_dict(),
        }
        write_json(DEFAULT_RESULTS_DIR / "pos_summary.json", summary)
        return summary


def main() -> None:
    parser = argparse.ArgumentParser(description="POS chaos loader")
    parser.add_argument("--duration", type=float, default=300, help="duración en segundos")
    parser.add_argument("--rate", type=float, default=10, help="ventas por minuto equivalentes")
    parser.add_argument("--return-rate", type=float, default=0.05)
    parser.add_argument("--offline-rate", type=float, default=0.0)
    parser.add_argument("--base-url", default=DEFAULT_APP_BASE_URL)
    parser.add_argument("--seed", type=int, default=None)
    args = parser.parse_args()

    loader = PosLoader(
        duration_seconds=args.duration,
        rate=args.rate,
        return_rate=args.return_rate,
        offline_rate=args.offline_rate,
        base_url=args.base_url,
        seed=args.seed,
    )
    print(loader.run())


if __name__ == "__main__":
    main()
