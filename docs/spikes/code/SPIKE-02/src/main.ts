// SPIKE-02 · Plugin smoke test UI (TypeScript)
// 3 个 plugin 的最小闭环验证：clipboard-manager / fs / (IME echo 走浏览器原生)
import { writeText, readText } from "@tauri-apps/plugin-clipboard-manager";
import {
  writeTextFile,
  readTextFile,
  BaseDirectory,
} from "@tauri-apps/plugin-fs";

// 测试用文本 · 含中日英 + emoji · 验证 UTF-8 全链路
const TEST_TEXT = "Hello · 你好 · こんにちは · 🎉 SPIKE-02";
const FS_FILE = ".vibestation-spike-02-test.txt";

function getErrorMessage(error: unknown): string {
  if (error instanceof Error) return error.message;
  return String(error);
}

function setResult(elId: string, msg: string, isError = false): void {
  const el = document.querySelector<HTMLParagraphElement>(`#${elId}`);
  if (el) {
    el.textContent = msg;
    el.className = isError ? "result error" : "result ok";
  }
}

async function handleClipboardWrite(): Promise<void> {
  try {
    await writeText(TEST_TEXT);
    setResult(
      "clipboard-result",
      `✅ 已写入剪贴板：${TEST_TEXT}（切到别的 app Cmd+V 验证）`,
    );
  } catch (e: unknown) {
    setResult("clipboard-result", `❌ 写入失败：${getErrorMessage(e)}`, true);
  }
}

async function handleClipboardRead(): Promise<void> {
  try {
    const text = await readText();
    setResult(
      "clipboard-result",
      `✅ 读出剪贴板（长度 ${text?.length ?? 0}）：${text}`,
    );
  } catch (e: unknown) {
    setResult("clipboard-result", `❌ 读取失败：${getErrorMessage(e)}`, true);
  }
}

async function handleFsWrite(): Promise<void> {
  try {
    await writeTextFile(FS_FILE, TEST_TEXT, { baseDir: BaseDirectory.Home });
    setResult(
      "fs-result",
      `✅ 已写入 ~/${FS_FILE}（${TEST_TEXT.length} 字符）`,
    );
  } catch (e: unknown) {
    setResult("fs-result", `❌ 写入失败：${getErrorMessage(e)}`, true);
  }
}

async function handleFsRead(): Promise<void> {
  try {
    const content = await readTextFile(FS_FILE, {
      baseDir: BaseDirectory.Home,
    });
    setResult(
      "fs-result",
      `✅ 读出 ~/${FS_FILE}（长度 ${content.length}）：${content}`,
    );
  } catch (e: unknown) {
    setResult("fs-result", `❌ 读取失败：${getErrorMessage(e)}`, true);
  }
}

function setupImeEcho(): void {
  const input = document.querySelector<HTMLInputElement>("#ime-input");
  const echo = document.querySelector<HTMLParagraphElement>("#ime-echo");
  input?.addEventListener("input", () => {
    if (echo) {
      echo.textContent = `当前输入：${input.value}（字符数 ${input.value.length} · byteLength ${new TextEncoder().encode(input.value).length}）`;
    }
  });
}

window.addEventListener("DOMContentLoaded", () => {
  document
    .querySelector("#clipboard-write")
    ?.addEventListener("click", handleClipboardWrite);
  document
    .querySelector("#clipboard-read")
    ?.addEventListener("click", handleClipboardRead);
  document.querySelector("#fs-write")?.addEventListener("click", handleFsWrite);
  document.querySelector("#fs-read")?.addEventListener("click", handleFsRead);
  setupImeEcho();
});
