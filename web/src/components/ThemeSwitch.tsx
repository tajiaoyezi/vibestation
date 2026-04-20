import { type Component } from "solid-js";
import { useTheme } from "../stores/theme";

export const ThemeSwitch: Component = () => {
  const { theme, setTheme } = useTheme();

  const themes: { id: "light" | "dark" | "auto"; icon: string }[] = [
    { id: "light", icon: "☀" },
    { id: "dark", icon: "☾" },
    { id: "auto", icon: "⊘" },
  ];

  return (
    <div class="vs-theme-switch" role="radiogroup" aria-label="Theme selection">
      {themes.map((t) => (
        <button
          type="button"
          class={`vs-theme-btn${theme() === t.id ? " vs-theme-btn-on" : ""}`}
          role="radio"
          aria-checked={theme() === t.id}
          aria-label={`${t.id} theme`}
          onClick={() => setTheme(t.id)}
        >
          {t.icon}
        </button>
      ))}
    </div>
  );
};
