import type { BackupSummary } from "@gym/contracts";
import { errorMessage } from "@gym/contracts";
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { useState, type FormEvent } from "react";

import { formatBytes, formatDateTime, shortenChecksum } from "../lib/format";
import { api, isTauriRuntime } from "../lib/tauri";

export function BackupsPage() {
  const queryClient = useQueryClient();
  const inTauri = isTauriRuntime();
  const [passphrase, setPassphrase] = useState("");

  const backups = useQuery<BackupSummary[]>({
    queryKey: ["backups"],
    queryFn: api.listBackups,
    enabled: inTauri,
  });

  const createBackup = useMutation({
    mutationFn: api.createBackup,
    onSuccess: async () => {
      setPassphrase("");
      await queryClient.invalidateQueries({ queryKey: ["backups"] });
    },
  });

  const restoreBackup = useMutation({
    mutationFn: ({ fileName, secret }: { fileName: string; secret: string }) =>
      api.restoreBackup(fileName, secret),
    onSuccess: async () => {
      await queryClient.invalidateQueries();
    },
  });

  function handleCreate(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (passphrase.trim().length >= 8) {
      createBackup.mutate(passphrase);
    }
  }

  function handleRestore(backup: BackupSummary) {
    const secret = window.prompt(
      `Frase de respaldo para restaurar ${backup.fileName}:`,
    );
    if (!secret) {
      return;
    }
    const confirmed = window.confirm(
      "La restauración reemplaza la base local actual. ¿Continuar?",
    );
    if (confirmed) {
      restoreBackup.mutate({ fileName: backup.fileName, secret });
    }
  }

  return (
    <div className="space-y-6">
      <div>
        <h1 className="text-xl font-semibold tracking-tight">Backups cifrados</h1>
        <p className="mt-1 text-sm text-slate-600">
          Snapshots locales con Argon2id + XChaCha20-Poly1305 y restauración verificada.
        </p>
      </div>

      <section className="rounded-2xl border border-slate-200 bg-white p-6 shadow-sm">
        <h2 className="text-base font-semibold">Crear snapshot</h2>
        <p className="mt-1 text-sm text-slate-600">
          La frase no se almacena. Necesitarás la misma frase para restaurar.
        </p>
        <form onSubmit={handleCreate} className="mt-4 flex flex-wrap gap-3">
          <input
            type="password"
            value={passphrase}
            onChange={(event) => setPassphrase(event.target.value)}
            placeholder="Frase de respaldo (mínimo 8 caracteres)"
            className="min-w-64 flex-1 rounded-lg border border-slate-300 px-3 py-2 text-sm outline-none focus:border-brand-500 focus:ring-2 focus:ring-brand-100"
          />
          <button
            type="submit"
            className="rounded-lg bg-brand-600 px-4 py-2 text-sm font-medium text-white hover:bg-brand-700 disabled:opacity-50"
            disabled={createBackup.isPending || passphrase.trim().length < 8}
          >
            {createBackup.isPending ? "Creando…" : "Crear backup"}
          </button>
        </form>
        {createBackup.isError && (
          <p className="mt-3 text-sm text-rose-700">{errorMessage(createBackup.error)}</p>
        )}
        {createBackup.isSuccess && (
          <p className="mt-3 text-sm text-emerald-700">
            Snapshot {createBackup.data.fileName} creado.
          </p>
        )}
      </section>

      <section className="rounded-2xl border border-slate-200 bg-white shadow-sm">
        <div className="flex items-center justify-between gap-3 border-b border-slate-200 px-6 py-4">
          <h2 className="text-base font-semibold">Snapshots locales</h2>
          <button
            type="button"
            onClick={() => backups.refetch()}
            className="text-sm font-medium text-brand-700 hover:text-brand-600"
            disabled={backups.isFetching}
          >
            Actualizar
          </button>
        </div>

        {!inTauri ? (
          <p className="px-6 py-8 text-sm text-slate-500">
            Disponible únicamente dentro de la aplicación Tauri.
          </p>
        ) : backups.isLoading ? (
          <p className="px-6 py-8 text-sm text-slate-500">Cargando snapshots…</p>
        ) : backups.isError ? (
          <p className="px-6 py-8 text-sm text-rose-700">{errorMessage(backups.error)}</p>
        ) : backups.data && backups.data.length > 0 ? (
          <div className="overflow-x-auto">
            <table className="min-w-full divide-y divide-slate-200 text-sm">
              <thead className="bg-slate-50 text-left text-xs uppercase tracking-wide text-slate-500">
                <tr>
                  <th className="px-6 py-3 font-medium">Archivo</th>
                  <th className="px-6 py-3 font-medium">Fecha</th>
                  <th className="px-6 py-3 font-medium">Tamaño</th>
                  <th className="px-6 py-3 font-medium">Checksum</th>
                  <th className="px-6 py-3 font-medium">Esquema</th>
                  <th className="px-6 py-3" />
                </tr>
              </thead>
              <tbody className="divide-y divide-slate-100">
                {backups.data.map((backup) => (
                  <tr key={backup.id}>
                    <td className="px-6 py-3 font-mono text-xs">{backup.fileName}</td>
                    <td className="px-6 py-3">{formatDateTime(backup.createdAt)}</td>
                    <td className="px-6 py-3">{formatBytes(backup.sizeBytes)}</td>
                    <td className="px-6 py-3 font-mono text-xs">
                      {shortenChecksum(backup.checksum)}
                    </td>
                    <td className="px-6 py-3">v{backup.schemaVersion}</td>
                    <td className="px-6 py-3 text-right">
                      <button
                        type="button"
                        onClick={() => handleRestore(backup)}
                        className="rounded-lg border border-slate-300 px-3 py-1.5 text-xs font-medium text-slate-700 hover:bg-slate-50"
                        disabled={restoreBackup.isPending}
                      >
                        Restaurar
                      </button>
                    </td>
                  </tr>
                ))}
              </tbody>
            </table>
          </div>
        ) : (
          <p className="px-6 py-8 text-sm text-slate-500">Aún no hay snapshots.</p>
        )}

        {restoreBackup.isError && (
          <p className="border-t border-rose-100 bg-rose-50 px-6 py-3 text-sm text-rose-700">
            {errorMessage(restoreBackup.error)}
          </p>
        )}
        {restoreBackup.isSuccess && (
          <p className="border-t border-emerald-100 bg-emerald-50 px-6 py-3 text-sm text-emerald-700">
            Restauración completada desde {restoreBackup.data.backupId} · integridad{" "}
            {restoreBackup.data.integrityOk ? "ok" : "falló"}.
          </p>
        )}
      </section>
    </div>
  );
}
