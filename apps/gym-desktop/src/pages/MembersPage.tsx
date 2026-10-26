import type {
  CreateMembershipPlanInput,
  Member,
  MemberAccount,
  MemberStatus,
  Membership,
  MembershipCharge,
  MembershipPlan,
  PaymentMethod,
  Prospect,
  ProspectStatus,
} from "@gym/contracts";
import { errorMessage } from "@gym/contracts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

import { api } from "../lib/tauri";
import { formatDateTime } from "../lib/format";

const fieldClass =
  "w-full rounded-lg border border-slate-300 px-3 py-2 text-sm outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100";
const primaryButton =
  "rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-700 disabled:opacity-50";
const secondaryButton =
  "rounded-lg border border-slate-300 px-4 py-2 text-sm font-semibold text-slate-700 hover:bg-slate-50 disabled:opacity-50";
const dangerButton =
  "rounded-lg border border-rose-300 px-4 py-2 text-sm font-semibold text-rose-700 hover:bg-rose-50 disabled:opacity-50";

type Tab = "members" | "prospects" | "plans";

const memberStatusLabels: Record<MemberStatus, string> = {
  active: "Activo",
  inactive: "Inactivo",
  archived: "Archivado",
};

const prospectStatusLabels: Record<ProspectStatus, string> = {
  new: "Nuevo",
  contacted: "Contactado",
  trial: "Prueba",
  converted: "Convertido",
  lost: "Perdido",
};

const membershipStatusLabels: Record<Membership["status"], string> = {
  scheduled: "Programada",
  active: "Vigente",
  paused: "Pausada",
  expired: "Vencida",
  cancelled: "Cancelada",
};

export function MembersPage() {
  const [tab, setTab] = useState<Tab>("members");
  const dashboard = useQuery({
    queryKey: ["membership-dashboard"],
    queryFn: api.membershipDashboard,
  });

  return (
    <div className="space-y-6">
      <header className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Socios y membresías</h1>
          <p className="mt-1 text-sm text-slate-600">
            Prospectos, altas, planes, renovaciones y cobros en un solo lugar.
          </p>
        </div>
        <div className="flex rounded-xl border border-slate-200 bg-white p-1 shadow-sm">
          {(["members", "prospects", "plans"] as const).map((value) => (
            <button
              key={value}
              type="button"
              onClick={() => setTab(value)}
              className={[
                "rounded-lg px-3 py-2 text-sm font-medium transition",
                tab === value
                  ? "bg-brand-50 text-brand-700"
                  : "text-slate-600 hover:bg-slate-100",
              ].join(" ")}
            >
              {value === "members" ? "Socios" : value === "prospects" ? "Prospectos" : "Planes"}
            </button>
          ))}
        </div>
      </header>

      {dashboard.isError ? (
        <div className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
          {errorMessage(dashboard.error)}
        </div>
      ) : (
        <section className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <SummaryCard label="Socios activos" value={dashboard.data?.activeMembers ?? "—"} />
          <SummaryCard label="Membresías vigentes" value={dashboard.data?.activeMemberships ?? "—"} />
          <SummaryCard
            label="Por vencer"
            value={dashboard.data?.expiringSoon ?? "—"}
            hint="Próximos 7 días"
            warning={(dashboard.data?.expiringSoon ?? 0) > 0}
          />
          <SummaryCard
            label="Deuda pendiente"
            value={dashboard.data ? money(dashboard.data.debtMinor) : "—"}
            hint={dashboard.data ? `${dashboard.data.pastDue} socios con cargos vencidos` : undefined}
            warning={(dashboard.data?.pastDue ?? 0) > 0}
          />
        </section>
      )}

      {tab === "members" ? (
        <MembersTab />
      ) : tab === "prospects" ? (
        <ProspectsTab />
      ) : (
        <PlansTab />
      )}
    </div>
  );
}

