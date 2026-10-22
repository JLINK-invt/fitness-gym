import type {
  CatalogProduct,
  CatalogStatus,
  CatalogUnit,
  CatalogVariant,
  CatalogVariantInput,
} from "@gym/contracts";
import { errorMessage } from "@gym/contracts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { useState } from "react";

import { api } from "../lib/tauri";

const unitLabels: Record<CatalogUnit, string> = {
  piece: "Pieza",
  service: "Servicio",
  session: "Sesión",
  package: "Paquete",
};

const statusLabels: Record<CatalogStatus, string> = {
  active: "Activo",
  archived: "Archivado",
};

type CreateForm = {
  name: string;
  category: string;
  variantName: string;
  sku: string;
  barcode: string;
  unit: CatalogUnit;
  cost: string;
  price: string;
  tax: string;
  priceIncludesTax: boolean;
};

const emptyForm: CreateForm = {
  name: "",
  category: "",
  variantName: "",
  sku: "",
  barcode: "",
  unit: "service",
  cost: "0",
  price: "",
  tax: "16",
  priceIncludesTax: true,
};

export function CatalogPage() {
  const queryClient = useQueryClient();
  const [form, setForm] = useState<CreateForm>(emptyForm);
  const catalog = useQuery({ queryKey: ["catalog"], queryFn: api.listCatalog });

  const createItem = useMutation({
    mutationFn: () =>
      api.createCatalogItem({
        name: form.name.trim(),
        category: form.category.trim() || undefined,
        variant: {
          name: form.variantName.trim(),
          sku: form.sku.trim(),
          barcode: form.barcode.trim() || undefined,
          unit: form.unit,
          costMinor: toMinor(form.cost),
          price: {
            amountMinor: toMinor(form.price),
            taxRateBps: Math.round(Number(form.tax) * 100),
            priceIncludesTax: form.priceIncludesTax,
          },
        },
      }),
    onSuccess: () => {
      setForm(emptyForm);
      void queryClient.invalidateQueries({ queryKey: ["catalog"] });
    },
  });

  const addVariant = useMutation({
    mutationFn: api.addCatalogVariant,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["catalog"] });
    },
  });

  const updateProduct = useMutation({
    mutationFn: api.updateCatalogProduct,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["catalog"] });
    },
  });

  const updateVariant = useMutation({
    mutationFn: api.updateCatalogVariant,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["catalog"] });
    },
  });

  const addPrice = useMutation({
    mutationFn: api.addCatalogPrice,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: ["catalog"] });
    },
  });

  const error = [createItem, addVariant, updateProduct, updateVariant, addPrice].find(
    (mutation) => mutation.isError,
  )?.error;
  const canCreate =
    form.name.trim().length >= 2 &&
    form.variantName.trim().length >= 1 &&
    form.sku.trim().length >= 1 &&
    validMoney(form.price) &&
    validMoney(form.cost) &&
    validPercent(form.tax);

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold tracking-tight">Catálogo</h1>
        <p className="mt-1 text-sm text-slate-600">
          Precios e impuestos configurables. Cambiar un precio crea una versión nueva y
          conserva las ventas anteriores.
        </p>
      </div>

      <form
        className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm"
        onSubmit={(event) => {
          event.preventDefault();
          createItem.mutate();
        }}
      >
        <h2 className="text-base font-semibold">Nuevo producto</h2>
        <div className="mt-4 grid gap-4 sm:grid-cols-2 lg:grid-cols-4">
          <Field label="Nombre del producto">
            <input
              value={form.name}
              onChange={(event) => setForm({ ...form, name: event.target.value })}
              placeholder="Membresía mensual"
              className={inputClass}
            />
          </Field>
          <Field label="Categoría">
            <input
              value={form.category}
              onChange={(event) => setForm({ ...form, category: event.target.value })}
              placeholder="Membresías"
              className={inputClass}
            />
          </Field>
          <Field label="Nombre de la variante">
            <input
              value={form.variantName}
              onChange={(event) => setForm({ ...form, variantName: event.target.value })}
              placeholder="Mensual"
              className={inputClass}
            />
          </Field>
          <Field label="SKU">
            <input
              value={form.sku}
              onChange={(event) => setForm({ ...form, sku: event.target.value })}
              placeholder="MEM-MEN"
              className={inputClass}
            />
          </Field>
          <Field label="Código de barras">
            <input
              value={form.barcode}
              onChange={(event) => setForm({ ...form, barcode: event.target.value })}
              placeholder="Opcional"
              className={inputClass}
            />
          </Field>
          <Field label="Unidad">
            <select
              value={form.unit}
              onChange={(event) =>
                setForm({ ...form, unit: event.target.value as CatalogUnit })
              }
              className={inputClass}
            >
              {Object.entries(unitLabels).map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </select>
          </Field>
          <Field label="Costo">
            <input
              inputMode="decimal"
              value={form.cost}
              onChange={(event) => setForm({ ...form, cost: event.target.value })}
              className={inputClass}
            />
          </Field>
          <Field label="Precio">
            <input
              inputMode="decimal"
              value={form.price}
              onChange={(event) => setForm({ ...form, price: event.target.value })}
              placeholder="350.00"
              className={inputClass}
            />
          </Field>
          <Field label="Impuesto (%)">
            <input
              inputMode="decimal"
              value={form.tax}
              onChange={(event) => setForm({ ...form, tax: event.target.value })}
              className={inputClass}
            />
          </Field>
          <label className="flex items-end gap-2 pb-2 text-sm text-slate-700">
            <input
              type="checkbox"
              checked={form.priceIncludesTax}
              onChange={(event) =>
                setForm({ ...form, priceIncludesTax: event.target.checked })
              }
              className="h-4 w-4 rounded border-slate-300"
            />
            El precio ya incluye impuesto
          </label>
        </div>
        <div className="mt-4 flex flex-wrap items-center gap-3">
          <button
            type="submit"
            disabled={!canCreate || createItem.isPending}
            className="rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-700 disabled:opacity-50"
          >
            {createItem.isPending ? "Creando…" : "Crear producto"}
          </button>
          {createItem.isSuccess && (
            <p className="text-sm text-emerald-700">Producto agregado al catálogo.</p>
          )}
        </div>
      </form>

      {error && (
        <div className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
          {errorMessage(error)}
        </div>
      )}

      <section className="space-y-3">
        <div className="flex items-center justify-between">
          <h2 className="text-base font-semibold">Productos</h2>
          <span className="text-xs text-slate-500">
            {catalog.data?.length ?? 0} producto(s)
          </span>
        </div>
        {catalog.isError ? (
          <p className="rounded-2xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
            {errorMessage(catalog.error)}
          </p>
        ) : catalog.isLoading ? (
          <p className="text-sm text-slate-500">Cargando catálogo…</p>
        ) : catalog.data && catalog.data.length > 0 ? (
          catalog.data.map((product) => (
            <ProductCard
              key={`${product.id}-${product.status}-${product.updatedAt}`}
              product={product}
              saving={
                addVariant.isPending ||
                updateProduct.isPending ||
                updateVariant.isPending ||
                addPrice.isPending
              }
              onAddVariant={(variant) =>
                addVariant.mutate({ productId: product.id, variant })
              }
              onUpdateProduct={(name, category, status) =>
                updateProduct.mutate({ productId: product.id, name, category, status })
              }
              onUpdateVariant={(variant, update) =>
                updateVariant.mutate({
                  variantId: variant.id,
                  name: update.name,
                  sku: update.sku,
                  barcode: update.barcode,
                  unit: update.unit,
                  costMinor: update.costMinor,
                  status: update.status,
                })
              }
              onAddPrice={(variantId, amount, tax, priceIncludesTax, effectiveFrom) =>
                addPrice.mutate({
                  variantId,
                  price: {
                    amountMinor: amount,
                    taxRateBps: tax,
                    priceIncludesTax,
                    effectiveFrom,
                  },
                })
              }
            />
          ))
        ) : (
          <p className="rounded-2xl border border-slate-200 bg-white p-6 text-sm text-slate-500">
            Aún no hay productos. Crea el primero para empezar a cobrar.
          </p>
        )}
      </section>
    </div>
  );
}

