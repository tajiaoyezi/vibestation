import { type Component } from "solid-js";
import { useTheme } from "../stores/theme";
import { useSettings, type ThemeSetting } from "../stores/settings";
import { SunIcon, MoonIcon, AutoIcon } from "./Icons";

export const ThemeSwitch: Component = () => {
  const { theme, setTheme } = useTheme();
  const { updateSettings } = useSettings();

  const themes: { id: ThemeSetting; Icon: Component }[] = [
    { id: "light", Icon: SunIcon },
    { id: "dark", Icon: MoonIcon },
    { id: "auto", Icon: AutoIcon },
  ];

  const handleClick = (id: ThemeSetting) => {
    setTheme(id);
    void updateSettings({ theme: id });
  };

  return (
    <div class="vs-theme-switch" role="radiogroup" aria-label="Theme selection">
      {themes.map((t) => (
        <button
          type="button"
          class={`vs-theme-btn${theme() === t.id ? " vs-theme-btn-on" : ""}`}
          data-theme={t.id}
          role="radio"
          aria-checked={theme() === t.id}
          aria-label={`${t.id} theme`}
          onClick={() => handleClick(t.id)}
        >
          <t.Icon />
        </button>
      ))}
    </div>
  );
};
