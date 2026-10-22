import type { SiteSummary, StaffRole, StaffStatus, StaffSummary } from "@gym/contracts";
import { errorMessage } from "@gym/contracts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { useOutletContext } from "react-router-dom";

import type { LayoutContext } from "../components/Layout";
import { api } from "../lib/tauri";

const roleLabels: Record<StaffRole, string> = {
  owner: "Dueño",
  manager: "Encargado",
  cashier: "Cajero",
  accounting: "Contabilidad",
};

export function StaffPage() {
  const { session } = useOutletContext<LayoutContext>();
  const queryClient = useQueryClient();
  const [name, setName] = useState("");
  const [role, setRole] = useState<StaffRole>("cashier");
  const [siteId, setSiteId] = useState(session.siteId ?? "");
  const [password, setPassword] = useState("");

  const sites = useQuery({ queryKey: ["sites"], queryFn: api.listSites });
  const staff = useQuery({ queryKey: ["staff"], queryFn: api.listStaff });
  const activeSites = (sites.data ?? []).filter((site) => site.status === "active");

  useEffect(() => {
    if (!siteId && activeSites[0]) {
      setSiteId(activeSites[0].id);
    }
  }, [activeSites, siteId]);

  const createStaff = useMutation({
    mutationFn: () => api.createStaff(name.trim(), role, siteId, password),
    onSuccess: () => {
      setName("");
      setPassword("");
      void queryClient.invalidateQueries({ queryKey: ["staff"] });
    },
  });

  const updateStaff = useMutation({
    mutationFn: ({
      userId,
      nextRole,
      nextSiteId,
      status,
    }: {
      userId: string;
      nextRole: StaffRole;
      nextSiteId: string;
      status: StaffStatus;
    }) => api.updateStaff(userId, nextRole, nextSiteId, status),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["staff"] });
    },
  });

  const canCreate = name.trim().length >= 2 && Boolean(siteId) && password.length >= 8;

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold tracking-tight">Equipo</h1>
        <p className="mt-1 text-sm text-slate-600">
          Crea accesos individuales, asigna rol y sede, y desactiva a quien deja el gimnasio.
        </p>
      </div>

      <form
        className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm"
        onSubmit={(event) => {
          event.preventDefault();
          createStaff.mutate();
        }}
      >
        <h2 className="text-base font-semibold">Agregar empleado</h2>
        <div className="mt-4 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <label className="text-sm">
            <span className="font-medium text-slate-700">Nombre del empleado</span>
            <input
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder="Carlos Caja"
              className="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100"
            />
          </label>
          <label className="text-sm">
            <span className="font-medium text-slate-700">Rol</span>
            <select
              aria-label="Rol de la nueva cuenta"
              value={role}
              onChange={(event) => setRole(event.target.value as StaffRole)}
              className="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100"
            >
              {roleOptions(session.role).map((option) => (
                <option key={option} value={option}>
                  {roleLabels[option]}
                </option>
              ))}
            </select>
          </label>
          <label className="text-sm">
            <span className="font-medium text-slate-700">Sede</span>
            <select
              aria-label="Sede de la nueva cuenta"
              value={siteId}
              onChange={(event) => setSiteId(event.target.value)}
              disabled={session.role === "manager"}
              className="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100 disabled:bg-slate-100"
            >
              <option value="">Selecciona una sede</option>
              {activeSites.map((site) => (
                <option key={site.id} value={site.id}>
                  {site.name}
                </option>
              ))}
            </select>
          </label>
          <label className="text-sm">
            <span className="font-medium text-slate-700">Contraseña inicial</span>
            <input
              type="password"
              value={password}
              onChange={(event) => setPassword(event.target.value)}
              placeholder="Mínimo 8 caracteres"
              className="mt-1 w-full rounded-lg border border-slate-300 px-3 py-2 outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100"
            />
          </label>
        </div>
        <div className="mt-4 flex flex-wrap items-center gap-3">
          <button
            type="submit"
            disabled={!canCreate || createStaff.isPending}
            className="rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-700 disabled:opacity-50"
          >
            {createStaff.isPending ? "Creando…" : "Crear acceso"}
          </button>
          {activeSites.length === 0 && (
            <p className="text-sm text-amber-700">
              Primero crea una sede activa para asignar personal.
            </p>
          )}
          {createStaff.isError && (
            <p className="text-sm text-rose-700">{errorMessage(createStaff.error)}</p>
          )}
          {createStaff.isSuccess && (
            <p className="text-sm text-emerald-700">Acceso creado. Ya puede iniciar sesión.</p>
          )}
        </div>
      </form>

      {updateStaff.isError && (
        <div className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
          {errorMessage(updateStaff.error)}
        </div>
      )}

      <section className="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm">
        <div className="border-b border-slate-200 px-6 py-4">
          <h2 className="text-base font-semibold">Personal registrado</h2>
        </div>
        {staff.isError ? (
          <p className="p-6 text-sm text-rose-700">{errorMessage(staff.error)}</p>
        ) : staff.isLoading ? (
          <p className="p-6 text-sm text-slate-500">Cargando personal…</p>
        ) : staff.data && staff.data.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-slate-200 text-sm">
              <thead className="bg-slate-50 text-left text-xs font-medium uppercase tracking-wide text-slate-500">
                <tr>
                  <th className="px-6 py-3">Persona</th>
                  <th className="px-6 py-3">Rol</th>
                  <th className="px-6 py-3">Sede</th>
                  <th className="px-6 py-3">Último ingreso</th>
                  <th className="px-6 py-3">Estado</th>
                  <th className="px-6 py-3 text-right">Acciones</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {staff.data.map((member) => (
                  <StaffRow
                    key={`${member.userId}-${member.role}-${member.siteId ?? "none"}-${member.status}`}
                    member={member}
                    sessionRole={session.role}
                    self={member.userId === session.userId}
                    sites={sites.data ?? []}
                    saving={updateStaff.isPending}
                    onSave={(nextRole, nextSiteId, status) =>
                      updateStaff.mutate({
                        userId: member.userId,
                        nextRole,
                        nextSiteId,
                        status,
                      })
                    }
                  />
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p className="p-6 text-sm text-slate-500">Aún no hay personal registrado.</p>
        )}
      </section>
    </div>
  );
}

