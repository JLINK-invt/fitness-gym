import type { Permission, SessionSummary } from "@gym/contracts";
import { CONTRACT_VERSION } from "@gym/contracts";
import { useEffect, useMemo, useRef, useState } from "react";
import { createPortal } from "react-dom";
import { NavLink, Outlet, useLocation } from "react-router-dom";

import { roleLabels } from "../lib/permissions";
import { isTauriRuntime } from "../lib/tauri";
import { ChevronDownIcon, SignOutIcon } from "./icons";

export type LayoutContext = {
  session: SessionSummary;
};

type NavigationItem = {
  to: string;
  label: string;
  title?: string;
  permission?: Permission;
  anyPermission?: Permission[];
};

type NavigationGroup = {
  label: string;
  items: NavigationItem[];
};

const navigationGroups: NavigationGroup[] = [
  {
    label: "Inicio",
    items: [
      { to: "/", label: "Panel", title: "Panel" },
      { to: "/reports", label: "Reportes", title: "Reportes", permission: "viewReports" },
    ],
  },
  {
    label: "Socios",
    items: [
      {
        to: "/members",
        label: "Socios",
        title: "Socios y membresías",
        permission: "viewMembers",
      },
      {
        to: "/access",
        label: "Acceso y asistencia",
        title: "Accesos y asistencia",
        permission: "viewAccess",
      },
      {
        to: "/classes",
        label: "Clases y reservas",
        title: "Clases y reservas",
        permission: "viewClasses",
      },
    ],
  },
  {
    label: "Ventas y caja",
    items: [
      { to: "/sales", label: "Ventas", title: "Ventas", permission: "viewSales" },
      {
        to: "/cash",
        label: "Caja",
        title: "Caja y turnos",
        anyPermission: ["operatePos", "superviseCash", "viewFinance"],
      },
      {
        to: "/inventory",
        label: "Inventario",
        title: "Inventario",
        permission: "viewInventory",
      },
      {
        to: "/catalog",
        label: "Catálogo",
        title: "Catálogo",
        permission: "viewCatalog",
      },
    ],
  },
  {
    label: "Marketing",
    items: [
      {
        to: "/marketing",
        label: "Marketing y retención",
        title: "Marketing y retención",
        permission: "viewMarketing",
      },
    ],
  },
  {
    label: "Finanzas",
    items: [
      {
        to: "/finance",
        label: "Finanzas",
        title: "Finanzas",
        permission: "viewFinance",
      },
    ],
  },
  {
    label: "Configuración",
    items: [
      {
        to: "/settings",
        label: "General",
        title: "General",
        permission: "viewSettings",
      },
      {
        to: "/staff",
        label: "Colaboradores",
        title: "Colaboradores",
        permission: "viewStaff",
      },
      {
        to: "/maintenance",
        label: "Mantenimiento",
        title: "Mantenimiento",
        permission: "viewMaintenance",
      },
      {
        to: "/integrations",
        label: "Complementos",
        title: "Complementos",
        permission: "viewIntegrations",
      },
      {
        to: "/backups",
        label: "Backups",
        title: "Backups cifrados",
        permission: "manageBackups",
      },
      {
        to: "/audit",
        label: "Auditoría",
        title: "Auditoría",
        permission: "viewAudit",
      },
      {
        to: "/diagnostics",
        label: "Diagnóstico",
        title: "Diagnóstico del core",
      },
    ],
  },
];

const SESSION_MENU = "session";

