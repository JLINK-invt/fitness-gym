import type {
  CreateMembershipPlanInput,
  Member,
  MemberAccount,
  MemberCredential,
  MemberStatus,
  Membership,
  MembershipCharge,
  MembershipPayment,
  MembershipPaymentReceipt,
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
          <p className="text-sm text-slate-600">
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
        <div className="space-y-4">
          <section className="grid gap-4 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-6">
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
            <SummaryCard
              label="Ingresos del mes"
              value={dashboard.data ? money(dashboard.data.incomeThisMonthMinor) : "—"}
              hint={dashboard.data ? `${dashboard.data.newMembersThisMonth} altas nuevas` : undefined}
            />
            <SummaryCard
              label="Renovaciones"
              value={dashboard.data?.renewalsThisMonth ?? "—"}
              hint={dashboard.data ? `${dashboard.data.cancellationsThisMonth} cancelaciones` : undefined}
            />
          </section>
          {dashboard.data && (
            <section className="grid gap-4 lg:grid-cols-2">
              <div className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
                <div className="flex items-center justify-between gap-3">
                  <h2 className="text-base font-semibold">Tendencia 6 meses</h2>
                  <span className="text-xs text-slate-500">
                    {dashboard.data.periodStart} a {dashboard.data.periodEnd} · {dashboard.data.timezone}
                  </span>
                </div>
                <div className="mt-4 space-y-2">
                  {dashboard.data.trend.map((point) => {
                    const maxIncome = Math.max(...dashboard.data!.trend.map((item) => item.incomeMinor), 1);
                    return (
                      <div key={point.month} className="grid grid-cols-[4rem_1fr_auto] items-center gap-3 text-xs">
                        <span className="font-medium text-slate-600">{point.month}</span>
                        <div className="h-2 overflow-hidden rounded-full bg-slate-100">
                          <div className="h-full rounded-full bg-brand-500" style={{ width: `${Math.max((point.incomeMinor / maxIncome) * 100, 2)}%` }} />
                        </div>
                        <span className="text-right text-slate-600">
                          {money(point.incomeMinor)} · {point.newMembers} altas · {point.renewals} ren.
                        </span>
                      </div>
                    );
                  })}
                </div>
              </div>
              <div className="grid gap-4 sm:grid-cols-2">
                <div className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
                  <h2 className="text-base font-semibold">Pagos recientes</h2>
                  <div className="mt-3 space-y-2">
                    {dashboard.data.recentPayments.length === 0 ? (
                      <p className="text-sm text-slate-500">Sin pagos registrados.</p>
                    ) : dashboard.data.recentPayments.map((receipt) => (
                      <div key={receipt.paymentId} className="flex items-start justify-between gap-2 text-xs text-slate-600">
                        <div>
                          <p className="font-medium text-slate-800">{receipt.memberName}</p>
                          <p>{receipt.receiptNumber} · {receipt.chargeDescription}</p>
                        </div>
                        <span className="whitespace-nowrap font-medium">{money(receipt.amountMinor, receipt.currency)}</span>
                      </div>
                    ))}
                  </div>
                </div>
                <div className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
                  <h2 className="text-base font-semibold">Próximos vencimientos</h2>
                  <div className="mt-3 space-y-2">
                    {dashboard.data.expiringMemberships.length === 0 ? (
                      <p className="text-sm text-slate-500">Sin vencimientos próximos.</p>
                    ) : dashboard.data.expiringMemberships.map((membership) => (
                      <div key={membership.id} className="flex items-start justify-between gap-2 text-xs text-slate-600">
                        <div>
                          <p className="font-medium text-slate-800">{membership.memberName}</p>
                          <p>{membership.planName}</p>
                        </div>
                        <span className="whitespace-nowrap">{membership.endDate}</span>
                      </div>
                    ))}
                  </div>
                </div>
              </div>
            </section>
          )}
        </div>
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
            <CredentialsPanel member={selected} />
          </div>

          <AccountPanel
            account={account.data ?? null}
            loading={account.isLoading}
            busy={pay.isPending}
            onPay={(charge, amountMinor, method, reference) =>
              pay.mutateAsync({ charge, amountMinor, method, reference })
            }
          />
        </section>
      )}
    </div>
  );
}

