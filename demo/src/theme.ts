import type { ThemeName } from "./types";

export const validThemes: ThemeName[] = [
  "midnight",
  "paper",
  "arcade",
  "blueprint",
  "ember",
  "forest",
  "mono",
];

const THEME_STORAGE_KEY = "rustoku-theme";

export function getStoredTheme(): ThemeName | null {
  try {
    const storedTheme = localStorage.getItem(THEME_STORAGE_KEY);
    if (storedTheme && validThemes.includes(storedTheme as ThemeName)) {
      return storedTheme as ThemeName;
    }
  } catch (_err) {
    // Ignore storage access failures and fall back to the default theme.
  }

  return null;
}

export function applyTheme(theme: ThemeName): void {
  document.documentElement.classList.add("no-transition");
  document.documentElement.setAttribute("data-theme", theme);
  try {
    localStorage.setItem(THEME_STORAGE_KEY, theme);
  } catch (_err) {
    // Ignore storage access failures so theme application still succeeds.
  }

  // Force layout reflow to apply the theme instantly without transitions
  void document.documentElement.offsetHeight;

  // Re-enable transitions on the next paint cycles
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      document.documentElement.classList.remove("no-transition");
    });
  });
}