function MembersTab() {
  const queryClient = useQueryClient();
  const [search, setSearch] = useState("");
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [showNew, setShowNew] = useState(false);
  const [firstName, setFirstName] = useState("");
  const [lastName, setLastName] = useState("");
  const [phone, setPhone] = useState("");
  const [email, setEmail] = useState("");
  const [birthDate, setBirthDate] = useState("");
  const [notes, setNotes] = useState("");

  const members = useQuery({
    queryKey: ["members", search],
    queryFn: () => api.listMembers(search),
  });
  const plans = useQuery({
    queryKey: ["membership-plans"],
    queryFn: api.listMembershipPlans,
  });
  const selected = members.data?.find((member) => member.id === selectedId) ?? null;
  const memberships = useQuery({
    queryKey: ["memberships", selectedId],
    queryFn: () => api.listMemberships(selectedId!),
    enabled: Boolean(selectedId),
  });
  const account = useQuery({
    queryKey: ["member-account", selectedId],
    queryFn: () => api.memberAccount(selectedId!),
    enabled: Boolean(selectedId),
  });

  const invalidate = () => {
    void queryClient.invalidateQueries({ queryKey: ["members"] });
    void queryClient.invalidateQueries({ queryKey: ["memberships"] });
    void queryClient.invalidateQueries({ queryKey: ["member-account"] });
    void queryClient.invalidateQueries({ queryKey: ["membership-dashboard"] });
  };

  const createMember = useMutation({
    mutationFn: () =>
      api.createMember({
        firstName: firstName.trim(),
        lastName: lastName.trim(),
        phone: phone.trim() || undefined,
        email: email.trim() || undefined,
        birthDate: birthDate || undefined,
        notes: notes.trim() || undefined,
      }),
    onSuccess: (member) => {
      setShowNew(false);
      setFirstName("");
      setLastName("");
      setPhone("");
      setEmail("");
      setBirthDate("");
      setNotes("");
      setSelectedId(member.id);
      invalidate();
    },
  });

  const createMembership = useMutation({
    mutationFn: ({ memberId, planId, startDate }: { memberId: string; planId: string; startDate?: string }) =>
      api.createMembership({ memberId, planId, startDate }),
    onSuccess: invalidate,
  });

  const renew = useMutation({
    mutationFn: (membership: Membership) =>
      api.renewMembership({ membershipId: membership.id }),
    onSuccess: invalidate,
  });

  const pause = useMutation({
    mutationFn: ({ membershipId, untilDate }: { membershipId: string; untilDate: string }) =>
      api.pauseMembership({ membershipId, untilDate }),
    onSuccess: invalidate,
  });

  const resume = useMutation({
    mutationFn: (membershipId: string) =>
      api.resumeMembership({ membershipId }),
    onSuccess: invalidate,
  });

  const cancel = useMutation({
    mutationFn: ({ membershipId, reason }: { membershipId: string; reason: string }) =>
      api.cancelMembership({ membershipId, reason }),
    onSuccess: invalidate,
  });

  const pay = useMutation({
    mutationFn: ({ charge, amountMinor, method, reference }: {
      charge: MembershipCharge;
      amountMinor: number;
      method: PaymentMethod;
      reference?: string;
    }) =>
      api.recordMembershipPayment({
        chargeId: charge.id,
        amountMinor,
        paymentMethod: method,
        reference,
        idempotencyKey: crypto.randomUUID(),
      }),
    onSuccess: invalidate,
  });

  const canCreate =
    firstName.trim().length > 0 &&
    lastName.trim().length > 0 &&
    (!email.trim() || email.includes("@"));

  const actionError =
    createMembership.error ??
    renew.error ??
    pause.error ??
    resume.error ??
    cancel.error ??
    pay.error;

  return (
    <div className="space-y-5">
      <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <input
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="Buscar por nombre, número, teléfono o email"
            className={`${fieldClass} max-w-md`}
          />
          <button
            type="button"
            onClick={() => setShowNew((value) => !value)}
            className={showNew ? secondaryButton : primaryButton}
          >
            {showNew ? "Cerrar alta" : "Nuevo socio"}
          </button>
        </div>

        {showNew && (
          <form
            className="mt-5 grid gap-4 border-t border-slate-100 pt-5 sm:grid-cols-2 lg:grid-cols-3"
            onSubmit={(event) => {
              event.preventDefault();
              createMember.mutate();
            }}
          >
            <Field label="Nombre">
              <input value={firstName} onChange={(event) => setFirstName(event.target.value)} className={fieldClass} />
            </Field>
            <Field label="Apellido">
              <input value={lastName} onChange={(event) => setLastName(event.target.value)} className={fieldClass} />
            </Field>
            <Field label="Teléfono">
              <input value={phone} onChange={(event) => setPhone(event.target.value)} className={fieldClass} />
            </Field>
            <Field label="Email">
              <input type="email" value={email} onChange={(event) => setEmail(event.target.value)} className={fieldClass} />
            </Field>
            <Field label="Fecha de nacimiento">
              <input type="date" value={birthDate} onChange={(event) => setBirthDate(event.target.value)} className={fieldClass} />
            </Field>
            <Field label="Notas">
              <input value={notes} onChange={(event) => setNotes(event.target.value)} className={fieldClass} />
            </Field>
            <div className="flex items-center gap-3 sm:col-span-2 lg:col-span-3">
              <button type="submit" disabled={!canCreate || createMember.isPending} className={primaryButton}>
                {createMember.isPending ? "Guardando…" : "Dar de alta"}
              </button>
              {createMember.isError && (
                <p className="text-sm text-rose-700">{errorMessage(createMember.error)}</p>
              )}
            </div>
          </form>
        )}
      </section>

      {members.isError ? (
        <p className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
          {errorMessage(members.error)}
        </p>
      ) : members.isLoading ? (
        <p className="text-sm text-slate-500">Cargando socios…</p>
      ) : (
        <section className="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm">
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-slate-200 text-sm">
              <thead className="bg-slate-50 text-left text-xs font-medium uppercase tracking-wide text-slate-500">
                <tr>
                  <th className="px-5 py-3">Socio</th>
                  <th className="px-5 py-3">Estado</th>
                  <th className="px-5 py-3">Membresía</th>
                  <th className="px-5 py-3">Deuda</th>
                  <th className="px-5 py-3 text-right">Acción</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {(members.data ?? []).map((member) => (
                  <tr key={member.id} className={selectedId === member.id ? "bg-brand-50/40" : undefined}>
                    <td className="px-5 py-3">
                      <p className="font-medium text-slate-900">
                        {member.firstName} {member.lastName}
                      </p>
                      <p className="text-xs text-slate-500">
                        {member.memberNumber}
                        {member.phone ? ` · ${member.phone}` : ""}
                      </p>
                    </td>
                    <td className="px-5 py-3">
                      <span className="rounded-full bg-slate-100 px-2.5 py-1 text-xs font-medium text-slate-600">
                        {memberStatusLabels[member.status]}
                      </span>
                    </td>
                    <td className="px-5 py-3">
                      {member.activeMembership ? (
                        <span className="text-slate-700">
                          {member.activeMembership.planName} · hasta {member.activeMembership.endDate}
                        </span>
                      ) : (
                        <span className="text-slate-400">Sin membresía vigente</span>
                      )}
                    </td>
                    <td className="px-5 py-3">
                      <span className={member.debtMinor > 0 ? "font-medium text-rose-700" : "text-slate-500"}>
                        {money(member.debtMinor)}
                      </span>
                    </td>
                    <td className="px-5 py-3 text-right">
                      <button
                        type="button"
                        onClick={() => setSelectedId(member.id)}
                        className="rounded-lg border border-slate-300 px-3 py-1.5 text-xs font-semibold text-slate-700 hover:bg-slate-50"
                      >
                        Gestionar
                      </button>
                    </td>
                  </tr>
                ))}
                {(members.data ?? []).length === 0 && (
                  <tr>
                    <td colSpan={5} className="px-5 py-8 text-center text-sm text-slate-500">
                      No hay socios que coincidan con la búsqueda.
                    </td>
                  </tr>
                )}
              </tbody>
            </table>
          </div>
        </section>
      )}

      {actionError && (
        <div className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
          {errorMessage(actionError)}
        </div>
      )}

      {selected && (
        <section className="grid gap-5 lg:grid-cols-[1.1fr_0.9fr]">
          <div className="space-y-5">
            <div className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
              <div className="flex flex-wrap items-start justify-between gap-3">
                <div>
                  <h2 className="text-lg font-semibold">{selected.firstName} {selected.lastName}</h2>
                  <p className="text-xs text-slate-500">
                    {selected.memberNumber} · alta {selected.joinedAt}
                  </p>
                </div>
                <button type="button" onClick={() => setSelectedId(null)} className="text-sm text-slate-500 hover:text-slate-800">
                  Cerrar
                </button>
              </div>
              <div className="mt-4 grid gap-2 text-sm text-slate-600 sm:grid-cols-2">
                <p>Teléfono: {selected.phone ?? "—"}</p>
                <p>Email: {selected.email ?? "—"}</p>
                <p>Nacimiento: {selected.birthDate ?? "—"}</p>
                <p>Estado: {memberStatusLabels[selected.status]}</p>
              </div>
              {selected.notes && <p className="mt-3 rounded-lg bg-slate-50 p-3 text-sm text-slate-600">{selected.notes}</p>}
            </div>

            <MembershipPanel
              member={selected}
              memberships={memberships.data ?? []}
              plans={plans.data ?? []}
              loading={memberships.isLoading || plans.isLoading}
              busy={createMembership.isPending || renew.isPending || pause.isPending || resume.isPending || cancel.isPending}
              onCreate={(planId, startDate) => createMembership.mutate({ memberId: selected.id, planId, startDate })}
              onRenew={(membership) => renew.mutate(membership)}
              onPause={(membershipId, untilDate) => pause.mutate({ membershipId, untilDate })}
              onResume={(membershipId) => resume.mutate(membershipId)}
              onCancel={(membershipId, reason) => cancel.mutate({ membershipId, reason })}
            />
          </div>

          <AccountPanel
            account={account.data ?? null}
            loading={account.isLoading}
            busy={pay.isPending}
            onPay={(charge, amountMinor, method, reference) =>
              pay.mutate({ charge, amountMinor, method, reference })
            }
          />
        </section>
      )}
    </div>
  );
}