function CredentialsPanel({ member }: { member: Member }) {
  const queryClient = useQueryClient();
  const credentials = useQuery<MemberCredential[]>({
    queryKey: ["member-credentials", member.id],
    queryFn: () => api.listMemberCredentials(member.id),
  });
  const issue = useMutation({
    mutationFn: () => api.issueMemberCredential(member.id),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: ["member-credentials", member.id],
      });
    },
  });
  const revoke = useMutation({
    mutationFn: (credentialId: string) =>
      api.revokeMemberCredential(credentialId),
    onSuccess: () => {
      void queryClient.invalidateQueries({
        queryKey: ["member-credentials", member.id],
      });
    },
  });

  return (
    <div className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
      <div className="flex flex-wrap items-center justify-between gap-2">
        <div>
          <h2 className="text-base font-semibold">Credenciales de acceso</h2>
          <p className="mt-1 text-xs text-slate-500">
            Cada emisión o revocación actualiza la revisión del socio.
          </p>
        </div>
        <button
          type="button"
          onClick={() => issue.mutate()}
          disabled={issue.isPending}
          className={primaryButton}
        >
          {issue.isPending ? "Emitiendo…" : "Emitir credencial"}
        </button>
      </div>

      {credentials.isLoading ? (
        <p className="mt-3 text-sm text-slate-500">Cargando credenciales…</p>
      ) : credentials.isError ? (
        <p className="mt-3 text-sm text-rose-700">
          {errorMessage(credentials.error)}
        </p>
      ) : credentials.data && credentials.data.length > 0 ? (
        <div className="mt-4 space-y-2">
          {credentials.data.map((credential) => (
            <div
              key={credential.id}
              className="flex flex-wrap items-center justify-between gap-2 rounded-xl border border-slate-200 px-3 py-2"
            >
              <div>
                <p className="font-mono text-sm font-medium">
                  {credential.code}
                </p>
                <p className="text-xs text-slate-500">
                  Rev. {credential.revision} · emitida{" "}
                  {formatDateTime(credential.issuedAt)}
                  {credential.revokedAt
                    ? ` · revocada ${formatDateTime(credential.revokedAt)}`
                    : ""}
                </p>
              </div>
              <div className="flex items-center gap-2">
                <span
                  className={[
                    "rounded-full px-2 py-0.5 text-xs font-medium",
                    credential.status === "active"
                      ? "bg-emerald-50 text-emerald-700"
                      : "bg-slate-100 text-slate-600",
                  ].join(" ")}
                >
                  {credential.status === "active" ? "Activa" : "Revocada"}
                </span>
                {credential.status === "active" && (
                  <button
                    type="button"
                    onClick={() => revoke.mutate(credential.id)}
                    disabled={revoke.isPending}
                    className={dangerButton}
                  >
                    Revocar
                  </button>
                )}
              </div>
            </div>
          ))}
        </div>
      ) : (
        <p className="mt-3 text-sm text-slate-500">
          Este socio aún no tiene credenciales.
        </p>
      )}
      {(issue.isError || revoke.isError) && (
        <p className="mt-3 text-sm text-rose-700">
          {errorMessage(issue.error ?? revoke.error)}
        </p>
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
  onPay: (charge: MembershipCharge, amountMinor: number, method: PaymentMethod, reference?: string) => Promise<MembershipPayment>;
}) {
  const [chargeId, setChargeId] = useState<string | null>(null);
  const [amount, setAmount] = useState("");
  const [method, setMethod] = useState<PaymentMethod>("cash");
  const [reference, setReference] = useState("");
  const [receipt, setReceipt] = useState<MembershipPaymentReceipt | null>(null);
  const [receiptBusy, setReceiptBusy] = useState(false);
  const pending = (account?.charges ?? []).filter((charge) => charge.status === "pending");
  const selectedCharge = pending.find((charge) => charge.id === chargeId) ?? null;
  const remaining = selectedCharge ? selectedCharge.amountMinor - selectedCharge.paidMinor : 0;
  const amountMinor = toMinor(amount);
  const loadReceipt = async (paymentId: string) => {
    setReceiptBusy(true);
    try {
      setReceipt(await api.membershipPaymentReceipt(paymentId));
    } finally {
      setReceiptBusy(false);
    }
  };

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
              void onPay(selectedCharge, amountMinor, method, reference.trim() || undefined).then((payment) => {
                setChargeId(null);
                setAmount("");
                setReference("");
                void loadReceipt(payment.id);
              });
            }
          }}
        >
          <div className="grid gap-2 sm:grid-cols-2">
            <input value={amount} onChange={(event) => setAmount(event.target.value)} inputMode="decimal" aria-label="Monto" className={fieldClass} />
            <select value={method} onChange={(event) => setMethod(event.target.value as PaymentMethod)} aria-label="Método de pago" className={fieldClass}>
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
              <span>{formatDateTime(payment.createdAt)} · {payment.paymentMethod} · {payment.receiptNumber}</span>
              <div className="flex items-center gap-2">
                <span>{money(payment.amountMinor, account.currency)}</span>
                <button
                  type="button"
                  disabled={receiptBusy}
                  onClick={() => void loadReceipt(payment.id)}
                  className="rounded border border-slate-300 px-2 py-1 text-[11px] font-medium text-slate-700 hover:bg-slate-50 disabled:opacity-50"
                >
                  Recibo
                </button>
              </div>
            </div>
          ))}
        </div>
      )}

      {receipt && (
        <div className="mt-4 rounded-xl border border-emerald-200 bg-emerald-50 p-4 text-xs text-emerald-900">
          <div className="flex items-start justify-between gap-3">
            <div>
              <p className="text-sm font-semibold">Recibo {receipt.receiptNumber}</p>
              <p>{receipt.gymName} · {formatDateTime(receipt.createdAt)}</p>
              <p className="mt-2">{receipt.memberName} ({receipt.memberNumber})</p>
              <p>{receipt.chargeDescription}</p>
              <p className="mt-1 font-semibold">{money(receipt.amountMinor, receipt.currency)} · {receipt.paymentMethod}</p>
              {receipt.receivedByName && <p>Recibido por {receipt.receivedByName}</p>}
            </div>
            <div className="flex gap-2">
              <button type="button" onClick={() => printReceipt(receipt)} className={secondaryButton}>
                Imprimir
              </button>
              <button type="button" onClick={() => setReceipt(null)} className="text-xs text-emerald-800 hover:underline">
                Cerrar
              </button>
            </div>
          </div>
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

const weekDays = [
  ["mon", "Lun"],
  ["tue", "Mar"],
  ["wed", "Mié"],
  ["thu", "Jue"],
  ["fri", "Vie"],
  ["sat", "Sáb"],
  ["sun", "Dom"],
] as const;

function PlansTab() {
  const queryClient = useQueryClient();
  const [editing, setEditing] = useState<MembershipPlan | null>(null);
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [durationDays, setDurationDays] = useState("30");
  const [price, setPrice] = useState("");
  const [graceDays, setGraceDays] = useState("0");
  const [accessDays, setAccessDays] = useState<string[]>(weekDays.map(([value]) => value));
  const [accessStart, setAccessStart] = useState("06:00");
  const [accessEnd, setAccessEnd] = useState("22:00");
  const [minAge, setMinAge] = useState("");
  const [maxAge, setMaxAge] = useState("");

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
    setAccessDays(weekDays.map(([value]) => value));
    setAccessStart("06:00");
    setAccessEnd("22:00");
    setMinAge("");
    setMaxAge("");
  };

  const save = useMutation({
    mutationFn: () => {
      const input: CreateMembershipPlanInput = {
        name: name.trim(),
        description: description.trim() || undefined,
        durationDays: Number(durationDays),
        priceMinor: toMinor(price),
        graceDays: Number(graceDays),
        accessDays: accessDays.join(","),
        accessStartMinute: timeToMinute(accessStart),
        accessEndMinute: timeToMinute(accessEnd),
        minAge: minAge.trim() ? Number(minAge) : undefined,
        maxAge: maxAge.trim() ? Number(maxAge) : undefined,
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
        accessDays: plan.accessDays,
        accessStartMinute: plan.accessStartMinute,
        accessEndMinute: plan.accessEndMinute,
        minAge: plan.minAge ?? undefined,
        maxAge: plan.maxAge ?? undefined,
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
    setAccessDays(plan.accessDays.split(","));
    setAccessStart(minuteToTime(plan.accessStartMinute));
    setAccessEnd(minuteToTime(plan.accessEndMinute));
    setMinAge(plan.minAge === null ? "" : String(plan.minAge));
    setMaxAge(plan.maxAge === null ? "" : String(plan.maxAge));
  };

  const valid =
    name.trim().length > 0 &&
    Number(durationDays) > 0 &&
    validMoney(price) &&
    Number(graceDays) >= 0 &&
    accessDays.length > 0 &&
    timeToMinute(accessStart) < timeToMinute(accessEnd) &&
    (!minAge.trim() || (Number(minAge) >= 0 && Number(minAge) <= 120)) &&
    (!maxAge.trim() || (Number(maxAge) >= 0 && Number(maxAge) <= 120)) &&
    (!minAge.trim() || !maxAge.trim() || Number(maxAge) >= Number(minAge));
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
          <Field label="Días de duración">
            <input value={durationDays} onChange={(event) => setDurationDays(event.target.value)} inputMode="numeric" className={fieldClass} />
          </Field>
          <Field label="Precio">
            <input value={price} onChange={(event) => setPrice(event.target.value)} inputMode="decimal" className={fieldClass} />
          </Field>
          <Field label="Días de gracia">
            <input value={graceDays} onChange={(event) => setGraceDays(event.target.value)} inputMode="numeric" className={fieldClass} />
          </Field>
        </div>
        <Field label="Días de acceso">
          <div className="flex flex-wrap gap-1.5">
            {weekDays.map(([value, label]) => {
              const checked = accessDays.includes(value);
              return (
                <button
                  key={value}
                  type="button"
                  aria-pressed={checked}
                  onClick={() => setAccessDays((current) => checked ? current.filter((day) => day !== value) : [...current, value])}
                  className={[
                    "rounded-lg border px-2.5 py-1.5 text-xs font-medium transition",
                    checked
                      ? "border-brand-300 bg-brand-50 text-brand-700"
                      : "border-slate-300 text-slate-600 hover:bg-slate-50",
                  ].join(" ")}
                >
                  {label}
                </button>
              );
            })}
          </div>
        </Field>
        <div className="grid gap-3 sm:grid-cols-2">
          <Field label="Acceso desde">
            <input type="time" value={accessStart} onChange={(event) => setAccessStart(event.target.value)} className={fieldClass} />
          </Field>
          <Field label="Acceso hasta">
            <input type="time" value={accessEnd} onChange={(event) => setAccessEnd(event.target.value)} className={fieldClass} />
          </Field>
        </div>
        <div className="grid gap-3 sm:grid-cols-2">
          <Field label="Edad mínima (opcional)">
            <input value={minAge} onChange={(event) => setMinAge(event.target.value)} inputMode="numeric" className={fieldClass} />
          </Field>
          <Field label="Edad máxima (opcional)">
            <input value={maxAge} onChange={(event) => setMaxAge(event.target.value)} inputMode="numeric" className={fieldClass} />
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
                    v{plan.version} · {plan.durationDays} días · gracia {plan.graceDays} días
                    {plan.description ? ` · ${plan.description}` : ""}
                  </p>
                  <p className="text-xs text-slate-500">
                    {plan.accessDays.split(",").map(dayLabel).join(" ")} · {minuteToTime(plan.accessStartMinute)}–{minuteToTime(plan.accessEndMinute)}
                    {plan.minAge !== null || plan.maxAge !== null
                      ? ` · ${plan.minAge ?? 0}–${plan.maxAge ?? 120} años`
                      : " · sin límite de edad"}
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

function dayLabel(value: string): string {
  return weekDays.find(([day]) => day === value)?.[1] ?? value;
}

function minuteToTime(minute: number): string {
  const hours = Math.floor(minute / 60);
  const minutes = minute % 60;
  return `${String(hours).padStart(2, "0")}:${String(minutes).padStart(2, "0")}`;
}

function timeToMinute(value: string): number {
  const [hours = 0, minutes = 0] = value.split(":").map(Number);
  return hours * 60 + minutes;
}

function printReceipt(receipt: MembershipPaymentReceipt) {
  const printWindow = window.open("", "_blank", "width=420,height=640");
  if (!printWindow) {
    return;
  }
  const doc = printWindow.document;
  doc.title = `Recibo ${receipt.receiptNumber}`;
  const style = doc.createElement("style");
  style.textContent = "body{font-family:ui-monospace,monospace;padding:24px;font-size:12px;line-height:1.5}h1{font-size:16px;margin:0 0 4px}p{margin:4px 0}.total{font-size:16px;font-weight:700;margin-top:12px}.muted{color:#555}";
  doc.head.appendChild(style);
  const title = doc.createElement("h1");
  title.textContent = receipt.gymName;
  doc.body.appendChild(title);
  const lines = [
    `Recibo ${receipt.receiptNumber}`,
    formatDateTime(receipt.createdAt),
    "",
    `${receipt.memberName} (${receipt.memberNumber})`,
    receipt.chargeDescription,
    receipt.reference ? `Referencia: ${receipt.reference}` : "",
    receipt.receivedByName ? `Recibido por: ${receipt.receivedByName}` : "",
  ].filter(Boolean);
  for (const line of lines) {
    const paragraph = doc.createElement("p");
    paragraph.textContent = line;
    doc.body.appendChild(paragraph);
  }
  const total = doc.createElement("p");
  total.className = "total";
  total.textContent = `${money(receipt.amountMinor, receipt.currency)} · ${receipt.paymentMethod}`;
  doc.body.appendChild(total);
  const note = doc.createElement("p");
  note.className = "muted";
  note.textContent = "Comprobante no fiscal.";
  doc.body.appendChild(note);
  printWindow.focus();
  printWindow.setTimeout(() => printWindow.print(), 50);
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
