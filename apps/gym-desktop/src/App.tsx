import type { Permission, SessionSummary } from "@gym/contracts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import type { ReactNode } from "react";
import { Navigate, Route, Routes, useOutletContext } from "react-router-dom";

import type { LayoutContext } from "./components/Layout";
import { Layout } from "./components/Layout";
import { AccessPage } from "./pages/AccessPage";
import { AuthPage } from "./pages/AuthPage";
import { BackupsPage } from "./pages/BackupsPage";
import { DiagnosticsPage } from "./pages/DiagnosticsPage";
import { SitesPage } from "./pages/SitesPage";
import { StaffPage } from "./pages/StaffPage";
import { api, isTauriRuntime } from "./lib/tauri";

export function App() {
  const inTauri = isTauriRuntime();
  const queryClient = useQueryClient();

  const session = useQuery<SessionSummary | null>({
    queryKey: ["session"],
    queryFn: api.currentSession,
    enabled: inTauri,
    retry: false,
    staleTime: 0,
    refetchInterval: 30_000,
  });

  const logout = useMutation({
    mutationFn: api.logout,
    onSuccess: () => {
      queryClient.clear();
    },
  });

  if (!inTauri) {
    return <WebPreview />;
  }

  if (session.isLoading) {
    return (
      <div className="flex min-h-full items-center justify-center text-sm text-slate-500">
        Abriendo el gimnasio…
      </div>
    );
  }

  if (!session.data) {
    return (
      <AuthPage
        onAuthenticated={(authenticated) => {
          queryClient.setQueryData(["session"], authenticated);
        }}
      />
    );
  }

  return (
    <Routes>
      <Route
        element={
          <Layout
            session={session.data}
            onLogout={() => logout.mutate()}
            signingOut={logout.isPending}
          />
        }
      >
        <Route index element={<DiagnosticsPage />} />
        <Route
          path="access"
          element={
            <Guarded permission="operatePos">
              <AccessPage />
            </Guarded>
          }
        />
        <Route
          path="staff"
          element={
            <Guarded permission="manageStaff">
              <StaffPage />
            </Guarded>
          }
        />
        <Route
          path="sites"
          element={
            <Guarded permission="manageSettings">
              <SitesPage />
            </Guarded>
          }
        />
        <Route
          path="backups"
          element={
            <Guarded permission="manageBackups">
              <BackupsPage />
            </Guarded>
          }
        />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Route>
    </Routes>
  );
}

function Guarded({
  permission,
  children,
}: {
  permission: Permission;
  children: ReactNode;
}) {
  const { session } = useOutletContext<LayoutContext>();
  return session.permissions.includes(permission) ? (
    children
  ) : (
    <Navigate to="/" replace />
  );
}

function WebPreview() {
  return (
    <div className="flex min-h-full items-center justify-center px-6 py-12">
      <section className="max-w-2xl rounded-2xl border border-slate-200 bg-white p-8 shadow-sm">
        <h1 className="text-xl font-semibold tracking-tight">Vista web sin runtime Tauri</h1>
        <p className="mt-2 text-sm text-slate-600">
          La autenticación, la SQLite cifrada y los backups requieren el proceso
          Tauri. Ejecuta <code className="font-mono">pnpm tauri dev</code> para
          abrir la aplicación real.
        </p>
        <pre className="mt-6 overflow-x-auto rounded-xl bg-slate-900 p-4 text-xs text-slate-100">
{`# Dependencias del sistema (Fedora)
sudo dnf install webkit2gtk4.1-devel gtk3-devel libappindicator-gtk3-devel \\
  librsvg2-devel openssl-devel sqlcipher-devel

pnpm install
pnpm tauri dev`}
        </pre>
      </section>
    </div>
  );
}
