// Task 4.2 · shortcut-display · 文件级断言
//
// 覆盖 spec §6 Acceptance：
// - AC2 SCEN-4.2.2 TEST-4.2.2 user-facing 文案无残留裸 ⌘（helper mac 参数除外）
// - AC3 SCEN-4.2.3 TEST-4.2.3 styles.css maximized chip 含 :root[data-platform="windows"] 规则
// 读源文件做静态断言（CSS content 无法在 jsdom 直接渲染 ::before content）。

import { describe, it, expect } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";

const srcRoot = resolve(__dirname, "../../src");
const read = (rel: string): string =>
  readFileSync(resolve(srcRoot, rel), "utf8");

describe("TEST-4.2.3: styles.css maximized chip CSS content 平台切换", () => {
  const css = read("panels/Terminal/styles.css");

  it("默认/mac 规则保留 ⌘Enter", () => {
    expect(css).toContain('content: "maximized · ⌘Enter or Esc"');
  });

  it('Windows 规则用 :root[data-platform="windows"] 覆盖为 Ctrl+Enter', () => {
    expect(css).toMatch(
      /:root\[data-platform="windows"\][^}]*content:\s*"maximized · Ctrl\+Enter or Esc"/,
    );
  });
});

describe("TEST-4.2.2: 替换后的组件文案改为 formatShortcut 调用", () => {
  // 每个文件曾硬编码 ⌘，替换后应 import + 调 formatShortcut
  const replaced: { file: string; macToken: string }[] = [
    { file: "components/TopBar.tsx", macToken: "⌘B" },
    { file: "components/ActivityStrip.tsx", macToken: "⌘2" },
    { file: "panels/Terminal/PaneTerminal.tsx", macToken: "⌘⇧O" },
    { file: "panels/CommitBar/CommitBar.tsx", macToken: "⌘↵" },
    { file: "App.tsx", macToken: "⌘," },
    { file: "dialogs/ConfigImport/ConfigImportDialog.tsx", macToken: "⌘T" },
  ];

  for (const { file, macToken } of replaced) {
    it(`${file} 调 formatShortcut（mac 参数仍含 ${macToken}）`, () => {
      const src = read(file);
      expect(src).toContain("formatShortcut");
      // mac 符号只应作为 formatShortcut 的字符串参数出现（含多行 prettier 排版），
      // 不再裸用于 title / placeholder 文案
      expect(src).toContain(`"${macToken}`);
    });
  }
});
