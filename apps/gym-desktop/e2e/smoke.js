async (page) => {
  const passed = [];
  const pass = (name) => {
    passed.push(name);
    console.log(`PASS ${name}`);
  };
  let recoveryCode = "";

  await page.addInitScript(() => {
    window.__e2eAnswers = [];
    window.prompt = () => window.__e2eAnswers.shift() ?? "clave-segura-123";
    window.confirm = () => true;
  });

  await page.goto("http://localhost:1420/?e2e=1#/");
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
  await page.getByRole("link", { name: "Diagnóstico" }).click();
  await page.getByRole("heading", { name: "Diagnóstico del core" }).waitFor();
  await page.getByText("Ana E2E").waitFor();
  await page.getByText("Dueño", { exact: true }).waitFor();
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

  await page.getByRole("link", { name: "Backups" }).click();
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

  await page.getByRole("link", { name: "Catálogo" }).click();
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

  await page.getByRole("link", { name: "Equipo" }).click();
  await page.getByRole("heading", { name: "Equipo" }).waitFor();
  await page.getByLabel("Nombre del empleado").fill("Carlos Caja");
  await page.getByLabel("Contraseña inicial").fill("clave-carlos-123");
  await page.getByRole("button", { name: "Crear acceso" }).click();
  await page.getByText("Acceso creado. Ya puede iniciar sesión.").waitFor();
  await page.getByText("Carlos Caja").waitFor();
  pass("crear acceso de cajero");

  await page.getByRole("button", { name: "Salir" }).click();
  await page.getByRole("heading", { name: "Entra a tu turno" }).waitFor();
  await page.getByLabel("Nombre", { exact: true }).fill("Carlos Caja");
  await page.getByLabel("Contraseña").fill("clave-carlos-123");
  await page.getByRole("button", { name: "Entrar" }).click();
  await page.getByRole("heading", { name: "Panel" }).waitFor();
  await page.getByText("Cajero", { exact: true }).waitFor();
  if ((await page.getByRole("link", { name: "Backups" }).count()) > 0) {
    throw new Error("El cajero no debería ver Backups.");
  }
  if ((await page.getByRole("link", { name: "Equipo" }).count()) > 0) {
    throw new Error("El cajero no debería ver Equipo.");
  }
  if ((await page.getByRole("link", { name: "Catálogo" }).count()) > 0) {
    throw new Error("El cajero no debería ver Catálogo.");
  }
  if ((await page.getByRole("link", { name: "Auditoría" }).count()) > 0) {
    throw new Error("El cajero no debería ver Auditoría.");
  }
  if ((await page.getByRole("link", { name: "Acceso" }).count()) > 0) {
    throw new Error("El módulo Acceso simulado ya no debería mostrarse.");
  }
  pass("cajero no ve módulos administrativos");

  await page.getByRole("link", { name: "Ventas" }).click();
  await page
    .getByText("Abre la caja y registra el fondo inicial antes de cobrar.")
    .waitFor();
  pass("venta exige un turno de caja abierto");

  await page.getByRole("link", { name: "Caja", exact: true }).click();
  await page.getByRole("heading", { name: "Caja y turnos" }).waitFor();
  await page.getByLabel("Monto inicial").fill("500");
  await page.getByRole("button", { name: "Abrir caja" }).click();
  await page.getByText("Turno abierto", { exact: true }).waitFor();
  await page.getByText("Esperado en caja").waitFor();
  pass("cajero abre la caja con fondo inicial");

  await page.getByLabel("Tipo").selectOption("expense");
  await page.getByLabel("Monto").fill("50");
  await page.getByLabel("Categoría").fill("Limpieza");
  await page.getByLabel("Método").selectOption("cash");
  await page.getByLabel("Evidencia").fill("Nota 42");
  await page.getByRole("button", { name: "Registrar movimiento" }).click();
  await page.getByText("Limpieza").waitFor();
  await page.getByText("$50.00").first().waitFor();
  pass("cajero registra un gasto de caja con evidencia");

  await page.getByRole("link", { name: "Inventario" }).click();
  await page.getByRole("heading", { name: "Inventario", exact: true }).waitFor();
  await page.getByLabel("Producto").selectOption({ label: "Playera · M · PLAY-M" });
  await page.getByLabel("Cantidad").fill("5");
  await page.getByLabel("Costo unitario").fill("100");
  await page.getByLabel("Referencia").fill("OC-E2E");
  await page.getByLabel("Nota").fill("Compra E2E");
  await page.getByRole("button", { name: "Registrar entrada" }).click();
  await page.getByText("Compra E2E").waitFor();
  await page.getByText("$500.00").first().waitFor();
  pass("cajero registra una entrada y el valor del inventario");

  const threshold = page
    .locator("tbody tr", { hasText: "Playera" })
    .getByRole("spinbutton");
  await threshold.fill("5");
  await threshold.press("Tab");

  await page.getByRole("link", { name: "Panel" }).click();
  await page.getByRole("heading", { name: "Panel", exact: true }).waitFor();
  await page.getByText("1 SKU en o bajo mínimo").waitFor();
  await page.locator("tbody").getByText("Playera").waitFor();
  await page.getByText("5 u", { exact: true }).waitFor();
  await page.getByText("$500.00").first().waitFor();
  pass("panel refleja existencias y alerta de mínimo");

  await page.getByRole("link", { name: "Ventas" }).click();
  await page.getByRole("heading", { name: "Ventas", exact: true }).waitFor();
  await page.getByRole("button", { name: /Membresía mensual/ }).first().click();
  await page.getByLabel("Efectivo recibido").fill("500");
  await page.getByText("Cambio: $100.00").waitFor();
  await page.getByRole("button", { name: "Completar venta" }).click();
  await page.getByRole("heading", { name: "Ticket de venta" }).waitFor();
  await page.getByText("Ticket no fiscal").waitFor();
  pass("cajero cobra una venta en efectivo");

  await page.getByRole("button", { name: "Devolver" }).first().click();
  await page
    .getByLabel("Cantidad a devolver Membresía mensual")
    .fill("1");
  await page
    .getByLabel("Motivo de la devolución")
    .fill("Cliente canceló la membresía");
  await page.getByRole("button", { name: "Registrar devolución" }).click();
  await page.getByText("Devolución registrada").waitFor();
  pass("cajero devuelve una venta con motivo");

  await page.getByRole("link", { name: "Caja", exact: true }).click();
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
  pass("cajero cierra caja y audita la diferencia");

  await page.getByRole("button", { name: "Salir" }).click();
  await page.getByLabel("Nombre", { exact: true }).fill("Ana E2E");
  await page.getByLabel("Contraseña").fill("clave-segura-123");
  await page.getByRole("button", { name: "Entrar" }).click();
  await page.getByRole("link", { name: "Caja", exact: true }).click();
  await page.getByRole("heading", { name: "Caja y turnos" }).waitFor();
  await page.getByRole("heading", { name: "Turnos recientes" }).waitFor();
  const shiftRows = page.locator("tbody tr");
  if ((await shiftRows.count()) !== 1) {
    throw new Error(`Se esperaba 1 turno, hay ${await shiftRows.count()}.`);
  }
  await page.getByText("$440.00").first().waitFor();
  pass("dueño consulta el historial de turnos");
  await page.getByRole("link", { name: "Reportes" }).click();
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

  await page.getByRole("link", { name: "Auditoría" }).click();
  await page.getByRole("heading", { name: "Auditoría" }).waitFor();
  await page.getByText("report.exported").waitFor();
  await page.getByText(/Cadena íntegra:/).waitFor();
  if ((await page.locator("tbody tr").count()) === 0) {
    throw new Error("La auditoría no mostró eventos.");
  }
  pass("auditoría lista eventos y verifica la cadena");

  await page.getByRole("link", { name: "Equipo" }).click();
  const cashierRow = page.locator("tbody tr", { hasText: "Carlos Caja" });
  await cashierRow.getByRole("button", { name: "Desactivar" }).click();
  await cashierRow.getByText("Inactivo").waitFor();
  pass("desactivar acceso de empleado");

  await page.getByRole("button", { name: "Salir" }).click();
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
  await page.getByRole("heading", { name: "Equipo" }).waitFor();
  await page.getByRole("link", { name: "Diagnóstico" }).click();
  await page.getByRole("heading", { name: "Diagnóstico del core" }).waitFor();
  pass("código de recuperación permite fijar contraseña y entrar");

  await page.getByRole("button", { name: "Salir" }).click();
  await page.getByLabel("Nombre", { exact: true }).fill("Ana E2E");
  await page.getByLabel("Contraseña").fill("clave-nueva-456");
  await page.getByRole("button", { name: "Entrar" }).click();
  await page.getByRole("heading", { name: "Diagnóstico del core" }).waitFor();
  pass("la contraseña recuperada inicia sesión");

  await page.screenshot({ path: "e2e-smoke.png", fullPage: true });
  return { checks: passed.length, passed };
}
