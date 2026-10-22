import type { HealthReport } from "@gym/contracts";
import { errorMessage } from "@gym/contracts";
import { useQuery } from "@tanstack/react-query";

import { StatusBadge } from "../components/StatusBadge";
import { api, isTauriRuntime } from "../lib/tauri";

export function DiagnosticsPage() {
  const inTauri = isTauriRuntime();

  const health = useQuery<HealthReport>({
    queryKey: ["health"],
    queryFn: api.health,
    enabled: inTauri,
    refetchInterval: 5_000,
  });

  if (!inTauri) {
    return (
      <section className="rounded-2xl border border-slate-200 bg-white p-8 shadow-sm">
        <h1 className="text-xl font-semibold tracking-tight">Diagnóstico del core</h1>
        <p className="mt-2 max-w-2xl text-sm text-slate-600">
          La interfaz está corriendo en el navegador. El chequeo de salud, la SQLite
          cifrada y los backups requieren el proceso Tauri.
        </p>
        <pre className="mt-6 overflow-x-auto rounded-xl bg-slate-900 p-4 text-xs text-slate-100">
{`# 1. Dependencias del sistema (Fedora)
sudo dnf install webkit2gtk4.1-devel gtk3-devel libappindicator-gtk3-devel \\
  librsvg2-devel openssl-devel sqlcipher-devel

# 2. Dependencias JS y app desktop
pnpm install
pnpm tauri dev`}
        </pre>
      </section>
    );
  }

  return (
    <div className="space-y-6">
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <h1 className="text-xl font-semibold tracking-tight">Diagnóstico del core</h1>
          <p className="mt-1 text-sm text-slate-600">
            Health local de Tauri, backend Rust, SQLCipher y contratos.
          </p>
        </div>
        <button
          type="button"
          onClick={() => health.refetch()}
          className="rounded-lg bg-slate-900 px-4 py-2 text-sm font-medium text-white hover:bg-slate-700 disabled:opacity-50"
          disabled={health.isFetching}
        >
          {health.isFetching ? "Consultando…" : "Actualizar"}
        </button>
      </div>

      {health.isError && (
        <div className="rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
          {errorMessage(health.error)}
        </div>
      )}

      {health.data && (
        <>
          <section className="grid gap-4 md:grid-cols-3">
            <article className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
              <p className="text-xs font-medium uppercase tracking-wide text-slate-500">
                Servicio
              </p>
              <p className="mt-2 text-lg font-semibold">{health.data.service}</p>
              <p className="text-sm text-slate-500">v{health.data.version}</p>
              <div className="mt-4">
                <StatusBadge status={health.data.status} />
              </div>
            </article>

            <article className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
              <p className="text-xs font-medium uppercase tracking-wide text-slate-500">
                Contratos
              </p>
              <p className="mt-2 text-lg font-semibold">v{health.data.contractVersion}</p>
              <p className="text-sm text-slate-500">Chequeo {formatTime(health.data.checkedAt)}</p>
            </article>

            <article className="rounded-2xl border border-slate-200 bg-white p-5 shadow-sm">
              <p className="text-xs font-medium uppercase tracking-wide text-slate-500">
                Módulos
              </p>
              <ul className="mt-2 space-y-2">
                {health.data.modules.map((module) => (
                  <li key={module.name} className="flex items-center justify-between gap-3">
                    <span className="text-sm text-slate-700">{module.name}</span>
                    <StatusBadge status={module.status} />
                  </li>
                ))}
              </ul>
            </article>
          </section>

          <section className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
            <div className="flex flex-wrap items-center justify-between gap-3">
              <h2 className="text-base font-semibold">Base de datos local</h2>
              {health.data.database?.encrypted ? (
                <span className="rounded-full bg-emerald-50 px-3 py-1 text-xs font-semibold text-emerald-700 ring-1 ring-inset ring-emerald-600/20">
                  SQLCipher {health.data.database.cipherVersion ?? "activo"}
                </span>
              ) : (
                <span className="rounded-full bg-amber-50 px-3 py-1 text-xs font-semibold text-amber-700 ring-1 ring-inset ring-amber-600/20">
                  Sin cifrado
                </span>
              )}
            </div>

            {health.data.database ? (
              <dl className="mt-5 grid gap-4 text-sm sm:grid-cols-2 lg:grid-cols-4">
                <Detail label="Ruta" value={health.data.database.path} mono />
                <Detail
                  label="Esquema"
                  value={`migración ${health.data.database.schemaVersion}`}
                />
                <Detail
                  label="Integridad"
                  value={health.data.database.integrityOk ? "ok" : "falló"}
                />
                <Detail
                  label="Configuración"
                  value={
                    (health.data.database.organizations ?? 0) > 0
                      ? "Gimnasio configurado"
                      : "Pendiente de configurar"
                  }
                />
              </dl>
            ) : (
              <p className="mt-4 text-sm text-rose-700">
                No fue posible abrir la base de datos. Revisa los logs locales.
              </p>
            )}
          </section>
        </>
      )}

    </div>
  );
}

function Detail({
  label,
  value,
  mono = false,
}: {
  label: string;
  value: string;
  mono?: boolean;
}) {
  return (
    <div>
      <dt className="text-xs font-medium uppercase tracking-wide text-slate-500">{label}</dt>
      <dd className={`mt-1 break-all ${mono ? "font-mono text-xs" : ""}`}>{value}</dd>
    </div>
  );
}

function formatTime(value: string): string {
  const date = new Date(value);
  return Number.isNaN(date.getTime())
    ? value
    : new Intl.DateTimeFormat("es-MX", { timeStyle: "medium" }).format(date);
}
