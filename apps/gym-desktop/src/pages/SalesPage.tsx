import type {
  CatalogProduct,
  CatalogVariant,
  CreateSaleInput,
  PaymentMethod,
  Sale,
} from "@gym/contracts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useMemo, useState } from "react";

import { api } from "../lib/tauri";

type CartLine = {
  variantId: string;
  quantity: number;
};

const paymentLabels: Record<PaymentMethod, string> = {
  cash: "Efectivo",
  transfer: "Transferencia",
  point: "Terminal Point",
};

const fieldClass =
  "w-full rounded-lg border border-slate-300 px-3 py-2 text-sm outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100";
const primaryButton =
  "rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-700 disabled:opacity-50";
const secondaryButton =
  "rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium text-slate-700 hover:bg-slate-100 disabled:opacity-50";

export function SalesPage() {
  const queryClient = useQueryClient();
  const [search, setSearch] = useState("");
  const [cart, setCart] = useState<CartLine[]>([]);
  const [paymentMethod, setPaymentMethod] = useState<PaymentMethod>("cash");
  const [cashReceived, setCashReceived] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [receipt, setReceipt] = useState<Sale | null>(null);

  const catalog = useQuery<CatalogProduct[]>({
    queryKey: ["catalog"],
    queryFn: api.listCatalog,
  });
  const sales = useQuery<Sale[]>({
    queryKey: ["sales"],
    queryFn: api.listSales,
  });

  const sellable = useMemo(
    () =>
      (catalog.data ?? []).flatMap((product) =>
        product.variants
          .filter((variant) => product.status === "active" && variant.status === "active")
          .map((variant) => ({ product, variant })),
      ),
    [catalog.data],
  );

  const byVariant = useMemo(() => {
    const map = new Map<string, { product: CatalogProduct; variant: CatalogVariant }>();
    for (const entry of sellable) {
      map.set(entry.variant.id, entry);
    }
    return map;
  }, [sellable]);

  const filtered = useMemo(() => {
    const term = search.trim().toLowerCase();
    if (!term) {
      return sellable;
    }
    return sellable.filter(({ product, variant }) =>
      [product.name, variant.name, variant.sku, variant.barcode ?? ""]
        .join(" ")
        .toLowerCase()
        .includes(term),
    );
  }, [search, sellable]);

  const lines = useMemo(
    () =>
      cart.flatMap((line) => {
        const entry = byVariant.get(line.variantId);
        const price = entry?.variant.currentPrice;
        if (!entry || !price) {
          return [];
        }
        const totals = lineTotals(
          price.amountMinor,
          line.quantity,
          price.taxRateBps,
          price.priceIncludesTax,
        );
        return [
          {
            ...line,
            productName: entry.product.name,
            variantName: entry.variant.name,
            sku: entry.variant.sku,
            price,
            ...totals,
          },
        ];
      }),
    [byVariant, cart],
  );

  const subtotal = lines.reduce((total, line) => total + line.subtotal, 0);
  const tax = lines.reduce((total, line) => total + line.tax, 0);
  const total = lines.reduce((total, line) => total + line.total, 0);
  const change =
    paymentMethod === "cash" && cashReceived.trim()
      ? Math.max(0, toMinor(cashReceived) - total)
      : 0;
  const cashIsValid =
    paymentMethod !== "cash" ||
    !cashReceived.trim() ||
    (validMoney(cashReceived) && toMinor(cashReceived) >= total);

  const create = useMutation({
    mutationFn: (input: CreateSaleInput) => api.createSale(input),
    onSuccess: (sale) => {
      setReceipt(sale);
      setCart([]);
      setCashReceived("");
      setError(null);
      queryClient.invalidateQueries({ queryKey: ["sales"] });
      queryClient.invalidateQueries({ queryKey: ["audit"] });
    },
    onError: (cause) => {
      setError(cause instanceof Error ? cause.message : "No se pudo registrar la venta.");
    },
  });

  const add = (variantId: string) => {
    setError(null);
    setCart((current) => {
      const existing = current.find((line) => line.variantId === variantId);
      if (existing) {
        return current.map((line) =>
          line.variantId === variantId
            ? { ...line, quantity: Math.min(999, line.quantity + 1) }
            : line,
        );
      }
      return [...current, { variantId, quantity: 1 }];
    });
  };

  const setQuantity = (variantId: string, quantity: number) => {
    if (quantity < 1) {
      setCart((current) => current.filter((line) => line.variantId !== variantId));
      return;
    }
    setCart((current) =>
      current.map((line) =>
        line.variantId === variantId
          ? { ...line, quantity: Math.min(999, quantity) }
          : line,
      ),
    );
  };

  const complete = () => {
    if (lines.length === 0) {
      return;
    }
    const input: CreateSaleInput = {
      items: cart.map((line) => ({
        variantId: line.variantId,
        quantity: line.quantity,
      })),
      paymentMethod,
    };
    if (paymentMethod === "cash" && cashReceived.trim()) {
      input.cashReceivedMinor = toMinor(cashReceived);
    }
    create.mutate(input);
  };

  return (
    <div className="space-y-6">
      <header className="flex flex-wrap items-end justify-between gap-3">
        <div>
          <h1 className="text-2xl font-semibold tracking-tight">Ventas</h1>
          <p className="mt-1 text-sm text-slate-600">
            Cobra productos y servicios con el precio vigente del catálogo.
          </p>
        </div>
        <span className="rounded-full bg-brand-50 px-3 py-1 text-xs font-medium text-brand-700">
          {paymentLabels[paymentMethod]}
        </span>
      </header>

      <div className="grid gap-6 lg:grid-cols-[1.4fr_1fr]">
        <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <h2 className="text-sm font-semibold uppercase tracking-wide text-slate-500">
              Catálogo
            </h2>
            <input
              type="search"
              value={search}
              onChange={(event) => setSearch(event.target.value)}
              placeholder="Buscar por nombre, SKU o código"
              className={`${fieldClass} max-w-xs`}
            />
          </div>

          {catalog.isLoading ? (
            <p className="mt-4 text-sm text-slate-500">Cargando catálogo…</p>
          ) : filtered.length === 0 ? (
            <p className="mt-4 text-sm text-slate-500">
              No hay variantes con precio vigente para vender.
            </p>
          ) : (
            <ul className="mt-4 grid gap-2 sm:grid-cols-2">
              {filtered.map(({ product, variant }) => (
                <li key={variant.id}>
                  <button
                    type="button"
                    onClick={() => add(variant.id)}
                    disabled={!variant.currentPrice}
                    className="flex w-full items-center justify-between gap-3 rounded-xl border border-slate-200 px-3 py-3 text-left transition hover:border-brand-500 hover:bg-brand-50 disabled:cursor-not-allowed disabled:opacity-50"
                  >
                    <span className="min-w-0">
                      <span className="block truncate text-sm font-medium">
                        {product.name}
                      </span>
                      <span className="block truncate text-xs text-slate-500">
                        {variant.name} · {variant.sku}
                      </span>
                    </span>
                    <span className="shrink-0 text-right">
                      <span className="block text-sm font-semibold">
                        {variant.currentPrice
                          ? formatMinor(variant.currentPrice.amountMinor, "MXN")
                          : "Sin precio"}
                      </span>
                      <span className="block text-xs text-slate-400">
                        {variant.currentPrice?.priceIncludesTax ? "IVA incluido" : "+ IVA"}
                      </span>
                    </span>
                  </button>
                </li>
              ))}
            </ul>
          )}
        </section>

        <section className="space-y-4 rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
          <div className="flex items-center justify-between">
            <h2 className="text-sm font-semibold uppercase tracking-wide text-slate-500">
              Carrito
            </h2>
            <span className="text-xs text-slate-400">
              {cart.reduce((count, line) => count + line.quantity, 0)} artículos
            </span>
          </div>

          {lines.length === 0 ? (
            <p className="rounded-xl border border-dashed border-slate-300 px-4 py-8 text-center text-sm text-slate-500">
              Agrega productos del catálogo para iniciar una venta.
            </p>
          ) : (
            <ul className="divide-y divide-slate-100">
              {lines.map((line) => (
                <li key={line.variantId} className="flex items-center gap-3 py-3">
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-medium">{line.productName}</p>
                    <p className="truncate text-xs text-slate-500">
                      {line.variantName} · {formatMinor(line.price.amountMinor, "MXN")}
                    </p>
                  </div>
                  <div className="flex items-center gap-1">
                    <button
                      type="button"
                      onClick={() => setQuantity(line.variantId, line.quantity - 1)}
                      className="h-8 w-8 rounded-lg border border-slate-300 text-sm font-semibold hover:bg-slate-100"
                      aria-label={`Quitar una unidad de ${line.productName}`}
                    >
                      −
                    </button>
                    <span className="w-8 text-center text-sm tabular-nums">
                      {line.quantity}
                    </span>
                    <button
                      type="button"
                      onClick={() => setQuantity(line.variantId, line.quantity + 1)}
                      className="h-8 w-8 rounded-lg border border-slate-300 text-sm font-semibold hover:bg-slate-100"
                      aria-label={`Agregar una unidad de ${line.productName}`}
                    >
                      +
                    </button>
                  </div>
                  <span className="w-24 text-right text-sm font-semibold tabular-nums">
                    {formatMinor(line.total, "MXN")}
                  </span>
                </li>
              ))}
            </ul>
          )}

          <dl className="space-y-1 border-t border-slate-200 pt-3 text-sm">
            <div className="flex justify-between text-slate-600">
              <dt>Subtotal</dt>
              <dd className="tabular-nums">{formatMinor(subtotal, "MXN")}</dd>
            </div>
            <div className="flex justify-between text-slate-600">
              <dt>Impuestos</dt>
              <dd className="tabular-nums">{formatMinor(tax, "MXN")}</dd>
            </div>
            <div className="flex justify-between text-base font-semibold">
              <dt>Total</dt>
              <dd className="tabular-nums">{formatMinor(total, "MXN")}</dd>
            </div>
          </dl>

          <div className="space-y-3 border-t border-slate-200 pt-3">
            <label className="block text-sm font-medium text-slate-700">
              Método de pago
              <select
                value={paymentMethod}
                onChange={(event) => {
                  setPaymentMethod(event.target.value as PaymentMethod);
                  setCashReceived("");
                }}
                className={`${fieldClass} mt-1`}
              >
                <option value="cash">Efectivo</option>
                <option value="transfer">Transferencia</option>
                <option value="point">Terminal Point</option>
              </select>
            </label>

            {paymentMethod === "cash" && (
              <label className="block text-sm font-medium text-slate-700">
                Efectivo recibido
                <input
                  inputMode="decimal"
                  value={cashReceived}
                  onChange={(event) => setCashReceived(event.target.value)}
                  placeholder="Exacto"
                  className={`${fieldClass} mt-1`}
                />
                {!cashIsValid && (
                  <span className="mt-1 block text-xs text-red-600">
                    El efectivo recibido no cubre el total.
                  </span>
                )}
                {cashIsValid && cashReceived.trim() && (
                  <span className="mt-1 block text-xs text-slate-500">
                    Cambio: {formatMinor(change, "MXN")}
                  </span>
                )}
              </label>
            )}

            {error && <p className="text-sm text-red-600">{error}</p>}

            <button
              type="button"
              onClick={complete}
              disabled={create.isPending || lines.length === 0 || !cashIsValid}
              className={`${primaryButton} w-full`}
            >
              {create.isPending ? "Registrando…" : "Completar venta"}
            </button>
          </div>
        </section>
      </div>

      {receipt && (
        <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm print:border-0 print:shadow-none">
          <div className="flex flex-wrap items-center justify-between gap-3">
            <h2 className="text-sm font-semibold uppercase tracking-wide text-slate-500">
              Ticket de venta
            </h2>
            <div className="flex gap-2 print:hidden">
              <button type="button" onClick={() => window.print()} className={secondaryButton}>
                Imprimir
              </button>
              <button type="button" onClick={() => setReceipt(null)} className={secondaryButton}>
                Cerrar
              </button>
            </div>
          </div>
          <Receipt sale={receipt} />
        </section>
      )}

      <section className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
        <h2 className="text-sm font-semibold uppercase tracking-wide text-slate-500">
          Ventas recientes
        </h2>
        {sales.isLoading ? (
          <p className="mt-4 text-sm text-slate-500">Cargando ventas…</p>
        ) : (sales.data ?? []).length === 0 ? (
          <p className="mt-4 text-sm text-slate-500">Todavía no hay ventas registradas.</p>
        ) : (
          <ul className="mt-3 divide-y divide-slate-100">
            {(sales.data ?? []).map((sale) => (
              <li key={sale.id} className="flex flex-wrap items-center gap-3 py-3">
                <div className="min-w-0 flex-1">
                  <p className="text-sm font-medium">
                    {formatMinor(sale.totalMinor, sale.currency)} ·{" "}
                    {paymentLabels[sale.paymentMethod]}
                  </p>
                  <p className="text-xs text-slate-500">
                    {new Date(sale.createdAt).toLocaleString("es-MX")} ·{" "}
                    {sale.items.reduce((count, item) => count + item.quantity, 0)} artículos
                    {sale.soldByName ? ` · ${sale.soldByName}` : ""}
                  </p>
                </div>
                <button
                  type="button"
                  onClick={() => setReceipt(sale)}
                  className={secondaryButton}
                >
                  Ver ticket
                </button>
              </li>
            ))}
          </ul>
        )}
      </section>
    </div>
  );
}