function roleOptions(sessionRole: StaffRole): StaffRole[] {
  return sessionRole === "owner"
    ? ["manager", "cashier", "accounting"]
    : ["cashier", "accounting"];
}

function StaffRow({
  member,
  sessionRole,
  self,
  sites,
  saving,
  onSave,
}: {
  member: StaffSummary;
  sessionRole: StaffRole;
  self: boolean;
  sites: SiteSummary[];
  saving: boolean;
  onSave: (role: StaffRole, siteId: string, status: StaffStatus) => void;
}) {
  const [role, setRole] = useState<StaffRole>(member.role);
  const [siteId, setSiteId] = useState(member.siteId ?? "");
  const owner = member.role === "owner";
  const locked = owner || self;
  const options = roleOptions(sessionRole);
  if (!options.includes(member.role)) {
    options.unshift(member.role);
  }
  const availableSites = sites.filter(
    (site) => site.status === "active" || site.id === member.siteId,
  );
  const dirty = role !== member.role || siteId !== (member.siteId ?? "");

  if (owner) {
    return (
      <tr>
        <td className="px-6 py-4 font-medium">{member.name}</td>
        <td className="px-6 py-4">Dueño</td>
        <td className="px-6 py-4 text-slate-500">Todas las sedes</td>
        <td className="px-6 py-4 text-slate-500">{formatDate(member.lastLoginAt)}</td>
        <td className="px-6 py-4">
          <span className="rounded-full bg-emerald-50 px-2.5 py-1 text-xs font-semibold text-emerald-700">
            Activo
          </span>
        </td>
        <td className="px-6 py-4 text-right text-xs text-slate-400">Cuenta principal</td>
      </tr>
    );
  }

  return (
    <tr>
      <td className="px-6 py-4 font-medium">{member.name}</td>
      <td className="px-6 py-4">
        <select
          aria-label={`Rol de ${member.name}`}
          value={role}
          disabled={locked}
          onChange={(event) => setRole(event.target.value as StaffRole)}
          className="rounded-lg border border-slate-300 px-2 py-1.5 text-sm disabled:bg-slate-100"
        >
          {options.map((option) => (
            <option key={option} value={option}>
              {roleLabels[option]}
            </option>
          ))}
        </select>
      </td>
      <td className="px-6 py-4">
        <select
          aria-label={`Sede de ${member.name}`}
          value={siteId}
          disabled={locked || sessionRole === "manager"}
          onChange={(event) => setSiteId(event.target.value)}
          className="rounded-lg border border-slate-300 px-2 py-1.5 text-sm disabled:bg-slate-100"
        >
          {availableSites.map((site) => (
            <option key={site.id} value={site.id}>
              {site.name}
              {site.status === "closed" ? " (cerrada)" : ""}
            </option>
          ))}
        </select>
      </td>
      <td className="px-6 py-4 text-slate-500">{formatDate(member.lastLoginAt)}</td>
      <td className="px-6 py-4">
        <span
          className={[
            "rounded-full px-2.5 py-1 text-xs font-semibold",
            member.status === "active"
              ? "bg-emerald-50 text-emerald-700"
              : "bg-slate-100 text-slate-600",
          ].join(" ")}
        >
          {member.status === "active" ? "Activo" : "Inactivo"}
        </span>
      </td>
      <td className="px-6 py-4">
        <div className="flex justify-end gap-2">
          <button
            type="button"
            disabled={locked || saving || !dirty || !siteId}
            onClick={() => onSave(role, siteId, member.status)}
            className="rounded-lg border border-slate-300 px-3 py-1.5 text-sm font-medium text-slate-700 hover:bg-slate-100 disabled:opacity-50"
          >
            Guardar
          </button>
          <button
            type="button"
            disabled={locked || saving}
            onClick={() =>
              onSave(
                role,
                siteId,
                member.status === "active" ? "revoked" : "active",
              )
            }
            className="rounded-lg border border-slate-300 px-3 py-1.5 text-sm font-medium text-slate-700 hover:bg-slate-100 disabled:opacity-50"
          >
            {member.status === "active" ? "Desactivar" : "Reactivar"}
          </button>
        </div>
      </td>
    </tr>
  );
}

function formatDate(value: string | null): string {
  if (!value) {
    return "Nunca";
  }
  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? value
    : new Intl.DateTimeFormat("es-MX", { dateStyle: "medium" }).format(date);
}