function ProductCard({
  product,
  saving,
  onAddVariant,
  onUpdateProduct,
  onUpdateVariant,
  onAddPrice,
}: {
  product: CatalogProduct;
  saving: boolean;
  onAddVariant: (variant: CatalogVariantInput) => void;
  onUpdateProduct: (name: string, category: string | undefined, status: CatalogStatus) => void;
  onUpdateVariant: (
    variant: CatalogVariant,
    update: {
      name: string;
      sku: string;
      barcode: string | undefined;
      unit: CatalogUnit;
      costMinor: number;
      status: CatalogStatus;
    },
  ) => void;
  onAddPrice: (
    variantId: string,
    amount: number,
    tax: number,
    priceIncludesTax: boolean,
    effectiveFrom: string | undefined,
  ) => void;
}) {
  const [name, setName] = useState(product.name);
  const [category, setCategory] = useState(product.category ?? "");
  const [addingVariant, setAddingVariant] = useState(false);
  const nextStatus: CatalogStatus = product.status === "active" ? "archived" : "active";
  const dirty =
    name.trim() !== product.name || category.trim() !== (product.category ?? "");

  return (
    <article className="overflow-hidden rounded-2xl border border-slate-200 bg-white shadow-sm">
      <header className="flex flex-col gap-3 border-b border-slate-200 px-6 py-4 sm:flex-row sm:items-center">
        <div className="grid flex-1 gap-2 sm:grid-cols-2">
          <input
            aria-label={`Nombre de ${product.name}`}
            value={name}
            onChange={(event) => setName(event.target.value)}
            className={inputClass}
          />
          <input
            aria-label={`Categoría de ${product.name}`}
            value={category}
            onChange={(event) => setCategory(event.target.value)}
            placeholder="Sin categoría"
            className={inputClass}
          />
        </div>
        <span
          className={[
            "self-start rounded-full px-3 py-1 text-xs font-semibold ring-1 ring-inset",
            product.status === "active"
              ? "bg-emerald-50 text-emerald-700 ring-emerald-600/20"
              : "bg-slate-100 text-slate-600 ring-slate-500/20",
          ].join(" ")}
        >
          {statusLabels[product.status]}
        </span>
        <div className="flex gap-2">
          <button
            type="button"
            disabled={!dirty || saving}
            onClick={() =>
              onUpdateProduct(name.trim(), category.trim() || undefined, product.status)
            }
            className={secondaryButton}
          >
            Guardar
          </button>
          <button
            type="button"
            disabled={saving}
            onClick={() => onUpdateProduct(product.name, product.category ?? undefined, nextStatus)}
            className={secondaryButton}
          >
            {product.status === "active" ? "Archivar" : "Reactivar"}
          </button>
        </div>
      </header>

      <div className="divide-y divide-slate-100">
        {product.variants.map((variant) => (
          <VariantRow
            key={`${variant.id}-${variant.updatedAt}-${variant.currentPrice?.id ?? "none"}`}
            variant={variant}
            saving={saving}
            onUpdate={(update) => onUpdateVariant(variant, update)}
            productArchived={product.status === "archived"}
            onAddPrice={(amount, tax, includesTax, effectiveFrom) =>
              onAddPrice(variant.id, amount, tax, includesTax, effectiveFrom)
            }
          />
        ))}
        {product.variants.length === 0 && (
          <p className="px-6 py-4 text-sm text-slate-500">Este producto no tiene variantes.</p>
        )}
      </div>

      {product.status === "active" && (
        <footer className="border-t border-slate-200 bg-slate-50 px-6 py-4">
          {addingVariant ? (
            <NewVariantForm
              saving={saving}
              onCancel={() => setAddingVariant(false)}
              onSubmit={onAddVariant}
            />
          ) : (
            <button
              type="button"
              onClick={() => setAddingVariant(true)}
              className={secondaryButton}
            >
              Agregar variante
            </button>
          )}
        </footer>
      )}
    </article>
  );
}

