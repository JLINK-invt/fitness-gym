import type { SiteStatus, SiteSummary } from "@gym/contracts";
import { errorMessage } from "@gym/contracts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { api } from "../lib/tauri";

export function SitesPage() {
  const queryClient = useQueryClient();
  const [name, setName] = useState("");
  const sites = useQuery({
    queryKey: ["sites"],
    queryFn: api.listSites,
  });

  const createSite = useMutation({
    mutationFn: (siteName: string) => api.createSite(siteName),
    onSuccess: () => {
      setName("");
      void queryClient.invalidateQueries({ queryKey: ["sites"] });
    },
  });

  const updateSite = useMutation({
    mutationFn: ({
      siteId,
      siteName,
      status,
    }: {
      siteId: string;
      siteName: string;
      status: SiteStatus;
    }) => api.updateSite(siteId, siteName, status),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["sites"] });
      void queryClient.invalidateQueries({ queryKey: ["staff"] });
    },
  });

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold tracking-tight">Sedes</h1>
        <p className="mt-1 text-sm text-slate-600">
          Cada sede define dónde opera el personal. Cerrar una sede no borra su historial.
        </p>
      </div>

      <form
        className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm"
        onSubmit={(event) => {
          event.preventDefault();
          createSite.mutate(name.trim());
        }}
      >
        <h2 className="text-base font-semibold">Nueva sede</h2>
        <div className="mt-4 flex flex-col gap-3 sm:flex-row sm:items-end">
          <label className="flex-1 text-sm">
            <span className="font-medium text-slate-700">Nombre de la sede</span>
            <input
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder="Sucursal Norte"
              className="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100"
            />
          </label>
          <button
            type="submit"
            disabled={name.trim().length < 2 || createSite.isPending}
            className="rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-700 disabled:opacity-50"
          >
            {createSite.isPending ? "Creando…" : "Crear sede"}
          </button>
        </div>
        {createSite.isError && (
          <p className="mt-3 text-sm text-rose-700">{errorMessage(createSite.error)}</p>
        )}
      </form>

      {updateSite.isError && (
        <div className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
          {errorMessage(updateSite.error)}
        </div>
      )}

      <section className="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm">
        <div className="border-b border-slate-200 px-6 py-4">
          <h2 className="text-base font-semibold">Sedes registradas</h2>
        </div>
        {sites.isError ? (
          <p className="p-6 text-sm text-rose-700">{errorMessage(sites.error)}</p>
        ) : sites.isLoading ? (
          <p className="p-6 text-sm text-slate-500">Cargando sedes…</p>
        ) : sites.data && sites.data.length > 0 ? (
          <ul className="divide-y divide-slate-100">
            {sites.data.map((site) => (
              <SiteRow
                key={site.id}
                site={site}
                saving={updateSite.isPending}
                onSave={(siteName, status) =>
                  updateSite.mutate({ siteId: site.id, siteName, status })
                }
              />
            ))}
          </ul>
        ) : (
          <p className="p-6 text-sm text-slate-500">Aún no hay sedes registradas.</p>
        )}
      </section>
    </div>
  );
}

function SiteRow({
  site,
  saving,
  onSave,
}: {
  site: SiteSummary;
  saving: boolean;
  onSave: (name: string, status: SiteStatus) => void;
}) {
  const [name, setName] = useState(site.name);
  const nextStatus: SiteStatus = site.status === "active" ? "closed" : "active";
  const dirty = name.trim() !== site.name;

  return (
    <li className="flex flex-col gap-3 px-6 py-4 sm:flex-row sm:items-center">
      <div className="flex-1">
        <input
          aria-label={`Nombre de ${site.name}`}
          value={name}
          onChange={(event) => setName(event.target.value)}
          className="w-full max-w-sm rounded-lg border border-slate-300 px-3 py-2 text-sm outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100"
        />
      </div>
      <span
        className={[
          "self-start rounded-full px-3 py-1 text-xs font-semibold ring-1 ring-inset",
          site.status === "active"
            ? "bg-emerald-50 text-emerald-700 ring-emerald-600/20"
            : "bg-slate-100 text-slate-600 ring-slate-500/20",
        ].join(" ")}
      >
        {site.status === "active" ? "Activa" : "Cerrada"}
      </span>
      <div className="flex gap-2">
        <button
          type="button"
          disabled={!dirty || saving}
          onClick={() => onSave(name.trim(), site.status)}
          className="rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium text-slate-700 hover:bg-slate-100 disabled:opacity-50"
        >
          Guardar
        </button>
        <button
          type="button"
          disabled={saving}
          onClick={() => onSave(site.name, nextStatus)}
          className="rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium text-slate-700 hover:bg-slate-100 disabled:opacity-50"
        >
          {site.status === "active" ? "Cerrar sede" : "Reabrir sede"}
        </button>
      </div>
    </li>
  );
}
