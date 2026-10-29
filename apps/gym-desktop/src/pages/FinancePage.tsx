import type { FinancialReport, PaymentMethod } from "@gym/contracts";
import { errorMessage } from "@gym/contracts";
import { useMutation, useQuery } from "@tanstack/react-query";
import { useState } from "react";
import { Link, useOutletContext } from "react-router-dom";

import type { LayoutContext } from "../components/Layout";
import { formatDateTime } from "../lib/format";
import { api } from "../lib/tauri";

const fieldClass =
  "w-full rounded-lg border border-slate-300 px-3 py-2 text-sm outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100";
const primaryButton =
  "rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-700 disabled:opacity-50";

const paymentLabels: Record<PaymentMethod, string> = {
  cash: "Efectivo",
  transfer: "Transferencia",
  point: "Terminal Point",
};

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

function formatMinor(amount: number, currency: string): string {
  return new Intl.NumberFormat("es-MX", {
    style: "currency",
    currency,
  }).format(amount / 100);
}

export function FinancePage() {
  const { session } = useOutletContext<LayoutContext>();
  const canExport = session.permissions.includes("exportFinance");
  const now = new Date();
  const [from, setFrom] = useState(inputDate(new Date(now.getFullYear(), now.getMonth(), 1)));
  const [to, setTo] = useState(inputDate(now));
  const [exported, setExported] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);

  const report = useQuery<FinancialReport>({
    queryKey: ["finance-report", from, to],
    queryFn: () =>
      api.financialReport({ from: dayStart(from), to: dayEnd(to) }),
    enabled: Boolean(from && to),
  });

  const exportCsv = useMutation({
    mutationFn: () => api.exportFinancialReportCsv({ from: dayStart(from), to: dayEnd(to) }),
    onSuccess: (result) => {
      setError(null);
      setExported(result.path);
    },
    onError: (cause) => setError(errorMessage(cause)),
  });

  const data = report.data;
  const currency = data?.currency ?? "MXN";

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-end justify-between gap-3">
        <p className="text-sm text-slate-600">
          Libro financiero del periodo: ventas, devoluciones, margen y cobros por método.
          El detalle operativo está en{" "}
          <Link className="font-semibold text-brand-700 underline" to="/reports">
            Reportes
          </Link>
          .
        </p>
        <div className="flex flex-wrap items-end gap-2">
          <label className="text-xs font-medium text-slate-600">
            Desde
            <input
              type="date"
              className={`${fieldClass} mt-1`}
              value={from}
              max={to}
              onChange={(event) => setFrom(event.target.value)}
            />
          </label>
          <label className="text-xs font-medium text-slate-600">
            Hasta
            <input
              type="date"
              className={`${fieldClass} mt-1`}
              value={to}
              min={from}
              onChange={(event) => setTo(event.target.value)}
            />
          </label>
          {canExport && (
            <button
              type="button"
              className={primaryButton}
              disabled={exportCsv.isPending}
              onClick={() => exportCsv.mutate()}
            >
              {exportCsv.isPending ? "Exportando…" : "Exportar CSV"}
            </button>
          )}
        </div>
      </div>

      {report.isError && (
        <div className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
          {errorMessage(report.error)}
        </div>
      )}
      {error && (
        <div className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
          {error}
        </div>
      )}
      {exported && (
        <div className="rounded-xl border border-emerald-200 bg-emerald-50 p-4 text-sm text-emerald-700">
          Exportado en {exported}
        </div>
      )}

      {data && (
        <>
          <div className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
            <Metric label="Venta bruta" value={formatMinor(data.grossSalesMinor, currency)} />
            <Metric label="Venta neta" value={formatMinor(data.netSalesMinor, currency)} />
            <Metric label="Devoluciones" value={formatMinor(data.refundedMinor, currency)} />
            <Metric label="Margen" value={formatMinor(data.marginMinor, currency)} />
            <Metric label="Ticket promedio" value={formatMinor(data.averageSaleMinor, currency)} />
            <Metric label="Impuestos" value={formatMinor(data.taxMinor, currency)} />
            <Metric
              label="Diferencia de caja"
              value={formatMinor(data.cashDifferenceMinor, currency)}
            />
            <Metric
              label="Costo de ventas"
              value={formatMinor(data.costOfGoodsMinor, currency)}
            />
          </div>

          <section className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
            <h2 className="text-sm font-semibold text-slate-900">Cobros por método</h2>
            <div className="mt-4 overflow-x-auto">
              <table className="min-w-full text-sm">
                <thead>
                  <tr className="text-left text-xs uppercase tracking-wide text-slate-500">
                    <th className="py-2 pr-4">Método</th>
                    <th className="py-2 pr-4">Ventas</th>
                    <th className="py-2 pr-4">Bruto</th>
                    <th className="py-2 pr-4">Devoluciones</th>
                    <th className="py-2">Neto</th>
                  </tr>
                </thead>
                <tbody className="divide-y divide-slate-100">
                  {data.paymentMethods.map((row) => (
                    <tr key={row.paymentMethod}>
                      <td className="py-2 pr-4 font-medium text-slate-900">
                        {paymentLabels[row.paymentMethod] ?? row.paymentMethod}
                      </td>
                      <td className="py-2 pr-4 text-slate-600">{row.salesCount}</td>
                      <td className="py-2 pr-4 text-slate-600">
                        {formatMinor(row.grossSalesMinor, currency)}
                      </td>
                      <td className="py-2 pr-4 text-slate-600">
                        {formatMinor(row.refundedMinor, currency)}
                      </td>
                      <td className="py-2 font-semibold text-slate-900">
                        {formatMinor(row.netSalesMinor, currency)}
                      </td>
                    </tr>
                  ))}
                  {data.paymentMethods.length === 0 && (
                    <tr>
                      <td className="py-4 text-sm text-slate-500" colSpan={5}>
                        Sin cobros en el periodo.
                      </td>
                    </tr>
                  )}
                </tbody>
              </table>
            </div>
            <p className="mt-4 text-xs text-slate-500">
              Generado {formatDateTime(data.generatedAt)} · {data.salesCount} ventas
            </p>
          </section>
        </>
      )}
    </div>
  );
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="rounded-2xl border border-slate-200 bg-white p-4 shadow-sm">
      <p className="text-xs font-medium uppercase tracking-wide text-slate-500">{label}</p>
      <p className="mt-1 text-xl font-semibold text-slate-900">{value}</p>
    </div>
  );
}
