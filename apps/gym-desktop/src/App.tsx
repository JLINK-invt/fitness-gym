import type { Permission, SessionSummary } from "@gym/contracts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState, type ReactNode } from "react";
import { Navigate, Route, Routes, useOutletContext } from "react-router-dom";

import type { LayoutContext } from "./components/Layout";
import { Layout } from "./components/Layout";
import { AccessPage } from "./pages/AccessPage";
import { AuditPage } from "./pages/AuditPage";
import { AuthPage } from "./pages/AuthPage";
import { BackupsPage } from "./pages/BackupsPage";
import { CashPage } from "./pages/CashPage";
import { CatalogPage } from "./pages/CatalogPage";
import { DashboardPage } from "./pages/DashboardPage";
import { DiagnosticsPage } from "./pages/DiagnosticsPage";
import { InventoryPage } from "./pages/InventoryPage";
import { MembersPage } from "./pages/MembersPage";
import { ReportsPage } from "./pages/ReportsPage";
import { SalesPage } from "./pages/SalesPage";
import { StaffPage } from "./pages/StaffPage";
import { api, isTauriRuntime } from "./lib/tauri";

export function App() {
  const inTauri = isTauriRuntime();
  const queryClient = useQueryClient();
  const [recoveryPending, setRecoveryPending] = useState(false);

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

  if (!session.data || recoveryPending) {
    return (
      <AuthPage
        onAuthenticated={(authenticated) => {
          queryClient.setQueryData(["session"], authenticated);
        }}
        onRecoveryPendingChange={setRecoveryPending}
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
        <Route index element={<DashboardPage />} />
        <Route path="diagnostics" element={<DiagnosticsPage />} />
        <Route
          path="cash"
          element={
            <Guarded anyPermission={["operatePos", "viewFinance"]}>
              <CashPage />
            </Guarded>
          }
        />
        <Route
          path="sales"
          element={
            <Guarded permission="operatePos">
              <SalesPage />
            </Guarded>
          }
        />
        <Route
          path="inventory"
          element={
            <Guarded permission="manageInventory">
              <InventoryPage />
            </Guarded>
          }
        />
        <Route
          path="members"
          element={
            <Guarded permission="manageMembers">
              <MembersPage />
            </Guarded>
          }
        />
        <Route
          path="access"
          element={
            <Guarded anyPermission={["operatePos", "manageMembers"]}>
              <AccessPage />
            </Guarded>
          }
        />
        <Route
          path="catalog"
          element={
            <Guarded permission="manageCatalog">
              <CatalogPage />
            </Guarded>
          }
        />
        <Route
          path="reports"
          element={
            <Guarded permission="viewFinance">
              <ReportsPage />
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
          path="backups"
          element={
            <Guarded permission="manageBackups">
              <BackupsPage />
            </Guarded>
          }
        />
        <Route
          path="audit"
          element={
            <Guarded permission="viewAudit">
              <AuditPage />
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
  anyPermission,
  children,
}: {
  permission?: Permission;
  anyPermission?: Permission[];
  children: ReactNode;
}) {
  const { session } = useOutletContext<LayoutContext>();
  const allowed = permission
    ? session.permissions.includes(permission)
    : (anyPermission ?? []).some((value) => session.permissions.includes(value));
  return allowed ? children : <Navigate to="/" replace />;
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
