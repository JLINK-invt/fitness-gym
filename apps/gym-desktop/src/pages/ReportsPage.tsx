import type {
  FinancialReport,
  FinancialReportExport,
  InventoryAdjustmentTotal,
  PaymentMethod,
  PaymentMethodTotal,
  ProductSalesTotal,
  ReportRun,
  ReportSchedule,
} from "@gym/contracts";
import { errorMessage } from "@gym/contracts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useOutletContext } from "react-router-dom";

import type { LayoutContext } from "../components/Layout";
import { formatDateTime } from "../lib/format";
import { api } from "../lib/tauri";

const paymentLabels: Record<PaymentMethod, string> = {
  cash: "Efectivo",
  transfer: "Transferencia",
  point: "Terminal Point",
};

const fieldClass =
  "w-full rounded-lg border border-slate-300 px-3 py-2 text-sm outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100";
const primaryButton =
  "rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-700 disabled:opacity-50";

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

export function ReportsPage() {
  const queryClient = useQueryClient();
  const { session } = useOutletContext<LayoutContext>();
  const canExport = session.permissions.includes("exportReports");
  const today = new Date();
  const monthAgo = new Date(today.getTime() - 29 * 24 * 60 * 60 * 1_000);
  const [fromDate, setFromDate] = useState(inputDate(monthAgo));
  const [toDate, setToDate] = useState(inputDate(today));
  const [exported, setExported] = useState<FinancialReportExport | null>(null);
  const rangeValid = fromDate !== "" && toDate !== "" && fromDate <= toDate;

  const report = useQuery<FinancialReport>({
    queryKey: ["financial-report", fromDate, toDate],
    queryFn: () =>
      api.financialReport({ from: dayStart(fromDate), to: dayEnd(toDate) }),
    enabled: rangeValid,
  });

  const exportCsv = useMutation({
    mutationFn: () =>
      api.exportReportCsv({
        from: dayStart(fromDate),
        to: dayEnd(toDate),
      }),
    onSuccess: (result) => {
      setExported(result);
      void queryClient.invalidateQueries({ queryKey: ["audit-events"] });
    },
    onError: () => setExported(null),
  });

  const data = report.data;

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <p className="text-sm text-slate-600">
            Cierre de ventas, métodos de pago, margen por producto, ajustes de
            inventario y diferencia de caja. El CSV incluye la razón social y el
            RFC configurados en General. Cada exportación queda auditada.
          </p>
        </div>
        {canExport ? (
          <button
            type="button"
            onClick={() => exportCsv.mutate()}
            disabled={!rangeValid || report.isLoading || exportCsv.isPending}
            className={primaryButton}
          >
            {exportCsv.isPending ? "Exportando…" : "Exportar CSV"}
          </button>
        ) : (
          <p className="rounded-lg border border-slate-200 bg-white px-3 py-2 text-xs text-slate-500">
            Puedes consultar los reportes, pero tu rol no permite exportarlos.
          </p>
        )}
      </div>

      <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
        <div className="flex flex-wrap items-end gap-3">
          <label className="text-sm">
            <span className="font-medium text-slate-700">Desde</span>
            <input
              type="date"
              value={fromDate}
              max={toDate}
              onChange={(event) => {
                setFromDate(event.target.value);
                setExported(null);
              }}
              className={`mt-1 ${fieldClass}`}
            />
          </label>
          <label className="text-sm">
            <span className="font-medium text-slate-700">Hasta</span>
            <input
              type="date"
              value={toDate}
              min={fromDate}
              onChange={(event) => {
                setToDate(event.target.value);
                setExported(null);
              }}
              className={`mt-1 ${fieldClass}`}
            />
          </label>
          <button
            type="button"
            onClick={() => void report.refetch()}
            disabled={!rangeValid || report.isFetching}
            className="rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium text-slate-700 hover:bg-slate-100 disabled:opacity-50"
          >
            {report.isFetching ? "Actualizando…" : "Actualizar"}
          </button>
          {data && (
            <p className="ml-auto text-xs text-slate-500">
              Generado {formatDateTime(data.generatedAt)} · Moneda {data.currency}
            </p>
          )}
        </div>
        {!rangeValid && (
          <p className="mt-3 text-sm text-rose-700">
            Elige un rango válido: la fecha inicial no puede ser posterior a la final.
          </p>
        )}
      </section>

      {exported && (
        <p className="rounded-xl border border-emerald-200 bg-emerald-50 px-4 py-3 text-sm text-emerald-800">
          Exportado {exported.fileName} ({formatBytes(exported.bytes)}) en{" "}
          <span className="font-mono text-xs">{exported.path}</span>
        </p>
      )}
      {exportCsv.isError && (
        <p className="rounded-xl border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-700">
          {errorMessage(exportCsv.error)}
        </p>
      )}
      {report.isError && (
        <p className="rounded-xl border border-rose-200 bg-rose-50 px-4 py-3 text-sm text-rose-700">
          {errorMessage(report.error)}
        </p>
      )}

      {report.isLoading && rangeValid ? (
        <p className="rounded-xl border border-slate-200 bg-white p-4 text-sm text-slate-500">
          Calculando el reporte…
        </p>
      ) : data ? (
        <>
          <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
            <Summary label="Venta neta" value={formatMinor(data.netSalesMinor, data.currency)} />
            <Summary label="Venta bruta" value={formatMinor(data.grossSalesMinor, data.currency)} />
            <Summary label="Devoluciones" value={formatMinor(data.refundedMinor, data.currency)} />
            <Summary label="Ticket promedio" value={formatMinor(data.averageSaleMinor, data.currency)} />
            <Summary label="Ventas" value={String(data.salesCount)} />
            <Summary label="Impuestos" value={formatMinor(data.taxMinor, data.currency)} />
            <Summary
              label="Diferencia de caja"
              value={formatMinor(data.cashDifferenceMinor, data.currency)}
              tone={data.cashDifferenceMinor === 0 ? "ok" : "warn"}
            />
            <Summary
              label="Turnos cerrados"
              value={`${data.closedCashSessions} de ${data.cashSessionsCount}`}
              tone={data.openCashSessions > 0 ? "warn" : "neutral"}
            />
          </div>

          <Analytics data={data} />
          <PaymentMethods data={data.paymentMethods} currency={data.currency} />
          <ProductSales data={data.productSales} currency={data.currency} />
          <Adjustments data={data.inventoryAdjustments} currency={data.currency} />
        </>
      ) : null}

      <ScheduledReports canExport={canExport} />
    </div>
  );
}