function Receipt({ sale }: { sale: Sale }) {
  return (
    <div id="receipt" className="mt-4 max-w-sm text-sm">
      <p className="text-base font-semibold">Fitness Gym</p>
      <p className="text-xs text-slate-500">
        {new Date(sale.createdAt).toLocaleString("es-MX")}
      </p>
      <p className="text-xs text-slate-500">Venta {sale.id.slice(0, 8)}</p>
      <table className="mt-3 w-full text-xs">
        <thead>
          <tr className="border-b border-slate-200 text-left text-slate-500">
            <th className="py-1 font-medium">Artículo</th>
            <th className="py-1 text-right font-medium">Cant.</th>
            <th className="py-1 text-right font-medium">Importe</th>
          </tr>
        </thead>
        <tbody>
          {sale.items.map((item) => (
            <tr key={item.id} className="align-top">
              <td className="py-1 pr-2">
                {item.productName}
                <span className="block text-slate-500">
                  {item.variantName} · {item.sku}
                </span>
              </td>
              <td className="py-1 text-right tabular-nums">{item.quantity}</td>
              <td className="py-1 text-right tabular-nums">
                {formatMinor(item.totalMinor, sale.currency)}
              </td>
            </tr>
          ))}
        </tbody>
      </table>
      <dl className="mt-3 space-y-1 border-t border-slate-200 pt-2">
        <div className="flex justify-between">
          <dt>Subtotal</dt>
          <dd className="tabular-nums">{formatMinor(sale.subtotalMinor, sale.currency)}</dd>
        </div>
        <div className="flex justify-between">
          <dt>Impuestos</dt>
          <dd className="tabular-nums">{formatMinor(sale.taxMinor, sale.currency)}</dd>
        </div>
        <div className="flex justify-between text-base font-semibold">
          <dt>Total</dt>
          <dd className="tabular-nums">{formatMinor(sale.totalMinor, sale.currency)}</dd>
        </div>
      </dl>
      <p className="mt-3 text-xs text-slate-500">
        Pago: {paymentLabels[sale.paymentMethod]}
        {sale.cashReceivedMinor != null
          ? ` · Recibido ${formatMinor(sale.cashReceivedMinor, sale.currency)}`
          : ""}
        {sale.changeMinor != null
          ? ` · Cambio ${formatMinor(sale.changeMinor, sale.currency)}`
          : ""}
      </p>
      {sale.soldByName && (
        <p className="mt-1 text-xs text-slate-500">Atendió: {sale.soldByName}</p>
      )}
      <p className="mt-2 text-xs text-slate-400">Ticket no fiscal</p>
    </div>
  );
}

function lineTotals(
  unitAmountMinor: number,
  quantity: number,
  taxRateBps: number,
  priceIncludesTax: boolean,
): { subtotal: number; tax: number; total: number } {
  const gross = unitAmountMinor * quantity;
  if (priceIncludesTax) {
    const denominator = 10_000 + taxRateBps;
    const subtotal = Math.floor((gross * 20_000 + denominator) / (denominator * 2));
    return { subtotal, tax: gross - subtotal, total: gross };
  }
  const tax = Math.floor((gross * taxRateBps + 5_000) / 10_000);
  return { subtotal: gross, tax, total: gross + tax };
}

function validMoney(value: string): boolean {
  if (!value.trim()) {
    return false;
  }
  const amount = Number(value);
  return Number.isFinite(amount) && amount >= 0;
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
