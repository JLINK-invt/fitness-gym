import type {
  ClassBooking,
  ClassSession,
  Member,
  StaffSummary,
} from "@gym/contracts";
import { errorMessage } from "@gym/contracts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useOutletContext } from "react-router-dom";

import type { LayoutContext } from "../components/Layout";
import { api } from "../lib/tauri";

const fieldClass =
  "w-full rounded-lg border border-slate-300 px-3 py-2 text-sm outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100";
const primaryButton =
  "rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-700 disabled:opacity-50";
const secondaryButton =
  "rounded-lg border border-slate-300 px-4 py-2 text-sm font-semibold text-slate-700 hover:bg-slate-50 disabled:opacity-50";

const weekdays = [
  "Domingo",
  "Lunes",
  "Martes",
  "Miércoles",
  "Jueves",
  "Viernes",
  "Sábado",
];

const classStatusLabels: Record<string, string> = {
  active: "Activa",
  paused: "Pausada",
  cancelled: "Cancelada",
};

const bookingStatusLabels: Record<string, string> = {
  booked: "Reservada",
  attended: "Asistió",
  cancelled: "Cancelada",
  no_show: "No asistió",
};

export function ClassesPage() {
  const queryClient = useQueryClient();
  const { session } = useOutletContext<LayoutContext>();
  const canManage = session.permissions.includes("manageClasses");
  const canReserve = session.permissions.includes("manageReservations");
  const [name, setName] = useState("");
  const [coachUserId, setCoachUserId] = useState("");
  const [weekday, setWeekday] = useState("1");
  const [startTime, setStartTime] = useState("07:00");
  const [durationMinutes, setDurationMinutes] = useState("60");
  const [capacity, setCapacity] = useState("20");
  const [room, setRoom] = useState("");
  const [notes, setNotes] = useState("");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [memberId, setMemberId] = useState("");
  const [bookingNote, setBookingNote] = useState("");
  const [error, setError] = useState<string | null>(null);

  const classes = useQuery<ClassSession[]>({
    queryKey: ["classes"],
    queryFn: api.listClasses,
  });
  const staff = useQuery<StaffSummary[]>({
    queryKey: ["staff"],
    queryFn: api.listStaff,
  });
  const members = useQuery<Member[]>({
    queryKey: ["members", "class-booking"],
    queryFn: () => api.listMembers(),
  });
  const selected = classes.data?.find((item) => item.id === selectedId) ?? null;
  const bookings = useQuery<ClassBooking[]>({
    queryKey: ["class-bookings", selectedId],
    queryFn: () => api.listClassBookings(selectedId ?? ""),
    enabled: Boolean(selectedId),
  });

  const refresh = () => {
    void queryClient.invalidateQueries({ queryKey: ["classes"] });
    void queryClient.invalidateQueries({ queryKey: ["class-bookings"] });
  };

  const createClass = useMutation({
    mutationFn: () =>
      api.createClass({
        name,
        coachUserId: coachUserId || null,
        weekday: Number(weekday),
        startTime,
        durationMinutes: Number(durationMinutes),
        capacity: Number(capacity),
        room: room || null,
        notes: notes || null,
      }),
    onSuccess: (created) => {
      setError(null);
      setName("");
      setRoom("");
      setNotes("");
      setSelectedId(created.id);
      refresh();
    },
    onError: (cause) => setError(errorMessage(cause)),
  });

  const changeStatus = useMutation({
    mutationFn: (target: ClassSession) =>
      api.updateClass({
        classId: target.id,
        name: target.name,
        coachUserId: target.coachUserId,
        weekday: target.weekday,
        startTime: target.startTime,
        durationMinutes: target.durationMinutes,
        capacity: target.capacity,
        room: target.room,
        status: target.status === "active" ? "paused" : "active",
        notes: target.notes,
      }),
    onSuccess: () => {
      setError(null);
      refresh();
    },
    onError: (cause) => setError(errorMessage(cause)),
  });

  const book = useMutation({
    mutationFn: () =>
      api.bookClass({
        classId: selectedId ?? "",
        memberId,
        note: bookingNote || null,
      }),
    onSuccess: () => {
      setError(null);
      setMemberId("");
      setBookingNote("");
      refresh();
    },
    onError: (cause) => setError(errorMessage(cause)),
  });

  const updateBooking = useMutation({
    mutationFn: (input: { booking: ClassBooking; status: string }) =>
      api.updateClassBooking({ bookingId: input.booking.id, status: input.status }),
    onSuccess: () => {
      setError(null);
      refresh();
    },
    onError: (cause) => setError(errorMessage(cause)),
  });

  return (
    <div className="space-y-6">
      <p className="text-sm text-slate-600">
        Agenda semanal, cupo por clase y reservas de socios con control de asistencia.
      </p>

      {error && (
        <div className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
          {error}
        </div>
      )}

      {canManage && (
        <form
          className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm"
          onSubmit={(event) => {
            event.preventDefault();
            createClass.mutate();
          }}
        >
          <h2 className="text-sm font-semibold text-slate-900">Nueva clase</h2>
          <div className="mt-4 grid gap-3 md:grid-cols-3">
            <label className="text-xs font-medium text-slate-600">
              Nombre
              <input
                className={`${fieldClass} mt-1`}
                value={name}
                onChange={(event) => setName(event.target.value)}
                required
              />
            </label>
            <label className="text-xs font-medium text-slate-600">
              Entrenador
              <select
                className={`${fieldClass} mt-1`}
                value={coachUserId}
                onChange={(event) => setCoachUserId(event.target.value)}
              >
                <option value="">Sin asignar</option>
                {(staff.data ?? []).map((member) => (
                  <option key={member.userId} value={member.userId}>
                    {member.name}
                  </option>
                ))}
              </select>
            </label>
            <label className="text-xs font-medium text-slate-600">
              Día
              <select
                className={`${fieldClass} mt-1`}
                value={weekday}
                onChange={(event) => setWeekday(event.target.value)}
              >
                {weekdays.map((label, index) => (
                  <option key={label} value={index}>
                    {label}
                  </option>
                ))}
              </select>
            </label>
            <label className="text-xs font-medium text-slate-600">
              Inicio
              <input
                type="time"
                className={`${fieldClass} mt-1`}
                value={startTime}
                onChange={(event) => setStartTime(event.target.value)}
                required
              />
            </label>
            <label className="text-xs font-medium text-slate-600">
              Duración (min)
              <input
                type="number"
                min={15}
                max={480}
                className={`${fieldClass} mt-1`}
                value={durationMinutes}
                onChange={(event) => setDurationMinutes(event.target.value)}
              />
            </label>
            <label className="text-xs font-medium text-slate-600">
              Capacidad
              <input
                type="number"
                min={1}
                max={500}
                className={`${fieldClass} mt-1`}
                value={capacity}
                onChange={(event) => setCapacity(event.target.value)}
              />
            </label>
            <label className="text-xs font-medium text-slate-600">
              Sala
              <input
                className={`${fieldClass} mt-1`}
                value={room}
                onChange={(event) => setRoom(event.target.value)}
              />
            </label>
            <label className="text-xs font-medium text-slate-600 md:col-span-2">
              Notas
              <input
                className={`${fieldClass} mt-1`}
                value={notes}
                onChange={(event) => setNotes(event.target.value)}
              />
            </label>
          </div>
          <button className={`${primaryButton} mt-4`} disabled={createClass.isPending}>
            {createClass.isPending ? "Guardando…" : "Crear clase"}
          </button>
        </form>
      )}

      <div className="grid gap-6 lg:grid-cols-2">
        <section className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
          <h2 className="text-sm font-semibold text-slate-900">Agenda</h2>
          <div className="mt-4 space-y-2">
            {(classes.data ?? []).map((item) => (
              <button
                key={item.id}
                type="button"
                onClick={() => setSelectedId(item.id)}
                className={`w-full rounded-xl border p-4 text-left text-sm ${
                  item.id === selectedId
                    ? "border-brand-400 bg-brand-50"
                    : "border-slate-200 hover:bg-slate-50"
                }`}
              >
                <div className="flex items-center justify-between gap-3">
                  <span className="font-semibold text-slate-900">
                    {weekdays[item.weekday]} {item.startTime} · {item.name}
                  </span>
                  <span className="text-xs font-semibold text-slate-600">
                    {classStatusLabels[item.status] ?? item.status}
                  </span>
                </div>
                <div className="mt-1 text-xs text-slate-500">
                  {item.coachName ?? "Sin entrenador"} · {item.booked}/{item.capacity} reservas
                  {item.room ? ` · ${item.room}` : ""}
                </div>
              </button>
            ))}
            {(classes.data ?? []).length === 0 && (
              <p className="text-sm text-slate-500">Aún no hay clases programadas.</p>
            )}
          </div>
        </section>

        <section className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
          <h2 className="text-sm font-semibold text-slate-900">
            Reservas {selected ? `· ${selected.name}` : ""}
          </h2>
          {selected ? (
            <>
              <div className="mt-3 flex flex-wrap items-center gap-2 text-xs text-slate-600">
                <span>
                  {weekdays[selected.weekday]} {selected.startTime} ·{" "}
                  {selected.attended} asistencias
                </span>
                {canManage && (
                  <button
                    type="button"
                    className={secondaryButton}
                    disabled={changeStatus.isPending}
                    onClick={() => changeStatus.mutate(selected)}
                  >
                    {selected.status === "active" ? "Pausar clase" : "Activar clase"}
                  </button>
                )}
              </div>
              {canReserve && (
                <form
                  className="mt-4 grid gap-2 sm:grid-cols-[1fr_1fr_auto]"
                  onSubmit={(event) => {
                    event.preventDefault();
                    book.mutate();
                  }}
                >
                  <select
                    className={fieldClass}
                    value={memberId}
                    onChange={(event) => setMemberId(event.target.value)}
                    required
                  >
                    <option value="">Selecciona un socio</option>
                    {(members.data ?? []).map((member) => (
                      <option key={member.id} value={member.id}>
                        {member.firstName} {member.lastName}
                      </option>
                    ))}
                  </select>
                  <input
                    className={fieldClass}
                    placeholder="Nota (opcional)"
                    value={bookingNote}
                    onChange={(event) => setBookingNote(event.target.value)}
                  />
                  <button className={primaryButton} disabled={book.isPending || !memberId}>
                    Reservar
                  </button>
                </form>
              )}
              <ul className="mt-4 space-y-2">
                {(bookings.data ?? []).map((booking) => (
                  <li
                    key={booking.id}
                    className="flex flex-wrap items-center justify-between gap-2 rounded-xl border border-slate-200 p-3 text-sm"
                  >
                    <span className="font-medium text-slate-900">{booking.memberName}</span>
                    <span className="text-xs text-slate-500">
                      {bookingStatusLabels[booking.status] ?? booking.status}
                    </span>
                    {canReserve && booking.status === "booked" && (
                      <span className="flex gap-2">
                        <button
                          type="button"
                          className={secondaryButton}
                          disabled={updateBooking.isPending}
                          onClick={() =>
                            updateBooking.mutate({ booking, status: "attended" })
                          }
                        >
                          Asistió
                        </button>
                        <button
                          type="button"
                          className={secondaryButton}
                          disabled={updateBooking.isPending}
                          onClick={() =>
                            updateBooking.mutate({ booking, status: "no_show" })
                          }
                        >
                          No asistió
                        </button>
                        <button
                          type="button"
                          className={secondaryButton}
                          disabled={updateBooking.isPending}
                          onClick={() =>
                            updateBooking.mutate({ booking, status: "cancelled" })
                          }
                        >
                          Cancelar
                        </button>
                      </span>
                    )}
                  </li>
                ))}
                {(bookings.data ?? []).length === 0 && (
                  <li className="text-sm text-slate-500">Esta clase no tiene reservas.</li>
                )}
              </ul>
            </>
          ) : (
            <p className="mt-3 text-sm text-slate-500">
              Selecciona una clase para ver y gestionar sus reservas.
            </p>
          )}
        </section>
      </div>
    </div>
  );
}
