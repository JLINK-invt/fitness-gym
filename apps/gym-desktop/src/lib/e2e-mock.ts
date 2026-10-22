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
type Site = {
  id: string;
  name: string;
  status: "active" | "closed";
  createdAt: string;
};
type StaffMember = {
  id: string;
  name: string;
  password: string;
  role: "manager" | "cashier" | "accounting";
  siteId: string;
  status: "active" | "revoked";
  lastLoginAt: string | null;
  createdAt: string;
};
type Backup = {
  id: string;
  fileName: string;
  createdAt: string;
  sizeBytes: number;
  checksum: string;
  schemaVersion: number;
};
type AuditEvent = {
  id: string;
  sequence: number;
  actorId: string | null;
  actorName: string | null;
  siteId: string | null;
  action: string;
  resource: string;
  result: "allowed" | "denied" | "failed";
  origin: string;
  correlationId: string | null;
  createdAt: string;
};

(() => {
  const state: {
    organizations: Organization[];
    owner: Owner | null;
    sites: Site[];
    staff: StaffMember[];
    session: Session | null;
    accessEvents: AccessEvent[];
    accessEntries: Map<string, AccessEntry>;
    backups: Backup[];
    secrets: Map<string, string>;
    auditEvents: AuditEvent[];
  } = {
    organizations: [],
    owner: null,
    sites: [],
    staff: [],
    session: null,
    accessEvents: [],
    accessEntries: new Map(),
    backups: [],
    secrets: new Map(),
    auditEvents: [],
  };

  const now = () => new Date().toISOString();
  const uuid = () => crypto.randomUUID();
  const apiError = (code: string, message: string) => ({ code, message });
  const permissionsFor = (role: Session["role"]) => {
    switch (role) {
      case "owner":
        return [
          "manageSettings",
          "manageStaff",
          "viewFinance",
          "operatePos",
          "manageInventory",
          "manageBackups",
          "viewAudit",
        ];
      case "manager":
        return [
          "manageStaff",
          "viewFinance",
          "operatePos",
          "manageInventory",
          "manageBackups",
          "viewAudit",
        ];
      case "cashier":
        return ["operatePos", "manageInventory"];
      case "accounting":
        return ["viewFinance", "viewAudit"];
    }
  };
  const requireSession = () => {
    if (!state.session) {
      throw apiError("unauthorized", "Inicia sesión para continuar.");
    }
    return state.session;
  };
  const requirePermission = (permission: string) => {
    const session = requireSession();
    if (!session.permissions.includes(permission)) {
      throw apiError("unauthorized", "Tu rol no permite realizar esta acción.");
    }
    return session;
  };
  const ownerStaff = () =>
    state.owner
      ? {
          userId: state.owner.id,
          name: state.owner.name,
          role: "owner" as const,
          siteId: null,
          siteName: null,
          status: "active" as const,
          lastLoginAt: null,
          createdAt: now(),
        }
      : null;
  const staffSummary = (member: StaffMember) => {
    const site = state.sites.find((item) => item.id === member.siteId);
    return {
      userId: member.id,
      name: member.name,
      role: member.role,
      siteId: member.siteId,
      siteName: site?.name ?? null,
      status: member.status,
      lastLoginAt: member.lastLoginAt,
      createdAt: member.createdAt,
    };
  };

  const recordAudit = (input: {
    actor?: Session | null;
    siteId?: string | null;
    action: string;
    resource: string;
    result?: "allowed" | "denied" | "failed";
    correlationId?: string | null;
  }) => {
    if (state.organizations.length === 0) {
      return;
    }
    state.auditEvents.unshift({
      id: uuid(),
      sequence: state.auditEvents.length + 1,
      actorId: input.actor?.userId ?? null,
      actorName: input.actor?.name ?? null,
      siteId: input.siteId ?? input.actor?.siteId ?? null,
      action: input.action,
      resource: input.resource,
      result: input.result ?? "allowed",
      origin: "tauri",
      correlationId: input.correlationId ?? null,
      createdAt: now(),
    });
  };

  const health = () => ({
    service: "gym-desktop",
    version: "0.1.0",
    contractVersion: "1.4.0",
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
      schemaVersion: 4,
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
            permissions: permissionsFor("owner"),
          };
          recordAudit({
            actor: state.session,
            action: "auth.owner_created",
            resource: "staff_membership",
          });
          return { ...state.session };
        }

        case "login": {
          const name = String(args.name ?? "").trim();
          const password = String(args.password ?? "");
          const owner =
            state.owner &&
            state.owner.name.toLowerCase() === name.toLowerCase() &&
            state.owner.password === password
              ? state.owner
              : null;
          const member = state.staff.find(
            (item) =>
              item.status === "active" &&
              item.name.toLowerCase() === name.toLowerCase() &&
              item.password === password,
          );
          if (!owner && !member) {
            recordAudit({
              action: "auth.login",
              resource: "session",
              result: "failed",
            });
            throw apiError("unauthorized", "Nombre o contraseña incorrectos.");
          }
          if (owner) {
            state.session = {
              userId: owner.id,
              organizationId: state.organizations[0]!.id,
              name: owner.name,
              role: "owner",
              permissions: permissionsFor("owner"),
            };
          } else if (member) {
            member.lastLoginAt = now();
            state.session = {
              userId: member.id,
              organizationId: state.organizations[0]!.id,
              siteId: member.siteId,
              name: member.name,
              role: member.role,
              permissions: permissionsFor(member.role),
            };
          }
          if (state.session) {
            recordAudit({
              actor: state.session,
              action: "auth.login",
              resource: "session",
            });
          }
          return state.session ? { ...state.session } : null;
        }

        case "logout":
          if (state.session) {
            recordAudit({
              actor: state.session,
              action: "auth.logout",
              resource: "session",
            });
          }
          state.session = null;
          return null;

        case "current_session":
          return state.session ? { ...state.session } : null;

        case "list_sites": {
          const session = requirePermission("manageStaff");
          return state.sites
            .filter((site) => session.role !== "manager" || site.id === session.siteId)
            .map((site) => ({ ...site }));
        }

        case "create_site": {
          requirePermission("manageSettings");
          const name = String(args.name ?? "").trim();
          if (name.length < 2) {
            throw apiError("invalidRequest", "El nombre de la sede es demasiado corto.");
          }
          if (state.sites.some((site) => site.name.toLowerCase() === name.toLowerCase())) {
            throw apiError("invalidRequest", "Ya existe una sede con ese nombre.");
          }
          const site: Site = {
            id: uuid(),
            name,
            status: "active",
            createdAt: now(),
          };
          state.sites.push(site);
          recordAudit({
            actor: state.session,
            siteId: site.id,
            action: "site.created",
            resource: "site",
          });
          return { ...site };
        }

        case "update_site": {
          requirePermission("manageSettings");
          const site = state.sites.find((item) => item.id === String(args.siteId ?? ""));
          if (!site) {
            throw apiError("notFound", "La sede no existe.");
          }
          const name = String(args.name ?? "").trim();
          const status = String(args.status ?? "");
          if (name.length < 2) {
            throw apiError("invalidRequest", "El nombre de la sede es demasiado corto.");
          }
          if (status !== "active" && status !== "closed") {
            throw apiError("invalidRequest", "Estado de sede inválido.");
          }
          if (
            state.sites.some(
              (item) =>
                item.id !== site.id && item.name.toLowerCase() === name.toLowerCase(),
            )
          ) {
            throw apiError("invalidRequest", "Ya existe una sede con ese nombre.");
          }
          if (
            status === "closed" &&
            state.staff.some(
              (member) => member.siteId === site.id && member.status === "active",
            )
          ) {
            throw apiError(
              "invalidRequest",
              "No puedes cerrar una sede con personal activo; reasigna o desactiva primero.",
            );
          }
          site.name = name;
          site.status = status;
          recordAudit({
            actor: state.session,
            siteId: site.id,
            action: "site.updated",
            resource: "site",
          });
          return { ...site };
        }

        case "list_staff": {
          const session = requirePermission("manageStaff");
          const all = [...(ownerStaff() ? [ownerStaff()!] : []), ...state.staff.map(staffSummary)];
          return all.filter(
            (member) =>
              session.role !== "manager" ||
              (member.role !== "owner" && member.siteId === session.siteId),
          );
        }

        case "create_staff": {
          const session = requirePermission("manageStaff");
          const name = String(args.name ?? "").trim();
          const password = String(args.password ?? "");
          const role = String(args.role ?? "");
          let siteId = String(args.siteId ?? "");
          if (name.length < 2) {
            throw apiError("invalidRequest", "El nombre del empleado es demasiado corto.");
          }
          if (password.length < 8) {
            throw apiError("invalidRequest", "La contraseña debe tener al menos 8 caracteres.");
          }
          if (!["manager", "cashier", "accounting"].includes(role)) {
            throw apiError("invalidRequest", "Rol inválido.");
          }
          if (
            state.owner?.name.toLowerCase() === name.toLowerCase() ||
            state.staff.some((member) => member.name.toLowerCase() === name.toLowerCase())
          ) {
            throw apiError("invalidRequest", "Ya existe una persona con ese nombre.");
          }
          if (session.role === "manager") {
            if (role === "manager") {
              throw apiError("unauthorized", "Un encargado no puede crear otro encargado.");
            }
            siteId = session.siteId ?? "";
          }
          const site = state.sites.find((item) => item.id === siteId);
          if (!site) {
            throw apiError("invalidRequest", "Selecciona una sede válida.");
          }
          if (site.status !== "active") {
            throw apiError("invalidRequest", "No puedes asignar personal a una sede cerrada.");
          }
          const member: StaffMember = {
            id: uuid(),
            name,
            password,
            role: role as StaffMember["role"],
            siteId,
            status: "active",
            lastLoginAt: null,
            createdAt: now(),
          };
          state.staff.push(member);
          recordAudit({
            actor: state.session,
            siteId: member.siteId,
            action: "staff.created",
            resource: "staff_membership",
          });
          return staffSummary(member);
        }

        case "update_staff": {
          const session = requirePermission("manageStaff");
          const userId = String(args.userId ?? "");
          if (userId === session.userId) {
            throw apiError("invalidRequest", "No puedes modificar tu propia cuenta.");
          }
          const member = state.staff.find((item) => item.id === userId);
          if (!member) {
            throw apiError("invalidRequest", "La cuenta del dueño no se puede modificar.");
          }
          const role = String(args.role ?? "");
          const status = String(args.status ?? "");
          let siteId = String(args.siteId ?? "");
          if (!["manager", "cashier", "accounting"].includes(role)) {
            throw apiError("invalidRequest", "Rol inválido.");
          }
          if (status !== "active" && status !== "revoked") {
            throw apiError("invalidRequest", "Estado inválido.");
          }
          if (session.role === "manager") {
            if (member.siteId !== session.siteId || role === "manager") {
              throw apiError(
                "unauthorized",
                "Solo puedes administrar cajeros y contabilidad de tu sede.",
              );
            }
            siteId = session.siteId ?? "";
          }
          const site = state.sites.find((item) => item.id === siteId);
          if (!site) {
            throw apiError("invalidRequest", "Selecciona una sede válida.");
          }
          if (status === "active" && site.status !== "active") {
            throw apiError("invalidRequest", "No puedes activar personal en una sede cerrada.");
          }
          member.role = role as StaffMember["role"];
          member.siteId = siteId;
          member.status = status;
          recordAudit({
            actor: state.session,
            siteId: member.siteId,
            action: "staff.updated",
            resource: "staff_membership",
          });
          return staffSummary(member);
        }

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
          recordAudit({
            actor: state.session,
            action: "access.simulated",
            resource: "access_decision",
            result: allowed ? "allowed" : "denied",
            correlationId: event.eventId,
          });
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
            schemaVersion: 4,
          };
          state.backups.unshift(backup);
          state.secrets.set(fileName, passphrase);
          recordAudit({
            actor: state.session,
            action: "backup.created",
            resource: "backup",
            correlationId: backup.id,
          });
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

        case "list_audit_events": {
          const session = requirePermission("viewAudit");
          const limit = Math.max(1, Math.min(200, Number(args.limit ?? 100)));
          return state.auditEvents
            .filter(
              (event) =>
                session.role === "owner" ||
                event.siteId === (session.siteId ?? "__unassigned__"),
            )
            .slice(0, limit)
            .map((event) => ({ ...event }));
        }

        case "verify_audit_chain": {
          requirePermission("viewAudit");
          const ordered = [...state.auditEvents].sort((left, right) => left.sequence - right.sequence);
          const valid = ordered.every(
            (event, index) => index === 0 || event.sequence === ordered[index - 1]!.sequence + 1,
          );
          return {
            valid,
            checkedEvents: ordered.length,
            firstBrokenSequence: valid
              ? null
              : ordered.find(
                  (event, index) =>
                    index > 0 && event.sequence !== ordered[index - 1]!.sequence + 1,
                )?.sequence ?? null,
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
