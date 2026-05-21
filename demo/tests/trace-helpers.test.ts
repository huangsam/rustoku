import { describe, it, expect } from "vitest";
import {
  getTraceCellIndex,
  formatCellLabel,
  titleCaseTechnique,
  isPlacementStep,
  createEmptyCandidateGrid,
  normalizeCandidateGrid,
  cloneCandidateGrid,
  formatDigitList,
  getCurrentTraceStep,
  findNextTraceStepIndex,
  getTraceAffectedIndices,
  buildTraceBoard,
  buildTraceCandidateGrid,
  getDisplayedBoard,
} from "../src/trace-helpers";
import type { SolveTraceStep, SolveTraceState } from "../src/types";

describe("trace-helpers.ts module", () => {
  describe("getTraceCellIndex", () => {
    it("should return index if cell property is a valid number", () => {
      expect(getTraceCellIndex({ cell: 42, technique: "test", value: 1 })).toBe(
        42,
      );
      expect(getTraceCellIndex({ cell: 0, technique: "test", value: 1 })).toBe(
        0,
      );
      expect(getTraceCellIndex({ cell: 80, technique: "test", value: 1 })).toBe(
        80,
      );
    });

    it("should return null if cell is out of range", () => {
      expect(
        getTraceCellIndex({ cell: -1, technique: "test", value: 1 }),
      ).toBeNull();
      expect(
        getTraceCellIndex({ cell: 81, technique: "test", value: 1 }),
      ).toBeNull();
    });

    it("should calculate cell index from row and col if cell is missing", () => {
      expect(
        getTraceCellIndex({ row: 0, col: 0, technique: "test", value: 1 }),
      ).toBe(0);
      expect(
        getTraceCellIndex({ row: 1, col: 2, technique: "test", value: 1 }),
      ).toBe(11);
      expect(
        getTraceCellIndex({ row: 8, col: 8, technique: "test", value: 1 }),
      ).toBe(80);
    });

    it("should return null if row or col is out of range", () => {
      expect(
        getTraceCellIndex({ row: -1, col: 0, technique: "test", value: 1 }),
      ).toBeNull();
      expect(
        getTraceCellIndex({ row: 9, col: 0, technique: "test", value: 1 }),
      ).toBeNull();
      expect(
        getTraceCellIndex({ row: 0, col: -1, technique: "test", value: 1 }),
      ).toBeNull();
      expect(
        getTraceCellIndex({ row: 0, col: 9, technique: "test", value: 1 }),
      ).toBeNull();
    });

    it("should return null if both cell and row/col are missing", () => {
      expect(getTraceCellIndex({ technique: "test", value: 1 })).toBeNull();
    });
  });

  describe("formatCellLabel", () => {
    it("should return R?C? when index is null", () => {
      expect(formatCellLabel(null)).toBe("R?C?");
    });

    it("should format index 0 as R1C1", () => {
      expect(formatCellLabel(0)).toBe("R1C1");
    });

    it("should format index 80 as R9C9", () => {
      expect(formatCellLabel(80)).toBe("R9C9");
    });

    it("should format intermediate cells correctly", () => {
      expect(formatCellLabel(9)).toBe("R2C1");
      expect(formatCellLabel(40)).toBe("R5C5");
    });
  });

  describe("titleCaseTechnique", () => {
    it("should convert snake_case or kebab-case to Title Case with spaces", () => {
      expect(titleCaseTechnique("naked_single")).toBe("Naked Single");
      expect(titleCaseTechnique("hidden-pair")).toBe("Hidden Pair");
      expect(titleCaseTechnique("pointing_triple")).toBe("Pointing Triple");
    });

    it("should handle multiple separators and extra whitespace cleanly", () => {
      expect(titleCaseTechnique("  naked__single-step  ")).toBe(
        "Naked Single Step",
      );
    });

    it("should correctly format and restore hyphens for Wing techniques", () => {
      expect(titleCaseTechnique("x-wing")).toBe("X-Wing");
      expect(titleCaseTechnique("x_wing")).toBe("X-Wing");
      expect(titleCaseTechnique("XY-Wing")).toBe("XY-Wing");
      expect(titleCaseTechnique("xy_wing")).toBe("XY-Wing");
      expect(titleCaseTechnique("xyz-wing")).toBe("XYZ-Wing");
      expect(titleCaseTechnique("w_wing")).toBe("W-Wing");
      expect(titleCaseTechnique("W Wing")).toBe("W-Wing");
    });
  });

  describe("isPlacementStep", () => {
    it("should return true for steps not marked as elimination", () => {
      expect(
        isPlacementStep({ type: "placement", technique: "test", value: 1 }),
      ).toBe(true);
      expect(isPlacementStep({ technique: "test", value: 1 })).toBe(true);
    });

    it("should return false for elimination steps", () => {
      expect(
        isPlacementStep({ type: "elimination", technique: "test", value: 1 }),
      ).toBe(false);
    });
  });

  describe("normalizeCandidateGrid", () => {
    it("should pass through a valid 9x9 candidate grid", () => {
      const grid = createEmptyCandidateGrid();
      grid[0][0] = [1, 2];
      const normalized = normalizeCandidateGrid(grid);
      expect(normalized[0][0]).toEqual([1, 2]);
      expect(normalized.length).toBe(9);
      expect(normalized[0].length).toBe(9);
    });

    it("should return empty grid if input is not an array of size 9", () => {
      expect(normalizeCandidateGrid(null)).toHaveLength(9);
      expect(normalizeCandidateGrid([])).toHaveLength(9);
      expect(normalizeCandidateGrid({ foo: "bar" })).toHaveLength(9);
    });

    it("should filter out non-number elements from candidate lists and handle non-array rows or cells", () => {
      const grid = createEmptyCandidateGrid();
      // Row 0 is an array, but cell [0][0] has mixed types
      // @ts-expect-error: deliberately testing invalid input types
      grid[0][0] = [1, "two", 3, null];
      // Row 1 is completely replaced by a non-array value
      // @ts-expect-error: deliberately testing invalid input types
      grid[1] = "not-an-array-row";
      // Row 2 cell [2][0] is replaced by a non-array value
      // @ts-expect-error: deliberately testing invalid input types
      grid[2][0] = { not: "array" };

      const normalized = normalizeCandidateGrid(grid);
      expect(normalized[0][0]).toEqual([1, 3]);
      expect(normalized[1]).toHaveLength(9);
      expect(normalized[1].every((cell) => cell.length === 0)).toBe(true);
      expect(normalized[2][0]).toEqual([]);
    });
  });

  describe("cloneCandidateGrid", () => {
    it("should create a deep clone of the candidate grid", () => {
      const grid = createEmptyCandidateGrid();
      grid[0][0] = [1, 2];
      const cloned = cloneCandidateGrid(grid);
      expect(cloned).toEqual(grid);
      expect(cloned).not.toBe(grid);
      expect(cloned[0][0]).not.toBe(grid[0][0]);
    });
  });

  describe("formatDigitList", () => {
    it("should format lists of numbers with commas", () => {
      expect(formatDigitList([1])).toBe("1");
      expect(formatDigitList([1, 2, 3])).toBe("1, 2, 3");
    });
  });

  describe("getCurrentTraceStep", () => {
    it("should return null for null trace state", () => {
      expect(getCurrentTraceStep(null)).toBeNull();
    });

    it("should return step at currentStep index", () => {
      const steps: SolveTraceStep[] = [
        { cell: 0, technique: "A", value: 1 },
        { cell: 1, technique: "B", value: 2 },
      ];
      const trace: SolveTraceState = {
        initialBoard: "0".repeat(81),
        initialCandidateGrid: createEmptyCandidateGrid(),
        solvedBoard: "1".repeat(81),
        steps,
        currentStep: 1,
        isPlaying: false,
        playbackTimer: null,
      };
      expect(getCurrentTraceStep(trace)).toEqual(steps[1]);
    });

    it("should return null if currentStep is out of bounds", () => {
      const steps: SolveTraceStep[] = [{ cell: 0, technique: "A", value: 1 }];
      const trace: SolveTraceState = {
        initialBoard: "0".repeat(81),
        initialCandidateGrid: createEmptyCandidateGrid(),
        solvedBoard: "1".repeat(81),
        steps,
        currentStep: 5,
        isPlaying: false,
        playbackTimer: null,
      };
      expect(getCurrentTraceStep(trace)).toBeNull();
    });
  });

  describe("findNextTraceStepIndex", () => {
    it("should return null if solveTrace is null", () => {
      expect(findNextTraceStepIndex(null, 0, () => true)).toBeNull();
    });

    it("should return the index of the next step that matches the filter", () => {
      const steps: SolveTraceStep[] = [
        { type: "placement", technique: "A", value: 1 },
        { type: "elimination", technique: "B", value: 2 },
        { type: "placement", technique: "C", value: 3 },
      ];
      const trace: SolveTraceState = {
        initialBoard: "0".repeat(81),
        initialCandidateGrid: createEmptyCandidateGrid(),
        solvedBoard: "1".repeat(81),
        steps,
        currentStep: 0,
        isPlaying: false,
        playbackTimer: null,
      };

      const nextPlacement = findNextTraceStepIndex(trace, 0, isPlacementStep);
      expect(nextPlacement).toBe(2);

      const nextElimination = findNextTraceStepIndex(
        trace,
        0,
        (step) => !isPlacementStep(step),
      );
      expect(nextElimination).toBe(1);
    });

    it("should return null if no matching step is found after startIndex", () => {
      const steps: SolveTraceStep[] = [
        { type: "placement", technique: "A", value: 1 },
        { type: "placement", technique: "B", value: 2 },
      ];
      const trace: SolveTraceState = {
        initialBoard: "0".repeat(81),
        initialCandidateGrid: createEmptyCandidateGrid(),
        solvedBoard: "1".repeat(81),
        steps,
        currentStep: 0,
        isPlaying: false,
        playbackTimer: null,
      };

      expect(findNextTraceStepIndex(trace, 1, isPlacementStep)).toBeNull();
    });
  });

  describe("getTraceAffectedIndices", () => {
    it("should return empty set if step is null", () => {
      expect(getTraceAffectedIndices(null).size).toBe(0);
    });

    it("should include the step cell index", () => {
      const step: SolveTraceStep = { cell: 10, technique: "A", value: 5 };
      const affected = getTraceAffectedIndices(step);
      expect(affected.has(10)).toBe(true);
      expect(affected.size).toBe(1);
    });

    it("should include any candidate changes indices", () => {
      const step: SolveTraceStep = {
        cell: 10,
        technique: "A",
        value: 5,
        candidate_changes: [
          { row: 0, col: 0, before: [], after: [], removed: [], added: [] },
          { row: 1, col: 2, before: [], after: [], removed: [], added: [] },
        ],
      };
      const affected = getTraceAffectedIndices(step);
      expect(affected.has(10)).toBe(true);
      expect(affected.has(0)).toBe(true);
      expect(affected.has(11)).toBe(true);
      expect(affected.size).toBe(3);
    });
  });

  describe("buildTraceBoard", () => {
    const initial = "0".repeat(81);
    const solved = "1".repeat(81);

    it("should return solvedBoard if steps are empty", () => {
      expect(buildTraceBoard(initial, [], 0, solved)).toBe(solved);
    });

    it("should return initialBoard if currentStep is less than 0", () => {
      const steps = [{ cell: 0, technique: "A", value: 9 }];
      expect(buildTraceBoard(initial, steps, -1, solved)).toBe(initial);
    });

    it("should return solvedBoard if currentStep is at or past last step index", () => {
      const steps = [{ cell: 0, technique: "A", value: 9 }];
      expect(buildTraceBoard(initial, steps, 0, solved)).toBe(solved);
      expect(buildTraceBoard(initial, steps, 1, solved)).toBe(solved);
    });

    it("should play steps incrementally up to currentStep", () => {
      const steps: SolveTraceStep[] = [
        { cell: 0, type: "placement", value: 5, technique: "test" },
        { cell: 1, type: "placement", value: 3, technique: "test" },
        { cell: 2, type: "elimination", value: 4, technique: "test" }, // should be ignored for board digits
        { cell: 3, type: "placement", value: 9, technique: "test" },
      ];

      // Play step 0 (only cell 0 set)
      const res0 = buildTraceBoard(initial, steps, 0, solved);
      expect(res0.startsWith("5000")).toBe(true);

      // Play step 1 (cell 0 and 1 set)
      const res1 = buildTraceBoard(initial, steps, 1, solved);
      expect(res1.startsWith("5300")).toBe(true);

      // Play step 2 (cell 2 is elimination, so board digits don't change from step 1)
      const res2 = buildTraceBoard(initial, steps, 2, solved);
      expect(res2.startsWith("5300")).toBe(true);

      // Play step 3 is the last step (currentStep = 3 is >= steps.length - 1, returns solvedBoard)
      const res3 = buildTraceBoard(initial, steps, 3, solved);
      expect(res3).toBe(solved);
    });
  });

  describe("buildTraceCandidateGrid", () => {
    it("should return clone of initial if currentStep is < 0", () => {
      const initial = createEmptyCandidateGrid();
      initial[0][0] = [1, 2];
      const result = buildTraceCandidateGrid(initial, [], -1);
      expect(result).toEqual(initial);
      expect(result).not.toBe(initial);
    });

    it("should apply candidate_changes incrementally up to currentStep", () => {
      const initial = createEmptyCandidateGrid();
      const steps: SolveTraceStep[] = [
        {
          technique: "A",
          value: 1,
          candidate_changes: [
            { row: 0, col: 0, before: [], after: [9], removed: [], added: [] },
          ],
        },
        {
          technique: "B",
          value: 2,
          candidate_changes: [
            {
              row: 0,
              col: 1,
              before: [],
              after: [4, 5],
              removed: [],
              added: [],
            },
          ],
        },
      ];

      const res0 = buildTraceCandidateGrid(initial, steps, 0);
      expect(res0[0][0]).toEqual([9]);
      expect(res0[0][1]).toEqual([]);

      const res1 = buildTraceCandidateGrid(initial, steps, 1);
      expect(res1[0][0]).toEqual([9]);
      expect(res1[0][1]).toEqual([4, 5]);
    });

    it("should ignore out-of-bounds candidate changes gracefully", () => {
      const initial = createEmptyCandidateGrid();
      const steps: SolveTraceStep[] = [
        {
          technique: "A",
          value: 1,
          candidate_changes: [
            { row: -1, col: 0, before: [], after: [9], removed: [], added: [] },
            { row: 9, col: 0, before: [], after: [9], removed: [], added: [] },
            { row: 0, col: -1, before: [], after: [9], removed: [], added: [] },
            { row: 0, col: 9, before: [], after: [9], removed: [], added: [] },
          ],
        },
      ];

      const res = buildTraceCandidateGrid(initial, steps, 0);
      // All cells should remain empty
      expect(res.every((row) => row.every((cell) => cell.length === 0))).toBe(
        true,
      );
    });
  });

  describe("getDisplayedBoard", () => {
    it("should return currentBoard if solveTrace is null", () => {
      expect(getDisplayedBoard(null, "someBoard")).toBe("someBoard");
    });

    it("should call buildTraceBoard if solveTrace is provided", () => {
      const steps = [
        { cell: 0, type: "placement" as const, value: 5, technique: "test" },
      ];
      const trace: SolveTraceState = {
        initialBoard: "0".repeat(81),
        initialCandidateGrid: createEmptyCandidateGrid(),
        solvedBoard: "1".repeat(81),
        steps,
        currentStep: -1,
        isPlaying: false,
        playbackTimer: null,
      };
      // For currentStep = -1, should return initialBoard
      expect(getDisplayedBoard(trace, "currentBoard")).toBe("0".repeat(81));
    });
  });
});
