async (page) => {
  const passed = [];
  const pass = (name) => {
    passed.push(name);
    console.log(`PASS ${name}`);
  };

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

  await page.getByRole("link", { name: "Acceso" }).click();
  await page.getByRole("heading", { name: "Acceso simulado" }).waitFor();
  await page.getByRole("button", { name: "Simular acceso" }).click();
  await page.getByText("Acceso permitido").waitFor();
  const accessRows = page.locator("tbody tr");
  if ((await accessRows.count()) !== 1) {
    throw new Error(`Se esperaba 1 evento de acceso, hay ${await accessRows.count()}.`);
  }
  await page.getByRole("button", { name: "Repetir última solicitud" }).click();
  await page.getByText("evento ya existente").waitFor();
  if ((await accessRows.count()) !== 1) {
    throw new Error("La repetición duplicó el evento de acceso.");
  }
  await page.getByRole("combobox", { name: "Credencial" }).selectOption("CRED-DENY");
  await page.getByRole("button", { name: "Simular acceso" }).click();
  await page.getByText("Acceso denegado").waitFor();
  if ((await accessRows.count()) !== 2) {
    throw new Error("El acceso denegado no se reflejó en la tabla.");
  }
  pass("vertical slice de acceso con evento idempotente");

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

  await page.getByRole("link", { name: "Sedes" }).click();
  await page.getByRole("heading", { name: "Sedes", exact: true }).waitFor();
  await page.getByLabel("Nombre de la sede").fill("Sucursal Norte");
  await page.getByRole("button", { name: "Crear sede" }).click();
  await page.getByLabel("Nombre de Sucursal Norte").waitFor();
  pass("crear sede y listarla");

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

  await page.getByRole("link", { name: "Equipo" }).click();
  await page.getByRole("heading", { name: "Equipo" }).waitFor();
  await page.getByLabel("Nombre del empleado").fill("Carlos Caja");
  await page.getByLabel("Sede de la nueva cuenta").selectOption({ label: "Sucursal Norte" });
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
  await page.getByRole("heading", { name: "Diagnóstico del core" }).waitFor();
  await page.getByText("Cajero", { exact: true }).waitFor();
  if ((await page.getByRole("link", { name: "Backups" }).count()) > 0) {
    throw new Error("El cajero no debería ver Backups.");
  }
  if ((await page.getByRole("link", { name: "Equipo" }).count()) > 0) {
    throw new Error("El cajero no debería ver Equipo.");
  }
  if ((await page.getByRole("link", { name: "Sedes" }).count()) > 0) {
    throw new Error("El cajero no debería ver Sedes.");
  }
  if ((await page.getByRole("link", { name: "Catálogo" }).count()) > 0) {
    throw new Error("El cajero no debería ver Catálogo.");
  }
  if ((await page.getByRole("link", { name: "Auditoría" }).count()) > 0) {
    throw new Error("El cajero no debería ver Auditoría.");
  }
  await page.getByRole("link", { name: "Acceso" }).waitFor();
  pass("cajero no ve módulos administrativos");

  await page.getByRole("button", { name: "Salir" }).click();
  await page.getByLabel("Nombre", { exact: true }).fill("Ana E2E");
  await page.getByLabel("Contraseña").fill("clave-segura-123");
  await page.getByRole("button", { name: "Entrar" }).click();
  await page.getByRole("link", { name: "Auditoría" }).click();
  await page.getByRole("heading", { name: "Auditoría" }).waitFor();
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

  await page.screenshot({ path: "e2e-smoke.png", fullPage: true });
  return { checks: passed.length, passed };
}
