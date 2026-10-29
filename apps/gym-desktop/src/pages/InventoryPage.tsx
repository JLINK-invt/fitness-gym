import type {
  CatalogProduct,
  InventoryItem,
  InventoryMovement,
} from "@gym/contracts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo, useState, type ReactNode } from "react";

import { api } from "../lib/tauri";

type Operation = "entry" | "adjust";

const movementLabels: Record<InventoryMovement["movementType"], string> = {
  entry: "Entrada",
  sale: "Venta",
  return: "Devolución",
  adjustment: "Ajuste",
};

const fieldClass =
  "w-full rounded-lg border border-slate-300 px-3 py-2 text-sm outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100";
const primaryButton =
  "rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-700 disabled:opacity-50";

export function InventoryPage() {
  const queryClient = useQueryClient();
  const [search, setSearch] = useState("");
  const [lowOnly, setLowOnly] = useState(false);
  const [operation, setOperation] = useState<Operation>("entry");
  const [error, setError] = useState<string | null>(null);
  const [entry, setEntry] = useState({
    variantId: "",
    quantity: "1",
    unitCost: "",
    reference: "",
    note: "",
  });
  const [adjustment, setAdjustment] = useState({
    variantId: "",
    delta: "",
    unitCost: "",
    reason: "",
  });
  const [thresholds, setThresholds] = useState<Record<string, string>>({});

  const catalog = useQuery<CatalogProduct[]>({
    queryKey: ["catalog"],
    queryFn: api.listCatalog,
  });
  const inventory = useQuery<InventoryItem[]>({
    queryKey: ["inventory", "all"],
    queryFn: () => api.listInventory(),
  });
  const movements = useQuery<InventoryMovement[]>({
    queryKey: ["inventory-movements", "all", 40],
    queryFn: () =>
      api.listInventoryMovements({
        limit: 40,
      }),
  });

  const pieces = useMemo(
    () =>
      (catalog.data ?? []).flatMap((product) =>
        product.variants
          .filter(
            (variant) =>
              variant.unit === "piece" &&
              variant.status === "active" &&
              product.status === "active",
          )
          .map((variant) => ({
            id: variant.id,
            label: `${product.name} · ${variant.name}`,
            sku: variant.sku,
            costMinor: variant.costMinor,
          })),
      ),
    [catalog.data],
  );

  const filtered = useMemo(() => {
    const term = search.trim().toLowerCase();
    return (inventory.data ?? []).filter((item) => {
      if (lowOnly && !item.lowStock) {
        return false;
      }
      if (!term) {
        return true;
      }
      return [item.productName, item.variantName, item.sku].some((value) =>
        value.toLowerCase().includes(term),
      );
    });
  }, [inventory.data, lowOnly, search]);

  const totals = useMemo(
    () =>
      (inventory.data ?? []).reduce(
        (accumulator, item) => ({
          skus: accumulator.skus + 1,
          units: accumulator.units + item.stock,
          value: accumulator.value + item.valueMinor,
          low: accumulator.low + (item.lowStock ? 1 : 0),
        }),
        { skus: 0, units: 0, value: 0, low: 0 },
      ),
    [inventory.data],
  );

  const refresh = () => {
    void queryClient.invalidateQueries({ queryKey: ["inventory"] });
    void queryClient.invalidateQueries({ queryKey: ["inventory-movements"] });
    void queryClient.invalidateQueries({ queryKey: ["audit"] });
  };

  const recordEntry = useMutation({
    mutationFn: () =>
      api.recordInventoryEntry({
        variantId: entry.variantId,
        quantity: Number(entry.quantity),
        unitCostMinor: toMinor(entry.unitCost),
        reference: entry.reference.trim() || undefined,
        note: entry.note.trim() || undefined,
      }),
    onSuccess: () => {
      setEntry((current) => ({
        ...current,
        quantity: "1",
        reference: "",
        note: "",
      }));
      setError(null);
      refresh();
    },
    onError: (cause) => setError(messageOf(cause, "No se pudo registrar la entrada.")),
  });

  const adjust = useMutation({
    mutationFn: () =>
      api.adjustInventory({
        variantId: adjustment.variantId,
        quantityDelta: Number(adjustment.delta),
        unitCostMinor: adjustment.unitCost.trim()
          ? toMinor(adjustment.unitCost)
          : undefined,
        reason: adjustment.reason.trim(),
      }),
    onSuccess: () => {
      setAdjustment((current) => ({ ...current, delta: "", reason: "" }));
      setError(null);
      refresh();
    },
    onError: (cause) => setError(messageOf(cause, "No se pudo registrar el ajuste.")),
  });

  const threshold = useMutation({
    mutationFn: (input: { variantId: string; minStock: number }) =>
      api.updateInventoryThreshold(input),
    onSuccess: (updated) => {
      setThresholds((current) => ({
        ...current,
        [updated.variantId]: String(updated.minStock),
      }));
      setError(null);
      refresh();
    },
    onError: (cause) => setError(messageOf(cause, "No se pudo guardar el mínimo.")),
  });

  const chooseVariant = (variantId: string, operationToOpen: Operation) => {
    setOperation(operationToOpen);
    const variant = pieces.find((item) => item.id === variantId);
    if (operationToOpen === "entry") {
      setEntry((current) => ({
        ...current,
        variantId,
        unitCost: current.unitCost || minorInput(variant?.costMinor ?? 0),
      }));
      return;
    }
    setAdjustment({
      variantId,
      delta: "",
      unitCost: "",
      reason: "",
    });
  };

  return (
    <div className="space-y-6">
      <div>
        <p className="text-sm text-slate-600">
          Existencias y valor derivados de movimientos inmutables. Recibe o ajusta
          stock sin editar el historial.
        </p>
      </div>

      <div className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4">
        <Summary label="SKU rastreados" value={String(totals.skus)} />
        <Summary label="Unidades" value={String(totals.units)} />
        <Summary label="Valor inventario" value={formatMinor(totals.value)} />
        <Summary label="Bajo mínimo" value={String(totals.low)} tone={totals.low > 0 ? "warn" : "ok"} />
      </div>

      {error && (
        <p className="rounded-xl border border-red-200 bg-red-50 px-4 py-3 text-sm text-red-700">
          {error}
        </p>
      )}

      <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
        <div className="flex flex-wrap items-end gap-3">
          <Field label="Buscar">
            <input
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder="Producto, variante o SKU"
              className={fieldClass}
            />
          </Field>
          <label className="flex items-center gap-2 pb-2 text-sm text-slate-700">
            <input
              type="checkbox"
              checked={lowOnly}
              onChange={(event) => setLowOnly(event.target.checked)}
              className="h-4 w-4 rounded border-slate-300"
            />
            Solo bajo mínimo
          </label>
        </div>

        <div className="mt-5 overflow-x-auto">
          {inventory.isLoading ? (
            <p className="py-8 text-center text-sm text-slate-500">Cargando inventario…</p>
          ) : filtered.length === 0 ? (
            <p className="py-8 text-center text-sm text-slate-500">
              No hay productos rastreados con este filtro.
            </p>
          ) : (
            <table className="w-full min-w-[860px] text-left text-sm">
              <thead className="border-b border-slate-200 text-xs uppercase tracking-wide text-slate-500">
                <tr>
                  <th className="pb-3 pr-3 font-medium">Producto</th>
                  <th className="pb-3 pr-3 text-right font-medium">Existencia</th>
                  <th className="pb-3 pr-3 text-right font-medium">Costo promedio</th>
                  <th className="pb-3 pr-3 text-right font-medium">Valor</th>
                  <th className="pb-3 pr-3 text-right font-medium">Mínimo</th>
                  <th className="pb-3 text-right font-medium">Acciones</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {filtered.map((item) => (
                  <tr key={item.variantId}>
                    <td className="py-3 pr-3">
                      <span className="block font-medium">{item.productName}</span>
                      <span className="block text-xs text-slate-500">
                        {item.variantName} · {item.sku}
                      </span>
                    </td>
                    <td className="py-3 pr-3 text-right tabular-nums">
                      <span className={item.lowStock ? "font-semibold text-amber-700" : ""}>
                        {item.stock}
                      </span>
                    </td>
                    <td className="py-3 pr-3 text-right tabular-nums">
                      {formatMinor(item.avgUnitCostMinor)}
                    </td>
                    <td className="py-3 pr-3 text-right tabular-nums">
                      {formatMinor(item.valueMinor)}
                    </td>
                    <td className="py-3 pr-3 text-right">
                      <input
                        type="number"
                        min={0}
                        max={999999}
                        value={thresholds[item.variantId] ?? String(item.minStock)}
                        onChange={(event) =>
                          setThresholds((current) => ({
                            ...current,
                            [item.variantId]: event.target.value,
                          }))
                        }
                        onBlur={() => {
                          const value = Number(
                            thresholds[item.variantId] ?? item.minStock,
                          );
                          if (Number.isInteger(value) && value !== item.minStock) {
                            threshold.mutate({ variantId: item.variantId, minStock: value });
                          }
                        }}
                        className="w-24 rounded-lg border border-slate-300 px-2 py-1 text-right text-sm outline-none focus:border-brand-500"
                      />
                    </td>
                    <td className="py-3 text-right">
                      <div className="flex justify-end gap-2">
                        <button
                          type="button"
                          onClick={() => chooseVariant(item.variantId, "adjust")}
                          className="rounded-lg border border-slate-300 px-2 py-1 text-xs font-medium text-slate-700 hover:bg-slate-50"
                        >
                          Ajustar
                        </button>
                      </div>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </section>

      <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <h2 className="text-base font-semibold">Operación de inventario</h2>
          <div className="flex gap-1 rounded-lg bg-slate-100 p-1">
            {(
              [
                ["entry", "Entrada"],
                ["adjust", "Ajuste"],
              ] as const
            ).map(([value, label]) => (
              <button
                key={value}
                type="button"
                onClick={() => {
                  setOperation(value);
                  if (value === "entry" && !entry.variantId && pieces[0]) {
                    chooseVariant(pieces[0].id, "entry");
                  }
                }}
                className={[
                  "rounded-md px-3 py-1.5 text-sm font-medium transition",
                  operation === value
                    ? "bg-white text-brand-700 shadow-sm"
                    : "text-slate-600 hover:text-slate-900",
                ].join(" ")}
              >
                {label}
              </button>
            ))}
          </div>
        </div>

        {operation === "entry" && (
          <form
            className="mt-4 grid gap-4 sm:grid-cols-2 lg:grid-cols-3"
            onSubmit={(event) => {
              event.preventDefault();
              recordEntry.mutate();
            }}
          >
            <VariantSelect
              variants={pieces}
              value={entry.variantId}
              onChange={(variantId) => chooseVariant(variantId, "entry")}
            />
            <Field label="Cantidad">
              <input
                type="number"
                min={1}
                max={999999}
                value={entry.quantity}
                onChange={(event) => setEntry({ ...entry, quantity: event.target.value })}
                className={fieldClass}
              />
            </Field>
            <Field label="Costo unitario">
              <input
                inputMode="decimal"
                value={entry.unitCost}
                onChange={(event) => setEntry({ ...entry, unitCost: event.target.value })}
                className={fieldClass}
              />
            </Field>
            <Field label="Referencia">
              <input
                value={entry.reference}
                onChange={(event) => setEntry({ ...entry, reference: event.target.value })}
                placeholder="Factura, proveedor u orden"
                className={fieldClass}
              />
            </Field>
            <Field label="Nota">
              <input
                value={entry.note}
                onChange={(event) => setEntry({ ...entry, note: event.target.value })}
                placeholder="Detalle opcional"
                className={fieldClass}
              />
            </Field>
            <div className="flex items-end">
              <button
                type="submit"
                disabled={
                  !entry.variantId ||
                  !Number.isInteger(Number(entry.quantity)) ||
                  Number(entry.quantity) < 1 ||
                  entry.unitCost.trim() === "" ||
                  !Number.isFinite(Number(entry.unitCost)) ||
                  recordEntry.isPending
                }
                className={primaryButton}
              >
                Registrar entrada
              </button>
            </div>
          </form>
        )}

        {operation === "adjust" && (
          <form
            className="mt-4 grid gap-4 sm:grid-cols-2 lg:grid-cols-3"
            onSubmit={(event) => {
              event.preventDefault();
              adjust.mutate();
            }}
          >
            <VariantSelect
              variants={pieces}
              value={adjustment.variantId}
              onChange={(variantId) => chooseVariant(variantId, "adjust")}
            />
            <Field label="Diferencia">
              <input
                type="number"
                value={adjustment.delta}
                onChange={(event) => setAdjustment({ ...adjustment, delta: event.target.value })}
                placeholder="-1 o 3"
                className={fieldClass}
              />
            </Field>
            <Field label="Costo unitario (opcional)">
              <input
                inputMode="decimal"
                value={adjustment.unitCost}
                onChange={(event) =>
                  setAdjustment({ ...adjustment, unitCost: event.target.value })
                }
                placeholder="Usa el costo promedio"
                className={fieldClass}
              />
            </Field>
            <Field label="Motivo">
              <input
                value={adjustment.reason}
                onChange={(event) =>
                  setAdjustment({ ...adjustment, reason: event.target.value })
                }
                placeholder="Merma, conteo físico…"
                className={fieldClass}
              />
            </Field>
            <div className="flex items-end">
              <button
                type="submit"
                disabled={
                  !adjustment.variantId ||
                  !Number.isInteger(Number(adjustment.delta)) ||
                  Number(adjustment.delta) === 0 ||
                  adjustment.reason.trim().length === 0 ||
                  adjust.isPending
                }
                className={primaryButton}
              >
                Registrar ajuste
              </button>
            </div>
          </form>
        )}

      </section>

      <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
        <h2 className="text-base font-semibold">Movimientos recientes</h2>
        <div className="mt-4 overflow-x-auto">
          {movements.isLoading ? (
            <p className="py-6 text-center text-sm text-slate-500">Cargando movimientos…</p>
          ) : (movements.data ?? []).length === 0 ? (
            <p className="py-6 text-center text-sm text-slate-500">
              Todavía no hay movimientos de inventario.
            </p>
          ) : (
            <table className="w-full min-w-[760px] text-left text-sm">
              <thead className="border-b border-slate-200 text-xs uppercase tracking-wide text-slate-500">
                <tr>
                  <th className="pb-3 pr-3 font-medium">Fecha</th>
                  <th className="pb-3 pr-3 font-medium">Tipo</th>
                  <th className="pb-3 pr-3 text-right font-medium">Cantidad</th>
                  <th className="pb-3 pr-3 text-right font-medium">Costo</th>
                  <th className="pb-3 font-medium">Detalle</th>
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {(movements.data ?? []).map((item) => (
                  <tr key={item.id}>
                    <td className="py-3 pr-3 text-xs text-slate-500">
                      {formatDateTime(item.createdAt)}
                    </td>
                    <td className="py-3 pr-3">{movementLabels[item.movementType]}</td>
                    <td
                      className={[
                        "py-3 pr-3 text-right font-medium tabular-nums",
                        item.quantityDelta > 0 ? "text-emerald-700" : "text-rose-700",
                      ].join(" ")}
                    >
                      {item.quantityDelta > 0 ? "+" : ""}
                      {item.quantityDelta}
                    </td>
                    <td className="py-3 pr-3 text-right tabular-nums">
                      {formatMinor(item.totalCostMinor)}
                    </td>
                    <td className="py-3 text-slate-600">
                      {item.note ?? item.reference ?? "—"}
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          )}
        </div>
      </section>
    </div>
  );
}

function VariantSelect({
  variants,
  value,
  onChange,
}: {
  variants: { id: string; label: string; sku: string }[];
  value: string;
  onChange: (value: string) => void;
}) {
  return (
    <Field label="Producto">
      <select
        value={value}
        onChange={(event) => onChange(event.target.value)}
        className={fieldClass}
      >
        <option value="">Selecciona una variante</option>
        {variants.map((variant) => (
          <option key={variant.id} value={variant.id}>
            {variant.label} · {variant.sku}
          </option>
        ))}
      </select>
    </Field>
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
      <p className="mt-1 text-xl font-semibold tabular-nums">{value}</p>
    </div>
  );
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="text-sm">
      <span className="font-medium text-slate-700">{label}</span>
      <div className="mt-1">{children}</div>
    </label>
  );
}

function messageOf(cause: unknown, fallback: string): string {
  return cause instanceof Error && cause.message ? cause.message : fallback;
}

function toMinor(value: string): number {
  return Math.round(Number(value) * 100);
}

function minorInput(amount: number): string {
  return (amount / 100).toFixed(2);
}

function formatMinor(amount: number): string {
  return new Intl.NumberFormat("es-MX", {
    style: "currency",
    currency: "MXN",
  }).format(amount / 100);
}

function formatDateTime(value: string): string {
  return new Intl.DateTimeFormat("es-MX", {
    dateStyle: "short",
    timeStyle: "short",
  }).format(new Date(value));
}
