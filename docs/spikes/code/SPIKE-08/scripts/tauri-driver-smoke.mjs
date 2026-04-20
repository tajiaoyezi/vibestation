import fs from "node:fs/promises";
import path from "node:path";
import { Builder, By, Capabilities, until } from "selenium-webdriver";

const driverUrl = process.env.SPIKE08_DRIVER_URL ?? "http://127.0.0.1:4444/";
const application = process.env.SPIKE08_APP_PATH;
const screenshotPath = process.env.SPIKE08_SCREENSHOT;
const workspaceName = process.env.SPIKE08_WORKSPACE_NAME ?? "Driver Repo";
const workspaceRoot =
  process.env.SPIKE08_WORKSPACE_ROOT ?? "/tmp/spike-08/driver-repo";
const workspaceNote = process.env.SPIKE08_WORKSPACE_NOTE ?? "tauri-driver smoke";

if (!application) {
  throw new Error("SPIKE08_APP_PATH is required");
}

function log(step, detail) {
  const suffix = detail ? ` ${detail}` : "";
  console.log(`[tauri-driver] ${step}${suffix}`);
}

async function ensureParent(filePath) {
  if (!filePath) {
    return;
  }
  await fs.mkdir(path.dirname(filePath), { recursive: true });
}

async function main() {
  const capabilities = new Capabilities();
  capabilities.setBrowserName("wry");
  capabilities.set("tauri:options", { application });

  const driver = await new Builder()
    .usingServer(driverUrl)
    .withCapabilities(capabilities)
    .build();

  try {
    const nameInput = await driver.wait(
      until.elementLocated(By.css("[data-testid='workspace-name-input']")),
      30000,
    );
    await nameInput.clear();
    await nameInput.sendKeys(workspaceName);

    const rootInput = await driver.findElement(
      By.css("[data-testid='workspace-root-path-input']"),
    );
    await rootInput.clear();
    await rootInput.sendKeys(workspaceRoot);

    const noteInput = await driver.findElement(
      By.css("[data-testid='workspace-note-input']"),
    );
    await noteInput.clear();
    await noteInput.sendKeys(workspaceNote);

    log("create", workspaceName);
    await driver
      .findElement(By.css("[data-testid='create-workspace-button']"))
      .click();

    const firstCard = await driver.wait(
      until.elementLocated(By.css("[data-testid^='workspace-card-']")),
      30000,
    );
    const cardText = await firstCard.getText();
    if (!cardText.includes(workspaceName)) {
      throw new Error(`expected card to include ${workspaceName}, got: ${cardText}`);
    }

    const cardId = await firstCard.getAttribute("data-testid");
    if (!cardId) {
      throw new Error("missing workspace card test id");
    }

    const workspaceId = cardId.replace("workspace-card-", "");
    log("created-id", workspaceId);

    await driver
      .findElement(By.css(`[data-testid='delete-workspace-button-${workspaceId}']`))
      .click();
    await driver.wait(
      until.elementLocated(By.css("[data-testid='delete-modal']")),
      30000,
    );
    await driver
      .findElement(By.css("[data-testid='confirm-delete-button']"))
      .click();
    await driver.wait(
      until.elementLocated(By.css("[data-testid='empty-state']")),
      30000,
    );

    const statusText = await driver
      .findElement(By.css("[data-testid='status-message']"))
      .getText();
    if (!statusText.includes("已删除")) {
      throw new Error(`expected delete status, got: ${statusText}`);
    }

    log("delete-ok", statusText);

    if (screenshotPath) {
      await ensureParent(screenshotPath);
      const encoded = await driver.takeScreenshot();
      await fs.writeFile(screenshotPath, encoded, "base64");
      log("screenshot", screenshotPath);
    }
  } finally {
    await driver.quit();
  }
}

main().catch((error) => {
  console.error("[tauri-driver] failed", error);
  process.exitCode = 1;
});
