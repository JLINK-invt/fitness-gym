import type { RecoveryKit, SessionSummary, SetupState } from "@gym/contracts";
import { errorMessage } from "@gym/contracts";
import { useMutation, useQuery } from "@tanstack/react-query";
import { useState, type FormEvent, type ReactNode } from "react";

import { RecoveryCodePanel } from "../components/RecoveryCodePanel";
import { api } from "../lib/tauri";

export function AuthPage({
  onAuthenticated,
  onRecoveryPendingChange,
}: {
  onAuthenticated: (session: SessionSummary) => void;
  onRecoveryPendingChange: (pending: boolean) => void;
}) {
  const setup = useQuery<SetupState>({
    queryKey: ["setup"],
    queryFn: api.setupState,
    retry: false,
  });

  if (setup.isLoading) {
    return (
      <AuthShell>
        <p className="text-sm text-slate-500">Abriendo la base local…</p>
      </AuthShell>
    );
  }

  if (setup.isError || !setup.data) {
    return (
      <AuthShell>
        <h1 className="text-xl font-semibold tracking-tight">No se pudo abrir el gimnasio</h1>
        <p className="mt-3 rounded-xl border border-rose-200 bg-rose-50 p-4 text-sm text-rose-700">
          {setup.isError ? errorMessage(setup.error) : "La configuración local no está disponible."}
        </p>
      </AuthShell>
    );
  }

  if (!setup.data.ownerReady) {
    return (
      <OwnerSetup
        state={setup.data}
        onAuthenticated={onAuthenticated}
        onRecoveryPendingChange={onRecoveryPendingChange}
      />
    );
  }

  return (
    <LoginForm
      gymName={setup.data.gymName}
      onAuthenticated={onAuthenticated}
      onRecoveryPendingChange={onRecoveryPendingChange}
    />
  );
}

function OwnerSetup({
  state,
  onAuthenticated,
  onRecoveryPendingChange,
}: {
  state: SetupState;
  onAuthenticated: (session: SessionSummary) => void;
  onRecoveryPendingChange: (pending: boolean) => void;
}) {
  const [gymName, setGymName] = useState(state.gymName ?? "");
  const [ownerName, setOwnerName] = useState("");
  const [password, setPassword] = useState("");
  const [confirmation, setConfirmation] = useState("");
  const [localError, setLocalError] = useState<string | null>(null);
  const [recoveryKit, setRecoveryKit] = useState<RecoveryKit | null>(null);

  const createOwner = useMutation({
    mutationFn: () =>
      api.bootstrapOwner(
        ownerName.trim(),
        password,
        state.configured ? undefined : gymName.trim(),
      ),
    onSuccess: (kit) => {
      setRecoveryKit(kit);
      onRecoveryPendingChange(true);
    },
  });

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setLocalError(null);
    if (!state.configured && !gymName.trim()) {
      setLocalError("Escribe el nombre del gimnasio.");
      return;
    }
    if (ownerName.trim().length < 2) {
      setLocalError("Escribe el nombre de la persona dueña.");
      return;
    }
    if (password.length < 12) {
      setLocalError("La contraseña del dueño debe tener al menos 12 caracteres.");
      return;
    }
    if (password !== confirmation) {
      setLocalError("Las contraseñas no coinciden.");
      return;
    }
    createOwner.mutate();
  }

  const error = localError ?? (createOwner.isError ? errorMessage(createOwner.error) : null);

  if (recoveryKit) {
    return (
      <AuthShell>
        <RecoveryCodePanel
          code={recoveryKit.recoveryCode}
          title="Guarda tu código de recuperación"
          description="Es la única forma de volver a entrar si olvidas la contraseña del dueño. No hay recuperación por correo ni por teléfono."
          onContinue={() => {
            onRecoveryPendingChange(false);
            onAuthenticated(recoveryKit.session);
          }}
        />
      </AuthShell>
    );
  }

  return (
    <AuthShell>
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-brand-700">
        Primer arranque
      </p>
      <h1 className="mt-3 text-2xl font-semibold tracking-tight">
        {state.configured ? "Crea la cuenta del dueño" : "Configura tu gimnasio"}
      </h1>
      <p className="mt-2 text-sm leading-6 text-slate-600">
        Esta cuenta administra el equipo, las finanzas y los respaldos. No hay
        usuario por defecto ni recuperación por correo. Al terminar recibirás un
        código de recuperación de un solo uso.
      </p>

      <form onSubmit={handleSubmit} className="mt-6 space-y-4">
        {!state.configured && (
          <Field label="Nombre del gimnasio">
            <input
              value={gymName}
              onChange={(event) => setGymName(event.target.value)}
              autoComplete="organization"
              className={inputClass}
              placeholder="Gimnasio Centro"
            />
          </Field>
        )}

        <Field label="Tu nombre">
          <input
            value={ownerName}
            onChange={(event) => setOwnerName(event.target.value)}
            autoComplete="name"
            className={inputClass}
            placeholder="Ana Pérez"
          />
        </Field>

        <Field label="Contraseña" hint="Mínimo 12 caracteres para el dueño.">
          <input
            type="password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            autoComplete="new-password"
            className={inputClass}
          />
        </Field>

        <Field label="Repetir contraseña">
          <input
            type="password"
            value={confirmation}
            onChange={(event) => setConfirmation(event.target.value)}
            autoComplete="new-password"
            className={inputClass}
          />
        </Field>

        {error && <ErrorNotice message={error} />}

        <button type="submit" disabled={createOwner.isPending} className={buttonClass}>
          {createOwner.isPending ? "Creando cuenta…" : "Crear cuenta y entrar"}
        </button>
      </form>
    </AuthShell>
  );
}

