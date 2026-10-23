import { useState } from "react";

export function RecoveryCodePanel({
  code,
  title,
  description,
  requireConfirmation = true,
  onContinue,
  onCancel,
}: {
  code: string;
  title: string;
  description: string;
  requireConfirmation?: boolean;
  onContinue?: () => void;
  onCancel?: () => void;
}) {
  const [confirmed, setConfirmed] = useState(false);
  const [copied, setCopied] = useState(false);
  const [copyError, setCopyError] = useState(false);

  async function copyCode() {
    setCopyError(false);
    try {
      await navigator.clipboard.writeText(code);
      setCopied(true);
    } catch {
      setCopyError(true);
    }
  }

  function downloadCode() {
    const contents = [
      "Fitness Gym · código de recuperación del dueño",
      "",
      code,
      "",
      "Guarda este código fuera de este equipo. Sirve una sola vez para fijar",
      "una contraseña nueva. Si generas otro código, este deja de funcionar.",
      "",
    ].join("\n");
    const url = URL.createObjectURL(new Blob([contents], { type: "text/plain" }));
    const link = document.createElement("a");
    link.href = url;
    link.download = "fitness-gym-recuperacion.txt";
    link.click();
    URL.revokeObjectURL(url);
  }

  return (
    <div>
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-brand-700">
        Seguridad de la cuenta
      </p>
      <h1 className="mt-3 text-2xl font-semibold tracking-tight">{title}</h1>
      <p className="mt-2 text-sm leading-6 text-slate-600">{description}</p>

      <div className="mt-5 rounded-2xl border border-amber-200 bg-amber-50 p-4">
        <p className="text-xs font-semibold uppercase tracking-wide text-amber-800">
          Guárdalo ahora
        </p>
        <code className="mt-2 block select-all break-all rounded-lg bg-white px-3 py-3 font-mono text-base font-semibold tracking-wider text-slate-900">
          {code}
        </code>
        <p className="mt-2 text-xs leading-5 text-amber-800">
          No se volverá a mostrar. Solo guardamos su hash; sin este código y sin
          la contraseña no se puede recuperar la cuenta.
        </p>
      </div>

      <div className="mt-4 flex flex-wrap gap-2">
        <button
          type="button"
          onClick={copyCode}
          className="rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium text-slate-700 hover:bg-slate-100"
        >
          {copied ? "Copiado" : "Copiar código"}
        </button>
        <button
          type="button"
          onClick={downloadCode}
          className="rounded-lg border border-slate-300 px-3 py-2 text-sm font-medium text-slate-700 hover:bg-slate-100"
        >
          Descargar .txt
        </button>
      </div>
      {copyError && (
        <p className="mt-2 text-xs text-rose-700">
          No se pudo copiar. Selecciona el código y guárdalo manualmente.
        </p>
      )}

      {requireConfirmation && (
        <label className="mt-5 flex items-start gap-2 text-sm text-slate-700">
          <input
            type="checkbox"
            checked={confirmed}
            onChange={(event) => setConfirmed(event.target.checked)}
            className="mt-0.5 h-4 w-4 rounded border-slate-300"
          />
          Ya guardé el código de recuperación en un lugar seguro.
        </label>
      )}

      <div className="mt-5 flex flex-col gap-2">
        {onContinue && (
          <button
            type="button"
            disabled={requireConfirmation && !confirmed}
            onClick={onContinue}
            className="w-full rounded-lg bg-brand-600 px-4 py-2.5 text-sm font-semibold text-white transition hover:bg-brand-700 disabled:cursor-not-allowed disabled:opacity-50"
          >
            Continuar
          </button>
        )}
        {onCancel && (
          <button
            type="button"
            onClick={onCancel}
            className="text-sm font-medium text-slate-600 hover:text-slate-900"
          >
            Cerrar
          </button>
        )}
      </div>
    </div>
  );
}
