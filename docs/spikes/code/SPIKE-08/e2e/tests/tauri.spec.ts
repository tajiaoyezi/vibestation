import { expect, test } from "../fixtures";

test("create/list/delete golden path in real Tauri window", async ({ tauriPage }) => {
  await expect(tauriPage.getByTestId("empty-state")).toBeVisible();

  await tauriPage.fill("[data-testid='workspace-name-input']", "Native Tauri Repo");
  await tauriPage.fill(
    "[data-testid='workspace-root-path-input']",
    "/tmp/spike-08/native-tauri-repo",
  );
  await tauriPage.fill("[data-testid='workspace-note-input']", "tauri-plugin-playwright smoke");

  await tauriPage.click("[data-testid='create-workspace-button']");
  await expect(tauriPage.getByTestId("workspace-count")).toContainText("1 items");

  const deleteButton = tauriPage.locator("[data-testid^='delete-workspace-button-']").first();
  await expect(deleteButton).toBeVisible();
  await deleteButton.click();

  await expect(tauriPage.getByTestId("delete-modal")).toBeVisible();
  await tauriPage.click("[data-testid='confirm-delete-button']");
  await expect(tauriPage.getByTestId("empty-state")).toBeVisible();
  await expect(tauriPage.getByTestId("status-message")).toContainText("已删除");
});
