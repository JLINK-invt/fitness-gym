import type { Campaign, MarketingFunnel } from "@gym/contracts";
import { errorMessage } from "@gym/contracts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { Link } from "react-router-dom";

import { api } from "../lib/tauri";

const fieldClass =
  "w-full rounded-lg border border-slate-300 px-3 py-2 text-sm outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100";
const primaryButton =
  "rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-700 disabled:opacity-50";
const secondaryButton =
  "rounded-lg border border-slate-300 px-4 py-2 text-sm font-semibold text-slate-700 hover:bg-slate-50 disabled:opacity-50";

const channelLabels: Record<string, string> = {
  whatsapp: "WhatsApp",
  sms: "SMS",
  email: "Correo",
  in_app: "En la app",
  referral: "Referidos",
};

const campaignStatusLabels: Record<string, string> = {
  draft: "Borrador",
  active: "Activa",
  paused: "Pausada",
  completed: "Completada",
};

const funnelLabels: Record<string, string> = {
  new: "Nuevos",
  contacted: "Contactados",
  trial: "En prueba",
  converted: "Convertidos",
  lost: "Perdidos",
};

export function MarketingPage() {
  const queryClient = useQueryClient();
  const [name, setName] = useState("");
  const [channel, setChannel] = useState("whatsapp");
  const [audience, setAudience] = useState("");
  const [status, setStatus] = useState("draft");
  const [startsOn, setStartsOn] = useState("");
  const [endsOn, setEndsOn] = useState("");
  const [notes, setNotes] = useState("");
  const [error, setError] = useState<string | null>(null);

  const funnel = useQuery<MarketingFunnel>({
    queryKey: ["marketing-funnel"],
    queryFn: () => api.marketingFunnel(),
  });
  const campaigns = useQuery<Campaign[]>({
    queryKey: ["campaigns"],
    queryFn: api.listCampaigns,
  });

  const refresh = () => {
    void queryClient.invalidateQueries({ queryKey: ["marketing-funnel"] });
    void queryClient.invalidateQueries({ queryKey: ["campaigns"] });
  };

  const createCampaign = useMutation({
    mutationFn: () =>
      api.createCampaign({
        name,
        channel,
        audience,
        status,
        startsOn: startsOn || null,
        endsOn: endsOn || null,
        notes: notes || null,
      }),
    onSuccess: () => {
      setError(null);
      setName("");
      setAudience("");
      setStartsOn("");
      setEndsOn("");
      setNotes("");
      refresh();
    },
    onError: (cause) => setError(errorMessage(cause)),
  });

  const changeStatus = useMutation({
    mutationFn: (input: { campaign: Campaign; status: string }) =>
      api.updateCampaign({
        campaignId: input.campaign.id,
        name: input.campaign.name,
        channel: input.campaign.channel,
        audience: input.campaign.audience,
        status: input.status,
        startsOn: input.campaign.startsOn,
        endsOn: input.campaign.endsOn,
        notes: input.campaign.notes,
      }),
    onSuccess: () => {
      setError(null);
      refresh();
    },
    onError: (cause) => setError(errorMessage(cause)),
  });

  const data = funnel.data;

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-end justify-between gap-3">
        <p className="text-sm text-slate-600">
          Embudo comercial, campañas y alertas de retención. El alta de prospectos vive en{" "}
          <Link className="font-semibold text-brand-700 underline" to="/members">
            Socios → Prospectos
          </Link>
          .
        </p>
        <button
          type="button"
          className={secondaryButton}
          onClick={() => {
            void funnel.refetch();
            void campaigns.refetch();
          }}
        >
          Actualizar
        </button>
      </div>

      {funnel.isError && (
        <div className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
          {errorMessage(funnel.error)}
        </div>
      )}
      {campaigns.isError && (
        <div className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
          {errorMessage(campaigns.error)}
        </div>
      )}
      {error && (
        <div className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
          {error}
        </div>
      )}

      <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <Metric label="Prospectos" value={data?.prospectsTotal ?? 0} />
        <Metric
          label="Conversión"
          value={`${((data?.conversionBps ?? 0) / 100).toFixed(1)}%`}
        />
        <Metric label="Campañas activas" value={data?.activeCampaigns ?? 0} />
        <Metric
          label="Membresías por vencer"
          value={data?.expiringMemberships ?? 0}
        />
      </div>

      <div className="grid gap-6 lg:grid-cols-2">
        <section className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
          <h2 className="text-sm font-semibold text-slate-900">Embudo de prospectos</h2>
          <ul className="mt-4 space-y-2">
            {(data?.stages ?? []).map((stage) => (
              <li key={stage.status}>
                <div className="flex items-center justify-between text-sm">
                  <span className="text-slate-700">
                    {funnelLabels[stage.status] ?? stage.status}
                  </span>
                  <span className="font-semibold text-slate-900">{stage.count}</span>
                </div>
                <div className="mt-1 h-2 overflow-hidden rounded-full bg-slate-100">
                  <div
                    className="h-full rounded-full bg-brand-500"
                    style={{
                      width: `${data?.prospectsTotal ? (stage.count / data.prospectsTotal) * 100 : 0}%`,
                    }}
                  />
                </div>
              </li>
            ))}
            {(data?.stages ?? []).length === 0 && (
              <li className="text-sm text-slate-500">Sin prospectos registrados.</li>
            )}
          </ul>
          <div className="mt-5 grid gap-3 sm:grid-cols-2">
            <Metric label="Socios inactivos" value={data?.inactiveMembers ?? 0} />
            <Metric label="Convertidos" value={data?.prospectsConverted ?? 0} />
          </div>
        </section>

        <section className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
          <h2 className="text-sm font-semibold text-slate-900">Nueva campaña</h2>
          <form
            className="mt-4 space-y-3"
            onSubmit={(event) => {
              event.preventDefault();
              createCampaign.mutate();
            }}
          >
            <label className="block text-xs font-medium text-slate-600">
              Nombre
              <input
                className={`${fieldClass} mt-1`}
                value={name}
                onChange={(event) => setName(event.target.value)}
                required
              />
            </label>
            <div className="grid gap-3 sm:grid-cols-2">
              <label className="block text-xs font-medium text-slate-600">
                Canal
                <select
                  className={`${fieldClass} mt-1`}
                  value={channel}
                  onChange={(event) => setChannel(event.target.value)}
                >
                  {Object.entries(channelLabels).map(([value, label]) => (
                    <option key={value} value={value}>
                      {label}
                    </option>
                  ))}
                </select>
              </label>
              <label className="block text-xs font-medium text-slate-600">
                Audiencia
                <input
                  className={`${fieldClass} mt-1`}
                  value={audience}
                  onChange={(event) => setAudience(event.target.value)}
                  placeholder="Ej. socios por vencer"
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
                  {Object.entries(campaignStatusLabels).map(([value, label]) => (
                    <option key={value} value={value}>
                      {label}
                    </option>
                  ))}
                </select>
              </label>
              <label className="block text-xs font-medium text-slate-600">
                Inicio
                <input
                  type="date"
                  className={`${fieldClass} mt-1`}
                  value={startsOn}
                  onChange={(event) => setStartsOn(event.target.value)}
                />
              </label>
              <label className="block text-xs font-medium text-slate-600">
                Fin
                <input
                  type="date"
                  className={`${fieldClass} mt-1`}
                  value={endsOn}
                  onChange={(event) => setEndsOn(event.target.value)}
                />
              </label>
              <label className="block text-xs font-medium text-slate-600">
                Notas
                <input
                  className={`${fieldClass} mt-1`}
                  value={notes}
                  onChange={(event) => setNotes(event.target.value)}
                />
              </label>
            </div>
            <button className={primaryButton} disabled={createCampaign.isPending}>
              {createCampaign.isPending ? "Guardando…" : "Crear campaña"}
            </button>
          </form>
        </section>
      </div>

      <section className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
        <h2 className="text-sm font-semibold text-slate-900">Campañas</h2>
        <div className="mt-4 overflow-x-auto">
          <table className="min-w-full text-sm">
            <thead>
              <tr className="text-left text-xs uppercase tracking-wide text-slate-500">
                <th className="py-2 pr-4">Campaña</th>
                <th className="py-2 pr-4">Canal</th>
                <th className="py-2 pr-4">Audiencia</th>
                <th className="py-2 pr-4">Periodo</th>
                <th className="py-2 pr-4">Estado</th>
                <th className="py-2" />
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {(campaigns.data ?? []).map((campaign) => (
                <tr key={campaign.id}>
                  <td className="py-2 pr-4 font-medium text-slate-900">{campaign.name}</td>
                  <td className="py-2 pr-4 text-slate-600">
                    {channelLabels[campaign.channel] ?? campaign.channel}
                  </td>
                  <td className="py-2 pr-4 text-slate-600">{campaign.audience}</td>
                  <td className="py-2 pr-4 text-slate-500">
                    {campaign.startsOn ?? "—"} → {campaign.endsOn ?? "—"}
                  </td>
                  <td className="py-2 pr-4 text-slate-600">
                    {campaignStatusLabels[campaign.status] ?? campaign.status}
                  </td>
                  <td className="py-2 text-right">
                    {campaign.status !== "completed" && (
                      <button
                        type="button"
                        className={secondaryButton}
                        disabled={changeStatus.isPending}
                        onClick={() =>
                          changeStatus.mutate({
                            campaign,
                            status: campaign.status === "active" ? "paused" : "active",
                          })
                        }
                      >
                        {campaign.status === "active" ? "Pausar" : "Activar"}
                      </button>
                    )}
                  </td>
                </tr>
              ))}
              {(campaigns.data ?? []).length === 0 && (
                <tr>
                  <td className="py-4 text-sm text-slate-500" colSpan={6}>
                    Aún no hay campañas.
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

function Metric({ label, value }: { label: string; value: number | string }) {
  return (
    <div className="rounded-2xl border border-slate-200 bg-white p-4 shadow-sm">
      <p className="text-xs font-medium uppercase tracking-wide text-slate-500">{label}</p>
      <p className="mt-1 text-2xl font-semibold text-slate-900">{value}</p>
    </div>
  );
}
