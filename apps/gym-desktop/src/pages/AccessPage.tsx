import type { AccessRequest, AccessEventSummary } from "@gym/contracts";
import { errorMessage } from "@gym/contracts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState, type FormEvent } from "react";

import { formatDateTime, shortenChecksum } from "../lib/format";
import { api, isTauriRuntime } from "../lib/tauri";

export function AccessPage() {
  const queryClient = useQueryClient();
  const inTauri = isTauriRuntime();
  const [credentialId, setCredentialId] = useState("CRED-ALLOW");
  const [deviceId, setDeviceId] = useState("recepcion-1");
  const [lastRequest, setLastRequest] = useState<AccessRequest | null>(null);

  const events = useQuery<AccessEventSummary[]>({
    queryKey: ["access-events"],
    queryFn: () => api.listAccessEvents(20),
    enabled: inTauri,
    refetchInterval: 2000,
  });

  const simulate = useMutation({
    mutationFn: api.simulateAccess,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["access-events"] });
    },
  });

  function submitRequest(request: AccessRequest) {
    setLastRequest(request);
    simulate.mutate(request);
  }

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!credentialId.trim() || !deviceId.trim()) {
      return;
    }
    submitRequest({
      credentialId: credentialId.trim(),
      deviceId: deviceId.trim(),
      idempotencyKey: crypto.randomUUID(),
    });
  }

  function replayLast() {
    if (lastRequest) {
      simulate.mutate(lastRequest);
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold tracking-tight">Acceso simulado</h1>
        <p className="mt-1 text-sm text-slate-600">
          Prueba el flujo local de punta a punta: decisión, evento idempotente y
          actualización de la interfaz.
        </p>
      </div>

      <section className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
        <h2 className="text-base font-semibold">Nueva lectura</h2>
        <p className="mt-1 text-sm text-slate-600">
          Requiere una sesión con permiso de operación. No uses credenciales de personas.
        </p>
        <form onSubmit={handleSubmit} className="mt-4 grid gap-3 sm:grid-cols-[1fr_1fr_auto]">
          <label className="text-sm">
            <span className="mb-1 block font-medium text-slate-700">Credencial</span>
            <select
              value={credentialId}
              onChange={(event) => setCredentialId(event.target.value)}
              className="w-full rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100"
            >
              <option value="CRED-ALLOW">Permitida (CRED-ALLOW)</option>
              <option value="CRED-DENY">Denegada (CRED-DENY)</option>
              <option value="CRED-999">Desconocida (CRED-999)</option>
            </select>
          </label>
          <label className="text-sm">
            <span className="mb-1 block font-medium text-slate-700">Lector</span>
            <input
              value={deviceId}
              onChange={(event) => setDeviceId(event.target.value)}
              className="w-full rounded-lg border border-slate-300 px-3 py-2 text-sm outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100"
            />
          </label>
          <div className="flex items-end">
            <button
              type="submit"
              className="w-full rounded-lg bg-brand-600 px-4 py-2 text-sm font-medium text-white hover:bg-brand-700 disabled:opacity-50"
              disabled={simulate.isPending || !credentialId.trim() || !deviceId.trim()}
            >
              {simulate.isPending ? "Procesando…" : "Simular acceso"}
            </button>
          </div>
        </form>

        {simulate.isError && (
          <p className="mt-3 text-sm text-rose-700">{errorMessage(simulate.error)}</p>
        )}

        {simulate.isSuccess && (
          <div
            className={[
              "mt-4 rounded-xl border p-4",
              simulate.data.allowed
                ? "border-emerald-200 bg-emerald-50"
                : "border-rose-200 bg-rose-50",
            ].join(" ")}
          >
            <div className="flex flex-wrap items-center gap-2">
              <p
                className={[
                  "text-sm font-semibold",
                  simulate.data.allowed ? "text-emerald-800" : "text-rose-800",
                ].join(" ")}
              >
                {simulate.data.allowed ? "Acceso permitido" : "Acceso denegado"}
              </p>
              {simulate.data.duplicate && (
                <span className="rounded-full bg-white px-2 py-0.5 text-xs font-medium text-slate-600">
                  evento ya existente
                </span>
              )}
            </div>
            <p className="mt-1 text-sm text-slate-700">{simulate.data.reason}</p>
            <p className="mt-1 text-xs text-slate-500">
              Evento {shortenChecksum(simulate.data.eventId, 8)} ·{" "}
              {simulate.data.latencyMs} ms
            </p>
            {lastRequest && (
              <button
                type="button"
                onClick={replayLast}
                className="mt-3 rounded-lg border border-slate-300 bg-white px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-50"
                disabled={simulate.isPending}
              >
                Repetir última solicitud
              </button>
            )}
          </div>
        )}
      </section>

      <section className="rounded-2xl border border-slate-200 bg-white shadow-sm">
        <div className="flex items-center justify-between border-b border-slate-200 px-6 py-4">
          <h2 className="text-base font-semibold">Eventos de acceso</h2>
          <button
            type="button"
            onClick={() => events.refetch()}
            className="text-sm font-medium text-brand-700 hover:text-brand-600"
            disabled={events.isFetching}
          >
            Actualizar
          </button>
        </div>

        {!inTauri ? (
          <p className="px-6 py-8 text-sm text-slate-500">
            Disponible únicamente dentro de la aplicación Tauri.
          </p>
        ) : events.isLoading ? (
          <p className="px-6 py-8 text-sm text-slate-500">Cargando eventos…</p>
        ) : events.isError ? (
          <p className="px-6 py-8 text-sm text-rose-700">{errorMessage(events.error)}</p>
        ) : events.data && events.data.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-slate-200 text-sm">
              <thead className="bg-slate-50 text-left text-xs uppercase tracking-wide text-slate-500">
                <tr>
                  <th className="px-6 py-3 font-medium">Resultado</th>
                  <th className="px-6 py-3 font-medium">Evento</th>
                  <th className="px-6 py-3 font-medium">Solicitud</th>
                  <th className="px-6 py-3 font-medium">Fecha</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {events.data.map((event) => (
                  <tr key={event.eventId}>
                    <td className="px-6 py-3">
                      <span
                        className={[
                          "rounded-full px-2 py-0.5 text-xs font-medium",
                          event.allowed
                            ? "bg-emerald-50 text-emerald-700"
                            : "bg-rose-50 text-rose-700",
                        ].join(" ")}
                      >
                        {event.allowed ? "Permitido" : "Denegado"}
                      </span>
                    </td>
                    <td className="px-6 py-3 font-mono text-xs">
                      {shortenChecksum(event.eventId, 8)}
                    </td>
                    <td className="px-6 py-3 font-mono text-xs">
                      {shortenChecksum(event.idempotencyKey, 8)}
                    </td>
                    <td className="px-6 py-3">{formatDateTime(event.occurredAt)}</td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p className="px-6 py-8 text-sm text-slate-500">Aún no hay eventos.</p>
        )}
      </section>
    </div>
  );
}
