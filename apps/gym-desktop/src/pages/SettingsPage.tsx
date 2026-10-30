import type { GymSettings, UpdateGymSettingsInput } from "@gym/contracts";
import { errorMessage } from "@gym/contracts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import type { ReactNode } from "react";
import { useOutletContext } from "react-router-dom";

import type { LayoutContext } from "../components/Layout";
import { formatDateTime } from "../lib/format";
import { api } from "../lib/tauri";

const fieldClass =
  "w-full rounded-lg border border-slate-300 px-3 py-2 text-sm outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100";
const primaryButton =
  "rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-700 disabled:opacity-50";

const emptyDraft: UpdateGymSettingsInput = {
  businessName: "",
  legalName: null,
  taxId: null,
  currency: "MXN",
  timezone: "America/Mexico_City",
  locale: "es-MX",
  openingTime: "06:00",
  closingTime: "22:00",
  slotMinutes: 60,
  graceMinutes: 10,
  inactivityDays: 30,
  frozenDays: 0,
};

export function SettingsPage() {
  const queryClient = useQueryClient();
  const { session } = useOutletContext<LayoutContext>();
  const canManage = session.permissions.includes("manageSettings");
  const [draft, setDraft] = useState<UpdateGymSettingsInput>(emptyDraft);
  const [saved, setSaved] = useState<GymSettings | null>(null);
  const [error, setError] = useState<string | null>(null);

  const settings = useQuery<GymSettings>({
    queryKey: ["gym-settings"],
    queryFn: api.getGymSettings,
  });

  useEffect(() => {
    if (!settings.data) {
      return;
    }
    setDraft({
      businessName: settings.data.businessName,
      legalName: settings.data.legalName,
      taxId: settings.data.taxId,
      currency: settings.data.currency,
      timezone: settings.data.timezone,
      locale: settings.data.locale,
      openingTime: settings.data.openingTime,
      closingTime: settings.data.closingTime,
      slotMinutes: settings.data.slotMinutes,
      graceMinutes: settings.data.graceMinutes,
      inactivityDays: settings.data.inactivityDays,
      frozenDays: settings.data.frozenDays,
    });
  }, [settings.data]);

  const save = useMutation({
    mutationFn: () => api.updateGymSettings(draft),
    onSuccess: (updated) => {
      setError(null);
      setSaved(updated);
      queryClient.setQueryData(["gym-settings"], updated);
    },
    onError: (cause) => setError(errorMessage(cause)),
  });

  const update = <Key extends keyof UpdateGymSettingsInput>(
    key: Key,
    value: UpdateGymSettingsInput[Key],
  ) => setDraft((current) => ({ ...current, [key]: value }));

  if (settings.isError) {
    return (
      <div className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
        {errorMessage(settings.error)}
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <p className="text-sm text-slate-600">
        Datos del gimnasio, horario operativo y reglas que alimentan accesos, reservas y
        alertas de retención.
      </p>

      {error && (
        <div className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
          {error}
        </div>
      )}
      {saved && (
        <div className="rounded-xl border border-emerald-200 bg-emerald-50 p-4 text-sm text-emerald-700">
          Configuración guardada {formatDateTime(saved.updatedAt)}.
        </div>
      )}

      <form
        className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm"
        onSubmit={(event) => {
          event.preventDefault();
          save.mutate();
        }}
      >
        <h2 className="text-sm font-semibold text-slate-900">Identidad</h2>
        <div className="mt-4 grid gap-3 md:grid-cols-2">
          <Field label="Nombre comercial">
            <input
              className={fieldClass}
              value={draft.businessName}
              onChange={(event) => update("businessName", event.target.value)}
              required
              disabled={!canManage}
            />
          </Field>
          <Field label="Moneda">
            <input
              className={fieldClass}
              value={draft.currency}
              onChange={(event) => update("currency", event.target.value.toUpperCase())}
              maxLength={3}
              disabled={!canManage}
            />
          </Field>
          <Field label="Zona horaria">
            <input
              className={fieldClass}
              value={draft.timezone}
              onChange={(event) => update("timezone", event.target.value)}
              disabled={!canManage}
            />
          </Field>
          <Field label="Idioma / locale">
            <input
              className={fieldClass}
              value={draft.locale}
              onChange={(event) => update("locale", event.target.value)}
              disabled={!canManage}
            />
          </Field>
        </div>

        <h2 className="mt-6 text-sm font-semibold text-slate-900">Operación</h2>
        <div className="mt-4 grid gap-3 md:grid-cols-3">
          <Field label="Apertura">
            <input
              type="time"
              className={fieldClass}
              value={draft.openingTime}
              onChange={(event) => update("openingTime", event.target.value)}
              disabled={!canManage}
            />
          </Field>
          <Field label="Cierre">
            <input
              type="time"
              className={fieldClass}
              value={draft.closingTime}
              onChange={(event) => update("closingTime", event.target.value)}
              disabled={!canManage}
            />
          </Field>
          <Field label="Duración de turno (min)">
            <input
              type="number"
              min={15}
              max={240}
              className={fieldClass}
              value={draft.slotMinutes}
              onChange={(event) => update("slotMinutes", Number(event.target.value))}
              disabled={!canManage}
            />
          </Field>
          <Field label="Tolerancia de acceso (min)">
            <input
              type="number"
              min={0}
              max={120}
              className={fieldClass}
              value={draft.graceMinutes}
              onChange={(event) => update("graceMinutes", Number(event.target.value))}
              disabled={!canManage}
            />
          </Field>
          <Field label="Días de inactividad">
            <input
              type="number"
              min={1}
              max={365}
              className={fieldClass}
              value={draft.inactivityDays}
              onChange={(event) => update("inactivityDays", Number(event.target.value))}
              disabled={!canManage}
            />
          </Field>
          <Field label="Días de congelamiento">
            <input
              type="number"
              min={0}
              max={365}
              className={fieldClass}
              value={draft.frozenDays}
              onChange={(event) => update("frozenDays", Number(event.target.value))}
              disabled={!canManage}
            />
          </Field>
        </div>

        <h2 className="mt-6 text-sm font-semibold text-slate-900">
          Datos para exportación
        </h2>
        <p className="mt-1 text-xs text-slate-500">
          Solo acompañan al CSV que compartes con tu contador. La app no calcula
          impuestos ni lleva contabilidad.
        </p>
        <div className="mt-4 grid gap-3 md:grid-cols-2">
          <Field label="Razón social">
            <input
              className={fieldClass}
              value={draft.legalName ?? ""}
              onChange={(event) => update("legalName", event.target.value || null)}
              disabled={!canManage}
            />
          </Field>
          <Field label="RFC / Tax ID">
            <input
              className={fieldClass}
              value={draft.taxId ?? ""}
              onChange={(event) => update("taxId", event.target.value || null)}
              disabled={!canManage}
            />
          </Field>
        </div>

        {canManage && (
          <button className={`${primaryButton} mt-6`} disabled={save.isPending}>
            {save.isPending ? "Guardando…" : "Guardar configuración"}
          </button>
        )}
        {!canManage && (
          <p className="mt-6 text-xs text-slate-500">
            Tu rol puede consultar la configuración, pero no modificarla.
          </p>
        )}
      </form>
    </div>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="block text-xs font-medium text-slate-600">
      {label}
      <div className="mt-1">{children}</div>
    </label>
  );
}
