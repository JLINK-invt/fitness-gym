import type { Equipment, MaintenanceOrder } from "@gym/contracts";
import { errorMessage } from "@gym/contracts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState } from "react";
import { useOutletContext } from "react-router-dom";

import type { LayoutContext } from "../components/Layout";
import { api } from "../lib/tauri";

const fieldClass =
  "w-full rounded-lg border border-slate-300 px-3 py-2 text-sm outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100";
const primaryButton =
  "rounded-lg bg-brand-600 px-4 py-2 text-sm font-semibold text-white hover:bg-brand-700 disabled:opacity-50";
const secondaryButton =
  "rounded-lg border border-slate-300 px-4 py-2 text-sm font-semibold text-slate-700 hover:bg-slate-50 disabled:opacity-50";

const equipmentStatusLabels: Record<string, string> = {
  operational: "Operativo",
  maintenance: "En mantenimiento",
  out_of_service: "Fuera de servicio",
};

const orderStatusLabels: Record<string, string> = {
  open: "Abierta",
  in_progress: "En progreso",
  completed: "Completada",
  cancelled: "Cancelada",
};

const priorityLabels: Record<string, string> = {
  low: "Baja",
  normal: "Normal",
  high: "Alta",
};

export function MaintenancePage() {
  const queryClient = useQueryClient();
  const { session } = useOutletContext<LayoutContext>();
  const canManage = session.permissions.includes("manageMaintenance");
  const [equipmentName, setEquipmentName] = useState("");
  const [equipmentCode, setEquipmentCode] = useState("");
  const [equipmentArea, setEquipmentArea] = useState("");
  const [orderKind, setOrderKind] = useState("preventive");
  const [orderPriority, setOrderPriority] = useState("normal");
  const [orderSummary, setOrderSummary] = useState("");
  const [orderDetail, setOrderDetail] = useState("");
  const [orderDueOn, setOrderDueOn] = useState("");
  const [orderEquipmentId, setOrderEquipmentId] = useState("");
  const [statusFilter, setStatusFilter] = useState("");
  const [error, setError] = useState<string | null>(null);

  const equipment = useQuery<Equipment[]>({
    queryKey: ["equipment"],
    queryFn: api.listEquipment,
  });
  const orders = useQuery<MaintenanceOrder[]>({
    queryKey: ["maintenance-orders", statusFilter],
    queryFn: () => api.listMaintenanceOrders(statusFilter || undefined),
  });

  const refresh = () => {
    void queryClient.invalidateQueries({ queryKey: ["equipment"] });
    void queryClient.invalidateQueries({ queryKey: ["maintenance-orders"] });
  };

  const createEquipment = useMutation({
    mutationFn: () =>
      api.createEquipment({
        name: equipmentName,
        code: equipmentCode || null,
        area: equipmentArea || null,
      }),
    onSuccess: () => {
      setError(null);
      setEquipmentName("");
      setEquipmentCode("");
      setEquipmentArea("");
      refresh();
    },
    onError: (cause) => setError(errorMessage(cause)),
  });

  const changeEquipmentStatus = useMutation({
    mutationFn: (input: { item: Equipment; status: string }) =>
      api.updateEquipment({
        equipmentId: input.item.id,
        name: input.item.name,
        code: input.item.code,
        area: input.item.area,
        status: input.status,
        acquiredOn: input.item.acquiredOn,
        notes: input.item.notes,
      }),
    onSuccess: () => {
      setError(null);
      refresh();
    },
    onError: (cause) => setError(errorMessage(cause)),
  });

  const createOrder = useMutation({
    mutationFn: () =>
      api.createMaintenanceOrder({
        equipmentId: orderEquipmentId || null,
        kind: orderKind,
        priority: orderPriority,
        summary: orderSummary,
        detail: orderDetail || null,
        dueOn: orderDueOn || null,
      }),
    onSuccess: () => {
      setError(null);
      setOrderSummary("");
      setOrderDetail("");
      setOrderDueOn("");
      refresh();
    },
    onError: (cause) => setError(errorMessage(cause)),
  });

  const changeOrderStatus = useMutation({
    mutationFn: (input: { order: MaintenanceOrder; status: string }) =>
      api.updateMaintenanceOrder({
        orderId: input.order.id,
        status: input.status,
        priority: input.order.priority,
        summary: input.order.summary,
        detail: input.order.detail,
        dueOn: input.order.dueOn,
      }),
    onSuccess: () => {
      setError(null);
      refresh();
    },
    onError: (cause) => setError(errorMessage(cause)),
  });

  return (
    <div className="space-y-6">
      <p className="text-sm text-slate-600">
        Inventario de equipos, estado operativo y órdenes de mantenimiento preventivo o
        correctivo.
      </p>

      {error && (
        <div className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
          {error}
        </div>
      )}

      <div className="grid gap-6 lg:grid-cols-2">
        <section className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
          <h2 className="text-sm font-semibold text-slate-900">Equipos</h2>
          {canManage && (
            <form
              className="mt-4 grid gap-2 sm:grid-cols-[1.4fr_1fr_1fr_auto]"
              onSubmit={(event) => {
                event.preventDefault();
                createEquipment.mutate();
              }}
            >
              <input
                className={fieldClass}
                placeholder="Nombre"
                value={equipmentName}
                onChange={(event) => setEquipmentName(event.target.value)}
                required
              />
              <input
                className={fieldClass}
                placeholder="Código"
                value={equipmentCode}
                onChange={(event) => setEquipmentCode(event.target.value)}
              />
              <input
                className={fieldClass}
                placeholder="Área"
                value={equipmentArea}
                onChange={(event) => setEquipmentArea(event.target.value)}
              />
              <button className={primaryButton} disabled={createEquipment.isPending}>
                Agregar
              </button>
            </form>
          )}
          <ul className="mt-4 space-y-2">
            {(equipment.data ?? []).map((item) => (
              <li
                key={item.id}
                className="flex flex-wrap items-center justify-between gap-2 rounded-xl border border-slate-200 p-3 text-sm"
              >
                <div>
                  <p className="font-medium text-slate-900">
                    {item.name}
                    {item.code ? ` · ${item.code}` : ""}
                  </p>
                  <p className="text-xs text-slate-500">
                    {equipmentStatusLabels[item.status] ?? item.status}
                    {item.area ? ` · ${item.area}` : ""} · {item.openOrders} órdenes abiertas
                  </p>
                </div>
                {canManage && (
                  <select
                    className="rounded-lg border border-slate-300 px-2 py-1 text-xs"
                    value={item.status}
                    onChange={(event) =>
                      changeEquipmentStatus.mutate({
                        item,
                        status: event.target.value,
                      })
                    }
                  >
                    {Object.entries(equipmentStatusLabels).map(([value, label]) => (
                      <option key={value} value={value}>
                        {label}
                      </option>
                    ))}
                  </select>
                )}
              </li>
            ))}
            {(equipment.data ?? []).length === 0 && (
              <li className="text-sm text-slate-500">Aún no hay equipos registrados.</li>
            )}
          </ul>
        </section>

        <section className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
          <h2 className="text-sm font-semibold text-slate-900">Nueva orden</h2>
          {canManage ? (
            <form
              className="mt-4 space-y-3"
              onSubmit={(event) => {
                event.preventDefault();
                createOrder.mutate();
              }}
            >
              <div className="grid gap-3 sm:grid-cols-2">
                <label className="block text-xs font-medium text-slate-600">
                  Equipo
                  <select
                    className={`${fieldClass} mt-1`}
                    value={orderEquipmentId}
                    onChange={(event) => setOrderEquipmentId(event.target.value)}
                  >
                    <option value="">Sin equipo</option>
                    {(equipment.data ?? []).map((item) => (
                      <option key={item.id} value={item.id}>
                        {item.name}
                      </option>
                    ))}
                  </select>
                </label>
                <label className="block text-xs font-medium text-slate-600">
                  Tipo
                  <select
                    className={`${fieldClass} mt-1`}
                    value={orderKind}
                    onChange={(event) => setOrderKind(event.target.value)}
                  >
                    <option value="preventive">Preventivo</option>
                    <option value="corrective">Correctivo</option>
                  </select>
                </label>
                <label className="block text-xs font-medium text-slate-600">
                  Prioridad
                  <select
                    className={`${fieldClass} mt-1`}
                    value={orderPriority}
                    onChange={(event) => setOrderPriority(event.target.value)}
                  >
                    {Object.entries(priorityLabels).map(([value, label]) => (
                      <option key={value} value={value}>
                        {label}
                      </option>
                    ))}
                  </select>
                </label>
                <label className="block text-xs font-medium text-slate-600">
                  Vence
                  <input
                    type="date"
                    className={`${fieldClass} mt-1`}
                    value={orderDueOn}
                    onChange={(event) => setOrderDueOn(event.target.value)}
                  />
                </label>
              </div>
              <label className="block text-xs font-medium text-slate-600">
                Resumen
                <input
                  className={`${fieldClass} mt-1`}
                  value={orderSummary}
                  onChange={(event) => setOrderSummary(event.target.value)}
                  required
                />
              </label>
              <label className="block text-xs font-medium text-slate-600">
                Detalle
                <textarea
                  className={`${fieldClass} mt-1`}
                  rows={2}
                  value={orderDetail}
                  onChange={(event) => setOrderDetail(event.target.value)}
                />
              </label>
              <button className={primaryButton} disabled={createOrder.isPending}>
                {createOrder.isPending ? "Guardando…" : "Crear orden"}
              </button>
            </form>
          ) : (
            <p className="mt-3 text-sm text-slate-500">
              Tu rol puede consultar el mantenimiento, pero no crear órdenes.
            </p>
          )}
        </section>
      </div>

      <section className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
        <div className="flex flex-wrap items-center justify-between gap-3">
          <h2 className="text-sm font-semibold text-slate-900">Órdenes de mantenimiento</h2>
          <select
            className="rounded-lg border border-slate-300 px-2 py-1 text-xs"
            value={statusFilter}
            onChange={(event) => setStatusFilter(event.target.value)}
          >
            <option value="">Todas</option>
            {Object.entries(orderStatusLabels).map(([value, label]) => (
              <option key={value} value={value}>
                {label}
              </option>
            ))}
          </select>
        </div>
        <div className="mt-4 overflow-x-auto">
          <table className="min-w-full text-sm">
            <thead>
              <tr className="text-left text-xs uppercase tracking-wide text-slate-500">
                <th className="py-2 pr-4">Resumen</th>
                <th className="py-2 pr-4">Equipo</th>
                <th className="py-2 pr-4">Tipo</th>
                <th className="py-2 pr-4">Prioridad</th>
                <th className="py-2 pr-4">Vence</th>
                <th className="py-2 pr-4">Estado</th>
                <th className="py-2" />
              </tr>
            </thead>
            <tbody className="divide-y divide-slate-100">
              {(orders.data ?? []).map((order) => (
                <tr key={order.id}>
                  <td className="py-2 pr-4 font-medium text-slate-900">{order.summary}</td>
                  <td className="py-2 pr-4 text-slate-600">
                    {order.equipmentName ?? "General"}
                  </td>
                  <td className="py-2 pr-4 text-slate-600">
                    {order.kind === "preventive" ? "Preventivo" : "Correctivo"}
                  </td>
                  <td className="py-2 pr-4 text-slate-600">
                    {priorityLabels[order.priority] ?? order.priority}
                  </td>
                  <td className="py-2 pr-4 text-slate-500">{order.dueOn ?? "—"}</td>
                  <td className="py-2 pr-4 text-slate-600">
                    {orderStatusLabels[order.status] ?? order.status}
                  </td>
                  <td className="py-2 text-right">
                    {canManage && order.status !== "completed" && order.status !== "cancelled" && (
                      <span className="flex justify-end gap-2">
                        {order.status === "open" && (
                          <button
                            type="button"
                            className={secondaryButton}
                            disabled={changeOrderStatus.isPending}
                            onClick={() =>
                              changeOrderStatus.mutate({ order, status: "in_progress" })
                            }
                          >
                            Iniciar
                          </button>
                        )}
                        <button
                          type="button"
                          className={secondaryButton}
                          disabled={changeOrderStatus.isPending}
                          onClick={() =>
                            changeOrderStatus.mutate({ order, status: "completed" })
                          }
                        >
                          Completar
                        </button>
                      </span>
                    )}
                  </td>
                </tr>
              ))}
              {(orders.data ?? []).length === 0 && (
                <tr>
                  <td className="py-4 text-sm text-slate-500" colSpan={7}>
                    Sin órdenes para este filtro.
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
