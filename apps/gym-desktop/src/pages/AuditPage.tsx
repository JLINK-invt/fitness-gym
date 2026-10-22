import type { AuditEventSummary, AuditResult } from "@gym/contracts";
import { errorMessage } from "@gym/contracts";
import { useQuery } from "@tanstack/react-query";

import { formatDateTime } from "../lib/format";
import { api } from "../lib/tauri";

const resultLabels: Record<AuditResult, string> = {
  allowed: "Permitido",
  denied: "Denegado",
  failed: "Falló",
};

const resultClasses: Record<AuditResult, string> = {
  allowed: "bg-emerald-50 text-emerald-700 ring-emerald-600/20",
  denied: "bg-amber-50 text-amber-700 ring-amber-600/20",
  failed: "bg-rose-50 text-rose-700 ring-rose-600/20",
};

export function AuditPage() {
  const events = useQuery<AuditEventSummary[]>({
    queryKey: ["audit-events"],
    queryFn: () => api.listAuditEvents(200),
  });

  const integrity = useQuery({
    queryKey: ["audit-integrity"],
    queryFn: api.verifyAuditChain,
    staleTime: 0,
  });

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h1 className="text-xl font-semibold tracking-tight">Auditoría</h1>
          <p className="mt-1 text-sm text-slate-600">
            Registro append-only con cadena de hashes. Solo se puede consultar; no se edita ni
            se borra desde la aplicación.
          </p>
        </div>
        <button
          type="button"
          onClick={() => {
            void events.refetch();
            void integrity.refetch();
          }}
          disabled={events.isFetching || integrity.isFetching}
          className="rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium text-slate-700 hover:bg-slate-100 disabled:opacity-50"
        >
          {events.isFetching || integrity.isFetching ? "Verificando…" : "Verificar cadena"}
        </button>
      </div>

      <IntegrityBanner
        loading={integrity.isLoading}
        error={integrity.error}
        valid={integrity.data?.valid}
        checked={integrity.data?.checkedEvents}
        brokenSequence={integrity.data?.firstBrokenSequence ?? null}
      />

      <section className="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm">
        <div className="border-b border-slate-200 px-6 py-4">
          <h2 className="text-base font-semibold">Eventos recientes</h2>
          <p className="mt-1 text-xs text-slate-500">
            Los eventos de otras sedes no se muestran a encargados ni contabilidad.
          </p>
        </div>
        {events.isError ? (
          <p className="px-6 py-8 text-sm text-rose-700">{errorMessage(events.error)}</p>
        ) : events.isLoading ? (
          <p className="px-6 py-8 text-sm text-slate-500">Cargando auditoría…</p>
        ) : events.data && events.data.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-slate-200 text-sm">
              <thead className="bg-slate-50 text-left text-xs uppercase tracking-wide text-slate-500">
                <tr>
                  <th className="px-4 py-3 font-medium">#</th>
                  <th className="px-4 py-3 font-medium">Fecha</th>
                  <th className="px-4 py-3 font-medium">Actor</th>
                  <th className="px-4 py-3 font-medium">Acción</th>
                  <th className="px-4 py-3 font-medium">Recurso</th>
                  <th className="px-4 py-3 font-medium">Resultado</th>
                  <th className="px-4 py-3 font-medium">Origen</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {events.data.map((event) => (
                  <tr key={event.id}>
                    <td className="px-4 py-3 font-mono text-xs text-slate-500">
                      {event.sequence}
                    </td>
                    <td className="whitespace-nowrap px-4 py-3">
                      {formatDateTime(event.createdAt)}
                    </td>
                    <td className="px-4 py-3">
                      {event.actorName ?? event.actorId ?? "sistema"}
                    </td>
                    <td className="px-4 py-3 font-mono text-xs">{event.action}</td>
                    <td className="px-4 py-3 font-mono text-xs">{event.resource}</td>
                    <td className="px-4 py-3">
                      <span
                        className={[
                          "rounded-full px-2.5 py-1 text-xs font-semibold ring-1 ring-inset",
                          resultClasses[event.result],
                        ].join(" ")}
                      >
                        {resultLabels[event.result]}
                      </span>
                    </td>
                    <td className="px-4 py-3 text-xs text-slate-500">{event.origin}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p className="px-6 py-8 text-sm text-slate-500">Aún no hay eventos de auditoría.</p>
        )}
      </section>
    </div>
  );
}

function IntegrityBanner({
  loading,
  error,
  valid,
  checked,
  brokenSequence,
}: {
  loading: boolean;
  error: unknown;
  valid: boolean | undefined;
  checked: number | undefined;
  brokenSequence: number | null;
}) {
  if (loading) {
    return (
      <p className="rounded-xl border border-slate-200 bg-white p-4 text-sm text-slate-500">
        Verificando integridad de la cadena…
      </p>
    );
  }
  if (error) {
    return (
      <p className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
        {errorMessage(error)}
      </p>
    );
  }
  if (valid) {
    return (
      <p className="rounded-xl border border-emerald-200 bg-emerald-50 p-4 text-sm text-emerald-800">
        Cadena íntegra: {checked ?? 0} evento(s) verificados.
      </p>
    );
  }
  return (
    <p className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-800">
      La cadena de auditoría no es válida
      {brokenSequence !== null ? `; primer evento alterado: #${brokenSequence}` : ""}.
    </p>
  );
}
