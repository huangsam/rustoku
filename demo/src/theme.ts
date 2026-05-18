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

export function applyTheme(theme: ThemeName): void {
  document.documentElement.classList.add("no-transition");
  document.documentElement.setAttribute("data-theme", theme);
  localStorage.setItem("rustoku-theme", theme);

  // Force layout reflow to apply the theme instantly without transitions
  void document.documentElement.offsetHeight;

  // Re-enable transitions on the next paint cycles
  requestAnimationFrame(() => {
    requestAnimationFrame(() => {
      document.documentElement.classList.remove("no-transition");
    });
  });
}
