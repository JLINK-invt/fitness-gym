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
  await page.getByRole("heading", { name: "Diagnóstico del core" }).waitFor();
  if ((await page.getByText("Vista web sin runtime Tauri").count()) > 0) {
    throw new Error("La app no entró en modo Tauri simulado.");
  }
  if ((await page.getByText("SQLCipher 4.6.1 community").count()) === 0) {
    throw new Error("No se mostró el estado de SQLCipher.");
  }
  if ((await page.getByText(/organizaci[oó]n/i).count()) > 0) {
    throw new Error("La UI sigue mostrando el concepto de organización.");
  }
  pass("diagnóstico carga health y estado cifrado");

  await page
    .getByRole("textbox", { name: "Nombre del gimnasio" })
    .fill("Gimnasio E2E");
  await page.getByRole("button", { name: "Configurar gimnasio" }).click();
  await page.getByText(/Gimnasio configurado\. Identificador interno:/).waitFor();
  await page.getByText(/Este equipo ya está configurado para su gimnasio/).waitFor();
  if ((await page.getByRole("button", { name: "Configurar gimnasio" }).count()) > 0) {
    throw new Error("La app todavía permite configurar el gimnasio dos veces.");
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
  pass("configurar el gimnasio actualiza health");

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
  await page.getByText(/Restauración completada desde/).waitFor();
  if ((await rows.count()) !== 1) {
    throw new Error("El restore duplicó el snapshot en la tabla.");
  }
  pass("restore correcto e integridad ok");

  await page.screenshot({ path: "backups-e2e.png", fullPage: true });
  return { checks: passed.length, passed };
}
