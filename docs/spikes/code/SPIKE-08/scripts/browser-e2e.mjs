import fs from "node:fs/promises";
import path from "node:path";
import { chromium } from "playwright";

const baseUrl = process.env.SPIKE08_BASE_URL ?? "http://127.0.0.1:1420";
const tracePath = process.env.SPIKE08_TRACE;
const screenshotPath = process.env.SPIKE08_SCREENSHOT;
const workspaceName = process.env.SPIKE08_WORKSPACE_NAME ?? "Golden Path Repo";
const workspaceRoot =
  process.env.SPIKE08_WORKSPACE_ROOT ?? "/tmp/spike-08/golden-path-repo";
const workspaceNote = process.env.SPIKE08_WORKSPACE_NOTE ?? "Playwright browser smoke";

function log(step, detail) {
  const suffix = detail ? ` ${detail}` : "";
  console.log(`[browser-e2e] ${step}${suffix}`);
}

async function ensureParent(filePath) {
  if (!filePath) {
    return;
  }
  await fs.mkdir(path.dirname(filePath), { recursive: true });
}

async function main() {
  const browser = await chromium.launch({ headless: true });
  const context = await browser.newContext();
  const page = await context.newPage();

  if (tracePath) {
    await ensureParent(tracePath);
    await context.tracing.start({ screenshots: true, snapshots: true, sources: true });
  }

  try {
    log("goto", baseUrl);
    await page.goto(baseUrl, { waitUntil: "networkidle" });
    await page.waitForSelector("[data-testid='empty-state']");

    await page.fill("[data-testid='workspace-name-input']", workspaceName);
    await page.fill("[data-testid='workspace-root-path-input']", workspaceRoot);
    await page.fill("[data-testid='workspace-note-input']", workspaceNote);

    log("create", workspaceName);
    await page.click("[data-testid='create-workspace-button']");

    const firstCard = page.locator("[data-testid^='workspace-card-']").first();
    await firstCard.waitFor();
    const cardText = await firstCard.textContent();

    if (!cardText?.includes(workspaceName)) {
      throw new Error(`expected card text to include ${workspaceName}, got: ${cardText}`);
    }

    const cardTestId = await firstCard.getAttribute("data-testid");
    if (!cardTestId) {
      throw new Error("missing workspace card test id");
    }

    const workspaceId = cardTestId.replace("workspace-card-", "");
    log("created-id", workspaceId);

    await page.click(`[data-testid='delete-workspace-button-${workspaceId}']`);
    await page.waitForSelector("[data-testid='delete-modal']");
    await page.click("[data-testid='confirm-delete-button']");
    await page.waitForSelector("[data-testid='empty-state']");

    const statusText = await page.textContent("[data-testid='status-message']");
    if (!statusText?.includes("已删除")) {
      throw new Error(`expected delete status, got: ${statusText}`);
    }

    log("delete-ok", statusText.trim());

    if (screenshotPath) {
      await ensureParent(screenshotPath);
      await page.screenshot({ path: screenshotPath, fullPage: true });
      log("screenshot", screenshotPath);
    }
  } catch (error) {
    if (screenshotPath) {
      await ensureParent(screenshotPath);
      await page.screenshot({ path: screenshotPath, fullPage: true });
      log("failure-screenshot", screenshotPath);
    }
    throw error;
  } finally {
    if (tracePath) {
      await context.tracing.stop({ path: tracePath });
      log("trace", tracePath);
    }
    await browser.close();
  }
}

main().catch((error) => {
  console.error("[browser-e2e] failed", error);
  process.exitCode = 1;
});