function Analytics({ data }: { data: FinancialReport }) {
  return (
    <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
      <div>
        <h2 className="text-base font-semibold">Analítica de ventas e inventario</h2>
        <p className="mt-1 text-xs text-slate-500">
          Ticket medio, margen, attach rate, rotación y ajustes del periodo,
          calculados con el ledger y los movimientos de inventario.
        </p>
      </div>
      <div className="mt-4 grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
        <Summary
          label="Margen"
          value={formatMinor(data.marginMinor, data.currency)}
          tone={data.marginMinor < 0 ? "warn" : "neutral"}
        />
        <Summary label="Margen %" value={formatPercentBps(data.marginBps)} />
        <Summary label="Costo de ventas" value={formatMinor(data.costOfGoodsMinor, data.currency)} />
        <Summary label="Attach rate" value={formatPercentBps(data.attachRateBps)} />
        <Summary
          label="Artículos por venta"
          value={formatDecimalMilli(data.itemsPerSaleMilli)}
        />
        <Summary label="Rotación" value={formatTurnover(data.inventoryTurnoverBps)} />
        <Summary
          label="Inventario (inicio → fin)"
          value={`${formatMinor(data.inventoryValueStartMinor, data.currency)} → ${formatMinor(data.inventoryValueEndMinor, data.currency)}`}
        />
        <Summary
          label="Ajustes"
          value={`${data.adjustmentsUnits} u · ${formatMinor(data.adjustmentsCostMinor, data.currency)}`}
          tone={data.adjustmentsUnits < 0 ? "warn" : "neutral"}
        />
      </div>
    </section>
  );
}