function MembershipPanel({
  member,
  memberships,
  plans,
  loading,
  busy,
  onCreate,
  onRenew,
  onPause,
  onResume,
  onCancel,
}: {
  member: Member;
  memberships: Membership[];
  plans: MembershipPlan[];
  loading: boolean;
  busy: boolean;
  onCreate: (planId: string, startDate?: string) => void;
  onRenew: (membership: Membership) => void;
  onPause: (membershipId: string, untilDate: string) => void;
  onResume: (membershipId: string) => void;
  onCancel: (membershipId: string, reason: string) => void;
}) {
  const [planId, setPlanId] = useState("");
  const [startDate, setStartDate] = useState("");
  const [pauseDate, setPauseDate] = useState("");
  const [cancelReason, setCancelReason] = useState("");
  const current = member.activeMembership ?? memberships.find((item) => ["active", "scheduled", "paused"].includes(item.status));
  const availablePlans = plans.filter((plan) => plan.active);
  const effectivePlanId = planId || availablePlans[0]?.id || "";

  return (
    <div className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
      <h2 className="text-base font-semibold">Membresías</h2>
      {loading ? (
        <p className="mt-3 text-sm text-slate-500">Cargando membresías…</p>
      ) : (
        <>
          {current ? (
            <div className="mt-4 rounded-xl border border-slate-200 p-4">
              <div className="flex flex-wrap items-start justify-between gap-2">
                <div>
                  <p className="font-medium">{current.planName}</p>
                  <p className="text-xs text-slate-500">
                    {membershipStatusLabels[current.status]} · {current.startDate} a {current.endDate}
                  </p>
                </div>
                <p className="text-sm font-medium">{money(current.priceMinor, current.currency)}</p>
              </div>
              <div className="mt-4 flex flex-wrap gap-2">
                {["active", "scheduled"].includes(current.status) && (
                  <>
                    <button type="button" disabled={busy} onClick={() => onRenew(current)} className={secondaryButton}>
                      Renovar
                    </button>
                    <button type="button" disabled={busy} onClick={() => onPause(current.id, pauseDate)} className={secondaryButton}>
                      Pausar
                    </button>
                  </>
                )}
                {current.status === "paused" && (
                  <button type="button" disabled={busy} onClick={() => onResume(current.id)} className={primaryButton}>
                    Reanudar
                  </button>
                )}
                <button
                  type="button"
                  disabled={busy || cancelReason.trim().length === 0}
                  onClick={() => onCancel(current.id, cancelReason.trim())}
                  className={dangerButton}
                >
                  Cancelar
                </button>
              </div>
              <div className="mt-3 grid gap-2 sm:grid-cols-2">
                <input type="date" value={pauseDate} onChange={(event) => setPauseDate(event.target.value)} aria-label="Pausar hasta" className={fieldClass} />
                <input value={cancelReason} onChange={(event) => setCancelReason(event.target.value)} placeholder="Motivo de cancelación" className={fieldClass} />
              </div>
            </div>
          ) : (
            <form
              className="mt-4 grid gap-3 sm:grid-cols-[1fr_auto_auto]"
              onSubmit={(event) => {
                event.preventDefault();
                if (effectivePlanId) {
                  onCreate(effectivePlanId, startDate || undefined);
                }
              }}
            >
              <select value={effectivePlanId} onChange={(event) => setPlanId(event.target.value)} aria-label="Plan de membresía" className={fieldClass}>
                <option value="">Selecciona un plan</option>
                {availablePlans.map((plan) => (
                  <option key={plan.id} value={plan.id}>
                    {plan.name} · {money(plan.priceMinor, plan.currency)}
                  </option>
                ))}
              </select>
              <input type="date" value={startDate} onChange={(event) => setStartDate(event.target.value)} aria-label="Inicio" className={fieldClass} />
              <button type="submit" disabled={!effectivePlanId || busy} className={primaryButton}>
                Alta de membresía
              </button>
            </form>
          )}

          {memberships.length > 1 && (
            <div className="mt-4 space-y-2">
              <p className="text-xs font-medium uppercase tracking-wide text-slate-500">Historial</p>
              {memberships.map((membership) => (
                <div key={membership.id} className="flex flex-wrap items-center justify-between gap-2 rounded-lg bg-slate-50 px-3 py-2 text-xs text-slate-600">
                  <span>{membership.planName} · {membership.startDate} a {membership.endDate}</span>
                  <span>{membershipStatusLabels[membership.status]}</span>
                </div>
              ))}
            </div>
          )}
        </>
      )}
    </div>
  );
}

