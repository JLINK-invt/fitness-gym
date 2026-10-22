import type { Permission, SessionSummary, StaffRole } from "@gym/contracts";
import { CONTRACT_VERSION } from "@gym/contracts";
import { NavLink, Outlet } from "react-router-dom";

import { isTauriRuntime } from "../lib/tauri";

type NavigationItem = {
  to: string;
  label: string;
  permission?: Permission;
};

const navigation: NavigationItem[] = [
  { to: "/", label: "Diagnóstico" },
  { to: "/access", label: "Acceso", permission: "operatePos" },
  { to: "/backups", label: "Backups", permission: "manageBackups" },
];

const roleLabels: Record<StaffRole, string> = {
  owner: "Dueño",
  manager: "Encargado",
  cashier: "Cajero",
  accounting: "Contabilidad",
};

export function Layout({
  session,
  onLogout,
  signingOut,
}: {
  session: SessionSummary;
  onLogout: () => void;
  signingOut: boolean;
}) {
  const visibleNavigation = navigation.filter(
    (item) => !item.permission || session.permissions.includes(item.permission),
  );

  return (
    <div className="flex min-h-full flex-col">
      <header className="border-b border-slate-200 bg-white">
        <div className="mx-auto flex w-full max-w-6xl flex-wrap items-center gap-4 px-6 py-4">
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-brand-600 text-lg font-bold text-white">
              FG
            </div>
            <div>
              <p className="text-sm font-semibold tracking-tight">Fitness Gym</p>
              <p className="text-xs text-slate-500">Operación local</p>
            </div>
          </div>

          <nav className="order-3 flex w-full items-center gap-1 overflow-x-auto sm:order-none sm:w-auto sm:flex-1 sm:justify-center">
            {visibleNavigation.map((item) => (
              <NavLink
                key={item.to}
                to={item.to}
                end={item.to === "/"}
                className={({ isActive }) =>
                  [
                    "rounded-lg px-3 py-2 text-sm font-medium transition",
                    isActive
                      ? "bg-brand-50 text-brand-700"
                      : "text-slate-600 hover:bg-slate-100",
                  ].join(" ")
                }
              >
                {item.label}
              </NavLink>
            ))}
          </nav>

          <div className="ml-auto flex items-center gap-3">
            <div className="text-right">
              <p className="text-sm font-medium leading-tight">{session.name}</p>
              <p className="text-xs text-slate-500">{roleLabels[session.role]}</p>
            </div>
            <button
              type="button"
              onClick={onLogout}
              disabled={signingOut}
              className="rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium text-slate-700 transition hover:bg-slate-100 disabled:opacity-50"
            >
              {signingOut ? "Saliendo…" : "Salir"}
            </button>
          </div>
        </div>
      </header>

      {!isTauriRuntime() && (
        <div className="border-b border-amber-200 bg-amber-50 px-6 py-2 text-center text-xs text-amber-800">
          Vista web sin runtime Tauri. Ejecuta <code className="font-mono">pnpm tauri dev</code>{" "}
          para probar los comandos Rust, SQLite y backups.
        </div>
      )}

      <main className="mx-auto w-full max-w-6xl flex-1 px-6 py-8">
        <Outlet />
      </main>

      <footer className="border-t border-slate-200 bg-white px-6 py-3 text-center text-xs text-slate-400">
        Operación local · SQLite cifrada · Contratos v{CONTRACT_VERSION}
      </footer>
    </div>
  );
}