function PaymentMethods({
  data,
  currency,
}: {
  data: PaymentMethodTotal[];
  currency: string;
}) {
  return (
    <section className="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm">
      <div className="border-b border-slate-200 px-6 py-4">
        <h2 className="text-base font-semibold">Métodos de pago</h2>
        <p className="mt-1 text-xs text-slate-500">
          Cargos y reembolsos registrados en el ledger, conciliados con las ventas.
        </p>
      </div>
      <div className="overflow-x-auto">
        <table className="min-w-full divide-y divide-slate-200 text-sm">
          <thead className="bg-slate-50 text-left text-xs uppercase tracking-wide text-slate-500">
            <tr>
              <th className="px-4 py-3 font-medium">Método</th>
              <th className="px-4 py-3 text-right font-medium">Ventas</th>
              <th className="px-4 py-3 text-right font-medium">Bruto</th>
              <th className="px-4 py-3 text-right font-medium">Devoluciones</th>
              <th className="px-4 py-3 text-right font-medium">Neto</th>
            </tr>
          </thead>
          <tbody className="divide-y divide-slate-100">
            {data.map((method) => (
              <tr key={method.paymentMethod}>
                <td className="px-4 py-3">{paymentLabels[method.paymentMethod]}</td>
                <td className="px-4 py-3 text-right">{method.salesCount}</td>
                <td className="px-4 py-3 text-right">
                  {formatMinor(method.grossSalesMinor, currency)}
                </td>
                <td className="px-4 py-3 text-right">
                  {method.refundsCount} · {formatMinor(method.refundedMinor, currency)}
                </td>
                <td className="px-4 py-3 text-right font-medium">
                  {formatMinor(method.netSalesMinor, currency)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
      </div>
    </section>
  );
}

function ProductSales({
  data,
  currency,
}: {
  data: ProductSalesTotal[];
  currency: string;
}) {
  return (
    <section className="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm">
      <div className="border-b border-slate-200 px-6 py-4">
        <h2 className="text-base font-semibold">Ventas por producto</h2>
        <p className="mt-1 text-xs text-slate-500">
          Ordenados por venta neta (top primero). El margen usa el costo de los
          movimientos de inventario; los servicios usan costo cero.
        </p>
      </div>
      {data.length === 0 ? (
        <p className="px-6 py-8 text-sm text-slate-500">Sin ventas en el rango.</p>
      ) : (
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-slate-200 text-sm">
            <thead className="bg-slate-50 text-left text-xs uppercase tracking-wide text-slate-500">
              <tr>
                <th className="px-4 py-3 font-medium">Producto</th>
                <th className="px-4 py-3 text-right font-medium">Vendidas</th>
                <th className="px-4 py-3 text-right font-medium">Devueltas</th>
                <th className="px-4 py-3 text-right font-medium">Netas</th>
                <th className="px-4 py-3 text-right font-medium">Venta neta</th>
                <th className="px-4 py-3 text-right font-medium">Costo</th>
                <th className="px-4 py-3 text-right font-medium">Margen</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {data.map((product) => (
                <tr key={product.variantId}>
                  <td className="px-4 py-3">
                    <p className="font-medium">{product.productName}</p>
                    <p className="text-xs text-slate-500">
                      {product.variantName} · {product.sku}
                    </p>
                  </td>
                  <td className="px-4 py-3 text-right">{product.soldUnits}</td>
                  <td className="px-4 py-3 text-right">{product.refundedUnits}</td>
                  <td className="px-4 py-3 text-right">{product.netUnits}</td>
                  <td className="px-4 py-3 text-right">
                    {formatMinor(product.netSalesMinor, currency)}
                  </td>
                  <td className="px-4 py-3 text-right">
                    {formatMinor(product.costMinor, currency)}
                  </td>
                  <td className="px-4 py-3 text-right font-medium">
                    {formatMinor(product.marginMinor, currency)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}

function Adjustments({
  data,
  currency,
}: {
  data: InventoryAdjustmentTotal[];
  currency: string;
}) {
  return (
    <section className="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm">
      <div className="border-b border-slate-200 px-6 py-4">
        <h2 className="text-base font-semibold">Ajustes de inventario</h2>
        <p className="mt-1 text-xs text-slate-500">
          Impacto de mermas y correcciones en unidades y costo.
        </p>
      </div>
      {data.length === 0 ? (
        <p className="px-6 py-8 text-sm text-slate-500">Sin ajustes en el rango.</p>
      ) : (
        <div className="overflow-x-auto">
          <table className="min-w-full divide-y divide-slate-200 text-sm">
            <thead className="bg-slate-50 text-left text-xs uppercase tracking-wide text-slate-500">
              <tr>
                <th className="px-4 py-3 font-medium">Producto</th>
                <th className="px-4 py-3 text-right font-medium">Unidades</th>
                <th className="px-4 py-3 text-right font-medium">Costo</th>
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {data.map((adjustment) => (
                <tr key={adjustment.variantId}>
                  <td className="px-4 py-3">
                    <p className="font-medium">{adjustment.productName}</p>
                    <p className="text-xs text-slate-500">
                      {adjustment.variantName} · {adjustment.sku}
                    </p>
                  </td>
                  <td className="px-4 py-3 text-right">{adjustment.unitsDelta}</td>
                  <td className="px-4 py-3 text-right">
                    {formatMinor(adjustment.costMinor, currency)}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </section>
  );
}

function ScheduledReports({ canExport }: { canExport: boolean }) {
  const queryClient = useQueryClient();
  const [cadence, setCadence] =
    useState<"daily" | "weekly" | "monthly">("weekly");
  const [lookbackDays, setLookbackDays] = useState("7");

  const schedules = useQuery<ReportSchedule[]>({
    queryKey: ["report-schedules"],
    queryFn: api.listReportSchedules,
  });
  const runs = useQuery<ReportRun[]>({
    queryKey: ["report-runs"],
    queryFn: () => api.listReportRuns(20),
  });

  const invalidate = async () => {
    await Promise.all([
      queryClient.invalidateQueries({ queryKey: ["report-schedules"] }),
      queryClient.invalidateQueries({ queryKey: ["report-runs"] }),
    ]);
  };

  const create = useMutation({
    mutationFn: () =>
      api.createReportSchedule({
        cadence,
        lookbackDays: Math.max(1, Number(lookbackDays) || 1),
      }),
    onSuccess: invalidate,
  });
  const toggle = useMutation({
    mutationFn: (schedule: ReportSchedule) =>
      api.setReportScheduleEnabled({
        scheduleId: schedule.id,
        enabled: !schedule.enabled,
      }),
    onSuccess: invalidate,
  });
  const runDue = useMutation({
    mutationFn: api.runDueReportSchedules,
    onSuccess: invalidate,
  });

  return (
    <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
      <div className="flex flex-wrap items-start justify-between gap-3">
        <div>
          <h2 className="text-base font-semibold">Reportes programados</h2>
          <p className="mt-1 text-xs text-slate-500">
            Genera el CSV financiero automáticamente y conserva el checksum de
            cada ejecución.
          </p>
        </div>
        {canExport ? (
          <button
            type="button"
            onClick={() => runDue.mutate()}
            disabled={runDue.isPending}
            className={primaryButton}
          >
            {runDue.isPending ? "Ejecutando…" : "Ejecutar pendientes"}
          </button>
        ) : (
          <p className="rounded-lg border border-slate-200 bg-white px-3 py-2 text-xs text-slate-500">
            Programar y ejecutar reportes requiere permiso de exportación.
          </p>
        )}
      </div>

      <form
        className="mt-4 grid gap-3 sm:grid-cols-[1fr_1fr_auto]"
        onSubmit={(event) => {
          event.preventDefault();
          create.mutate();
        }}
      >
        <label className="text-sm">
          <span className="font-medium text-slate-700">Cadencia</span>
          <select
            value={cadence}
            onChange={(event) =>
              setCadence(event.target.value as typeof cadence)
            }
            className={`mt-1 ${fieldClass}`}
          >
            <option value="daily">Diaria</option>
            <option value="weekly">Semanal</option>
            <option value="monthly">Mensual</option>
          </select>
        </label>
        <label className="text-sm">
          <span className="font-medium text-slate-700">Días hacia atrás</span>
          <input
            type="number"
            min={1}
            max={366}
            value={lookbackDays}
            onChange={(event) => setLookbackDays(event.target.value)}
            className={`mt-1 ${fieldClass}`}
          />
        </label>
        <div className="flex items-end">
          <button
            type="submit"
            disabled={create.isPending || !canExport}
            className={primaryButton}
          >
            {create.isPending ? "Creando…" : "Programar"}
          </button>
        </div>
      </form>

      {(create.isError || toggle.isError || runDue.isError) && (
        <p className="mt-3 text-sm text-rose-700">
          {errorMessage(create.error ?? toggle.error ?? runDue.error)}
        </p>
      )}

      <div className="mt-4 grid gap-4 lg:grid-cols-2">
        <div>
          <h3 className="text-sm font-semibold">Programaciones</h3>
          {schedules.isLoading ? (
            <p className="mt-2 text-sm text-slate-500">Cargando…</p>
          ) : schedules.data && schedules.data.length > 0 ? (
            <div className="mt-2 space-y-2">
              {schedules.data.map((schedule) => (
                <div
                  key={schedule.id}
                  className="flex flex-wrap items-center justify-between gap-2 rounded-xl border border-slate-200 px-3 py-2 text-xs"
                >
                  <div>
                    <p className="font-medium">
                      {cadenceLabels[schedule.cadence] ?? schedule.cadence} ·{" "}
                      {schedule.lookbackDays} días
                    </p>
                    <p className="text-slate-500">
                      Próxima {formatDateTime(schedule.nextRunAt)}
                      {schedule.lastRunAt
                        ? ` · última ${formatDateTime(schedule.lastRunAt)}`
                        : ""}
                    </p>
                  </div>
                  <button
                    type="button"
                    onClick={() => toggle.mutate(schedule)}
                    disabled={toggle.isPending || !canExport}
                    className="rounded-lg border border-slate-300 px-2.5 py-1.5 font-medium text-slate-700 hover:bg-slate-50 disabled:opacity-50"
                  >
                    {schedule.enabled ? "Desactivar" : "Activar"}
                  </button>
                </div>
              ))}
            </div>
          ) : (
            <p className="mt-2 text-sm text-slate-500">
              No hay programaciones.
            </p>
          )}
        </div>

        <div>
          <h3 className="text-sm font-semibold">Últimas ejecuciones</h3>
          {runs.isLoading ? (
            <p className="mt-2 text-sm text-slate-500">Cargando…</p>
          ) : runs.data && runs.data.length > 0 ? (
            <div className="mt-2 space-y-2">
              {runs.data.map((run) => (
                <div
                  key={run.id}
                  className="rounded-xl border border-slate-200 px-3 py-2 text-xs"
                >
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <span className="font-medium">
                      {run.periodFrom.slice(0, 10)} →{" "}
                      {run.periodTo.slice(0, 10)}
                    </span>
                    <span
                      className={
                        run.status === "completed"
                          ? "text-emerald-700"
                          : "text-rose-700"
                      }
                    >
                      {run.status}
                    </span>
                  </div>
                  <p className="mt-1 text-slate-500">
                    {run.fileName ?? "Sin archivo"} ·{" "}
                    {shortChecksum(run.checksum)}
                  </p>
                  {run.error && (
                    <p className="mt-1 text-rose-700">{run.error}</p>
                  )}
                </div>
              ))}
            </div>
          ) : (
            <p className="mt-2 text-sm text-slate-500">Sin ejecuciones.</p>
          )}
        </div>
      </div>
    </section>
  );
}

const cadenceLabels: Record<string, string> = {
  daily: "Diaria",
  weekly: "Semanal",
  monthly: "Mensual",
};

function shortChecksum(value: string): string {
  return value.length > 16 ? `${value.slice(0, 16)}…` : value;
}

function Summary({
  label,
  value,
  tone = "neutral",
}: {
  label: string;
  value: string;
  tone?: "neutral" | "ok" | "warn";
}) {
  return (
    <div
      className={[
        "rounded-2xl border p-4 shadow-sm",
        tone === "warn"
          ? "border-amber-200 bg-amber-50"
          : tone === "ok"
            ? "border-emerald-200 bg-emerald-50"
            : "border-slate-200 bg-white",
      ].join(" ")}
    >
      <p className="text-xs font-medium uppercase tracking-wide text-slate-500">{label}</p>
      <p className="mt-1 text-lg font-semibold tracking-tight">{value}</p>
    </div>
  );
}

function formatMinor(amount: number, currency: string): string {
  return new Intl.NumberFormat("es-MX", {
    style: "currency",
    currency,
  }).format(amount / 100);
}

function formatPercentBps(bps: number): string {
  return `${(bps / 100).toFixed(1)}%`;
}

function formatDecimalMilli(value: number): string {
  return (value / 1_000).toFixed(2);
}

function formatTurnover(bps: number): string {
  return `${(bps / 10_000).toFixed(2)}×`;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  return `${(bytes / 1024).toFixed(1)} KB`;
}
