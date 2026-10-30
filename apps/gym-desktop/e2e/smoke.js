async (page) => {
  const passed = [];
  const pass = (name) => {
    passed.push(name);
    console.log(`PASS ${name}`);
  };
  let recoveryCode = "";

  const sectionNav = page.getByRole("navigation", { name: "Secciones" });
  const sectionButton = (name) =>
    sectionNav.getByRole("button", { name, exact: true });

  const openGroup = async (name) => {
    const button = sectionButton(name);
    if ((await button.getAttribute("aria-expanded")) !== "true") {
      await button.click();
    }
    await page.getByRole("menu").waitFor();
  };

  const gotoSection = async (group, item) => {
    await openGroup(group);
    await page.getByRole("menuitem", { name: item, exact: true }).click();
  };

  const signOut = async () => {
    await page.getByRole("button", { name: /Sesión de/ }).click();
    await page.getByRole("menuitem", { name: "Cerrar sesión" }).click();
  };

  const checkSessionRole = async (label) => {
    await page.getByRole("button", { name: /Sesión de/ }).click();
    await page.getByRole("menu").getByText(label, { exact: true }).waitFor();
    await page.keyboard.press("Escape");
    await page.getByRole("menu").waitFor({ state: "hidden" });
  };

  await page.addInitScript(() => {
    window.__e2eAnswers = [];
    window.prompt = () => window.__e2eAnswers.shift() ?? "clave-segura-123";
    window.confirm = () => true;
  });

  await page.goto("http://127.0.0.1:1420/?e2e=1#/");
  await page.reload();
  await page.getByRole("heading", { name: "Configura tu gimnasio" }).waitFor();
  if ((await page.getByText("Vista web sin runtime Tauri").count()) > 0) {
    throw new Error("La app no entró en modo Tauri simulado.");
  }
  await page.getByLabel("Nombre del gimnasio").fill("Gimnasio E2E");
  await page.getByLabel("Tu nombre").fill("Ana E2E");
  await page.getByLabel(/^Contraseña/).fill("clave-segura-123");
  await page.getByLabel("Repetir contraseña").fill("clave-segura-123");
  await page.getByRole("button", { name: "Crear cuenta y entrar" }).click();
  await page.getByRole("heading", { name: "Guarda tu código de recuperación" }).waitFor();
  recoveryCode = (await page.locator("code").first().innerText()).trim();
  if (!/^[A-Z2-9]{5}(-[A-Z2-9]{5}){3}$/.test(recoveryCode)) {
    throw new Error(`Código de recuperación con formato inesperado: ${recoveryCode}`);
  }
  await page.waitForTimeout(31_000);
  if ((await page.getByRole("heading", { name: "Guarda tu código de recuperación" }).count()) === 0) {
    throw new Error("La pantalla del código desapareció sin confirmación.");
  }
  await page
    .getByLabel("Ya guardé el código de recuperación en un lugar seguro.")
    .check();
  await page.getByRole("button", { name: "Continuar" }).click();
  await page.getByRole("heading", { name: "Panel" }).waitFor();
  const breadcrumb = page.getByRole("navigation", { name: "Ruta" });
  await breadcrumb.getByText("Inicio", { exact: true }).waitFor();
  await breadcrumb.getByText("Panel", { exact: true }).waitFor();
  if ((await sectionButton("Inicio").getAttribute("aria-current")) !== "page") {
    throw new Error("La sección activa no se marcó como pestaña.");
  }
  if ((await sectionNav.getByRole("button").count()) < 3) {
    throw new Error("El shell de escritorio no mostró las secciones del menú.");
  }
  await sectionButton("Configuración").click();
  await page.getByRole("menu").waitFor();
  if (
    (await page.getByRole("menuitem", { name: "Diagnóstico", exact: true }).count()) ===
    0
  ) {
    throw new Error("El desplegable de sección no listó sus opciones.");
  }
  await sectionButton("Configuración").click();
  await page.getByRole("menu").waitFor({ state: "hidden" });
  await sectionButton("Socios").click();
  await page.getByRole("menu").waitFor();
  await page.keyboard.press("Escape");
  await page.getByRole("menu").waitFor({ state: "hidden" });
  pass("shell de escritorio con secciones desplegables");

  await gotoSection("Configuración", "Diagnóstico");
  await page.getByRole("heading", { name: "Diagnóstico del core" }).waitFor();
  await breadcrumb.getByText("Configuración", { exact: true }).waitFor();
  await breadcrumb.getByText("Diagnóstico del core", { exact: true }).waitFor();
  if ((await sectionButton("Configuración").getAttribute("aria-current")) !== "page") {
    throw new Error("La ruta no reflejó la sección activa.");
  }
  await page.getByText("Ana E2E").waitFor();
  await checkSessionRole("Dueño");
  pass("primer arranque crea al dueño y abre sesión");

  if ((await page.getByText("SQLCipher 4.6.1 community").count()) === 0) {
    throw new Error("No se mostró el estado de SQLCipher.");
  }
  if ((await page.getByText(/organizaci[oó]n/i).count()) > 0) {
    throw new Error("La UI sigue mostrando el concepto de organización.");
  }
  const configuration = page
    .locator("dt", { hasText: "Configuración" })
    .locator("..")
    .locator("dd");
  if ((await configuration.textContent()) !== "Gimnasio configurado") {
    throw new Error(
      `El estado de configuración no se actualizó: ${await configuration.textContent()}`,
    );
  }
  pass("diagnóstico carga health y estado cifrado");

  await gotoSection("Configuración", "Backups");
  await page.getByText("Aún no hay snapshots.").waitFor();
  pass("lista de backups inicia vacía");

  const phrase = page.getByRole("textbox", { name: /Frase de respaldo/ });
  const create = page.getByRole("button", { name: "Crear backup" });
  await phrase.fill("corta");
  if (!(await create.isDisabled())) {
    throw new Error("El botón permitió una frase menor a 8 caracteres.");
  }
  await phrase.fill("clave-segura-123");
  if (await create.isDisabled()) {
    throw new Error("El botón no se habilitó con una frase válida.");
  }
  pass("validación local de la frase de respaldo");

  await create.click();
  await page.getByText(/Snapshot .* creado\./).waitFor();
  const rows = page.locator("tbody tr");
  if ((await rows.count()) !== 1) {
    throw new Error(`Se esperaba 1 snapshot, hay ${await rows.count()}.`);
  }
  pass("crear backup y reflejarlo en la tabla");

  const restore = page.getByRole("button", { name: "Restaurar" }).first();
  await page.evaluate(() => {
    window.__e2eAnswers = ["frase-incorrecta"];
  });
  await restore.click();
  await page.getByText(/No se pudo descifrar el backup/).waitFor();
  pass("restore rechaza frase incorrecta");

  await page.evaluate(() => {
    window.__e2eAnswers = [];
  });
  await restore.click();
  await page.getByRole("heading", { name: "Entra a tu turno" }).waitFor();
  pass("restore correcto cierra la sesión");

  await page.getByLabel("Nombre", { exact: true }).fill("Ana E2E");
  await page.getByLabel("Contraseña").fill("clave-segura-123");
  await page.getByRole("button", { name: "Entrar" }).click();
  await page.getByRole("heading", { name: "Backups cifrados" }).waitFor();
  await page.locator("tbody tr").first().waitFor();
  if ((await page.locator("tbody tr").count()) !== 1) {
    throw new Error("El restore duplicó el snapshot en la tabla.");
  }
  pass("login posterior al restore conserva el backup");

  await gotoSection("Ventas y caja", "Catálogo");
  await page.getByRole("heading", { name: "Catálogo" }).waitFor();
  await page.getByLabel("Nombre del producto").fill("Membresía mensual");
  await page.getByLabel("Categoría").fill("Membresías");
  await page.getByLabel("Nombre de la variante").fill("Mensual");
  await page.getByLabel("SKU").fill("MEM-MEN");
  if (!(await page.getByRole("button", { name: "Crear producto" }).isDisabled())) {
    throw new Error("El botón permitió crear un producto sin precio.");
  }
  await page.getByLabel("Precio", { exact: true }).fill("350");
  await page.getByRole("button", { name: "Crear producto" }).click();
  await page.getByText("Producto agregado al catálogo.").waitFor();
  await page.getByLabel("Nombre de Membresía mensual").waitFor();
  await page.getByText("$350.00").waitFor();
  pass("crear producto con precio inicial");

  await page.getByRole("button", { name: "Nuevo precio" }).click();
  await page.getByLabel("Nuevo precio").fill("400");
  await page.getByRole("button", { name: "Guardar precio" }).click();
  await page.getByText("$400.00").waitFor();
  if ((await page.getByText("$350.00").count()) > 0) {
    throw new Error("El precio anterior no debería seguir como precio vigente.");
  }
  pass("cambiar precio crea una versión vigente");

  await page.getByRole("button", { name: "Agregar variante" }).click();
  await page.getByLabel("Nombre de la nueva variante").fill("Mensual premium");
  await page.getByLabel("SKU de la nueva variante").fill("MEM-PRE");
  await page.getByLabel("Precio de la nueva variante").fill("550");
  await page.getByRole("button", { name: "Crear variante" }).click();
  await page.getByText("Mensual premium").waitFor();
  await page.getByText("$550.00").waitFor();
  pass("agregar una segunda variante");

  await page.getByLabel("Nombre del producto").fill("Playera");
  await page.getByLabel("Categoría", { exact: true }).fill("Ropa");
  await page.getByLabel("Nombre de la variante").fill("M");
  await page.getByLabel("SKU").fill("PLAY-M");
  await page.getByLabel("Unidad").selectOption("piece");
  await page.getByLabel("Costo", { exact: true }).fill("100");
  await page.getByLabel("Precio", { exact: true }).fill("250");
  await page.getByRole("button", { name: "Crear producto" }).click();
  await page.getByLabel("Nombre de Playera").waitFor();
  pass("crear producto rastreado por inventario");

  await gotoSection("Socios", "Socios");
  await page.getByRole("heading", { name: "Socios y membresías" }).waitFor();
  await page.getByRole("button", { name: "Planes" }).click();
  await page.getByLabel("Nombre").fill("Mensual E2E");
  await page.getByLabel("Precio").fill("450");
  await page.getByRole("button", { name: "Crear plan" }).click();
  await page.getByText("$450.00").waitFor();
  pass("crear plan de membresía");

  await page.getByRole("main").getByRole("button", { name: "Socios", exact: true }).click();
  await page.getByRole("button", { name: "Nuevo socio" }).click();
  await page.getByLabel("Nombre", { exact: true }).fill("Luisa");
  await page.getByLabel("Apellido").fill("Socia");
  await page.getByLabel("Teléfono").fill("555-0101");
  await page.getByRole("button", { name: "Dar de alta" }).click();
  await page.getByText("SOC-00001").first().waitFor();
  pass("dar de alta a un socio");

  await page.getByRole("button", { name: "Gestionar" }).click();
  await page.getByLabel("Plan de membresía").selectOption({ label: "Mensual E2E · $450.00" });
  await page.getByRole("button", { name: "Alta de membresía" }).click();
  await page.getByText(/Vigente .* a /).waitFor();
  pass("asignar membresía y generar cargo");

  await page.getByRole("button", { name: /Membresía Mensual E2E/ }).click();
  await page.getByLabel("Método de pago").selectOption("transfer");
  await page.getByRole("button", { name: /Registrar pago de/ }).click();
  await page.getByText("Sin cargos pendientes.").waitFor();
  await page.getByText(/Recibo REC-00001/).waitFor();
  pass("cobrar el cargo de la membresía y generar recibo");

  await page.getByRole("button", { name: "Emitir credencial" }).click();
  await page.getByText("GYM-SOC00001-01", { exact: true }).waitFor();
  pass("emitir credencial de acceso para el socio");

  await gotoSection("Socios", "Acceso y asistencia");
  await page.getByRole("heading", { name: "Accesos y asistencia" }).waitFor();
  await page.getByRole("button", { name: "Publicar snapshot" }).click();
  await page.getByText("Checksum").waitFor();
  await page.getByLabel("Credencial").fill("GYM-SOC00001-01");
  await page.getByRole("button", { name: "Simular acceso" }).click();
  await page.getByText("Acceso permitido").waitFor();
  await page.getByText("Luisa Socia").first().waitFor();
  await page.getByText("GYM-SOC00001-01").first().waitFor();
  await page.getByRole("heading", { name: "Asistencia y ocupación" }).waitFor();
  pass("publicar snapshot y validar acceso con la credencial emitida");

  await gotoSection("Socios", "Socios");
  await page.getByRole("heading", { name: "Socios y membresías" }).waitFor();
  await page.getByRole("button", { name: "Prospectos" }).click();
  await page.getByLabel("Nombre").fill("Pedro Prospecto");
  await page.getByLabel("Teléfono").fill("555-0202");
  await page.getByRole("button", { name: "Agregar prospecto" }).click();
  await page.getByText("Pedro Prospecto").waitFor();
  await page.getByRole("button", { name: "Convertir en socio" }).click();
  await page.getByText("Socio creado").waitFor();
  pass("convertir prospecto en socio");

  await gotoSection("Configuración", "Colaboradores");
  await page.getByRole("heading", { name: "Colaboradores" }).waitFor();
  await page.getByLabel("Nombre", { exact: true }).fill("Carlos Caja");
  await page.getByLabel("Contraseña inicial").fill("clave-carlos-123");
  await page.getByRole("button", { name: "Crear acceso" }).click();
  await page.getByText("Acceso creado. Ya puede iniciar sesión.").waitFor();
  await page.getByText("Carlos Caja").waitFor();
  pass("crear acceso de recepción");

  const carlosRow = page.locator("tbody tr", { hasText: "Carlos Caja" });
  await carlosRow.getByRole("button", { name: /^Permisos/ }).click();
  await page.getByRole("heading", { name: "Permisos de Carlos Caja" }).waitFor();
  const catalogPermission = page.getByLabel(
    "Permiso Administrar catálogo y precios para Carlos Caja",
  );
  await catalogPermission.selectOption("allow");
  await page.getByRole("button", { name: "Guardar permisos" }).click();
  await page.getByText("Permisos actualizados.").waitFor();
  await carlosRow.getByRole("button", { name: "Permisos (1)" }).waitFor();
  await catalogPermission.selectOption("inherit");
  await page.getByRole("button", { name: "Guardar permisos" }).click();
  await carlosRow.getByRole("button", { name: "Permisos", exact: true }).waitFor();
  pass("conceder y revertir un permiso granular a un colaborador");

  await page.getByLabel("Nombre", { exact: true }).fill("Marta Gerente");
  await page.getByLabel("Rol de la nueva cuenta").selectOption("manager");
  await page.getByLabel("Contraseña inicial").fill("clave-marta-123");
  await page.getByRole("button", { name: "Crear acceso" }).click();
  await page.getByText("Acceso creado. Ya puede iniciar sesión.").waitFor();
  await page.getByText("Marta Gerente").waitFor();
  pass("crear acceso de gerencia para aprobar cambios críticos");

  await signOut();
  await page.getByRole("heading", { name: "Entra a tu turno" }).waitFor();
  await page.getByLabel("Nombre", { exact: true }).fill("Carlos Caja");
  await page.getByLabel("Contraseña").fill("clave-carlos-123");
  await page.getByRole("button", { name: "Entrar" }).click();
  await page.getByRole("heading", { name: "Panel" }).waitFor();
  await checkSessionRole("Recepción");
  await openGroup("Configuración");
  const receptionMenu = page.getByRole("menu");
  if (
    (await receptionMenu.getByRole("menuitem", { name: "Backups", exact: true }).count()) >
    0
  ) {
    throw new Error("Recepción no debería ver Backups.");
  }
  if (
    (await receptionMenu.getByRole("menuitem", { name: "Colaboradores", exact: true }).count()) >
    0
  ) {
    throw new Error("Recepción no debería ver Colaboradores.");
  }
  if (
    (await receptionMenu.getByRole("menuitem", { name: "Auditoría", exact: true }).count()) >
    0
  ) {
    throw new Error("Recepción no debería ver Auditoría.");
  }
  if ((await sectionButton("Finanzas").count()) > 0) {
    throw new Error("Recepción no debería ver la sección Finanzas.");
  }
  await gotoSection("Socios", "Acceso y asistencia");
  await page.getByRole("heading", { name: "Accesos y asistencia" }).waitFor();
  await gotoSection("Inicio", "Panel");
  await page.getByRole("heading", { name: "Panel", exact: true }).waitFor();
  pass("recepción ve accesos pero no módulos administrativos");

  await gotoSection("Ventas y caja", "Ventas");
  await page
    .getByText("Abre la caja y registra el fondo inicial antes de cobrar.")
    .waitFor();
  pass("venta exige un turno de caja abierto");

  await gotoSection("Ventas y caja", "Caja");
  await page.getByRole("heading", { name: "Caja y turnos" }).waitFor();
  await page.getByLabel("Monto inicial").fill("500");
  await page.getByRole("button", { name: "Abrir caja" }).click();
  await page.getByText("Turno abierto", { exact: true }).waitFor();
  await page.getByText("Esperado en caja").waitFor();
  pass("recepción abre la caja con fondo inicial");

  await page.getByLabel("Tipo").selectOption("expense");
  await page.getByLabel("Monto").fill("50");
  await page.getByLabel("Categoría").fill("Limpieza");
  await page.getByLabel("Método").selectOption("cash");
  await page.getByLabel("Evidencia").fill("Nota 42");
  await page.getByRole("button", { name: "Registrar movimiento" }).click();
  await page.getByText("Limpieza").waitFor();
  await page.getByText("$50.00").first().waitFor();
  pass("recepción registra un gasto de caja con evidencia");

  await gotoSection("Ventas y caja", "Inventario");
  await page.getByRole("heading", { name: "Inventario", exact: true }).waitFor();
  await page.getByLabel("Producto").selectOption({ label: "Playera · M · PLAY-M" });
  await page.getByLabel("Cantidad").fill("5");
  await page.getByLabel("Costo unitario").fill("100");
  await page.getByLabel("Referencia").fill("OC-E2E");
  await page.getByLabel("Nota").fill("Compra E2E");
  await page.getByRole("button", { name: "Registrar entrada" }).click();
  await page.getByText("Compra E2E").waitFor();
  await page.getByText("$500.00").first().waitFor();
  pass("recepción registra una entrada y el valor del inventario");

  const threshold = page
    .locator("tbody tr", { hasText: "Playera" })
    .getByRole("spinbutton");
  await threshold.fill("5");
  await threshold.press("Tab");

  await gotoSection("Inicio", "Panel");
  await page.getByRole("heading", { name: "Panel", exact: true }).waitFor();
  await page.getByText("1 SKU en o bajo mínimo").waitFor();
  await page.locator("tbody").getByText("Playera").waitFor();
  await page.getByText("5 u", { exact: true }).waitFor();
  await page.getByText("$500.00").first().waitFor();
  pass("panel refleja existencias y alerta de mínimo");

  await gotoSection("Ventas y caja", "Ventas");
  await page.getByRole("heading", { name: "Ventas", exact: true }).waitFor();
  await page.getByRole("button", { name: /Membresía mensual/ }).first().click();
  await page.getByLabel("Efectivo recibido").fill("500");
  await page.getByText("Cambio: $100.00").waitFor();
  await page.getByRole("button", { name: "Completar venta" }).click();
  await page.getByRole("heading", { name: "Ticket de venta" }).waitFor();
  await page.getByText("Ticket no fiscal").waitFor();
  await page.evaluate(() => {
    window.__e2ePrintCount = 0;
    window.print = () => {
      window.__e2ePrintCount += 1;
    };
  });
  await page.getByRole("button", { name: "Imprimir", exact: true }).click();
  if ((await page.evaluate(() => window.__e2ePrintCount)) !== 1) {
    throw new Error("El ticket no invocó el diálogo de impresión del sistema.");
  }
  pass("recepción cobra una venta en efectivo e imprime el ticket no fiscal");

  await page.getByRole("button", { name: "Devolver" }).first().click();
  await page
    .getByLabel("Cantidad a devolver Membresía mensual")
    .fill("1");
  await page
    .getByLabel("Motivo de la devolución")
    .fill("Cliente canceló la membresía");
  await page.getByRole("button", { name: "Registrar devolución" }).click();
  await page.getByText("Devolución registrada").waitFor();
  pass("recepción devuelve una venta con motivo");

  await gotoSection("Ventas y caja", "Caja");
  await page.getByText("Devoluciones efectivo").waitFor();
  await page.getByText("$450.00").first().waitFor();
  await page.getByLabel("Efectivo contado").fill("440");
  await page.getByText("-$10.00").waitFor();
  if (
    !(await page.getByRole("button", { name: "Cerrar caja" }).isDisabled())
  ) {
    throw new Error("La caja permitió cerrar con diferencia sin motivo.");
  }
  await page.getByLabel("Motivo de la diferencia").fill("Faltante de cambio");
  await page.getByRole("button", { name: "Cerrar caja" }).click();
  await page.getByRole("heading", { name: "Turno cerrado" }).waitFor();
  await page.getByText("Faltante de cambio").waitFor();
  pass("recepción cierra caja y audita la diferencia");

  await signOut();
  await page.getByLabel("Nombre", { exact: true }).fill("Ana E2E");
  await page.getByLabel("Contraseña").fill("clave-segura-123");
  await page.getByRole("button", { name: "Entrar" }).click();
  await gotoSection("Ventas y caja", "Caja");
  await page.getByRole("heading", { name: "Caja y turnos" }).waitFor();
  await page.getByRole("heading", { name: "Turnos recientes" }).waitFor();
  const shiftRows = page.locator("tbody tr");
  if ((await shiftRows.count()) !== 1) {
    throw new Error(`Se esperaba 1 turno, hay ${await shiftRows.count()}.`);
  }
  await page.getByText("$440.00").first().waitFor();
  pass("dueño consulta el historial de turnos");
  await gotoSection("Inicio", "Reportes");
  await page.getByRole("heading", { name: "Reportes", exact: true }).waitFor();
  await page.getByRole("heading", { name: "Analítica de ventas e inventario" }).waitFor();
  await page.getByText("Attach rate", { exact: true }).waitFor();
  await page.getByText("Rotación", { exact: true }).waitFor();
  await page.getByRole("heading", { name: "Métodos de pago" }).waitFor();
  await page.getByRole("heading", { name: "Ventas por producto" }).waitFor();
  await page.getByText("Membresía mensual").first().waitFor();
  await page.getByText("$400.00").first().waitFor();
  await page.getByText("-$10.00").waitFor();
  pass("reporte de caja, ventas, margen y analítica reconcilia el rango");

  await page.getByRole("button", { name: "Exportar CSV" }).click();
  await page.getByText(/Exportado reporte-.*\.csv/).waitFor();
  pass("exportar CSV genera archivo auditado");

  await page.getByRole("button", { name: "Programar" }).click();
  await page.getByText("Semanal · 7 días").waitFor();
  await page.getByRole("button", { name: "Ejecutar pendientes" }).click();
  await page.getByText("completed", { exact: true }).waitFor();
  pass("programar y ejecutar reporte financiero");

  await gotoSection("Socios", "Clases y reservas");
  await page.getByRole("heading", { name: "Nueva clase" }).waitFor();
  const classForm = page.locator("form").filter({ hasText: "Nueva clase" });
  await classForm.getByLabel("Nombre").fill("Spinning E2E");
  await classForm.getByLabel("Sala").fill("Sala 1");
  await classForm.getByRole("button", { name: "Crear clase" }).click();
  await page.getByRole("button", { name: /Spinning E2E/ }).click();
  const bookingForm = page.locator("form").filter({ hasText: "Reservar" });
  await bookingForm.getByRole("combobox").selectOption({ label: "Pedro Prospecto" });
  await bookingForm.getByRole("button", { name: "Reservar" }).click();
  await page.locator("li", { hasText: "Pedro Prospecto" }).waitFor();
  pass("crear clase y reservar un socio");

  await gotoSection("Marketing", "Marketing y retención");
  await page.getByRole("heading", { name: "Embudo de prospectos" }).waitFor();
  const campaignForm = page.locator("section", { hasText: "Nueva campaña" });
  await campaignForm.getByLabel("Nombre").fill("Retención E2E");
  await campaignForm.getByLabel("Audiencia").fill("Socios por vencer");
  await campaignForm.getByRole("button", { name: "Crear campaña" }).click();
  await page.getByRole("cell", { name: "Retención E2E" }).waitFor();
  pass("crear campaña y consultar el embudo comercial");

  await gotoSection("Finanzas", "Finanzas");
  await page.getByRole("heading", { name: "Cobros por método" }).waitFor();
  await page.getByText("Venta neta").waitFor();
  await page.getByRole("button", { name: "Exportar CSV" }).click();
  await page.getByText(/Exportado en/).waitFor();
  pass("consultar finanzas y exportar el periodo");

  await gotoSection("Configuración", "General");
  await page.getByRole("heading", { name: "Identidad" }).waitFor();
  await page.getByLabel("Nombre comercial").fill("Gimnasio E2E Centro");
  await page.getByRole("button", { name: "Solicitar cambio" }).click();
  await page.getByText(/Solicitud enviada/).waitFor();
  pass("solicitar un cambio crítico de configuración");

  await signOut();
  await page.getByRole("heading", { name: "Entra a tu turno" }).waitFor();
  await page.getByLabel("Nombre", { exact: true }).fill("Marta Gerente");
  await page.getByLabel("Contraseña").fill("clave-marta-123");
  await page.getByRole("button", { name: "Entrar" }).click();
  await sectionNav.waitFor();
  await gotoSection("Configuración", "General");
  await page.getByRole("heading", { name: "Cambio de configuración pendiente" }).waitFor();
  await page.getByRole("button", { name: "Aprobar y aplicar" }).click();
  await page.getByText(/Cambio aprobado/).waitFor();
  pass("aprobar el cambio con una segunda persona y aplicar la configuración");

  await signOut();
  await page.getByRole("heading", { name: "Entra a tu turno" }).waitFor();
  await page.getByLabel("Nombre", { exact: true }).fill("Ana E2E");
  await page.getByLabel("Contraseña").fill("clave-segura-123");
  await page.getByRole("button", { name: "Entrar" }).click();
  await sectionNav.waitFor();
  await gotoSection("Configuración", "General");
  if ((await page.getByLabel("Nombre comercial").inputValue()) !== "Gimnasio E2E Centro") {
    throw new Error("La configuración aprobada no se aplicó.");
  }
  pass("verificar la configuración aplicada tras la aprobación");

  await gotoSection("Configuración", "Mantenimiento");
  await page.getByRole("heading", { name: "Equipos" }).waitFor();
  await page.getByPlaceholder("Nombre").fill("Caminadora E2E");
  await page.getByPlaceholder("Código").fill("EQ-001");
  await page.getByPlaceholder("Área").fill("Cardio");
  await page.getByRole("button", { name: "Agregar" }).click();
  await page.getByText("Caminadora E2E · EQ-001").waitFor();
  const orderForm = page.locator("section", { hasText: "Nueva orden" });
  await orderForm.getByLabel("Equipo").selectOption({ label: "Caminadora E2E" });
  await orderForm.getByLabel("Resumen").fill("Revisión E2E");
  await orderForm.getByRole("button", { name: "Crear orden" }).click();
  const orderRow = page.locator("tbody tr", { hasText: "Revisión E2E" });
  await orderRow.getByRole("button", { name: "Completar" }).click();
  await orderRow.getByText("Completada").waitFor();
  pass("registrar equipo y cerrar una orden de mantenimiento");

  await gotoSection("Configuración", "Complementos");
  await page.getByRole("heading", { name: "Módulos conectados" }).waitFor();
  await page.getByText("Pendientes").waitFor();
  await page.getByLabel("Identificador").fill("pos-e2e");
  await page.getByLabel("Nombre visible").fill("POS E2E");
  await page.getByLabel(/Capacidades/).fill("members.read, sales.write");
  await page.getByRole("button", { name: "Guardar integración" }).click();
  await page.getByRole("cell", { name: "POS E2E" }).waitFor();
  pass("conectar un complemento y ver el journal de sincronización");

  await page.getByRole("heading", { name: "Sincronización con el servidor" }).waitFor();
  if ((await page.getByLabel("Dirección del servidor").inputValue()) !== "https://sync.e2e.test") {
    throw new Error("La configuración de sincronización no se cargó.");
  }
  await page.getByRole("button", { name: "Sincronizar ahora" }).click();
  await page.getByText(/Enviados 3/).waitFor();
  pass("sincronizar con el servidor mutual TLS y ver el reporte");

  await gotoSection("Configuración", "Auditoría");
  await page.getByRole("heading", { name: "Auditoría" }).waitFor();
  await page.getByText("report.exported").first().waitFor();
  await page.getByText(/Cadena íntegra:/).waitFor();
  if ((await page.locator("tbody tr").count()) === 0) {
    throw new Error("La auditoría no mostró eventos.");
  }
  await page.getByRole("button", { name: "Exportar auditoría" }).click();
  await page.getByText(/Exportaste/).waitFor();
  await page.getByText(/SHA-256:/).waitFor();
  pass("auditoría lista eventos, verifica la cadena y exporta con checksum");

  await gotoSection("Configuración", "Colaboradores");
  const staffRow = page.locator("tbody tr", { hasText: "Carlos Caja" });
  await staffRow.getByRole("button", { name: "Desactivar" }).click();
  await staffRow.getByText("Inactivo").waitFor();
  pass("desactivar acceso de empleado");

  await signOut();
  await page.getByRole("heading", { name: "Entra a tu turno" }).waitFor();
  await page.getByLabel("Nombre", { exact: true }).fill("Carlos Caja");
  await page.getByLabel("Contraseña").fill("clave-carlos-123");
  await page.getByRole("button", { name: "Entrar" }).click();
  await page.getByText("Nombre o contraseña incorrectos.").waitFor();
  pass("empleado desactivado ya no puede entrar");

  await page.getByRole("button", { name: "¿Olvidaste la contraseña del dueño?" }).click();
  await page.getByRole("heading", { name: "Recupera la cuenta" }).waitFor();
  await page.getByLabel("Código de recuperación").fill(recoveryCode);
  await page.getByLabel(/^Contraseña nueva/).fill("clave-nueva-456");
  await page.getByLabel("Repetir contraseña nueva").fill("clave-nueva-456");
  await page.getByRole("button", { name: "Fijar contraseña y entrar" }).click();
  await page.getByRole("heading", { name: "Tu nuevo código de recuperación" }).waitFor();
  const nextRecoveryCode = (await page.locator("code").first().innerText()).trim();
  if (!nextRecoveryCode || nextRecoveryCode === recoveryCode) {
    throw new Error("La recuperación no rotó el código.");
  }
  await page
    .getByLabel("Ya guardé el código de recuperación en un lugar seguro.")
    .check();
  await page.getByRole("button", { name: "Continuar" }).click();
  await page.getByRole("heading", { name: "Colaboradores" }).waitFor();
  await gotoSection("Configuración", "Diagnóstico");
  await page.getByRole("heading", { name: "Diagnóstico del core" }).waitFor();
  pass("código de recuperación permite fijar contraseña y entrar");

  await signOut();
  await page.getByLabel("Nombre", { exact: true }).fill("Ana E2E");
  await page.getByLabel("Contraseña").fill("clave-nueva-456");
  await page.getByRole("button", { name: "Entrar" }).click();
  await page.getByRole("heading", { name: "Diagnóstico del core" }).waitFor();
  pass("la contraseña recuperada inicia sesión");

  await page.screenshot({ path: "e2e-smoke.png", fullPage: true });
  return { checks: passed.length, passed };
}
