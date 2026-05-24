import { describe, it, expect } from "vitest";
import {
  isFieldDisabled,
  getDisabledReason,
} from "../../../src/dialogs/ConfigImport/fieldRules";
import type { ImportFieldType } from "../../../src/bindings";

/**
 * Issue #205 Finding 2 (a) · v0.1 实务方案的字段 disabled 规则
 *
 * AnsiColor 在 v0.1 不入 app_settings · 不能让用户错觉勾选会生效。
 */
describe("fieldRules (Issue #205 Finding 2)", () => {
  describe("isFieldDisabled", () => {
    it("AnsiColor 字段 disabled (v0.1 不入 app_settings)", () => {
      const field: ImportFieldType = {
        kind: "ansiColor",
        index: 0,
        hex: "#000000",
      };
      expect(isFieldDisabled(field)).toBe(true);
    });

    it("FontFamily 字段不 disabled", () => {
      const field: ImportFieldType = {
        kind: "fontFamily",
        value: "JetBrains Mono",
      };
      expect(isFieldDisabled(field)).toBe(false);
    });

    it("Theme 字段不 disabled", () => {
      const field: ImportFieldType = { kind: "theme", value: "dark" };
      expect(isFieldDisabled(field)).toBe(false);
    });

    it("Shell 字段不 disabled", () => {
      const field: ImportFieldType = { kind: "shell", value: "/bin/zsh" };
      expect(isFieldDisabled(field)).toBe(false);
    });

    it("KeyBinding 字段不 disabled", () => {
      const field: ImportFieldType = {
        kind: "keyBinding",
        key: "Cmd+T",
        action: "new_tab",
      };
      expect(isFieldDisabled(field)).toBe(false);
    });
  });

  describe("getDisabledReason", () => {
    it("AnsiColor 返回 v0.2 支持文案 (含'v0.2')", () => {
      const field: ImportFieldType = {
        kind: "ansiColor",
        index: 7,
        hex: "#cccccc",
      };
      const reason = getDisabledReason(field);
      expect(reason).toBeDefined();
      expect(reason).toContain("v0.2");
    });

    it("非 disabled 字段返回 undefined", () => {
      const field: ImportFieldType = { kind: "theme", value: "dark" };
      expect(getDisabledReason(field)).toBeUndefined();
    });
  });
});
