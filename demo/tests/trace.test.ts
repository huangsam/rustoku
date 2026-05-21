import { vi, describe, it, expect } from "vitest";

// Mock elements to prevent real DOM operations and errors in tests
vi.mock("../src/elements", () => {
  const elem = () => ({
    style: {},
    classList: { add: () => {}, remove: () => {} },
    dataset: {},
    appendChild: () => {},
    addEventListener: () => {},
    hidden: false,
    textContent: "",
    innerHTML: "",
    disabled: false,
    scrollIntoView: () => {},
  });
  return {
    grid: elem(),
    selectGenDifficulty: elem(),
    btnGenerate: elem(),
    btnSolve: elem(),
    btnSolveSteps: elem(),
    btnCandidates: elem(),
    btnCheck: elem(),
    btnErase: elem(),
    btnReset: elem(),
    btnUndo: elem(),
    btnRedo: elem(),
    inputBoard: elem(),
    btnLoadBoard: elem(),
    btnCopyBoard: elem(),
    selectExportFormat: elem(),
    infoPanel: elem(),
    infoTitle: elem(),
    solveTracePanel: elem(),
    solveTraceStepCount: elem(),
    solveTraceStatus: elem(),
    solveTraceTechnique: elem(),
    solveTracePlacement: elem(),
    solveTraceDetail: elem(),
    solveTraceChanges: elem(),
    btnTracePrev: elem(),
    btnTracePlay: elem(),
    btnTraceNext: elem(),
    btnTraceNextPlacement: elem(),
    btnTraceNextElimination: elem(),
    btnCloseInfo: elem(),
    selectTheme: elem(),
    btnInfoHeader: elem(),
    projectModal: elem(),
    selectGenSymmetry: elem(),
    btnModalClose: elem(),
    toastContainer: elem(),
    statDifficulty: elem(),
    statGivens: elem(),
    statProgress: elem(),
    gridLoader: elem(),
    newGameModal: elem(),
    btnNewGameClose: elem(),
    tabGenerate: elem(),
    tabImportExport: elem(),
    tabContentGenerate: elem(),
    tabContentImportExport: elem(),
    btnClearBlank: elem(),
  };
});

import {
  TECHNIQUE_INFO,
  getSolveTrace,
  stopSolveTracePlayback,
  clearSolveTrace,
} from "../src/trace";

describe("trace.ts module", () => {
  describe("TECHNIQUE_INFO", () => {
    it("should contain exactly the 17 standard Sudoku techniques", () => {
      const techniques = Object.keys(TECHNIQUE_INFO);
      expect(techniques).toHaveLength(17);
    });

    it("should map every technique to a valid description and difficulty level", () => {
      const validDifficulties = ["easy", "medium", "hard", "expert"];

      for (const [name, info] of Object.entries(TECHNIQUE_INFO)) {
        expect(name.length).toBeGreaterThan(0);
        expect(info.desc.length).toBeGreaterThan(10); // Check that description is informative
        expect(validDifficulties).toContain(info.difficulty);
      }
    });

    it("should contain correct descriptions for complex/Wing techniques", () => {
      expect(TECHNIQUE_INFO["X-Wing"].desc).toContain("parallel rows");
      expect(TECHNIQUE_INFO["XY-Wing"].desc).toContain(
        "pivot cell and two pincers",
      );
      expect(TECHNIQUE_INFO["XYZ-Wing"].desc).toContain("three candidates");
      expect(TECHNIQUE_INFO["W-Wing"].desc).toContain("bi-value cells");
    });
  });

  describe("SolveTrace state functions", () => {
    it("should return null by default for solveTrace", () => {
      expect(getSolveTrace()).toBeNull();
    });

    it("should handle stop/clear playback gracefully when solveTrace is null", () => {
      expect(() => stopSolveTracePlayback()).not.toThrow();
      expect(() => clearSolveTrace()).not.toThrow();
    });
  });
});
