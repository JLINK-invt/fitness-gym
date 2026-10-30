import type {
  Permission,
  PermissionEffect,
  StaffRole,
  StaffStatus,
  StaffSummary,
} from "@gym/contracts";
import { errorMessage } from "@gym/contracts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useOutletContext } from "react-router-dom";

import type { LayoutContext } from "../components/Layout";
import { RecoveryCodePanel } from "../components/RecoveryCodePanel";
import {
  effectivePermission,
  ownerOnlyPermissions,
  permissionGroups,
  permissionLabels,
  roleDescriptions,
  roleLabels,
  rolePermissions,
} from "../lib/permissions";
import { api } from "../lib/tauri";

type DraftEffect = PermissionEffect | "inherit";

export function StaffPage() {
  const { session } = useOutletContext<LayoutContext>();
  const queryClient = useQueryClient();
  const [name, setName] = useState("");
  const [role, setRole] = useState<StaffRole>("receptionist");
  const [password, setPassword] = useState("");
  const [recoveryCode, setRecoveryCode] = useState<string | null>(null);
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [draft, setDraft] = useState<Record<string, DraftEffect>>({});

  const staff = useQuery({ queryKey: ["staff"], queryFn: api.listStaff });
  const selected = staff.data?.find((member) => member.userId === selectedId) ?? null;
  const canManagePermissions = session.permissions.includes("managePermissions");

  const generateRecovery = useMutation({
    mutationFn: api.generateOwnerRecovery,
    onSuccess: (result) => setRecoveryCode(result.recoveryCode),
  });

  const createStaff = useMutation({
    mutationFn: () => api.createStaff(name.trim(), role, password),
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
      status,
    }: {
      userId: string;
      nextRole: StaffRole;
      status: StaffStatus;
    }) => api.updateStaff(userId, nextRole, status),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["staff"] });
    },
  });

  const updatePermissions = useMutation({
    mutationFn: ({
      userId,
      overrides,
    }: {
      userId: string;
      overrides: { permission: Permission; effect: PermissionEffect }[];
    }) => api.updateStaffPermissions({ userId, overrides }),
    onSuccess: (updated) => {
      setDraft(draftFromMember(updated));
      void queryClient.invalidateQueries({ queryKey: ["staff"] });
    },
  });

  const canCreate = name.trim().length >= 2 && password.length >= 8;

  function openPermissions(member: StaffSummary) {
    setSelectedId(member.userId);
    setDraft(draftFromMember(member));
    updatePermissions.reset();
  }

  function savePermissions() {
    if (!selected) {
      return;
    }
    const overrides = Object.entries(draft)
      .filter((entry): entry is [Permission, PermissionEffect] => entry[1] !== "inherit")
      .map(([permission, effect]) => ({ permission, effect }));
    updatePermissions.mutate({ userId: selected.userId, overrides });
  }

  return (
    <div className="space-y-6">
      <div>
        <p className="text-sm text-slate-600">
          Crea accesos individuales, asigna roles, ajusta permisos y desactiva a
          quien deja el gimnasio.
        </p>
      </div>

      {session.role === "owner" && (
        <section className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
          {recoveryCode ? (
            <RecoveryCodePanel
              code={recoveryCode}
              title="Nuevo código de recuperación"
              description="El código anterior quedó invalidado. Guárdalo fuera de este equipo antes de cerrar."
              requireConfirmation={false}
              onCancel={() => setRecoveryCode(null)}
            />
          ) : (
            <div className="flex flex-wrap items-start justify-between gap-4">
              <div>
                <h2 className="text-base font-semibold">Recuperación del dueño</h2>
                <p className="mt-1 max-w-2xl text-sm text-slate-600">
                  Genera un código de un solo uso para volver a entrar si olvidas
                  la contraseña. Reemplaza el código anterior y solo se muestra
                  una vez.
                </p>
                {generateRecovery.isError && (
                  <p className="mt-2 text-sm text-rose-700">
                    {errorMessage(generateRecovery.error)}
                  </p>
                )}
              </div>
              <button
                type="button"
                disabled={generateRecovery.isPending}
                onClick={() => generateRecovery.mutate()}
                className="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium text-slate-700 hover:bg-slate-100 disabled:opacity-50"
              >
                {generateRecovery.isPending ? "Generando…" : "Generar código nuevo"}
              </button>
            </div>
          )}
        </section>
      )}

      <form
        className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm"
        onSubmit={(event) => {
          event.preventDefault();
          createStaff.mutate();
        }}
      >
        <h2 className="text-base font-semibold">Agregar colaborador</h2>
        <div className="mt-4 grid gap-4 sm:grid-cols-2 lg:grid-cols-3">
          <label className="text-sm">
            <span className="font-medium text-slate-700">Nombre</span>
            <input
              value={name}
              onChange={(event) => setName(event.target.value)}
              placeholder="Carlos Recepción"
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
            <span className="mt-1 block text-xs text-slate-500">
              {roleDescriptions[role]}
            </span>
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
                  <th className="px-6 py-3">Último ingreso</th>
                  <th className="px-6 py-3">Estado</th>
                  <th className="px-6 py-3 text-right">Acciones</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {staff.data.map((member) => (
                  <StaffRow
                    key={`${member.userId}-${member.role}-${member.status}-${member.permissionOverrides.length}`}
                    member={member}
                    sessionRole={session.role}
                    self={member.userId === session.userId}
                    canManagePermissions={canManagePermissions}
                    selected={member.userId === selectedId}
                    saving={updateStaff.isPending}
                    onSave={(nextRole, status) =>
                      updateStaff.mutate({
                        userId: member.userId,
                        nextRole,
                        status,
                      })
                    }
                    onPermissions={() => openPermissions(member)}
                  />
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p className="p-6 text-sm text-slate-500">Aún no hay personal registrado.</p>
        )}
      </section>

      {selected && canManagePermissions && (
        <section className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
          <div className="flex flex-wrap items-start justify-between gap-4">
            <div>
              <h2 className="text-base font-semibold">
                Permisos de {selected.name}
              </h2>
              <p className="mt-1 text-sm text-slate-600">
                Rol base: {roleLabels[selected.role]}. Los ajustes permiten
                conceder o denegar permisos sin cambiar el rol.
              </p>
            </div>
            <button
              type="button"
              onClick={() => setSelectedId(null)}
              className="rounded-lg border border-slate-300 px-3 py-1.5 text-sm font-medium text-slate-700 hover:bg-slate-100"
            >
              Cerrar
            </button>
          </div>

          <div className="mt-5 space-y-5">
            {permissionGroups.map((group) => (
              <div key={group.label}>
                <h3 className="text-xs font-semibold uppercase tracking-wide text-slate-500">
                  {group.label}
                </h3>
                <div className="mt-2 grid gap-2 md:grid-cols-2">
                  {group.permissions.map((permission) => {
                    const ownerOnly = ownerOnlyPermissions.includes(permission);
                    const inherited = rolePermissions[selected.role].includes(permission);
                    const value = draft[permission] ?? "inherit";
                    const allowed =
                      !ownerOnly &&
                      effectivePermission(
                        selected.role,
                        Object.entries(draft)
                          .filter(
                            (entry): entry is [Permission, PermissionEffect] =>
                              entry[1] !== "inherit",
                          )
                          .map(([key, effect]) => ({ permission: key, effect })),
                        permission,
                      );
                    return (
                      <div
                        key={permission}
                        className="flex items-center justify-between gap-3 rounded-xl border border-slate-200 px-3 py-2"
                      >
                        <div>
                          <p className="text-sm font-medium text-slate-800">
                            {permissionLabels[permission]}
                          </p>
                          <p className="text-xs text-slate-500">
                            {ownerOnly
                              ? "Solo el Dueño"
                              : allowed
                                ? "Permitido"
                                : "Denegado"}
                          </p>
                        </div>
                        <select
                          aria-label={`Permiso ${permissionLabels[permission]} para ${selected.name}`}
                          value={value}
                          disabled={ownerOnly}
                          onChange={(event) =>
                            setDraft((current) => ({
                              ...current,
                              [permission]: event.target.value as DraftEffect,
                            }))
                          }
                          className="rounded-lg border border-slate-300 px-2 py-1.5 text-xs disabled:bg-slate-100"
                        >
                          <option value="inherit">
                            Heredar ({inherited ? "sí" : "no"})
                          </option>
                          <option value="allow">Permitir</option>
                          <option value="deny">Denegar</option>
                        </select>
                      </div>
                    );
                  })}
                </div>
              </div>
            ))}
          </div>

          <div className="mt-5 flex flex-wrap items-center gap-3">
            <button
              type="button"
              disabled={updatePermissions.isPending}
              onClick={savePermissions}
              className="rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-700 disabled:opacity-50"
            >
              {updatePermissions.isPending ? "Guardando…" : "Guardar permisos"}
            </button>
            <button
              type="button"
              disabled={updatePermissions.isPending}
              onClick={() => setDraft(draftFromMember(selected))}
              className="rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium text-slate-700 hover:bg-slate-100 disabled:opacity-50"
            >
              Restablecer cambios
            </button>
            {updatePermissions.isError && (
              <p className="text-sm text-rose-700">
                {errorMessage(updatePermissions.error)}
              </p>
            )}
            {updatePermissions.isSuccess && (
              <p className="text-sm text-emerald-700">Permisos actualizados.</p>
            )}
          </div>
        </section>
      )}
    </div>
  );
}