function LoginForm({
  gymName,
  onAuthenticated,
  onRecoveryPendingChange,
}: {
  gymName?: string;
  onAuthenticated: (session: SessionSummary) => void;
  onRecoveryPendingChange: (pending: boolean) => void;
}) {
  const [name, setName] = useState("");
  const [password, setPassword] = useState("");
  const [recovering, setRecovering] = useState(false);

  const login = useMutation({
    mutationFn: () => api.login(name.trim(), password),
    onSuccess: onAuthenticated,
  });

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (name.trim() && password) {
      login.mutate();
    }
  }

  if (recovering) {
    return (
      <RecoveryForm
        gymName={gymName}
        onBack={() => setRecovering(false)}
        onAuthenticated={onAuthenticated}
        onRecoveryPendingChange={onRecoveryPendingChange}
      />
    );
  }

  return (
    <AuthShell>
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-brand-700">
        {gymName ?? "Fitness Gym"}
      </p>
      <h1 className="mt-3 text-2xl font-semibold tracking-tight">Entra a tu turno</h1>
      <p className="mt-2 text-sm leading-6 text-slate-600">
        La sesión vive solo en este equipo y se bloquea tras 15 minutos de
        inactividad.
      </p>

      <form onSubmit={handleSubmit} className="mt-6 space-y-4">
        <Field label="Nombre">
          <input
            value={name}
            onChange={(event) => setName(event.target.value)}
            autoComplete="username"
            className={inputClass}
            placeholder="Ana Pérez"
            autoFocus
          />
        </Field>

        <Field label="Contraseña">
          <input
            type="password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            autoComplete="current-password"
            className={inputClass}
          />
        </Field>

        {login.isError && <ErrorNotice message={errorMessage(login.error)} />}

        <button
          type="submit"
          disabled={login.isPending || !name.trim() || !password}
          className={buttonClass}
        >
          {login.isPending ? "Entrando…" : "Entrar"}
        </button>
      </form>

      <button
        type="button"
        onClick={() => setRecovering(true)}
        className="mt-4 w-full text-center text-sm font-medium text-brand-700 hover:text-brand-900"
      >
        ¿Olvidaste la contraseña del dueño?
      </button>
    </AuthShell>
  );
}

