import type {
  CashMovement,
  CashMovementType,
  CashSession,
  PaymentMethod,
} from "@gym/contracts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useEffect, useState } from "react";
import { useOutletContext } from "react-router-dom";

import type { LayoutContext } from "../components/Layout";
import { api } from "../lib/tauri";

const fieldClass =
  "w-full rounded-lg border border-slate-300 px-3 py-2 text-sm outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100";
const primaryButton =
  "rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-700 disabled:opacity-50";
const secondaryButton =
  "rounded-lg border border-slate-300 px-4 py-2 text-sm font-semibold text-slate-700 hover:bg-slate-50 disabled:opacity-50";

export function CashPage() {
  const queryClient = useQueryClient();
  const { session } = useOutletContext<LayoutContext>();
  const canOperate = session.permissions.includes("operatePos");
  const canViewHistory = session.permissions.includes("viewFinance");
  const [openingAmount, setOpeningAmount] = useState("0");
  const [countedAmount, setCountedAmount] = useState("");
  const [differenceReason, setDifferenceReason] = useState("");
  const [movementType, setMovementType] = useState<CashMovementType>("expense");
  const [movementAmount, setMovementAmount] = useState("");
  const [movementCategory, setMovementCategory] = useState("");
  const [movementMethod, setMovementMethod] = useState<PaymentMethod>("cash");
  const [movementEvidence, setMovementEvidence] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [closed, setClosed] = useState<CashSession | null>(null);

  const current = useQuery<CashSession | null>({
    queryKey: ["cash-session"],
    queryFn: api.currentCashSession,
    enabled: canOperate,
  });
  const history = useQuery<CashSession[]>({
    queryKey: ["cash-sessions"],
    queryFn: api.listCashSessions,
    enabled: canViewHistory,
  });
  const activeSessionId = current.data?.id;
  const movements = useQuery<CashMovement[]>({
    queryKey: ["cash-movements", activeSessionId],
    queryFn: () => api.listCashMovements(activeSessionId!),
    enabled: canOperate && Boolean(activeSessionId),
  });

  useEffect(() => {
    if (current.data) {
      setCountedAmount((current.data.expectedCashMinor / 100).toFixed(2));
      setDifferenceReason("");
      setClosed(null);
    }
  }, [current.data]);

  const refresh = () => {
    queryClient.invalidateQueries({ queryKey: ["cash-session"] });
    queryClient.invalidateQueries({ queryKey: ["cash-sessions"] });
    queryClient.invalidateQueries({ queryKey: ["cash-movements"] });
    queryClient.invalidateQueries({ queryKey: ["sales"] });
    queryClient.invalidateQueries({ queryKey: ["audit"] });
  };

  const open = useMutation({
    mutationFn: () =>
      api.openCashSession({ openingAmountMinor: toMinor(openingAmount) }),
    onSuccess: (opened) => {
      setOpeningAmount("0");
      setError(null);
      setClosed(null);
      queryClient.setQueryData(["cash-session"], opened);
      refresh();
    },
    onError: (cause) => {
      setError(cause instanceof Error ? cause.message : "No se pudo abrir la caja.");
    },
  });

  const recordMovement = useMutation({
    mutationFn: (active: CashSession) =>
      api.recordCashMovement({
        sessionId: active.id,
        movementType,
        category: movementCategory.trim(),
        amountMinor: toMinor(movementAmount),
        paymentMethod: movementMethod,
        evidence: movementEvidence.trim(),
      }),
    onSuccess: () => {
      setError(null);
      setMovementAmount("");
      setMovementCategory("");
      setMovementEvidence("");
      refresh();
    },
    onError: (cause) => {
      setError(cause instanceof Error ? cause.message : "No se pudo registrar el movimiento.");
    },
  });

  const close = useMutation({
    mutationFn: (active: CashSession) =>
      api.closeCashSession({
        sessionId: active.id,
        countedAmountMinor: toMinor(countedAmount),
        differenceReason: difference !== 0 ? differenceReason.trim() : null,
      }),
    onSuccess: (closedSession) => {
      setError(null);
      setClosed(closedSession);
      setDifferenceReason("");
      queryClient.setQueryData(["cash-session"], null);
      refresh();
    },
    onError: (cause) => {
      setError(cause instanceof Error ? cause.message : "No se pudo cerrar la caja.");
    },
  });

  const active = current.data ?? null;
  const openingValid = validMoney(openingAmount);
  const countedValid = validMoney(countedAmount);
  const movementValid =
    validPositiveMoney(movementAmount) &&
    movementCategory.trim().length > 0 &&
    movementEvidence.trim().length > 0;
  const expected = active?.expectedCashMinor ?? 0;
  const difference = countedValid ? toMinor(countedAmount) - expected : 0;
  const reasonValid = difference === 0 || differenceReason.trim().length > 0;
  const canClose = countedValid && reasonValid;

  return (
    <div className="space-y-6">
      <header className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <p className="text-sm text-slate-600">
            Abre la caja antes de vender y cierra el turno contando el efectivo.
          </p>
        </div>
        <span
          className={[
            "rounded-full px-3 py-1 text-xs font-medium",
            !canOperate
              ? "bg-slate-100 text-slate-600"
              : active
                ? "bg-emerald-50 text-emerald-700"
                : "bg-slate-100 text-slate-600",
          ].join(" ")}
        >
          {!canOperate ? "Solo lectura" : active ? "Turno abierto" : "Caja cerrada"}
        </span>
      </header>

      {!canOperate ? (
        <section className="rounded-2xl border border-slate-200 bg-white p-5 text-sm text-slate-600 shadow-sm">
          Tu rol puede consultar el historial de turnos. Abrir y cerrar la caja
          requiere el permiso de operación de mostrador.
        </section>
      ) : current.isLoading ? (
        <p className="text-sm text-slate-500">Consultando la caja…</p>
      ) : active ? (
        <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
          <div className="flex flex-wrap items-start justify-between gap-3">
            <div>
              <h2 className="text-sm font-semibold uppercase tracking-wide text-slate-500">
                Turno en curso
              </h2>
              <p className="mt-1 text-xs text-slate-500">
                Abierto por {active.openedByName ?? "usuario eliminado"} ·{" "}
                {formatDateTime(active.openedAt)}
              </p>
            </div>
            <span className="rounded-lg bg-slate-100 px-3 py-1 font-mono text-xs text-slate-600">
              {active.id.slice(0, 8)}
            </span>
          </div>

          <dl className="mt-5 grid gap-3 sm:grid-cols-2 lg:grid-cols-6">
            <Summary label="Fondo inicial" value={formatMinor(active.openingAmountMinor, active.currency)} />
            <Summary label="Ventas efectivo" value={formatMinor(active.cashSalesMinor, active.currency)} />
            <Summary label="Ingresos efectivo" value={formatMinor(active.cashIncomeMinor, active.currency)} />
            <Summary label="Gastos efectivo" value={formatMinor(active.cashExpenseMinor, active.currency)} />
            <Summary label="Devoluciones efectivo" value={formatMinor(active.cashRefundsMinor, active.currency)} />
            <Summary
              label="Esperado en caja"
              value={formatMinor(active.expectedCashMinor, active.currency)}
              strong
            />
          </dl>

          <p className="mt-3 text-xs text-slate-500">
            {active.salesCount} {active.salesCount === 1 ? "venta" : "ventas"},{" "}
            {active.movementsCount}{" "}
            {active.movementsCount === 1 ? "movimiento" : "movimientos"} y{" "}
            {active.refundsCount} {active.refundsCount === 1 ? "devolución" : "devoluciones"}{" "}
            en este turno.
          </p>

          <div className="mt-5 border-t border-slate-200 pt-4">
            <h3 className="text-sm font-semibold text-slate-700">Ingresos y gastos</h3>
            <p className="mt-1 text-xs text-slate-500">
              Solo los movimientos en efectivo alteran el esperado de la caja.
            </p>
            <div className="mt-3 grid gap-3 md:grid-cols-2 xl:grid-cols-5">
              <label className="text-sm font-medium text-slate-700">
                Tipo
                <select
                  value={movementType}
                  onChange={(event) =>
                    setMovementType(event.target.value as CashMovementType)
                  }
                  className={`${fieldClass} mt-1`}
                >
                  <option value="expense">Gasto</option>
                  <option value="income">Ingreso</option>
                </select>
              </label>
              <label className="text-sm font-medium text-slate-700">
                Monto
                <input
                  inputMode="decimal"
                  value={movementAmount}
                  onChange={(event) => setMovementAmount(event.target.value)}
                  className={`${fieldClass} mt-1`}
                />
              </label>
              <label className="text-sm font-medium text-slate-700">
                Categoría
                <input
                  value={movementCategory}
                  onChange={(event) => setMovementCategory(event.target.value)}
                  placeholder="Insumos, ajuste, servicio…"
                  className={`${fieldClass} mt-1`}
                />
              </label>
              <label className="text-sm font-medium text-slate-700">
                Método
                <select
                  value={movementMethod}
                  onChange={(event) =>
                    setMovementMethod(event.target.value as PaymentMethod)
                  }
                  className={`${fieldClass} mt-1`}
                >
                  <option value="cash">Efectivo</option>
                  <option value="transfer">Transferencia</option>
                  <option value="point">Point manual</option>
                </select>
              </label>
              <label className="text-sm font-medium text-slate-700">
                Evidencia
                <input
                  value={movementEvidence}
                  onChange={(event) => setMovementEvidence(event.target.value)}
                  placeholder="Recibo, nota o referencia"
                  className={`${fieldClass} mt-1`}
                />
              </label>
            </div>
            <button
              type="button"
              onClick={() => recordMovement.mutate(active)}
              disabled={recordMovement.isPending || !movementValid}
              className={`${secondaryButton} mt-3`}
            >
              {recordMovement.isPending ? "Registrando…" : "Registrar movimiento"}
            </button>

            {movements.isLoading ? (
              <p className="mt-4 text-sm text-slate-500">Cargando movimientos…</p>
            ) : (movements.data ?? []).length === 0 ? (
              <p className="mt-4 text-sm text-slate-500">
                Todavía no hay ingresos o gastos en este turno.
              </p>
            ) : (
              <div className="mt-4 overflow-x-auto">
                <table className="w-full text-left text-sm">
                  <thead className="border-b border-slate-200 text-xs uppercase tracking-wide text-slate-500">
                    <tr>
                      <th className="py-2 pr-3 font-medium">Tipo</th>
                      <th className="py-2 pr-3 font-medium">Categoría</th>
                      <th className="py-2 pr-3 font-medium">Método</th>
                      <th className="py-2 pr-3 font-medium">Evidencia</th>
                      <th className="py-2 pr-3 text-right font-medium">Monto</th>
                    </tr>
                  </thead>
                  <tbody className="divide-y divide-slate-100">
                    {(movements.data ?? []).map((movement) => (
                      <tr key={movement.id}>
                        <td className="py-2 pr-3">
                          <span
                            className={[
                              "rounded-full px-2 py-1 text-xs font-medium",
                              movement.movementType === "income"
                                ? "bg-emerald-50 text-emerald-700"
                                : "bg-amber-50 text-amber-700",
                            ].join(" ")}
                          >
                            {movement.movementType === "income" ? "Ingreso" : "Gasto"}
                          </span>
                        </td>
                        <td className="py-2 pr-3">{movement.category}</td>
                        <td className="py-2 pr-3">{paymentMethodLabel(movement.paymentMethod)}</td>
                        <td className="py-2 pr-3 text-slate-500">{movement.evidence}</td>
                        <td className="py-2 pr-3 text-right tabular-nums">
                          {movement.movementType === "income" ? "+" : "−"}
                          {formatMinor(movement.amountMinor, active.currency)}
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              </div>
            )}
          </div>

          <div className="mt-5 border-t border-slate-200 pt-4">
            <label className="block max-w-xs text-sm font-medium text-slate-700">
              Efectivo contado
              <input
                inputMode="decimal"
                value={countedAmount}
                onChange={(event) => setCountedAmount(event.target.value)}
                className={`${fieldClass} mt-1`}
              />
            </label>
            {countedValid ? (
              <p
                className={[
                  "mt-2 text-sm font-medium tabular-nums",
                  difference === 0
                    ? "text-emerald-700"
                    : "text-amber-700",
                ].join(" ")}
              >
                Diferencia: {formatMinor(difference, active.currency)}
              </p>
            ) : (
              <p className="mt-2 text-xs text-red-600">
                Ingresa un monto válido para cerrar el turno.
              </p>
            )}
            {difference !== 0 && countedValid && (
              <label className="mt-3 block max-w-xl text-sm font-medium text-slate-700">
                Motivo de la diferencia
                <textarea
                  value={differenceReason}
                  onChange={(event) => setDifferenceReason(event.target.value)}
                  rows={2}
                  placeholder="Explica el faltante o sobrante para dejarlo auditado"
                  className={`${fieldClass} mt-1`}
                />
              </label>
            )}
            {error && <p className="mt-2 text-sm text-red-600">{error}</p>}
            <button
              type="button"
              onClick={() => close.mutate(active)}
              disabled={close.isPending || !canClose}
              className={`${primaryButton} mt-3`}
            >
              {close.isPending ? "Cerrando…" : "Cerrar caja"}
            </button>
          </div>
        </section>
      ) : (
        <div className="grid gap-6 lg:grid-cols-2">
          <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
            <h2 className="text-sm font-semibold uppercase tracking-wide text-slate-500">
              Abrir turno
            </h2>
            <p className="mt-1 text-sm text-slate-600">
              Registra el efectivo con el que inicia la caja. Puede ser cero.
            </p>
            <label className="mt-4 block max-w-xs text-sm font-medium text-slate-700">
              Monto inicial
              <input
                inputMode="decimal"
                value={openingAmount}
                onChange={(event) => setOpeningAmount(event.target.value)}
                className={`${fieldClass} mt-1`}
              />
            </label>
            {error && <p className="mt-2 text-sm text-red-600">{error}</p>}
            <button
              type="button"
              onClick={() => open.mutate()}
              disabled={open.isPending || !openingValid}
              className={`${primaryButton} mt-3`}
            >
              {open.isPending ? "Abriendo…" : "Abrir caja"}
            </button>
          </section>

          {closed && <ClosedSummary session={closed} />}
        </div>
      )}

      {canViewHistory && (
        <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
          <h2 className="text-sm font-semibold uppercase tracking-wide text-slate-500">
            Turnos recientes
          </h2>
          {history.isLoading ? (
            <p className="mt-4 text-sm text-slate-500">Cargando turnos…</p>
          ) : (history.data ?? []).length === 0 ? (
            <p className="mt-4 text-sm text-slate-500">Todavía no hay turnos registrados.</p>
          ) : (
            <div className="mt-3 overflow-x-auto">
              <table className="w-full text-left text-sm">
                <thead className="border-b border-slate-200 text-xs uppercase tracking-wide text-slate-500">
                  <tr>
                    <th className="py-2 pr-3 font-medium">Apertura</th>
                    <th className="py-2 pr-3 font-medium">Cierre</th>
                    <th className="py-2 pr-3 text-right font-medium">Ingresos</th>
                    <th className="py-2 pr-3 text-right font-medium">Gastos</th>
                    <th className="py-2 pr-3 text-right font-medium">Devoluciones</th>
                    <th className="py-2 pr-3 text-right font-medium">Esperado</th>
                    <th className="py-2 pr-3 text-right font-medium">Contado</th>
                    <th className="py-2 pr-3 text-right font-medium">Diferencia</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100">
                  {(history.data ?? []).map((item) => (
                    <tr key={item.id}>
                      <td className="py-3 pr-3">
                        <span className="block">{formatDateTime(item.openedAt)}</span>
                        <span className="block text-xs text-slate-500">
                          {item.openedByName ?? "—"}
                        </span>
                      </td>
                      <td className="py-3 pr-3">
                        {item.closedAt ? (
                          <>
                            <span className="block">{formatDateTime(item.closedAt)}</span>
                            <span className="block text-xs text-slate-500">
                              {item.closedByName ?? "—"}
                            </span>
                          </>
                        ) : (
                          <span className="rounded-full bg-emerald-50 px-2 py-1 text-xs font-medium text-emerald-700">
                            Abierto
                          </span>
                        )}
                      </td>
                      <td className="py-3 pr-3 text-right tabular-nums">
                        {formatMinor(item.incomeMinor, item.currency)}
                      </td>
                      <td className="py-3 pr-3 text-right tabular-nums">
                        {formatMinor(item.expenseMinor, item.currency)}
                      </td>
                      <td className="py-3 pr-3 text-right tabular-nums">
                        {formatMinor(item.refundedMinor, item.currency)}
                      </td>
                      <td className="py-3 pr-3 text-right tabular-nums">
                        {formatMinor(item.expectedCashMinor, item.currency)}
                      </td>
                      <td className="py-3 pr-3 text-right tabular-nums">
                        {item.countedAmountMinor == null
                          ? "—"
                          : formatMinor(item.countedAmountMinor, item.currency)}
                      </td>
                      <td
                        className={[
                          "py-3 pr-3 text-right font-medium tabular-nums",
                          item.differenceMinor === 0
                            ? "text-emerald-700"
                            : item.differenceMinor == null
                              ? "text-slate-400"
                              : "text-amber-700",
                        ].join(" ")}
                      >
                        {item.differenceMinor == null
                          ? "—"
                          : formatMinor(item.differenceMinor, item.currency)}
                        {item.differenceReason && (
                          <span className="block max-w-48 text-xs font-normal text-slate-500">
                            {item.differenceReason}
                          </span>
                        )}
                      </td>
                    </tr>
                  ))}
                </tbody>
              </table>
            </div>
          )}
        </section>
      )}
    </div>
  );
}

function ClosedSummary({ session }: { session: CashSession }) {
  const difference = session.differenceMinor ?? 0;
  return (
    <section className="rounded-2xl border border-emerald-200 bg-emerald-50 p-5">
      <h2 className="text-sm font-semibold uppercase tracking-wide text-emerald-800">
        Turno cerrado
      </h2>
      <p className="mt-1 text-sm text-emerald-900">
        El cierre quedó registrado y ya puedes abrir otro turno.
      </p>
      <dl className="mt-4 space-y-2 text-sm text-emerald-950">
        <div className="flex justify-between gap-3">
          <dt>Fondo inicial</dt>
          <dd className="tabular-nums">
            {formatMinor(session.openingAmountMinor, session.currency)}
          </dd>
        </div>
        <div className="flex justify-between gap-3">
          <dt>Ingresos</dt>
          <dd className="tabular-nums">
            {formatMinor(session.incomeMinor, session.currency)}
          </dd>
        </div>
        <div className="flex justify-between gap-3">
          <dt>Gastos</dt>
          <dd className="tabular-nums">
            {formatMinor(session.expenseMinor, session.currency)}
          </dd>
        </div>
        <div className="flex justify-between gap-3">
          <dt>Devoluciones</dt>
          <dd className="tabular-nums">
            {formatMinor(session.refundedMinor, session.currency)}
          </dd>
        </div>
        <div className="flex justify-between gap-3">
          <dt>Esperado en caja</dt>
          <dd className="tabular-nums">
            {formatMinor(session.expectedCashMinor, session.currency)}
          </dd>
        </div>
        <div className="flex justify-between gap-3">
          <dt>Efectivo contado</dt>
          <dd className="tabular-nums">
            {formatMinor(session.countedAmountMinor ?? 0, session.currency)}
          </dd>
        </div>
        <div className="flex justify-between gap-3 border-t border-emerald-200 pt-2 font-semibold">
          <dt>Diferencia</dt>
          <dd className="tabular-nums">{formatMinor(difference, session.currency)}</dd>
        </div>
        {session.differenceReason && (
          <div className="border-t border-emerald-200 pt-2">
            <dt className="text-xs font-medium uppercase tracking-wide text-emerald-800">
              Motivo
            </dt>
            <dd className="mt-1">{session.differenceReason}</dd>
          </div>
        )}
      </dl>
    </section>
  );
}

function Summary({
  label,
  value,
  strong = false,
}: {
  label: string;
  value: string;
  strong?: boolean;
}) {
  return (
    <div className="rounded-xl bg-slate-50 px-4 py-3">
      <dt className="text-xs font-medium uppercase tracking-wide text-slate-500">{label}</dt>
      <dd className={strong ? "mt-1 text-lg font-semibold tabular-nums" : "mt-1 text-lg tabular-nums"}>
        {value}
      </dd>
    </div>
  );
}

function validMoney(value: string): boolean {
  if (!value.trim()) {
    return false;
  }
  const amount = Number(value);
  return Number.isFinite(amount) && amount >= 0;
}

function validPositiveMoney(value: string): boolean {
  if (!value.trim()) {
    return false;
  }
  const amount = Number(value);
  return Number.isFinite(amount) && amount > 0;
}

function toMinor(value: string): number {
  return Math.round(Number(value) * 100);
}

function formatMinor(amount: number, currency: string): string {
  return new Intl.NumberFormat("es-MX", {
    style: "currency",
    currency,
  }).format(amount / 100);
}

function formatDateTime(value: string): string {
  return new Date(value).toLocaleString("es-MX", {
    dateStyle: "short",
    timeStyle: "short",
  });
}

function paymentMethodLabel(method: PaymentMethod): string {
  switch (method) {
    case "cash":
      return "Efectivo";
    case "transfer":
      return "Transferencia";
    case "point":
      return "Point manual";
  }
}