function draftFromMember(member: StaffSummary): Record<string, DraftEffect> {
  return Object.fromEntries(
    member.permissionOverrides.map((override) => [
      override.permission,
      override.effect,
    ]),
  );
}

function roleOptions(sessionRole: StaffRole): StaffRole[] {
  return sessionRole === "owner"
    ? ["manager", "receptionist", "advisor", "trainer"]
    : ["receptionist", "advisor", "trainer"];
}

function StaffRow({
  member,
  sessionRole,
  self,
  canManagePermissions,
  selected,
  saving,
  onSave,
  onPermissions,
}: {
  member: StaffSummary;
  sessionRole: StaffRole;
  self: boolean;
  canManagePermissions: boolean;
  selected: boolean;
  saving: boolean;
  onSave: (role: StaffRole, status: StaffStatus) => void;
  onPermissions: () => void;
}) {
  const [role, setRole] = useState<StaffRole>(member.role);
  const owner = member.role === "owner";
  const locked =
    owner || self || (sessionRole === "manager" && member.role === "manager");
  const options = roleOptions(sessionRole);
  if (!options.includes(member.role)) {
    options.unshift(member.role);
  }
  const dirty = role !== member.role;
  const permissionCount = member.permissionOverrides.length;

  if (owner) {
    return (
      <tr>
        <td className="px-6 py-4 font-medium">{member.name}</td>
        <td className="px-6 py-4">Dueño</td>
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
    <tr className={selected ? "bg-brand-50/40" : undefined}>
      <td className="px-6 py-4 font-medium">
        {member.name}
        {self && <span className="ml-2 text-xs text-slate-400">(tú)</span>}
      </td>
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
          {canManagePermissions && !locked && (
            <button
              type="button"
              onClick={onPermissions}
              className="rounded-lg border border-slate-300 px-3 py-1.5 text-sm font-medium text-slate-700 hover:bg-slate-100"
            >
              Permisos{permissionCount > 0 ? ` (${permissionCount})` : ""}
            </button>
          )}
          <button
            type="button"
            disabled={locked || saving || !dirty}
            onClick={() => onSave(role, member.status)}
            className="rounded-lg border border-slate-300 px-3 py-1.5 text-sm font-medium text-slate-700 hover:bg-slate-100 disabled:opacity-50"
          >
            Guardar
          </button>
          <button
            type="button"
            disabled={locked || saving}
            onClick={() =>
              onSave(role, member.status === "active" ? "revoked" : "active")
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