function AccountPanel({
  account,
  loading,
  busy,
  onPay,
}: {
  account: MemberAccount | null;
  loading: boolean;
  busy: boolean;
  onPay: (charge: MembershipCharge, amountMinor: number, method: PaymentMethod, reference?: string) => void;
}) {
  const [chargeId, setChargeId] = useState<string | null>(null);
  const [amount, setAmount] = useState("");
  const [method, setMethod] = useState<PaymentMethod>("cash");
  const [reference, setReference] = useState("");
  const pending = (account?.charges ?? []).filter((charge) => charge.status === "pending");
  const selectedCharge = pending.find((charge) => charge.id === chargeId) ?? null;
  const remaining = selectedCharge ? selectedCharge.amountMinor - selectedCharge.paidMinor : 0;
  const amountMinor = toMinor(amount);

  return (
    <div className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
      <div className="flex items-center justify-between gap-3">
        <h2 className="text-base font-semibold">Estado de cuenta</h2>
        <span className={account && account.balanceMinor > 0 ? "text-sm font-semibold text-rose-700" : "text-sm font-semibold text-emerald-700"}>
          {account ? money(account.balanceMinor, account.currency) : "—"}
        </span>
      </div>
      {loading ? (
        <p className="mt-3 text-sm text-slate-500">Cargando cargos…</p>
      ) : pending.length === 0 ? (
        <p className="mt-3 text-sm text-emerald-700">Sin cargos pendientes.</p>
      ) : (
        <div className="mt-3 space-y-2">
          {pending.map((charge) => (
            <button
              key={charge.id}
              type="button"
              onClick={() => {
                setChargeId(charge.id);
                setAmount(((charge.amountMinor - charge.paidMinor) / 100).toFixed(2));
              }}
              className={[
                "w-full rounded-xl border p-3 text-left text-sm transition",
                chargeId === charge.id
                  ? "border-brand-300 bg-brand-50"
                  : "border-slate-200 hover:bg-slate-50",
              ].join(" ")}
            >
              <div className="flex items-center justify-between gap-2">
                <span className="font-medium">{charge.description}</span>
                <span>{money(charge.amountMinor - charge.paidMinor, charge.currency)}</span>
              </div>
              <p className="mt-1 text-xs text-slate-500">
                Vence {charge.dueDate}
                {charge.paidMinor > 0 ? ` · abonado ${money(charge.paidMinor, charge.currency)}` : ""}
              </p>
            </button>
          ))}
        </div>
      )}

      {selectedCharge && (
        <form
          className="mt-4 space-y-3 border-t border-slate-100 pt-4"
          onSubmit={(event) => {
            event.preventDefault();
            if (amountMinor > 0 && amountMinor <= remaining) {
              onPay(selectedCharge, amountMinor, method, reference.trim() || undefined);
              setChargeId(null);
              setAmount("");
              setReference("");
            }
          }}
        >
          <div className="grid gap-2 sm:grid-cols-2">
            <input value={amount} onChange={(event) => setAmount(event.target.value)} inputMode="decimal" aria-label="Monto" className={fieldClass} />
            <select value={method} onChange={(event) => setMethod(event.target.value as PaymentMethod)} className={fieldClass}>
              <option value="cash">Efectivo</option>
              <option value="transfer">Transferencia</option>
              <option value="point">Point</option>
            </select>
          </div>
          <input value={reference} onChange={(event) => setReference(event.target.value)} placeholder="Referencia (opcional)" className={fieldClass} />
          <button
            type="submit"
            disabled={busy || amountMinor <= 0 || amountMinor > remaining}
            className={primaryButton}
          >
            {busy ? "Registrando…" : `Registrar pago de ${money(remaining, selectedCharge.currency)}`}
          </button>
        </form>
      )}

      {account && account.payments.length > 0 && (
        <div className="mt-4 space-y-2 border-t border-slate-100 pt-4">
          <p className="text-xs font-medium uppercase tracking-wide text-slate-500">Pagos recientes</p>
          {account.payments.slice(0, 5).map((payment) => (
            <div key={payment.id} className="flex items-center justify-between gap-2 text-xs text-slate-600">
              <span>{formatDateTime(payment.createdAt)} · {payment.paymentMethod}</span>
              <span>{money(payment.amountMinor, account.currency)}</span>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}

function ProspectsTab() {
  const queryClient = useQueryClient();
  const [search, setSearch] = useState("");
  const [name, setName] = useState("");
  const [phone, setPhone] = useState("");
  const [email, setEmail] = useState("");
  const [source, setSource] = useState("");
  const [notes, setNotes] = useState("");

  const prospects = useQuery({
    queryKey: ["prospects", search],
    queryFn: () => api.listProspects(search),
  });

  const invalidate = () => {
    void queryClient.invalidateQueries({ queryKey: ["prospects"] });
    void queryClient.invalidateQueries({ queryKey: ["members"] });
    void queryClient.invalidateQueries({ queryKey: ["membership-dashboard"] });
  };

  const create = useMutation({
    mutationFn: () =>
      api.createProspect({
        name: name.trim(),
        phone: phone.trim() || undefined,
        email: email.trim() || undefined,
        source: source.trim() || undefined,
        notes: notes.trim() || undefined,
      }),
    onSuccess: () => {
      setName("");
      setPhone("");
      setEmail("");
      setSource("");
      setNotes("");
      invalidate();
    },
  });

  const update = useMutation({
    mutationFn: (prospect: Prospect) =>
      api.updateProspect({
        prospectId: prospect.id,
        name: prospect.name,
        phone: prospect.phone ?? undefined,
        email: prospect.email ?? undefined,
        source: prospect.source ?? undefined,
        status: prospect.status,
        notes: prospect.notes ?? undefined,
      }),
    onSuccess: invalidate,
  });

  const convert = useMutation({
    mutationFn: api.convertProspect,
    onSuccess: invalidate,
  });

  const error = create.error ?? update.error ?? convert.error;

  return (
    <div className="space-y-5">
      <form
        className="grid gap-3 rounded-2xl border border-slate-200 bg-white p-5 shadow-sm sm:grid-cols-2 lg:grid-cols-5"
        onSubmit={(event) => {
          event.preventDefault();
          create.mutate();
        }}
      >
        <Field label="Nombre">
          <input value={name} onChange={(event) => setName(event.target.value)} className={fieldClass} />
        </Field>
        <Field label="Teléfono">
          <input value={phone} onChange={(event) => setPhone(event.target.value)} className={fieldClass} />
        </Field>
        <Field label="Email">
          <input value={email} onChange={(event) => setEmail(event.target.value)} className={fieldClass} />
        </Field>
        <Field label="Origen">
          <input value={source} onChange={(event) => setSource(event.target.value)} placeholder="Instagram, referido…" className={fieldClass} />
        </Field>
        <div className="flex items-end">
          <button type="submit" disabled={!name.trim() || create.isPending} className={`${primaryButton} w-full`}>
            {create.isPending ? "Guardando…" : "Agregar prospecto"}
          </button>
        </div>
        <div className="sm:col-span-2 lg:col-span-5">
          <input value={notes} onChange={(event) => setNotes(event.target.value)} placeholder="Notas de seguimiento" className={fieldClass} />
        </div>
      </form>

      {error && (
        <p className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
          {errorMessage(error)}
        </p>
      )}

      <section className="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm">
        <div className="flex flex-wrap items-center justify-between gap-3 border-b border-slate-200 px-5 py-4">
          <h2 className="text-base font-semibold">Prospectos</h2>
          <input
            value={search}
            onChange={(event) => setSearch(event.target.value)}
            placeholder="Buscar prospecto"
            className={`${fieldClass} max-w-xs`}
          />
        </div>
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-slate-200 text-sm">
            <thead className="bg-slate-50 text-left text-xs font-medium uppercase tracking-wide text-slate-500">
              <tr>
                <th className="px-5 py-3">Prospecto</th>
                <th className="px-5 py-3">Origen</th>
                <th className="px-5 py-3">Estado</th>
                <th className="px-5 py-3 text-right">Acción</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {(prospects.data ?? []).map((prospect) => (
                <tr key={prospect.id}>
                  <td className="px-5 py-3">
                    <p className="font-medium text-slate-900">{prospect.name}</p>
                    <p className="text-xs text-slate-500">
                      {[prospect.phone, prospect.email].filter(Boolean).join(" · ") || "Sin contacto"}
                    </p>
                  </td>
                  <td className="px-5 py-3 text-slate-600">{prospect.source ?? "—"}</td>
                  <td className="px-5 py-3">
                    <select
                      value={prospect.status}
                      disabled={prospect.status === "converted" || update.isPending}
                      onChange={(event) => update.mutate({ ...prospect, status: event.target.value as ProspectStatus })}
                      className="rounded-lg border border-slate-300 px-2 py-1 text-xs"
                    >
                      {Object.entries(prospectStatusLabels).map(([value, label]) => (
                        <option key={value} value={value}>{label}</option>
                      ))}
                    </select>
                  </td>
                  <td className="px-5 py-3 text-right">
                    {prospect.status === "converted" ? (
                      <span className="text-xs text-emerald-700">Socio creado</span>
                    ) : (
                      <button
                        type="button"
                        disabled={convert.isPending}
                        onClick={() => convert.mutate(prospect.id)}
                        className="rounded-lg border border-slate-300 px-3 py-1.5 text-xs font-semibold text-slate-700 hover:bg-slate-50 disabled:opacity-50"
                      >
                        Convertir en socio
                      </button>
                    )}
                  </td>
                </tr>
              ))}
              {(prospects.data ?? []).length === 0 && (
                <tr>
                  <td colSpan={4} className="px-5 py-8 text-center text-sm text-slate-500">
                    Aún no hay prospectos registrados.
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

function PlansTab() {
  const queryClient = useQueryClient();
  const [editing, setEditing] = useState<MembershipPlan | null>(null);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [durationDays, setDurationDays] = useState("30");
  const [price, setPrice] = useState("");
  const [graceDays, setGraceDays] = useState("0");

  const plans = useQuery({
    queryKey: ["membership-plans"],
    queryFn: api.listMembershipPlans,
  });

  const reset = () => {
    setEditing(null);
    setName("");
    setDescription("");
    setDurationDays("30");
    setPrice("");
    setGraceDays("0");
  };

  const save = useMutation({
    mutationFn: () => {
      const input: CreateMembershipPlanInput = {
        name: name.trim(),
        description: description.trim() || undefined,
        durationDays: Number(durationDays),
        priceMinor: toMinor(price),
        graceDays: Number(graceDays),
      };
      return editing
        ? api.updateMembershipPlan({ ...input, planId: editing.id, active: editing.active })
        : api.createMembershipPlan(input);
    },
    onSuccess: () => {
      reset();
      void queryClient.invalidateQueries({ queryKey: ["membership-plans"] });
    },
  });

  const toggle = useMutation({
    mutationFn: (plan: MembershipPlan) =>
      api.updateMembershipPlan({
        planId: plan.id,
        name: plan.name,
        description: plan.description ?? undefined,
        durationDays: plan.durationDays,
        priceMinor: plan.priceMinor,
        graceDays: plan.graceDays,
        active: !plan.active,
      }),
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["membership-plans"] });
    },
  });

  const startEdit = (plan: MembershipPlan) => {
    setEditing(plan);
    setName(plan.name);
    setDescription(plan.description ?? "");
    setDurationDays(String(plan.durationDays));
    setPrice((plan.priceMinor / 100).toFixed(2));
    setGraceDays(String(plan.graceDays));
  };

  const valid =
    name.trim().length > 0 &&
    Number(durationDays) > 0 &&
    validMoney(price) &&
    Number(graceDays) >= 0;
  const error = save.error ?? toggle.error;

  return (
    <div className="grid gap-5 lg:grid-cols-[0.9fr_1.1fr]">
      <form
        className="space-y-3 rounded-2xl border border-slate-200 bg-white p-5 shadow-sm"
        onSubmit={(event) => {
          event.preventDefault();
          save.mutate();
        }}
      >
        <h2 className="text-base font-semibold">{editing ? "Editar plan" : "Nuevo plan"}</h2>
        <Field label="Nombre">
          <input value={name} onChange={(event) => setName(event.target.value)} className={fieldClass} />
        </Field>
        <Field label="Descripción">
          <input value={description} onChange={(event) => setDescription(event.target.value)} className={fieldClass} />
        </Field>
        <div className="grid gap-3 sm:grid-cols-3">
          <Field label="Días">
            <input value={durationDays} onChange={(event) => setDurationDays(event.target.value)} inputMode="numeric" className={fieldClass} />
          </Field>
          <Field label="Precio">
            <input value={price} onChange={(event) => setPrice(event.target.value)} inputMode="decimal" className={fieldClass} />
          </Field>
          <Field label="Gracia">
            <input value={graceDays} onChange={(event) => setGraceDays(event.target.value)} inputMode="numeric" className={fieldClass} />
          </Field>
        </div>
        {error && <p className="text-sm text-rose-700">{errorMessage(error)}</p>}
        <div className="flex gap-2">
          <button type="submit" disabled={!valid || save.isPending} className={primaryButton}>
            {save.isPending ? "Guardando…" : editing ? "Guardar cambios" : "Crear plan"}
          </button>
          {editing && (
            <button type="button" onClick={reset} className={secondaryButton}>
              Cancelar
            </button>
          )}
        </div>
      </form>

      <section className="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm">
        <div className="border-b border-slate-200 px-5 py-4">
          <h2 className="text-base font-semibold">Planes</h2>
        </div>
        {plans.isError ? (
          <p className="p-5 text-sm text-rose-700">{errorMessage(plans.error)}</p>
        ) : plans.isLoading ? (
          <p className="p-5 text-sm text-slate-500">Cargando planes…</p>
        ) : (
          <div className="divide-y divide-slate-100">
            {(plans.data ?? []).map((plan) => (
              <div key={plan.id} className="flex flex-wrap items-center justify-between gap-3 px-5 py-4">
                <div>
                  <p className="font-medium">
                    {plan.name}
                    {!plan.active && <span className="ml-2 text-xs text-slate-400">Inactivo</span>}
                  </p>
                  <p className="text-xs text-slate-500">
                    {plan.durationDays} días · gracia {plan.graceDays} días
                    {plan.description ? ` · ${plan.description}` : ""}
                  </p>
                </div>
                <div className="flex items-center gap-3">
                  <span className="text-sm font-semibold">{money(plan.priceMinor, plan.currency)}</span>
                  <button type="button" onClick={() => startEdit(plan)} className={secondaryButton}>
                    Editar
                  </button>
                  <button
                    type="button"
                    disabled={toggle.isPending}
                    onClick={() => toggle.mutate(plan)}
                    className={plan.active ? dangerButton : secondaryButton}
                  >
                    {plan.active ? "Desactivar" : "Activar"}
                  </button>
                </div>
              </div>
            ))}
            {(plans.data ?? []).length === 0 && (
              <p className="px-5 py-8 text-sm text-slate-500">Crea el primer plan de membresía.</p>
            )}
          </div>
        )}
      </section>
    </div>
  );
}

function SummaryCard({
  label,
  value,
  hint,
  warning,
}: {
  label: string;
  value: number | string;
  hint?: string;
  warning?: boolean;
}) {
  return (
    <div className={[
      "rounded-2xl border bg-white p-5 shadow-sm",
      warning ? "border-amber-200" : "border-slate-200",
    ].join(" ")}>
      <p className="text-xs font-medium uppercase tracking-wide text-slate-500">{label}</p>
      <p className={["mt-2 text-2xl font-semibold tracking-tight", warning ? "text-amber-900" : "text-slate-900"].join(" ")}>
        {value}
      </p>
      {hint && <p className="mt-1 text-xs text-slate-500">{hint}</p>}
    </div>
  );
}

function Field({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <label className="text-sm">
      <span className="font-medium text-slate-700">{label}</span>
      <div className="mt-1">{children}</div>
    </label>
  );
}

function money(minor: number, currency = "MXN"): string {
  return new Intl.NumberFormat("es-MX", {
    style: "currency",
    currency,
    maximumFractionDigits: 2,
  }).format(minor / 100);
}

function toMinor(value: string): number {
  return Math.round(Number(value) * 100);
}

function validMoney(value: string): boolean {
  const amount = Number(value);
  return Number.isFinite(amount) && amount >= 0;
}
