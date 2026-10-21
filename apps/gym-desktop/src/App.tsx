import { Navigate, Route, Routes } from "react-router-dom";

import { Layout } from "./components/Layout";
import { AccessPage } from "./pages/AccessPage";
import { BackupsPage } from "./pages/BackupsPage";
import { DiagnosticsPage } from "./pages/DiagnosticsPage";

export function App() {
  return (
    <Routes>
      <Route element={<Layout />}>
        <Route index element={<DiagnosticsPage />} />
        <Route path="access" element={<AccessPage />} />
        <Route path="backups" element={<BackupsPage />} />
        <Route path="*" element={<Navigate to="/" replace />} />
      </Route>
    </Routes>
  );
}
