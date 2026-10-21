import { CONTRACT_VERSION } from "@gym/contracts";
import { NavLink, Outlet } from "react-router-dom";

import { isTauriRuntime } from "../lib/tauri";

const navigation = [
  { to: "/", label: "Diagnóstico" },
  { to: "/access", label: "Acceso" },
  { to: "/backups", label: "Backups" },
];

export function Layout() {
  return (
    <div className="flex min-h-full flex-col">
      <header className="border-b border-slate-200 bg-white">
        <div className="mx-auto flex w-full max-w-6xl items-center justify-between gap-6 px-6 py-4">
          <div className="flex items-center gap-3">
            <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-brand-600 text-lg font-bold text-white">
              FG
            </div>
            <div>
              <p className="text-sm font-semibold tracking-tight">Fitness Gym</p>
              <p className="text-xs text-slate-500">Core desktop · V1</p>
            </div>
          </div>

          <nav className="flex items-center gap-1">
            {navigation.map((item) => (
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