function VariantRow({
  variant,
  productArchived,
  saving,
  onUpdate,
  onAddPrice,
}: {
  variant: CatalogVariant;
  productArchived: boolean;
  saving: boolean;
  onUpdate: (update: {
    name: string;
    sku: string;
    barcode: string | undefined;
    unit: CatalogUnit;
    costMinor: number;
    status: CatalogStatus;
  }) => void;
  onAddPrice: (
    amount: number,
    tax: number,
    includesTax: boolean,
    effectiveFrom: string | undefined,
  ) => void;
}) {
  const [editing, setEditing] = useState(false);
  const [pricing, setPricing] = useState(false);
  const [name, setName] = useState(variant.name);
  const [sku, setSku] = useState(variant.sku);
  const [barcode, setBarcode] = useState(variant.barcode ?? "");
  const [unit, setUnit] = useState(variant.unit);
  const [cost, setCost] = useState(String(variant.costMinor / 100));
  const [status, setStatus] = useState(variant.status);
  const [price, setPrice] = useState(
    variant.currentPrice ? String(variant.currentPrice.amountMinor / 100) : "",
  );
  const [tax, setTax] = useState(
    variant.currentPrice ? String(variant.currentPrice.taxRateBps / 100) : "16",
  );
  const [includesTax, setIncludesTax] = useState(
    variant.currentPrice?.priceIncludesTax ?? true,
  );
  const [effectiveFrom, setEffectiveFrom] = useState("");

  return (
    <div className="px-6 py-4">
      {editing ? (
        <form
          className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4"
          onSubmit={(event) => {
            event.preventDefault();
            onUpdate({
              name: name.trim(),
              sku: sku.trim(),
              barcode: barcode.trim() || undefined,
              unit,
              costMinor: toMinor(cost),
              status,
            });
            setEditing(false);
          }}
        >
          <Field label="Variante">
            <input
              value={name}
              onChange={(event) => setName(event.target.value)}
              className={inputClass}
            />
          </Field>
          <Field label="SKU">
            <input
              value={sku}
              onChange={(event) => setSku(event.target.value)}
              className={inputClass}
            />
          </Field>
          <Field label="Código de barras">
            <input
              value={barcode}
              onChange={(event) => setBarcode(event.target.value)}
              className={inputClass}
            />
          </Field>
          <Field label="Unidad">
            <select
              value={unit}
              onChange={(event) => setUnit(event.target.value as CatalogUnit)}
              className={inputClass}
            >
              {Object.entries(unitLabels).map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </select>
          </Field>
          <Field label="Costo">
            <input
              inputMode="decimal"
              value={cost}
              onChange={(event) => setCost(event.target.value)}
              className={inputClass}
            />
          </Field>
          <Field label="Estado">
            <select
              value={status}
              onChange={(event) => setStatus(event.target.value as CatalogStatus)}
              className={inputClass}
            >
              {Object.entries(statusLabels).map(([value, label]) => (
                <option key={value} value={value}>
                  {label}
                </option>
              ))}
            </select>
          </Field>
          <div className="flex items-end gap-2 sm:col-span-2">
            <button type="submit" disabled={saving} className={primaryButton}>
              Guardar variante
            </button>
            <button
              type="button"
              onClick={() => setEditing(false)}
              className={secondaryButton}
            >
              Cancelar
            </button>
          </div>
        </form>
      ) : (
        <div className="flex flex-col gap-3 sm:flex-row sm:items-center">
          <div className="flex-1">
            <p className="font-medium">
              {variant.name}{" "}
              <span className="text-xs font-normal text-slate-500">{variant.sku}</span>
            </p>
            <p className="text-xs text-slate-500">
              {unitLabels[variant.unit]} · Costo {formatMinor(variant.costMinor, "MXN")} ·{" "}
              {statusLabels[variant.status]}
              {variant.barcode ? ` · ${variant.barcode}` : ""}
            </p>
          </div>
          <div className="text-left sm:text-right">
            {variant.currentPrice ? (
              <>
                <p className="font-semibold">
                  {formatMinor(
                    variant.currentPrice.amountMinor,
                    variant.currentPrice.currency,
                  )}
                </p>
                <p className="text-xs text-slate-500">
                  Impuesto {formatTax(variant.currentPrice.taxRateBps)} ·{" "}
                  {variant.currentPrice.priceIncludesTax ? "incluido" : "agregado"}
                </p>
              </>
            ) : (
              <p className="text-sm text-amber-700">Sin precio vigente</p>
            )}
          </div>
          <div className="flex flex-wrap gap-2">
            <button
              type="button"
              onClick={() => setPricing((value) => !value)}
              disabled={
                productArchived || variant.status === "archived"
              }
              title={
                productArchived || variant.status === "archived"
                  ? "Reactiva el producto y la variante para agregar precios."
                  : undefined
              }
              className={secondaryButton}
            >
              {pricing ? "Cerrar precio" : "Nuevo precio"}
            </button>
            <button type="button" onClick={() => setEditing(true)} className={secondaryButton}>
              Editar
            </button>
          </div>
        </div>
      )}

      {pricing && (
        <form
          className="mt-4 grid gap-3 rounded-xl border border-slate-200 bg-slate-50 p-4 sm:grid-cols-2 lg:grid-cols-5"
          onSubmit={(event) => {
            event.preventDefault();
            onAddPrice(
              toMinor(price),
              Math.round(Number(tax) * 100),
              includesTax,
              effectiveFrom ? new Date(effectiveFrom).toISOString() : undefined,
            );
            setPricing(false);
          }}
        >
          <Field label="Nuevo precio">
            <input
              inputMode="decimal"
              value={price}
              onChange={(event) => setPrice(event.target.value)}
              className={inputClass}
            />
          </Field>
          <Field label="Impuesto (%)">
            <input
              inputMode="decimal"
              value={tax}
              onChange={(event) => setTax(event.target.value)}
              className={inputClass}
            />
          </Field>
          <Field label="Vigente desde">
            <input
              type="datetime-local"
              value={effectiveFrom}
              onChange={(event) => setEffectiveFrom(event.target.value)}
              className={inputClass}
            />
          </Field>
          <label className="flex items-end gap-2 pb-2 text-sm text-slate-700">
            <input
              type="checkbox"
              checked={includesTax}
              onChange={(event) => setIncludesTax(event.target.checked)}
              className="h-4 w-4 rounded border-slate-300"
            />
            Impuesto incluido
          </label>
          <div className="flex items-end">
            <button
              type="submit"
              disabled={
                saving ||
                !validMoney(price) ||
                !validPercent(tax)
              }
              className={primaryButton}
            >
              Guardar precio
            </button>
          </div>
        </form>
      )}
    </div>
  );
}

function NewVariantForm({
  saving,
  onCancel,
  onSubmit,
}: {
  saving: boolean;
  onCancel: () => void;
  onSubmit: (variant: CatalogVariantInput) => void;
}) {
  const [name, setName] = useState("");
  const [sku, setSku] = useState("");
  const [barcode, setBarcode] = useState("");
  const [unit, setUnit] = useState<CatalogUnit>("service");
  const [cost, setCost] = useState("0");
  const [price, setPrice] = useState("");
  const [tax, setTax] = useState("16");
  const [includesTax, setIncludesTax] = useState(true);
  const valid =
    name.trim().length >= 1 &&
    sku.trim().length >= 1 &&
    validMoney(cost) &&
    validMoney(price) &&
    validPercent(tax);

  return (
    <form
      className="grid gap-3 sm:grid-cols-2 lg:grid-cols-4"
      onSubmit={(event) => {
        event.preventDefault();
        onSubmit({
          name: name.trim(),
          sku: sku.trim(),
          barcode: barcode.trim() || undefined,
          unit,
          costMinor: toMinor(cost),
          price: {
            amountMinor: toMinor(price),
            taxRateBps: Math.round(Number(tax) * 100),
            priceIncludesTax: includesTax,
          },
        });
      }}
    >
      <Field label="Nombre de la nueva variante">
        <input
          value={name}
          onChange={(event) => setName(event.target.value)}
          className={inputClass}
        />
      </Field>
      <Field label="SKU de la nueva variante">
        <input
          value={sku}
          onChange={(event) => setSku(event.target.value)}
          className={inputClass}
        />
      </Field>
      <Field label="Código de barras de la nueva variante">
        <input
          value={barcode}
          onChange={(event) => setBarcode(event.target.value)}
          placeholder="Opcional"
          className={inputClass}
        />
      </Field>
      <Field label="Unidad de la nueva variante">
        <select
          value={unit}
          onChange={(event) => setUnit(event.target.value as CatalogUnit)}
          className={inputClass}
        >
          {Object.entries(unitLabels).map(([value, label]) => (
            <option key={value} value={value}>
              {label}
            </option>
          ))}
        </select>
      </Field>
      <Field label="Costo de la nueva variante">
        <input
          inputMode="decimal"
          value={cost}
          onChange={(event) => setCost(event.target.value)}
          className={inputClass}
        />
      </Field>
      <Field label="Precio de la nueva variante">
        <input
          inputMode="decimal"
          value={price}
          onChange={(event) => setPrice(event.target.value)}
          className={inputClass}
        />
      </Field>
      <Field label="Impuesto de la nueva variante (%)">
        <input
          inputMode="decimal"
          value={tax}
          onChange={(event) => setTax(event.target.value)}
          className={inputClass}
        />
      </Field>
      <label className="flex items-end gap-2 pb-2 text-sm text-slate-700">
        <input
          type="checkbox"
          checked={includesTax}
          onChange={(event) => setIncludesTax(event.target.checked)}
          className="h-4 w-4 rounded border-slate-300"
        />
        La nueva variante incluye impuesto
      </label>
      <div className="flex items-end gap-2 sm:col-span-2">
        <button type="submit" disabled={!valid || saving} className={primaryButton}>
          Crear variante
        </button>
        <button type="button" onClick={onCancel} className={secondaryButton}>
          Cancelar
        </button>
      </div>
    </form>
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

const inputClass =
  "w-full rounded-lg border border-slate-300 px-3 py-2 text-sm outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100";
const primaryButton =
  "rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-700 disabled:opacity-50";
const secondaryButton =
  "rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium text-slate-700 hover:bg-slate-100 disabled:opacity-50";

function validMoney(value: string): boolean {
  if (!value.trim()) {
    return false;
  }
  const amount = Number(value);
  return Number.isFinite(amount) && amount >= 0;
}

function validPercent(value: string): boolean {
  if (!value.trim()) {
    return false;
  }
  const amount = Number(value);
  return Number.isFinite(amount) && amount >= 0 && amount <= 100;
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

function formatTax(bps: number): string {
  return `${(bps / 100).toFixed(2)}%`;
}
