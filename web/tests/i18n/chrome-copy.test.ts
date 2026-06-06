import { describe, expect, it } from "vitest";
import { t } from "../../src/i18n";

describe("FEAT-02 first Settings chrome copy", () => {
  it("TEST-FEAT-02.6: settings shell labels translate", () => {
    expect(t("settings.title", "en")).toBe("Preferences");
    expect(t("settings.title", "zh-Hans")).toBe("偏好设置");
    expect(t("settings.import", "en")).toBe("Import...");
    expect(t("settings.import", "zh-Hans")).toBe("导入...");
    expect(t("settings.close", "en")).toBe("Close settings");
    expect(t("settings.close", "zh-Hans")).toBe("关闭设置");
  });

  it("TEST-FEAT-02.6: settings group labels translate", () => {
    expect(t("settings.groups.appearance", "en")).toBe("Appearance");
    expect(t("settings.groups.appearance", "zh-Hans")).toBe("外观");
    expect(t("settings.groups.terminal", "en")).toBe("Terminal");
    expect(t("settings.groups.terminal", "zh-Hans")).toBe("终端");
    expect(t("settings.groups.externalTerminal", "en")).toBe(
      "External Terminal",
    );
    expect(t("settings.groups.externalTerminal", "zh-Hans")).toBe("外部终端");
  });

  it("TEST-FEAT-02.6: appearance labels translate", () => {
    expect(t("settings.appearance.theme", "zh-Hans")).toBe("主题");
    expect(t("settings.appearance.auto", "zh-Hans")).toBe("自动");
    expect(t("settings.appearance.fontFamily", "zh-Hans")).toBe("字体");
    expect(t("settings.appearance.backgroundOpacity", "zh-Hans")).toBe(
      "背景不透明度",
    );
    expect(t("settings.appearance.windowPaddingX", "zh-Hans")).toBe(
      "窗口水平内边距",
    );
    expect(t("settings.appearance.cursorBlink", "zh-Hans")).toBe("光标闪烁");
  });
});
