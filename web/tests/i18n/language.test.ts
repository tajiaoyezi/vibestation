import { describe, expect, it } from "vitest";

const i18nModulePath = "../../src/i18n";
const loadI18n = () => import(/* @vite-ignore */ i18nModulePath);

const collectKeys = (value: unknown, prefix = ""): string[] => {
  // Leaf path rule: FEAT-02 dictionaries are nested objects with string leaves.
  // Functions or rich values are not allowed in this first slice.
  if (typeof value !== "object" || value === null) {
    return [prefix];
  }
  return Object.entries(value as Record<string, unknown>).flatMap(
    ([key, child]) => collectKeys(child, prefix ? `${prefix}.${key}` : key),
  );
};

describe("FEAT-02 i18n core", () => {
  it("TEST-FEAT-02.1: vitest exposes jsdom document", () => {
    expect(document.documentElement).toBeInstanceOf(HTMLHtmlElement);
  });

  it("TEST-FEAT-02.5: en and zh-Hans dictionaries have identical leaf keys", async () => {
    const { dictionaries } = await loadI18n();
    const enKeys = collectKeys(dictionaries.en).sort();
    const zhKeys = collectKeys(dictionaries["zh-Hans"]).sort();
    expect(zhKeys).toEqual(enKeys);
  });

  it("TEST-FEAT-02.4: invalid language falls back to en", async () => {
    const { normalizeLanguage } = await loadI18n();
    expect(normalizeLanguage("en")).toBe("en");
    expect(normalizeLanguage("zh-Hans")).toBe("zh-Hans");
    expect(normalizeLanguage("fr")).toBe("en");
    expect(normalizeLanguage("")).toBe("en");
    expect(normalizeLanguage("zh-CN")).toBe("en");
    expect(normalizeLanguage("ZH-HANS")).toBe("en");
  });

  it("TEST-FEAT-02.1: document lang syncs to selected language", async () => {
    const { setDocumentLanguage } = await loadI18n();
    setDocumentLanguage("zh-Hans");
    expect(document.documentElement.lang).toBe("zh-Hans");
    setDocumentLanguage("en");
    expect(document.documentElement.lang).toBe("en");
  });

  it("TEST-FEAT-02.6: translates known app chrome key", async () => {
    const { t } = await loadI18n();
    expect(t("settings.title", "en")).toBe("Preferences");
    expect(t("settings.title", "zh-Hans")).toBe("偏好设置");
  });

  it("TEST-FEAT-02.6: translates nested dot-notation key", async () => {
    const { t } = await loadI18n();
    expect(t("settings.appearance.language", "en")).toBe("Language");
    expect(t("settings.appearance.language", "zh-Hans")).toBe("语言");
  });

  it("TEST-FEAT-02.6: returns zh-Hans dictionary when selected", async () => {
    const { getDictionary } = await loadI18n();
    expect(getDictionary("zh-Hans").settings.title).toBe("偏好设置");
  });

  it("TEST-FEAT-02.6: returns en dictionary for invalid language", async () => {
    const { getDictionary } = await loadI18n();
    expect(getDictionary("fr").settings.title).toBe("Preferences");
  });

  it("TEST-FEAT-02.6: falls back to en when selected locale misses a key", async () => {
    const { dictionaries, t } = await loadI18n();
    const zhSettings = dictionaries["zh-Hans"].settings as Record<
      string,
      unknown
    >;
    const original = zhSettings.title;

    try {
      delete zhSettings.title;
      expect(t("settings.title", "zh-Hans")).toBe("Preferences");
    } finally {
      zhSettings.title = original;
    }
  });

  it("TEST-FEAT-02.6: returns key when no locale contains the key", async () => {
    const { t } = await loadI18n();
    expect(t("missing.key", "zh-Hans")).toBe("missing.key");
  });
});
