import type {
  FinancialReport,
  FinancialReportExport,
  InventoryAdjustmentTotal,
  PaymentMethod,
  PaymentMethodTotal,
  ProductSalesTotal,
} from "@gym/contracts";
import { errorMessage } from "@gym/contracts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";

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
      api.exportFinancialReportCsv({
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
          <h1 className="text-xl font-semibold tracking-tight">Reportes</h1>
          <p className="mt-1 text-sm text-slate-600">
            Cierre de ventas, métodos de pago, margen por producto, ajustes de
            inventario y diferencia de caja. Cada exportación queda auditada.
          </p>
        </div>
        <button
          type="button"
          onClick={() => exportCsv.mutate()}
          disabled={!rangeValid || report.isLoading || exportCsv.isPending}
          className={primaryButton}
        >
          {exportCsv.isPending ? "Exportando…" : "Exportar CSV"}
        </button>
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

          <PaymentMethods data={data.paymentMethods} currency={data.currency} />
          <ProductSales data={data.productSales} currency={data.currency} />
          <Adjustments data={data.inventoryAdjustments} currency={data.currency} />
        </>
      ) : null}
    </div>
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
          Margen calculado con el costo de los movimientos de inventario; los
          servicios usan costo cero.
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

function formatBytes(bytes: number): string {
  if (bytes < 1024) {
    return `${bytes} B`;
  }
  return `${(bytes / 1024).toFixed(1)} KB`;
}