function RecoveryForm({
  gymName,
  onBack,
  onAuthenticated,
  onRecoveryPendingChange,
}: {
  gymName?: string;
  onBack: () => void;
  onAuthenticated: (session: SessionSummary) => void;
  onRecoveryPendingChange: (pending: boolean) => void;
}) {
  const [recoveryCode, setRecoveryCode] = useState("");
  const [password, setPassword] = useState("");
  const [confirmation, setConfirmation] = useState("");
  const [localError, setLocalError] = useState<string | null>(null);
  const [recoveryKit, setRecoveryKit] = useState<RecoveryKit | null>(null);

  const recover = useMutation({
    mutationFn: () => api.recoverOwner(recoveryCode.trim(), password),
    onSuccess: (kit) => {
      setRecoveryKit(kit);
      onRecoveryPendingChange(true);
    },
  });

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    setLocalError(null);
    if (!recoveryCode.trim()) {
      setLocalError("Escribe el código de recuperación.");
      return;
    }
    if (password.length < 12) {
      setLocalError("La contraseña nueva debe tener al menos 12 caracteres.");
      return;
    }
    if (password !== confirmation) {
      setLocalError("Las contraseñas no coinciden.");
      return;
    }
    recover.mutate();
  }

  if (recoveryKit) {
    return (
      <AuthShell>
        <RecoveryCodePanel
          code={recoveryKit.recoveryCode}
          title="Tu nuevo código de recuperación"
          description="La contraseña anterior ya no funciona. Sustituye el código guardado por este; el anterior quedó invalidado."
          onContinue={() => {
            onRecoveryPendingChange(false);
            onAuthenticated(recoveryKit.session);
          }}
        />
      </AuthShell>
    );
  }

  const error = localError ?? (recover.isError ? errorMessage(recover.error) : null);

  return (
    <AuthShell>
      <p className="text-xs font-semibold uppercase tracking-[0.2em] text-brand-700">
        {gymName ?? "Fitness Gym"}
      </p>
      <h1 className="mt-3 text-2xl font-semibold tracking-tight">Recupera la cuenta</h1>
      <p className="mt-2 text-sm leading-6 text-slate-600">
        Solo funciona con el código de recuperación de la cuenta del dueño. Al
        usarlo se invalida y se genera uno nuevo.
      </p>

      <form onSubmit={handleSubmit} className="mt-6 space-y-4">
        <Field
          label="Código de recuperación"
          hint="Formato XXXXX-XXXXX-XXXXX-XXXXX; no distingue mayúsculas."
        >
          <input
            value={recoveryCode}
            onChange={(event) => setRecoveryCode(event.target.value)}
            autoComplete="one-time-code"
            className={`${inputClass} font-mono uppercase tracking-wider`}
            placeholder="XXXXX-XXXXX-XXXXX-XXXXX"
            autoFocus
          />
        </Field>

        <Field label="Contraseña nueva" hint="Mínimo 12 caracteres.">
          <input
            type="password"
            value={password}
            onChange={(event) => setPassword(event.target.value)}
            autoComplete="new-password"
            className={inputClass}
          />
        </Field>

        <Field label="Repetir contraseña nueva">
          <input
            type="password"
            value={confirmation}
            onChange={(event) => setConfirmation(event.target.value)}
            autoComplete="new-password"
            className={inputClass}
          />
        </Field>

        {error && <ErrorNotice message={error} />}

        <button
          type="submit"
          disabled={recover.isPending || !recoveryCode.trim() || !password}
          className={buttonClass}
        >
          {recover.isPending ? "Recuperando…" : "Fijar contraseña y entrar"}
        </button>
      </form>

      <button
        type="button"
        onClick={onBack}
        className="mt-4 w-full text-center text-sm font-medium text-slate-600 hover:text-slate-900"
      >
        Volver al inicio de sesión
      </button>
    </AuthShell>
  );
}

function AuthShell({ children }: { children: React.ReactNode }) {
  return (
    <div className="flex min-h-full items-center justify-center px-6 py-12">
      <div className="grid w-full max-w-5xl overflow-hidden rounded-3xl border border-slate-200 bg-white shadow-xl shadow-slate-200/60 lg:grid-cols-[1.05fr_1fr]">
        <aside className="hidden bg-slate-950 p-10 text-white lg:flex lg:flex-col lg:justify-between">
          <div>
            <div className="flex h-11 w-11 items-center justify-center rounded-xl bg-brand-500 text-lg font-bold">
              FG
            </div>
            <p className="mt-8 text-3xl font-semibold leading-tight tracking-tight">
              El mostrador empieza aquí.
            </p>
            <p className="mt-4 max-w-sm text-sm leading-6 text-slate-300">
              Operación local, base cifrada y permisos por persona. Sin nube
              obligatoria para cobrar, registrar accesos o respaldar el día.
            </p>
          </div>
          <ul className="space-y-3 text-sm text-slate-300">
            <li>Identidad local con Argon2id</li>
            <li>Sesión en memoria, nunca en el navegador</li>
            <li>Autorización revalidada en Rust</li>
          </ul>
        </aside>

        <div className="p-8 sm:p-10">{children}</div>
      </div>
    </div>
  );
}

function Field({
  label,
  hint,
  children,
}: {
  label: string;
  hint?: string;
  children: ReactNode;
}) {
  return (
    <label className="block text-sm">
      <span className="mb-1 block font-medium text-slate-700">{label}</span>
      {children}
      {hint && <span className="mt-1 block text-xs text-slate-500">{hint}</span>}
    </label>
  );
}

function ErrorNotice({ message }: { message: string }) {
  return (
    <p className="rounded-xl border border-rose-200 bg-rose-50 p-3 text-sm text-rose-700" role="alert">
      {message}
    </p>
  );
}

const inputClass =
  "w-full rounded-lg border border-slate-300 bg-white px-3 py-2 text-sm outline-none transition focus:border-brand-500 focus:ring-2 focus:ring-brand-100";

const buttonClass =
  "w-full rounded-lg bg-brand-600 px-4 py-2.5 text-sm font-semibold text-white transition hover:bg-brand-700 disabled:cursor-not-allowed disabled:opacity-50";
