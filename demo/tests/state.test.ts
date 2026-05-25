import { vi, describe, it, expect, beforeEach, afterEach } from "vitest";

// Mock elements to prevent real DOM operations and errors in tests
vi.mock("../src/elements", () => {
  return {
    inputBoard: { value: "" },
    selectExportFormat: { value: "zero" },
  };
});

import { inputBoard, selectExportFormat } from "../src/elements";
import {
  STORAGE_KEYS,
  state,
  normalizeBoardInput,
  isPlacementValid,
  computeInvalidCells,
  boardForExport,
  syncBoardInput,
  hydrateBoardState,
} from "../src/state";

describe("state.ts module", () => {
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

    // Reset mocked element values
    if (inputBoard) inputBoard.value = "";
    if (selectExportFormat) selectExportFormat.value = "zero";
    storage.clear();
    state.currentBoard = "0".repeat(81);
    state.givenMask = Array(81).fill(false);
    state.undoStack = [];
    state.redoStack = [];
    state.currentDifficulty = "custom";
  });

  afterEach(() => {
    vi.restoreAllMocks();
    vi.unstubAllGlobals();
  });

  describe("normalizeBoardInput", () => {
    it("should return the original string if it is already a valid 81-digit string", () => {
      const valid =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
      expect(normalizeBoardInput(valid)).toBe(valid);
    });

    it("should normalize dots and underscores to zeros", () => {
      const dots =
        "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
      const underscores =
        "53__7____6__195____98____6_8___6___34__8_3__17___2___6_6____28____419__5____8__79";
      const expected =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
      expect(normalizeBoardInput(dots)).toBe(expected);
      expect(normalizeBoardInput(underscores)).toBe(expected);
    });

    it("should strip out all whitespace characters", () => {
      const spacing =
        " 53..7....6..195. \n ...98....6.8...6. ..34..8.3..17...2...6.6.. \r\n ..28....419..5....8..79 ";
      const expected =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
      expect(normalizeBoardInput(spacing)).toBe(expected);
    });

    it("should return null for strings that are too short or too long", () => {
      expect(normalizeBoardInput("12345")).toBeNull();
      expect(normalizeBoardInput("0".repeat(80))).toBeNull();
      expect(normalizeBoardInput("0".repeat(82))).toBeNull();
    });

    it("should return null for strings with invalid/non-numeric characters", () => {
      expect(normalizeBoardInput("a".repeat(81))).toBeNull();
      expect(
        normalizeBoardInput(
          "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..7X",
        ),
      ).toBeNull();
    });

    it("should return null for empty string or strings with only whitespace", () => {
      expect(normalizeBoardInput("")).toBeNull();
      expect(normalizeBoardInput("   ")).toBeNull();
      expect(normalizeBoardInput(" ".repeat(81))).toBeNull();
    });
  });

  describe("isPlacementValid", () => {
    // A partially completed valid board
    const board =
      "530070000600195000098000060800060003400803001700020006060000280000419005000080079";

    it("should return true for a valid placement", () => {
      // Cell at index 2 (row 0, col 2) is '0'. Placing '4' is valid
      expect(isPlacementValid(board, 2, "4")).toBe(true);
    });

    it("should return false for row conflict", () => {
      // Row 0 already contains '5' (index 0) and '3' (index 1) and '7' (index 4)
      expect(isPlacementValid(board, 2, "5")).toBe(false);
      expect(isPlacementValid(board, 2, "3")).toBe(false);
    });

    it("should return false for col conflict", () => {
      // Col 2 already contains '8' at index 65 (row 7, col 2)
      expect(isPlacementValid(board, 2, "8")).toBe(false);
    });

    it("should return false for box conflict", () => {
      // Box 0 (top-left) already contains '6' at index 9 (row 1, col 0)
      expect(isPlacementValid(board, 2, "6")).toBe(false);
    });

    it("should ignore checking the cell against itself", () => {
      // Cell index 0 has '5'. Placing '5' there should be considered valid (it matches itself)
      expect(isPlacementValid(board, 0, "5")).toBe(true);
    });

    it("should return false for out-of-bounds indices", () => {
      expect(isPlacementValid(board, -1, "5")).toBe(false);
      expect(isPlacementValid(board, 81, "5")).toBe(false);
    });
  });

  describe("computeInvalidCells", () => {
    it("should return empty set for a clean board", () => {
      const empty = "0".repeat(81);
      expect(computeInvalidCells(empty).size).toBe(0);

      const validPart =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";
      expect(computeInvalidCells(validPart).size).toBe(0);
    });

    it("should identify invalid cells when there are conflicts", () => {
      // Create a conflict: put two '5's in the first row (indices 0 and 2)
      const conflictBoard =
        "535070000600195000098000060800060003400803001700020006060000280000419005000080079";
      const invalid = computeInvalidCells(conflictBoard);
      expect(invalid.has(0)).toBe(true);
      expect(invalid.has(2)).toBe(true);
      // Wait, let's see if index 15 (which is '5' in row 1, col 6) conflicts with them.
      // Row 1 doesn't conflict with row 0.
      // Box 0 contains index 0, 1, 2, 9, 10, 11, 18, 19, 20.
      // Since index 0 and 2 are in row 0 and box 0, they both violate row and box rules.
    });
  });

  describe("boardForExport", () => {
    const board =
      "530070000600195000098000060800060003400803001700020006060000280000419005000080079";

    it("should keep zeros if format is zero", () => {
      expect(boardForExport(board, "zero")).toBe(board);
    });

    it("should replace zeros with dots if format is dot", () => {
      const expected =
        "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
      expect(boardForExport(board, "dot")).toBe(expected);
    });
  });

  describe("syncBoardInput", () => {
    it("should write board to inputBoard value based on export format", () => {
      const board =
        "530070000600195000098000060800060003400803001700020006060000280000419005000080079";

      // Test format 'zero'
      if (selectExportFormat) selectExportFormat.value = "zero";
      syncBoardInput(board);
      expect(inputBoard?.value).toBe(board);

      // Test format 'dot'
      if (selectExportFormat) selectExportFormat.value = "dot";
      syncBoardInput(board);
      expect(inputBoard?.value).toBe(
        "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79",
      );
    });
  });

  describe("hydrateBoardState", () => {
    it("should keep only well-formed undo and redo snapshots", () => {
      const validSnapshot = {
        board:
          "530070000600195000098000060800060003400803001700020006060000280000419005000080079",
        givens: Array(81).fill(true),
      };

      localStorage.setItem(
        STORAGE_KEYS.undoStack,
        JSON.stringify([
          validSnapshot,
          { board: "123", givens: Array(81).fill(true) },
          { board: validSnapshot.board, givens: Array(80).fill(false) },
          { board: validSnapshot.board, givens: Array(81).fill("1") },
          null,
        ]),
      );
      localStorage.setItem(
        STORAGE_KEYS.redoStack,
        JSON.stringify([
          { board: validSnapshot.board, givens: Array(81).fill(false) },
          { foo: "bar" },
        ]),
      );

      hydrateBoardState();

      expect(state.undoStack).toEqual([validSnapshot]);
      expect(state.redoStack).toEqual([
        { board: validSnapshot.board, givens: Array(81).fill(false) },
      ]);
    });

    it("should fall back to empty history when stored snapshots are invalid JSON", () => {
      localStorage.setItem(STORAGE_KEYS.undoStack, "not-json");
      localStorage.setItem(STORAGE_KEYS.redoStack, "also-not-json");

      hydrateBoardState();

      expect(state.undoStack).toEqual([]);
      expect(state.redoStack).toEqual([]);
    });
  });
});
