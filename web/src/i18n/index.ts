import { dictionaries, type Language } from "./dictionaries";

export { dictionaries, type Language };

export function normalizeLanguage(value: string | null | undefined): Language {
  return value === "zh-Hans" ? "zh-Hans" : "en";
}

export function getDictionary(language: string | null | undefined) {
  return dictionaries[normalizeLanguage(language)];
}

export function setDocumentLanguage(language: string | null | undefined): void {
  document.documentElement.lang = normalizeLanguage(language);
}

function lookup(dictionary: unknown, key: string): string | undefined {
  const value = key.split(".").reduce<unknown>((current, part) => {
    if (typeof current !== "object" || current === null) {
      return undefined;
    }
    return (current as Record<string, unknown>)[part];
  }, dictionary);

  return typeof value === "string" ? value : undefined;
}

export function t(key: string, language: string | null | undefined): string {
  const selected = getDictionary(language);
  return lookup(selected, key) ?? lookup(dictionaries.en, key) ?? key;
}
