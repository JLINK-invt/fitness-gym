import type { InventoryItem, InventoryMovement } from "@gym/contracts";
import { errorMessage } from "@gym/contracts";
import { useQuery } from "@tanstack/react-query";
import { useMemo } from "react";
import { Link, useOutletContext } from "react-router-dom";

import type { LayoutContext } from "../components/Layout";
import { api } from "../lib/tauri";

const ROTATION_DAYS = 30;
const ROTATION_LIMIT = 200;

export function DashboardPage() {
  const { session } = useOutletContext<LayoutContext>();
  const canViewInventory = session.permissions.includes("manageInventory");
  const inventory = useQuery<InventoryItem[]>({
    queryKey: ["inventory", "all"],
    queryFn: () => api.listInventory(),
    enabled: canViewInventory,
  });
  const movements = useQuery<InventoryMovement[]>({
    queryKey: ["inventory-movements", "all", ROTATION_LIMIT],
    queryFn: () =>
      api.listInventoryMovements({
        limit: ROTATION_LIMIT,
      }),
    enabled: canViewInventory,
  });

  const items = useMemo(() => inventory.data ?? [], [inventory.data]);

  const totals = useMemo(
    () =>
      items.reduce(
        (accumulator, item) => ({
          skus: accumulator.skus + 1,
          units: accumulator.units + item.stock,
          value: accumulator.value + item.valueMinor,
          low: accumulator.low + (item.lowStock ? 1 : 0),
        }),
        { skus: 0, units: 0, value: 0, low: 0 },
      ),
    [items],
  );

  const lowItems = useMemo(
    () =>
      items
        .filter((item) => item.lowStock)
        .sort((left, right) => left.stock - right.stock),
    [items],
  );

  const rotation = useMemo(() => {
    const cutoff = Date.now() - ROTATION_DAYS * 86_400_000;
    const sold = new Map<string, number>();
    const returned = new Map<string, number>();
    for (const movement of movements.data ?? []) {
      const createdAt = Date.parse(movement.createdAt);
      if (Number.isNaN(createdAt) || createdAt < cutoff) {
        continue;
      }
      if (movement.movementType === "sale") {
        sold.set(
          movement.variantId,
          (sold.get(movement.variantId) ?? 0) + Math.abs(movement.quantityDelta),
        );
      } else if (movement.movementType === "return") {
        returned.set(
          movement.variantId,
          (returned.get(movement.variantId) ?? 0) + Math.abs(movement.quantityDelta),
        );
      }
    }
    const byVariant = new Map(items.map((item) => [item.variantId, item]));
    const rows = Array.from(byVariant.values())
      .map((item) => {
        const net = Math.max(
          0,
          (sold.get(item.variantId) ?? 0) - (returned.get(item.variantId) ?? 0),
        );
        return {
          item,
          sold: net,
          coverageDays:
            net > 0
              ? Math.max(0, Math.round(item.stock / (net / ROTATION_DAYS)))
              : null,
        };
      })
      .filter((row) => row.sold > 0)
      .sort((left, right) => right.sold - left.sold);
    return {
      rows: rows.slice(0, 5),
      units: rows.reduce((total, row) => total + row.sold, 0),
    };
  }, [items, movements.data]);

  if (!canViewInventory) {
    return (
      <div className="space-y-6">
        <header>
          <p className="text-sm text-slate-600">Resumen operativo del gimnasio.</p>
        </header>
        <section className="rounded-2xl border border-slate-200 bg-white p-8 shadow-sm">
          <h2 className="text-base font-semibold">Inventario restringido</h2>
          <p className="mt-2 max-w-2xl text-sm text-slate-600">
            Tu rol puede consultar finanzas, pero no existencias ni valor de
            inventario. El estado técnico sigue disponible en Diagnóstico.
          </p>
          <Link
            to="/diagnostics"
            className="mt-5 inline-flex rounded-lg border border-slate-300 px-4 py-2 text-sm font-medium text-slate-700 hover:bg-slate-50"
          >
            Ir a Diagnóstico
          </Link>
        </section>
      </div>
    );
  }

  const error = inventory.error ?? movements.error;

  return (
    <div className="space-y-6">
      <header>
        <p className="text-sm text-slate-600">
          Existencias, valor y rotación básica de los últimos {ROTATION_DAYS} días.
        </p>
      </header>

      {error && (
        <div className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
          {errorMessage(error)}
        </div>
      )}

      {lowItems.length > 0 && (
        <div className="flex flex-wrap items-center justify-between gap-4 rounded-2xl border border-amber-200 bg-amber-50 p-5">
          <div>
            <p className="text-sm font-semibold text-amber-900">
              {lowItems.length} SKU en o bajo mínimo
            </p>
            <p className="mt-1 text-sm text-amber-800">
              Revisa y repón antes de que se agoten. La alerta se calcula con el
              umbral configurado por producto.
            </p>
          </div>
          <Link
            to="/inventory"
            className="rounded-lg bg-amber-900 px-4 py-2 text-sm font-semibold text-white hover:bg-amber-800"
          >
            Revisar inventario
          </Link>
        </div>
      )}

      <section className="grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
        <SummaryCard label="SKU rastreados" value={String(totals.skus)} />
        <SummaryCard label="Existencias" value={`${totals.units} u`} />
        <SummaryCard label="Valor inventario" value={formatMinor(totals.value)} />
        <SummaryCard
          label="Bajo mínimo"
          value={String(totals.low)}
          hint="SKU en o bajo el umbral"
          warning={totals.low > 0}
        />
      </section>

      <section className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h2 className="text-base font-semibold">Productos bajo mínimo</h2>
            <p className="mt-1 text-sm text-slate-500">
              Ordenados por existencia ascendente.
            </p>
          </div>
          <p className="text-sm text-slate-500">
            {totals.low} de {totals.skus} SKU
          </p>
        </div>

        {inventory.isLoading ? (
          <p className="py-8 text-center text-sm text-slate-500">
            Cargando inventario…
          </p>
        ) : lowItems.length === 0 ? (
          <p className="py-8 text-center text-sm text-slate-500">
            Todo el inventario está por encima de su mínimo.
          </p>
        ) : (
          <div className="mt-5 overflow-x-auto">
            <table className="w-full min-w-[620px] text-left text-sm">
              <thead className="border-b border-slate-200 text-xs uppercase tracking-wide text-slate-500">
                <tr>
                  <th className="pb-3 pr-3 font-medium">Producto</th>
                  <th className="pb-3 pr-3 font-medium">SKU</th>
                  <th className="pb-3 pr-3 text-right font-medium">Existencia</th>
                  <th className="pb-3 pr-3 text-right font-medium">Mínimo</th>
                  <th className="pb-3 text-right font-medium">Estado</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {lowItems.map((item) => (
                  <tr key={item.variantId}>
                    <td className="py-3 pr-3">
                      <span className="block font-medium">{item.productName}</span>
                      <span className="block text-xs text-slate-500">
                        {item.variantName}
                      </span>
                    </td>
                    <td className="py-3 pr-3 font-mono text-xs text-slate-600">
                      {item.sku}
                    </td>
                    <td className="py-3 pr-3 text-right font-semibold tabular-nums text-amber-700">
                      {item.stock}
                    </td>
                    <td className="py-3 pr-3 text-right tabular-nums">
                      {item.minStock}
                    </td>
                    <td className="py-3 text-right">
                      <span
                        className={[
                          "rounded-full px-2.5 py-1 text-xs font-semibold ring-1 ring-inset",
                          item.stock <= 0
                            ? "bg-rose-50 text-rose-700 ring-rose-600/20"
                            : "bg-amber-50 text-amber-700 ring-amber-600/20",
                        ].join(" ")}
                      >
                        {item.stock <= 0 ? "Agotado" : "Bajo mínimo"}
                      </span>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>

      <section className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <div>
            <h2 className="text-base font-semibold">Rotación básica</h2>
            <p className="mt-1 text-sm text-slate-500">
              Salidas netas de ventas menos devoluciones en {ROTATION_DAYS} días.
            </p>
          </div>
          <p className="text-sm text-slate-500">
            {rotation.units} unidad(es) vendidas
          </p>
        </div>

        {movements.isLoading ? (
          <p className="py-8 text-center text-sm text-slate-500">
            Calculando rotación…
          </p>
        ) : rotation.rows.length === 0 ? (
          <p className="py-8 text-center text-sm text-slate-500">
            Aún no hay salidas de inventario en los últimos {ROTATION_DAYS} días.
          </p>
        ) : (
          <div className="mt-5 overflow-x-auto">
            <table className="w-full min-w-[620px] text-left text-sm">
              <thead className="border-b border-slate-200 text-xs uppercase tracking-wide text-slate-500">
                <tr>
                  <th className="pb-3 pr-3 font-medium">Producto</th>
                  <th className="pb-3 pr-3 font-medium">SKU</th>
                  <th className="pb-3 pr-3 text-right font-medium">Salidas</th>
                  <th className="pb-3 text-right font-medium">Cobertura</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {rotation.rows.map(({ item, sold, coverageDays }) => (
                  <tr key={item.variantId}>
                    <td className="py-3 pr-3">
                      <span className="block font-medium">{item.productName}</span>
                      <span className="block text-xs text-slate-500">
                        {item.variantName}
                      </span>
                    </td>
                    <td className="py-3 pr-3 font-mono text-xs text-slate-600">
                      {item.sku}
                    </td>
                    <td className="py-3 pr-3 text-right font-semibold tabular-nums">
                      {sold}
                    </td>
                    <td className="py-3 text-right tabular-nums">
                      {coverageDays === null ? "—" : `${coverageDays} días`}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        )}
      </section>

      {!inventory.isLoading && items.length === 0 && (
        <section className="rounded-2xl border border-dashed border-slate-300 bg-white p-8 text-center">
          <h2 className="text-base font-semibold">Sin productos rastreados</h2>
          <p className="mx-auto mt-2 max-w-xl text-sm text-slate-500">
            Crea una variante con unidad Pieza en el catálogo para empezar a
            controlar existencias.
          </p>
          <Link
            to="/catalog"
            className="mt-5 inline-flex rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-700"
          >
            Ir al catálogo
          </Link>
        </section>
      )}
    </div>
  );
}

function SummaryCard({
  label,
  value,
  hint,
  warning = false,
}: {
  label: string;
  value: string;
  hint?: string;
  warning?: boolean;
}) {
  return (
    <article className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
      <p className="text-xs font-medium uppercase tracking-wide text-slate-500">
        {label}
      </p>
      <p
        className={[
          "mt-2 text-2xl font-semibold tabular-nums",
          warning ? "text-amber-700" : "",
        ].join(" ")}
      >
        {value}
      </p>
      {hint && <p className="mt-1 text-xs text-slate-500">{hint}</p>}
    </article>
  );
}

function formatMinor(amount: number): string {
  return new Intl.NumberFormat("es-MX", {
    style: "currency",
    currency: "MXN",
  }).format(amount / 100);
}
