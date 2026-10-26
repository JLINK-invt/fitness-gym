import type {
  AccessEvent,
  AccessRequest,
  AccessSnapshot,
  AttendanceReport,
} from "@gym/contracts";
import { errorMessage } from "@gym/contracts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState, type FormEvent } from "react";
import { useOutletContext } from "react-router-dom";

import type { LayoutContext } from "../components/Layout";
import { formatDateTime, shortenChecksum } from "../lib/format";
import { api, isTauriRuntime } from "../lib/tauri";

function inputDate(date: Date): string {
  const offset = date.getTimezoneOffset() * 60_000;
  return new Date(date.getTime() - offset).toISOString().slice(0, 10);
}

function dayStart(value: string): string {
  return new Date(`${value}T00:00:00`).toISOString();
}

function dayEnd(value: string): string {
  return new Date(`${value}T23:59:59.999`).toISOString();
}

export function AccessPage() {
  const queryClient = useQueryClient();
  const inTauri = isTauriRuntime();
  const { session } = useOutletContext<LayoutContext>();
  const canManage = session.permissions.includes("manageMembers");
  const canOperate = session.permissions.includes("operatePos");
  const today = new Date();
  const monthAgo = new Date(today.getTime() - 29 * 24 * 60 * 60 * 1_000);
  const [credentialId, setCredentialId] = useState("CRED-ALLOW");
  const [deviceId, setDeviceId] = useState("recepcion-1");
  const [lastRequest, setLastRequest] = useState<AccessRequest | null>(null);
  const [fromDate, setFromDate] = useState(inputDate(monthAgo));
  const [toDate, setToDate] = useState(inputDate(today));

  const snapshot = useQuery<AccessSnapshot | null>({
    queryKey: ["access-snapshot"],
    queryFn: api.latestAccessSnapshot,
    enabled: inTauri && canManage,
  });

  const publishSnapshot = useMutation({
    mutationFn: api.publishAccessSnapshot,
    onSuccess: (published) => {
      queryClient.setQueryData(["access-snapshot"], published);
      void queryClient.invalidateQueries({ queryKey: ["audit-events"] });
    },
  });

  const events = useQuery<AccessEvent[]>({
    queryKey: ["access-event-log"],
    queryFn: () => api.listAccessEventLog(20),
    enabled: inTauri && canManage,
    refetchInterval: 2000,
  });

  const attendance = useQuery<AttendanceReport>({
    queryKey: ["attendance-report", fromDate, toDate],
    queryFn: () =>
      api.attendanceReport({ from: dayStart(fromDate), to: dayEnd(toDate) }),
    enabled:
      inTauri &&
      canManage &&
      fromDate !== "" &&
      toDate !== "" &&
      fromDate <= toDate,
  });

  const simulate = useMutation({
    mutationFn: api.simulateAccess,
    onSuccess: async () => {
      await queryClient.invalidateQueries({ queryKey: ["access-event-log"] });
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

  const rangeValid =
    fromDate !== "" && toDate !== "" && fromDate <= toDate;

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold tracking-tight">Accesos y asistencia</h1>
        <p className="mt-1 text-sm text-slate-600">
          Publica la nómina firmada, prueba decisiones locales y revisa la
          asistencia y ocupación por hora.
        </p>
      </div>

      <section
        hidden={!canManage}
        className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm"
      >
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div>
            <h2 className="text-base font-semibold">Snapshot de acceso</h2>
            <p className="mt-1 text-sm text-slate-600">
              Firma la nómina activa con la clave local. Cada publicación crea
              una nueva revisión.
            </p>
          </div>
          <button
            type="button"
            onClick={() => publishSnapshot.mutate()}
            disabled={publishSnapshot.isPending}
            className="rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-700 disabled:opacity-50"
          >
            {publishSnapshot.isPending ? "Publicando…" : "Publicar snapshot"}
          </button>
        </div>
        {snapshot.isLoading ? (
          <p className="mt-3 text-sm text-slate-500">Cargando snapshot…</p>
        ) : snapshot.data ? (
          <div className="mt-4 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
            <Metric label="Revisión" value={String(snapshot.data.revision)} />
            <Metric label="Socios" value={String(snapshot.data.memberCount)} />
            <Metric
              label="Credenciales"
              value={String(snapshot.data.credentialCount)}
            />
            <Metric
              label="Publicado"
              value={formatDateTime(snapshot.data.generatedAt)}
            />
            <p className="text-xs text-slate-500 sm:col-span-2 lg:col-span-4">
              Checksum{" "}
              <span className="font-mono">
                {shortenChecksum(snapshot.data.checksum, 16)}
              </span>{" "}
              · Firma{" "}
              <span className="font-mono">
                {shortenChecksum(snapshot.data.signature, 16)}
              </span>
            </p>
          </div>
        ) : (
          <p className="mt-3 text-sm text-slate-500">
            Aún no hay snapshots publicados.
          </p>
        )}
        {publishSnapshot.isError && (
          <p className="mt-3 text-sm text-rose-700">
            {errorMessage(publishSnapshot.error)}
          </p>
        )}
      </section>

      <section
        hidden={!canOperate}
        className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm"
      >
        <h2 className="text-base font-semibold">Nueva lectura</h2>
        <p className="mt-1 text-sm text-slate-600">
          Usa el código de una credencial emitida o las credenciales simuladas
          CRED-ALLOW / CRED-DENY.
        </p>
        <form
          onSubmit={handleSubmit}
          className="mt-4 grid gap-3 sm:grid-cols-[1fr_1fr_auto]"
        >
          <label className="text-sm">
            <span className="mb-1 block font-medium text-slate-700">Credencial</span>
            <input
              value={credentialId}
              onChange={(event) => setCredentialId(event.target.value)}
              className="w-full rounded-lg border border-slate-300 px-3 py-2 font-mono text-sm outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100"
            />
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
              disabled={
                simulate.isPending || !credentialId.trim() || !deviceId.trim()
              }
            >
              {simulate.isPending ? "Procesando…" : "Simular acceso"}
            </button>
          </div>
        </form>

        {simulate.isError && (
          <p className="mt-3 text-sm text-rose-700">
            {errorMessage(simulate.error)}
          </p>
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

      <section
        hidden={!canManage}
        className="rounded-2xl border border-slate-200 bg-white shadow-sm"
      >
        <div className="flex items-center justify-between border-b border-slate-200 px-6 py-4">
          <div>
            <h2 className="text-base font-semibold">Bitácora de accesos</h2>
            <p className="mt-1 text-xs text-slate-500">
              Eventos reconciliados, con credencial, socio y lector.
            </p>
          </div>
          <button
            type="button"
            onClick={() => events.refetch()}
            className="text-sm font-medium text-brand-700 hover:text-brand-600"
            disabled={events.isFetching}
          >
            Actualizar
          </button>
        </div>

        {events.isLoading ? (
          <p className="px-6 py-8 text-sm text-slate-500">Cargando eventos…</p>
        ) : events.isError ? (
          <p className="px-6 py-8 text-sm text-rose-700">
            {errorMessage(events.error)}
          </p>
        ) : events.data && events.data.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-slate-200 text-sm">
              <thead className="bg-slate-50 text-left text-xs uppercase tracking-wide text-slate-500">
                <tr>
                  <th className="px-6 py-3 font-medium">Resultado</th>
                  <th className="px-6 py-3 font-medium">Socio</th>
                  <th className="px-6 py-3 font-medium">Credencial</th>
                  <th className="px-6 py-3 font-medium">Lector</th>
                  <th className="px-6 py-3 font-medium">Estado</th>
                  <th className="px-6 py-3 font-medium">Fecha</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {events.data.map((event) => (
                  <tr key={event.id}>
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
                    <td className="px-6 py-3">
                      <p className="font-medium">
                        {event.memberName ?? "Sin socio"}
                      </p>
                      <p className="text-xs text-slate-500">
                        {event.memberNumber ?? "—"}
                      </p>
                    </td>
                    <td className="px-6 py-3 font-mono text-xs">
                      {event.credentialCode}
                    </td>
                    <td className="px-6 py-3 text-xs">
                      {event.deviceId}
                      {event.zone ? ` · ${event.zone}` : ""}
                    </td>
                    <td className="px-6 py-3 text-xs">
                      {event.reconciliationStatus}
                    </td>
                    <td className="px-6 py-3">
                      {formatDateTime(event.occurredAt)}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p className="px-6 py-8 text-sm text-slate-500">
            Aún no hay eventos reconciliados.
          </p>
        )}
      </section>

      <section
        hidden={!canManage}
        className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm"
      >
        <div className="flex flex-wrap items-end justify-between gap-3">
          <div>
            <h2 className="text-base font-semibold">Asistencia y ocupación</h2>
            <p className="mt-1 text-sm text-slate-600">
              Visitas únicas por día, hora y zona. Las horas de baja demanda se
              calculan contra la hora pico.
            </p>
          </div>
          <div className="flex flex-wrap items-end gap-2">
            <label className="text-sm">
              <span className="block font-medium text-slate-700">Desde</span>
              <input
                type="date"
                value={fromDate}
                max={toDate}
                onChange={(event) => setFromDate(event.target.value)}
                className="mt-1 rounded-lg border border-slate-300 px-3 py-2 text-sm"
              />
            </label>
            <label className="text-sm">
              <span className="block font-medium text-slate-700">Hasta</span>
              <input
                type="date"
                value={toDate}
                min={fromDate}
                onChange={(event) => setToDate(event.target.value)}
                className="mt-1 rounded-lg border border-slate-300 px-3 py-2 text-sm"
              />
            </label>
          </div>
        </div>

        {!rangeValid ? (
          <p className="mt-4 text-sm text-rose-700">
            La fecha inicial no puede ser posterior a la final.
          </p>
        ) : attendance.isLoading ? (
          <p className="mt-4 text-sm text-slate-500">Calculando asistencia…</p>
        ) : attendance.isError ? (
          <p className="mt-4 text-sm text-rose-700">
            {errorMessage(attendance.error)}
          </p>
        ) : attendance.data ? (
          <div className="mt-4 space-y-4">
            <div className="grid gap-3 sm:grid-cols-3">
              <Metric
                label="Visitas"
                value={String(attendance.data.totalVisits)}
              />
              <Metric
                label="Socios únicos"
                value={String(attendance.data.uniqueMembers)}
              />
              <Metric
                label="Hora pico"
                value={`${String(attendance.data.peakHour).padStart(2, "0")}:00`}
              />
            </div>
            <div className="grid gap-4 lg:grid-cols-2">
              <div>
                <h3 className="text-sm font-semibold">Por hora</h3>
                <div className="mt-2 space-y-1">
                  {attendance.data.hours.map((hour) => (
                    <div
                      key={hour.hour}
                      className="flex items-center gap-2 text-xs"
                    >
                      <span className="w-10 font-mono text-slate-500">
                        {String(hour.hour).padStart(2, "0")}:00
                      </span>
                      <div className="h-2 flex-1 overflow-hidden rounded-full bg-slate-100">
                        <div
                          className="h-full rounded-full bg-brand-500"
                          style={{
                            width: `${
                              attendance.data.totalVisits > 0
                                ? (hour.visits /
                                    Math.max(
                                      ...attendance.data.hours.map(
                                        (item) => item.visits,
                                      ),
                                      1,
                                    )) *
                                  100
                                : 0
                            }%`,
                          }}
                        />
                      </div>
                      <span className="w-8 text-right text-slate-600">
                        {hour.visits}
                      </span>
                    </div>
                  ))}
                </div>
                {attendance.data.lowDemandHours.length > 0 && (
                  <p className="mt-2 text-xs text-slate-500">
                    Baja demanda:{" "}
                    {attendance.data.lowDemandHours
                      .map((hour) => `${String(hour.hour).padStart(2, "0")}:00`)
                      .join(", ")}
                  </p>
                )}
              </div>
              <div>
                <h3 className="text-sm font-semibold">Por día</h3>
                {attendance.data.days.length === 0 ? (
                  <p className="mt-2 text-sm text-slate-500">
                    Sin visitas en el rango.
                  </p>
                ) : (
                  <div className="mt-2 space-y-1">
                    {attendance.data.days.map((day) => (
                      <div
                        key={day.date}
                        className="flex items-center justify-between rounded-lg bg-slate-50 px-3 py-1.5 text-xs"
                      >
                        <span>{day.date}</span>
                        <span>
                          {day.visits} visitas · {day.uniqueMembers} socios
                        </span>
                      </div>
                    ))}
                  </div>
                )}
              </div>
            </div>
          </div>
        ) : null}
      </section>
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-xl border border-slate-200 p-4">
      <p className="text-xs font-medium uppercase tracking-wide text-slate-500">
        {label}
      </p>
      <p className="mt-1 text-lg font-semibold tracking-tight">{value}</p>
    </div>
  );
}
