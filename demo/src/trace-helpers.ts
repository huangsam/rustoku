import type { CandidateGrid, SolveTraceStep, SolveTraceState } from "./types";

export function getTraceCellIndex(step: SolveTraceStep): number | null {
  if (typeof step.cell === "number") {
    return step.cell >= 0 && step.cell < 81 ? step.cell : null;
  }

  if (typeof step.row === "number" && typeof step.col === "number") {
    if (step.row >= 0 && step.row < 9 && step.col >= 0 && step.col < 9) {
      return step.row * 9 + step.col;
    }
    return null;
  }

  return null;
}

export function formatCellLabel(index: number | null): string {
  if (index === null) return "R?C?";
  return `R${Math.floor(index / 9) + 1}C${(index % 9) + 1}`;
}

export function titleCaseTechnique(technique: string): string {
  return technique
    .replace(/[_-]+/g, " ")
    .replace(/\s+/g, " ")
    .trim()
    .replace(/\b\w/g, (letter) => letter.toUpperCase());
}

export function isPlacementStep(step: SolveTraceStep): boolean {
  return step.type !== "elimination";
}

export function createEmptyCandidateGrid(): CandidateGrid {
  return Array.from({ length: 9 }, () => Array.from({ length: 9 }, () => []));
}

export function normalizeCandidateGrid(raw: unknown): CandidateGrid {
  if (!Array.isArray(raw) || raw.length !== 9) {
    return createEmptyCandidateGrid();
  }

  return raw.map((row) => {
    if (!Array.isArray(row) || row.length !== 9) {
      return Array.from({ length: 9 }, () => []);
    }

    return row.map((cell) =>
      Array.isArray(cell)
        ? cell.filter((value): value is number => typeof value === "number")
        : [],
    );
  });
}

export function cloneCandidateGrid(grid: CandidateGrid): CandidateGrid {
  return grid.map((row) => row.map((cell) => [...cell]));
}

export function formatDigitList(values: number[]): string {
  return values.join(", ");
}

export function getCurrentTraceStep(
  solveTrace: SolveTraceState | null,
): SolveTraceStep | null {
  if (!solveTrace) {
    return null;
  }
  return solveTrace.steps[solveTrace.currentStep] ?? null;
}

export function findNextTraceStepIndex(
  solveTrace: SolveTraceState | null,
  startIndex: number,
  matcher: (step: SolveTraceStep) => boolean,
): number | null {
  if (!solveTrace) {
    return null;
  }

  for (let index = startIndex + 1; index < solveTrace.steps.length; index++) {
    if (matcher(solveTrace.steps[index])) {
      return index;
    }
  }

  return null;
}

export function getTraceAffectedIndices(
  step: SolveTraceStep | null,
): Set<number> {
  const indices = new Set<number>();
  if (!step) {
    return indices;
  }

  const currentCell = getTraceCellIndex(step);
  if (currentCell !== null) {
    indices.add(currentCell);
  }

  for (const change of step.candidate_changes ?? []) {
    const index = change.row * 9 + change.col;
    if (index >= 0 && index < 81) {
      indices.add(index);
    }
  }

  return indices;
}

export function buildTraceBoard(
  initialBoard: string,
  steps: SolveTraceStep[],
  currentStep: number,
  solvedBoard: string,
): string {
  if (steps.length === 0) return solvedBoard;
  if (currentStep < 0) return initialBoard;
  if (currentStep >= steps.length - 1) return solvedBoard;

  const chars = initialBoard.split("");
  for (let i = 0; i <= currentStep; i++) {
    const step = steps[i];
    if (!isPlacementStep(step)) continue;

    const index = getTraceCellIndex(step);
    const value = String(step.value);

    if (index === null || !/^[1-9]$/.test(value)) continue;
    chars[index] = value;
  }

  return chars.join("");
}

export function buildTraceCandidateGrid(
  initialCandidateGrid: CandidateGrid,
  steps: SolveTraceStep[],
  currentStep: number,
): CandidateGrid {
  const grid = cloneCandidateGrid(initialCandidateGrid);

  if (currentStep < 0) {
    return grid;
  }

  for (let i = 0; i <= currentStep; i++) {
    const candidateChanges = steps[i]?.candidate_changes ?? [];
    for (const change of candidateChanges) {
      if (
        change.row < 0 ||
        change.row >= 9 ||
        change.col < 0 ||
        change.col >= 9
      ) {
        continue;
      }

      grid[change.row][change.col] = [...change.after];
    }
  }

  return grid;
}

export function getDisplayedBoard(
  solveTrace: SolveTraceState | null,
  currentBoard: string,
): string {
  if (!solveTrace) {
    return currentBoard;
  }

  return buildTraceBoard(
    solveTrace.initialBoard,
    solveTrace.steps,
    solveTrace.currentStep,
    solveTrace.solvedBoard,
  );
}
