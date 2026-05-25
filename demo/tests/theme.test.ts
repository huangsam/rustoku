import { beforeEach, describe, expect, it, vi, afterEach } from "vitest";
import { applyTheme, getStoredTheme } from "../src/theme";

describe("theme.ts module", () => {
  let storage: {
    getItem: (key: string) => string | null;
    setItem: (key: string, value: string) => void;
    clear: () => void;
  };

  beforeEach(() => {
    const values = new Map<string, string>();
    storage = {
      getItem: (key) => values.get(key) ?? null,
      setItem: (key, value) => {
        values.set(key, value);
      },
      clear: () => {
        values.clear();
      },
    };

    vi.stubGlobal("localStorage", storage);
    document.documentElement.removeAttribute("data-theme");
    document.documentElement.classList.remove("no-transition");
    storage.clear();
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
  });

  describe("getStoredTheme", () => {
    it("should return a valid stored theme", () => {
      storage.setItem("rustoku-theme", "forest");

      expect(getStoredTheme()).toBe("forest");
    });

    it("should return null for invalid stored themes", () => {
      storage.setItem("rustoku-theme", "invalid-theme");

      expect(getStoredTheme()).toBeNull();
    });

    it("should return null when storage access throws", () => {
      vi.spyOn(storage, "getItem").mockImplementation(() => {
        throw new Error("storage denied");
      });

      expect(getStoredTheme()).toBeNull();
    });
  });

  describe("applyTheme", () => {
    it("should apply the theme even when storage writes fail", () => {
      vi.spyOn(storage, "setItem").mockImplementation(() => {
        throw new Error("storage denied");
      });

      expect(() => applyTheme("midnight")).not.toThrow();
      expect(document.documentElement.getAttribute("data-theme")).toBe(
        "midnight",
      );
    });
  });
});
