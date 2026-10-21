import type { HealthStatus } from "@gym/contracts";

const styles: Record<HealthStatus, string> = {
  ok: "bg-emerald-50 text-emerald-700 ring-emerald-600/20",
  degraded: "bg-amber-50 text-amber-700 ring-amber-600/20",
  down: "bg-rose-50 text-rose-700 ring-rose-600/20",
};

const labels: Record<HealthStatus, string> = {
  ok: "Operativo",
  degraded: "Degradado",
  down: "Caído",
};

export function StatusBadge({ status }: { status: HealthStatus }) {
  return (
    <span
      className={`inline-flex items-center rounded-full px-2.5 py-1 text-xs font-semibold ring-1 ring-inset ${styles[status]}`}
    >
      {labels[status]}
    </span>
  );
}
