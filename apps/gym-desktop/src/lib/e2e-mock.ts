type Organization = { id: string; name: string };
type Owner = { id: string; name: string; password: string };
type Session = {
  userId: string;
  organizationId: string;
  siteId?: string;
  name: string;
  role: "owner" | "manager" | "cashier" | "accounting";
  permissions: string[];
};
type AccessEvent = {
  eventId: string;
  allowed: boolean;
  idempotencyKey: string;
  occurredAt: string;
};
type AccessEntry = { event: AccessEvent; requestHash: string };
type Backup = {
  id: string;
  fileName: string;
  createdAt: string;
  sizeBytes: number;
  checksum: string;
  schemaVersion: number;
};

(() => {
  const state: {
    organizations: Organization[];
    owner: Owner | null;
    session: Session | null;
    accessEvents: AccessEvent[];
    accessEntries: Map<string, AccessEntry>;
    backups: Backup[];
    secrets: Map<string, string>;
  } = {
    organizations: [],
    owner: null,
    session: null,
    accessEvents: [],
    accessEntries: new Map(),
    backups: [],
    secrets: new Map(),
  };

  const now = () => new Date().toISOString();
  const uuid = () => crypto.randomUUID();
  const apiError = (code: string, message: string) => ({ code, message });

  const health = () => ({
    service: "gym-desktop",
    version: "0.1.0",
    contractVersion: "1.2.0",
    status: "ok",
    modules: [
      { name: "gym-core", status: "ok" },
      { name: "gym-data", status: "ok" },
      { name: "gym-backup", status: "ok" },
    ],
    database: {
      path: "/tmp/fitness-gym-e2e/fitness-gym.db",
      encrypted: true,
      cipherVersion: "4.6.1 community",
      schemaVersion: 2,
      integrityOk: true,
      organizations: state.organizations.length,
    },
    checkedAt: now(),
  });

  const internals = {
    invoke: async (command: string, args: Record<string, unknown> = {}) => {
      switch (command) {
        case "health":
          return health();

        case "get_setup_state":
          return {
            configured: state.organizations.length > 0,
            gymName: state.organizations[0]?.name,
            ownerReady: state.owner !== null,
          };

        case "bootstrap_owner": {
          const name = String(args.name ?? "").trim();
          const password = String(args.password ?? "");
          const gymName = String(args.gymName ?? "").trim();
          if (state.organizations.length === 0) {
            if (!gymName) {
              throw apiError("invalidRequest", "Escribe el nombre del gimnasio.");
            }
            state.organizations.push({ id: uuid(), name: gymName });
          }
          if (state.owner) {
            throw apiError("invalidRequest", "Este gimnasio ya tiene una cuenta de dueño.");
          }
          if (name.length < 2) {
            throw apiError("invalidRequest", "El nombre del dueño es demasiado corto.");
          }
          if (password.length < 12) {
            throw apiError("invalidRequest", "La contraseña debe tener al menos 12 caracteres.");
          }
          const owner: Owner = { id: uuid(), name, password };
          state.owner = owner;
          state.session = {
            userId: owner.id,
            organizationId: state.organizations[0]!.id,
            name: owner.name,
            role: "owner",
            permissions: [
              "manageSettings",
              "manageStaff",
              "viewFinance",
              "operatePos",
              "manageInventory",
              "manageBackups",
              "viewAudit",
            ],
          };
          return { ...state.session };
        }

        case "login": {
          const name = String(args.name ?? "").trim();
          const password = String(args.password ?? "");
          if (
            !state.owner ||
            state.owner.name.toLowerCase() !== name.toLowerCase() ||
            state.owner.password !== password
          ) {
            throw apiError("unauthorized", "Nombre o contraseña incorrectos.");
          }
          state.session = {
            userId: state.owner.id,
            organizationId: state.organizations[0]!.id,
            name: state.owner.name,
            role: "owner",
            permissions: [
              "manageSettings",
              "manageStaff",
              "viewFinance",
              "operatePos",
              "manageInventory",
              "manageBackups",
              "viewAudit",
            ],
          };
          return { ...state.session };
        }

        case "logout":
          state.session = null;
          return null;

        case "current_session":
          return state.session ? { ...state.session } : null;

        case "simulate_access": {
          if (!state.session) {
            throw apiError("unauthorized", "Inicia sesión para continuar.");
          }
          if (state.organizations.length === 0) {
            throw apiError(
              "invalidRequest",
              "Configura el gimnasio antes de simular un acceso.",
            );
          }
          const request = (args.request ?? {}) as Record<string, unknown>;
          const credentialId = String(request.credentialId ?? "").trim();
          const deviceId = String(request.deviceId ?? "").trim();
          const idempotencyKey = String(request.idempotencyKey ?? "").trim();
          if (!credentialId || !deviceId || !idempotencyKey) {
            throw apiError("invalidRequest", "La solicitud está incompleta.");
          }

          const requestHash = `${credentialId}\n${deviceId}`;
          const existing = state.accessEntries.get(idempotencyKey);
          if (existing) {
            if (existing.requestHash !== requestHash) {
              throw apiError(
                "invalidRequest",
                "La clave de idempotencia ya se usó para otra solicitud.",
              );
            }
            return {
              eventId: existing.event.eventId,
              allowed: existing.event.allowed,
              reason: "evento ya registrado; se devuelve el resultado original",
              duplicate: true,
              idempotencyKey,
              latencyMs: 3,
              decidedAt: existing.event.occurredAt,
            };
          }

          const allowed = credentialId === "CRED-ALLOW";
          const reason = allowed
            ? "credencial simulada activa"
            : credentialId === "CRED-DENY"
              ? "credencial simulada sin acceso"
              : "credencial desconocida";
          const event: AccessEvent = {
            eventId: uuid(),
            allowed,
            idempotencyKey,
            occurredAt: now(),
          };
          state.accessEvents.unshift(event);
          state.accessEntries.set(idempotencyKey, { event, requestHash });
          return {
            eventId: event.eventId,
            allowed,
            reason,
            duplicate: false,
            idempotencyKey,
            latencyMs: 3,
            decidedAt: event.occurredAt,
          };
        }

        case "list_access_events": {
          if (!state.session) {
            throw apiError("unauthorized", "Inicia sesión para continuar.");
          }
          const limit = Math.max(1, Math.min(100, Number(args.limit ?? 20)));
          return state.accessEvents.slice(0, limit);
        }

        case "list_backups":
          if (!state.session) {
            throw apiError("unauthorized", "Inicia sesión para continuar.");
          }
          return state.backups.map((backup) => ({ ...backup }));

        case "create_backup": {
          if (!state.session) {
            throw apiError("unauthorized", "Inicia sesión para continuar.");
          }
          const passphrase = String(args.passphrase ?? "");
          if (passphrase.trim().length < 8) {
            throw apiError("backup", "La frase debe tener al menos 8 caracteres.");
          }
          const createdAt = now();
          const fileName = `fitness-gym-${createdAt.replace(/[-:.]/g, "")}.gymbackup`;
          const backup = {
            id: uuid(),
            fileName,
            createdAt,
            sizeBytes: 118784,
            checksum: "9f2c4e7a1b8d3f5061728394a5b6c7d8e9f00112233445566778899aabbccddee",
            schemaVersion: 2,
          };
          state.backups.unshift(backup);
          state.secrets.set(fileName, passphrase);
          return { ...backup };
        }

        case "restore_backup": {
          if (!state.session) {
            throw apiError("unauthorized", "Inicia sesión para continuar.");
          }
          const fileName = String(args.fileName ?? "");
          const backup = state.backups.find((item) => item.fileName === fileName);
          if (!backup) {
            throw apiError("notFound", `No existe el backup ${fileName}.`);
          }
          if (state.secrets.get(fileName) !== String(args.passphrase ?? "")) {
            throw apiError(
              "backup",
              "No se pudo descifrar el backup: frase incorrecta o archivo dañado.",
            );
          }
          state.session = null;
          return {
            backupId: backup.id,
            restoredAt: now(),
            schemaVersion: backup.schemaVersion,
            integrityOk: true,
          };
        }

        default:
          throw apiError("internal", `Comando no simulado: ${command}`);
      }
    },
    transformCallback: () => 1,
    unregisterCallback: () => {},
    convertFileSrc: (path: string) => path,
  };

  (window as unknown as { __TAURI_INTERNALS__: typeof internals }).__TAURI_INTERNALS__ =
    internals;
})();

export {};