export function Layout({
  session,
  onLogout,
  signingOut,
}: {
  session: SessionSummary;
  onLogout: () => void;
  signingOut: boolean;
}) {
  const location = useLocation();
  const [openMenu, setOpenMenu] = useState<{ id: string; rect: DOMRect } | null>(
    null,
  );
  const triggers = useRef(new Map<string, HTMLElement>());
  const panelRef = useRef<HTMLDivElement>(null);

  const visibleGroups = useMemo(
    () =>
      navigationGroups
        .map((group) => ({
          ...group,
          items: group.items.filter((item) => {
            if (item.permission) {
              return session.permissions.includes(item.permission);
            }
            if (item.anyPermission) {
              return item.anyPermission.some((permission) =>
                session.permissions.includes(permission),
              );
            }
            return true;
          }),
        }))
        .filter((group) => group.items.length > 0),
    [session.permissions],
  );

  const activeGroup = useMemo(
    () =>
      visibleGroups.find((group) =>
        group.items.some((item) => item.to === location.pathname),
      ) ?? null,
    [location.pathname, visibleGroups],
  );

  const activeItem = useMemo(
    () =>
      visibleGroups
        .flatMap((group) => group.items)
        .find((item) => item.to === location.pathname) ?? null,
    [location.pathname, visibleGroups],
  );

  const openGroup = openMenu
    ? visibleGroups.find((group) => group.label === openMenu.id) ?? null
    : null;

  useEffect(() => {
    setOpenMenu(null);
  }, [location.pathname]);

  useEffect(() => {
    if (!openMenu) {
      return;
    }
    const handlePointerDown = (event: MouseEvent) => {
      const target = event.target as Node;
      if (panelRef.current?.contains(target)) {
        return;
      }
      const trigger = triggers.current.get(openMenu.id);
      if (trigger?.contains(target)) {
        return;
      }
      setOpenMenu(null);
    };
    const handleKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        setOpenMenu(null);
        triggers.current.get(openMenu.id)?.focus();
      }
    };
    const handleViewportChange = () => {
      const trigger = triggers.current.get(openMenu.id);
      if (!trigger) {
        setOpenMenu(null);
        return;
      }
      setOpenMenu((current) =>
        current && current.id === openMenu.id
          ? { ...current, rect: trigger.getBoundingClientRect() }
          : current,
      );
    };
    document.addEventListener("mousedown", handlePointerDown);
    document.addEventListener("keydown", handleKeyDown);
    window.addEventListener("resize", handleViewportChange);
    window.addEventListener("scroll", handleViewportChange, true);
    return () => {
      document.removeEventListener("mousedown", handlePointerDown);
      document.removeEventListener("keydown", handleKeyDown);
      window.removeEventListener("resize", handleViewportChange);
      window.removeEventListener("scroll", handleViewportChange, true);
    };
  }, [openMenu]);

  const openMenuId = openMenu?.id ?? null;

  useEffect(() => {
    if (!openMenuId) {
      return;
    }
    const firstItem = panelRef.current?.querySelector<HTMLElement>(
      "a[role='menuitem'], button[role='menuitem']",
    );
    firstItem?.focus({ preventScroll: true });
  }, [openMenuId]);

  function toggleMenu(id: string) {
    if (openMenu?.id === id) {
      setOpenMenu(null);
      return;
    }
    const trigger = triggers.current.get(id);
    if (!trigger) {
      return;
    }
    setOpenMenu({ id, rect: trigger.getBoundingClientRect() });
  }

  function registerTrigger(id: string, element: HTMLElement | null) {
    if (element) {
      triggers.current.set(id, element);
      return;
    }
    triggers.current.delete(id);
  }

  return (
    <div className="flex h-full flex-col overflow-hidden bg-slate-100 print:h-auto print:overflow-visible">
      <nav
        aria-label="Secciones"
        className="flex h-9 shrink-0 items-end gap-0.5 bg-slate-100 px-2 print:hidden"
      >
        <div
          className="flex min-w-0 flex-1 items-end gap-0.5 overflow-x-auto"
          style={{ scrollbarWidth: "none" }}
        >
          {visibleGroups.map((group) => {
            const active = activeGroup?.label === group.label;
            const expanded = openMenu?.id === group.label;
            return (
              <button
                key={group.label}
                type="button"
                ref={(element) => registerTrigger(group.label, element)}
                aria-haspopup="menu"
                aria-expanded={expanded}
                aria-current={active ? "page" : undefined}
                onClick={() => toggleMenu(group.label)}
                data-merge={active ? "true" : undefined}
                className={[
                  "chrome-tab flex shrink-0 items-center gap-1 rounded-t-[6px] px-3 pb-1 pt-0.5 text-[13px] font-medium transition",
                  "focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-brand-600",
                  active
                    ? "-mb-px bg-white font-semibold text-brand-700"
                    : expanded
                      ? "-mb-px bg-white text-brand-700 shadow-[inset_0_0_0_1px_rgba(148,163,184,0.3)]"
                      : "text-slate-600 hover:bg-white/60 hover:text-slate-900",
                ].join(" ")}
              >
                {group.label}
                <ChevronDownIcon
                  className={`h-3.5 w-3.5 text-slate-400 transition-transform duration-150 ${
                    expanded ? "rotate-180" : ""
                  }`}
                />
              </button>
            );
          })}
        </div>

        <button
          type="button"
          ref={(element) => registerTrigger(SESSION_MENU, element)}
          aria-haspopup="menu"
          aria-expanded={openMenu?.id === SESSION_MENU}
          aria-label={`Sesión de ${session.name}`}
          onClick={() => toggleMenu(SESSION_MENU)}
          className={[
            "my-auto flex shrink-0 items-center gap-2 rounded-lg px-2 py-1.5 transition",
            "focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-brand-600",
            openMenu?.id === SESSION_MENU
              ? "bg-white shadow-[0_1px_2px_rgba(15,23,42,0.08)]"
              : "hover:bg-white/70",
          ].join(" ")}
        >
          <span className="flex h-6 w-6 items-center justify-center rounded-full bg-brand-100 text-[10px] font-semibold text-brand-700">
            {initials(session.name)}
          </span>
          <span className="max-w-[160px] truncate text-[13px] font-medium text-slate-700">
            {session.name}
          </span>
          <ChevronDownIcon
            className={`h-3.5 w-3.5 text-slate-400 transition-transform duration-150 ${
              openMenu?.id === SESSION_MENU ? "rotate-180" : ""
            }`}
          />
        </button>
      </nav>

      <div className="grid h-10 shrink-0 grid-cols-[1fr_auto_1fr] items-center gap-3 border-b border-slate-200 bg-white px-3 print:hidden">
        <nav aria-label="Ruta" className="min-w-0">
          <ol className="flex min-w-0 items-center gap-1.5 text-[11px] text-slate-500">
            <li className="truncate">{activeGroup?.label ?? "Inicio"}</li>
            <li aria-hidden="true" className="text-slate-300">
              /
            </li>
            <li className="truncate text-slate-600">{activeItem?.title ?? "Panel"}</li>
          </ol>
        </nav>
        <h1 className="truncate text-center text-[15px] font-semibold tracking-tight text-slate-900">
          {activeItem?.title ?? "Panel"}
        </h1>
        <div aria-hidden="true" className="min-w-0" />
      </div>

      {!isTauriRuntime() && (
        <div className="border-b border-amber-200 bg-amber-50 px-4 py-1.5 text-center text-[11px] text-amber-800 print:hidden">
          Vista web sin runtime Tauri. Ejecuta{" "}
          <code className="font-mono">pnpm tauri dev</code> para probar los comandos
          Rust, SQLite y backups.
        </div>
      )}

      <main className="flex-1 overflow-y-auto print:overflow-visible">
        <div className="mx-auto w-full max-w-6xl px-6 py-6">
          <Outlet context={{ session } satisfies LayoutContext} />
        </div>
      </main>

      {openMenu &&
        createPortal(
          <div
            ref={panelRef}
            role="menu"
            aria-label={openMenu.id === SESSION_MENU ? "Sesión" : openMenu.id}
            style={
              openMenu.id === SESSION_MENU
                ? {
                    top: openMenu.rect.bottom + 6,
                    right: Math.max(8, window.innerWidth - openMenu.rect.right),
                  }
                : { top: openMenu.rect.bottom + 6, left: openMenu.rect.left }
            }
            className="fixed z-50 min-w-[200px] rounded-xl border border-slate-200 bg-white p-1 shadow-[0_10px_28px_-12px_rgba(15,23,42,0.28)]"
          >
            {openMenu.id !== SESSION_MENU && openGroup ? (
              openGroup.items.map((item) => (
                <NavLink
                  key={item.to}
                  to={item.to}
                  role="menuitem"
                  onClick={() => setOpenMenu(null)}
                  className={({ isActive }) =>
                    [
                      "block rounded-lg px-3 py-1.5 text-[13px] transition",
                      "focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-brand-600",
                      isActive
                        ? "bg-brand-50 font-semibold text-brand-700"
                        : "font-medium text-slate-700 hover:bg-slate-100",
                    ].join(" ")
                  }
                >
                  {item.label}
                </NavLink>
              ))
            ) : (
              <>
                <div className="px-3 py-2">
                  <p className="text-[13px] font-semibold text-slate-900">
                    {session.name}
                  </p>
                  <p className="text-[11px] text-slate-500">
                    {roleLabels[session.role]}
                  </p>
                </div>
                <div className="my-1 border-t border-slate-100" />
                <button
                  type="button"
                  role="menuitem"
                  onClick={onLogout}
                  disabled={signingOut}
                  className="flex w-full items-center gap-2 rounded-lg px-3 py-1.5 text-left text-[13px] font-medium text-slate-700 transition hover:bg-slate-100 focus-visible:outline-2 focus-visible:outline-offset-[-2px] focus-visible:outline-brand-600 disabled:opacity-50"
                >
                  <SignOutIcon className="h-4 w-4 text-slate-400" />
                  {signingOut ? "Saliendo…" : "Cerrar sesión"}
                </button>
                <div className="mt-1 border-t border-slate-100 px-3 py-2 text-[11px] leading-tight text-slate-400">
                  Operación local · SQLite cifrada · Contratos v{CONTRACT_VERSION}
                </div>
              </>
            )}
          </div>,
          document.body,
        )}
    </div>
  );
}

function initials(name: string): string {
  const parts = name.trim().split(/\s+/).filter(Boolean);
  const letters = parts.slice(0, 2).map((part) => part[0]?.toUpperCase() ?? "");
  return letters.join("") || "?";
}
