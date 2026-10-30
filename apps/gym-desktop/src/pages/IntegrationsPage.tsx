import type { IntegrationJournalSummary, IntegrationLink } from "@gym/contracts";
import { CONTRACT_VERSION, errorMessage } from "@gym/contracts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useOutletContext } from "react-router-dom";

import type { LayoutContext } from "../components/Layout";
import { formatDateTime } from "../lib/format";
import { api } from "../lib/tauri";

const fieldClass =
  "w-full rounded-lg border border-slate-300 px-3 py-2 text-sm outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100";
const primaryButton =
  "rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-700 disabled:opacity-50";

const statusLabels: Record<string, string> = {
  connected: "Conectada",
  degraded: "Degradada",
  disconnected: "Desconectada",
};

export function IntegrationsPage() {
  const queryClient = useQueryClient();
  const { session } = useOutletContext<LayoutContext>();
  const canManage = session.permissions.includes("manageIntegrations");
  const [moduleId, setModuleId] = useState("");
  const [displayName, setDisplayName] = useState("");
  const [contractVersion, setContractVersion] = useState(CONTRACT_VERSION);
  const [status, setStatus] = useState("connected");
  const [capabilities, setCapabilities] = useState("");
  const [notes, setNotes] = useState("");
  const [error, setError] = useState<string | null>(null);

  const links = useQuery<IntegrationLink[]>({
    queryKey: ["integration-links"],
    queryFn: api.listIntegrationLinks,
  });
  const journal = useQuery<IntegrationJournalSummary>({
    queryKey: ["integration-journal"],
    queryFn: api.integrationJournalSummary,
  });

  const upsert = useMutation({
    mutationFn: () =>
      api.upsertIntegrationLink({
        moduleId,
        displayName,
        contractVersion,
        status,
        capabilities: capabilities
          .split(",")
          .map((value) => value.trim())
          .filter(Boolean),
        notes: notes || null,
      }),
    onSuccess: () => {
      setError(null);
      setModuleId("");
      setDisplayName("");
      setCapabilities("");
      setNotes("");
      void queryClient.invalidateQueries({ queryKey: ["integration-links"] });
      void queryClient.invalidateQueries({ queryKey: ["integration-journal"] });
    },
    onError: (cause) => setError(errorMessage(cause)),
  });

  return (
    <div className="space-y-6">
      <p className="text-sm text-slate-600">
        Módulos externos conectados por contrato firmado, su estado de salud y el journal de
        sincronización local.
      </p>

      {links.isError && (
        <div className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
          {errorMessage(links.error)}
        </div>
      )}
      {journal.isError && (
        <div className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
          {errorMessage(journal.error)}
        </div>
      )}
      {error && (
        <div className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
          {error}
        </div>
      )}

      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Metric label="Pendientes" value={journal.data?.pending ?? 0} />
        <Metric label="Enviados" value={journal.data?.sent ?? 0} />
        <Metric label="Dead letter" value={journal.data?.deadLetter ?? 0} />
        <Metric
          label="Secuencia del servidor"
          value={journal.data?.lastServerSequence ?? 0}
        />
      </div>

      {canManage && (
        <form
          className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm"
          onSubmit={(event) => {
            event.preventDefault();
            upsert.mutate();
          }}
        >
          <h2 className="text-sm font-semibold text-slate-900">Conectar módulo</h2>
          <div className="mt-4 grid gap-3 md:grid-cols-3">
            <label className="block text-xs font-medium text-slate-600">
              Identificador
              <input
                className={`${fieldClass} mt-1`}
                value={moduleId}
                onChange={(event) => setModuleId(event.target.value)}
                placeholder="pos-sucursal"
                required
              />
            </label>
            <label className="block text-xs font-medium text-slate-600">
              Nombre visible
              <input
                className={`${fieldClass} mt-1`}
                value={displayName}
                onChange={(event) => setDisplayName(event.target.value)}
                required
              />
            </label>
            <label className="block text-xs font-medium text-slate-600">
              Versión de contrato
              <input
                className={`${fieldClass} mt-1`}
                value={contractVersion}
                onChange={(event) => setContractVersion(event.target.value)}
                required
              />
            </label>
            <label className="block text-xs font-medium text-slate-600">
              Estado
              <select
                className={`${fieldClass} mt-1`}
                value={status}
                onChange={(event) => setStatus(event.target.value)}
              >
                {Object.entries(statusLabels).map(([value, label]) => (
                  <option key={value} value={value}>
                    {label}
                  </option>
                ))}
              </select>
            </label>
            <label className="block text-xs font-medium text-slate-600 md:col-span-2">
              Capacidades (separadas por coma)
              <input
                className={`${fieldClass} mt-1`}
                value={capabilities}
                onChange={(event) => setCapabilities(event.target.value)}
                placeholder="members.read, sales.write"
              />
            </label>
            <label className="block text-xs font-medium text-slate-600 md:col-span-3">
              Notas
              <input
                className={`${fieldClass} mt-1`}
                value={notes}
                onChange={(event) => setNotes(event.target.value)}
              />
            </label>
          </div>
          <button className={`${primaryButton} mt-4`} disabled={upsert.isPending}>
            {upsert.isPending ? "Guardando…" : "Guardar integración"}
          </button>
        </form>
      )}

      <section className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
        <h2 className="text-sm font-semibold text-slate-900">Módulos conectados</h2>
        <div className="mt-4 overflow-x-auto">
          <table className="min-w-full text-sm">
            <thead>
              <tr className="text-left text-xs uppercase tracking-wide text-slate-500">
                <th className="py-2 pr-4">Módulo</th>
                <th className="py-2 pr-4">Contrato</th>
                <th className="py-2 pr-4">Estado</th>
                <th className="py-2 pr-4">Capacidades</th>
                <th className="py-2">Última señal</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {(links.data ?? []).map((link) => (
                <tr key={link.id}>
                  <td className="py-2 pr-4">
                    <p className="font-medium text-slate-900">{link.displayName}</p>
                    <p className="text-xs text-slate-500">{link.moduleId}</p>
                  </td>
                  <td className="py-2 pr-4 text-slate-600">{link.contractVersion}</td>
                  <td className="py-2 pr-4 text-slate-600">
                    {statusLabels[link.status] ?? link.status}
                  </td>
                  <td className="py-2 pr-4 text-xs text-slate-500">
                    {link.capabilities.length > 0 ? link.capabilities.join(", ") : "—"}
                  </td>
                  <td className="py-2 text-slate-500">
                    {link.lastSeenAt ? formatDateTime(link.lastSeenAt) : "—"}
                  </td>
                </tr>
              ))}
              {(links.data ?? []).length === 0 && (
                <tr>
                  <td className="py-4 text-sm text-slate-500" colSpan={5}>
                    Aún no hay módulos conectados.
                  </td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </section>
    </div>
  );
}

function Metric({ label, value }: { label: string; value: number }) {
  return (
    <div className="rounded-2xl border border-slate-200 bg-white p-4 shadow-sm">
      <p className="text-xs font-medium uppercase tracking-wide text-slate-500">{label}</p>
      <p className="mt-1 text-2xl font-semibold text-slate-900">{value}</p>
    </div>
  );
}
