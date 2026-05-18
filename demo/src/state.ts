import type { HighlightMode } from "./types";
import { inputBoard, selectExportFormat } from "./elements";

export const STORAGE_KEYS = {
  board: "rustoku-board",
  givens: "rustoku-givens",
  difficulty: "rustoku-difficulty",
};

// Global reactive state object
export const state = {
  isWasmLoaded: false,
  currentBoard: "0".repeat(81),
  givenMask: Array(81).fill(false) as boolean[],
  selectedCell: null as number | null,
  lastPlacedCell: null as number | null,
  showPencilMarks: false,
  invalidCells: new Set<number>(),
  undoStack: [] as Array<{ board: string; givens: boolean[] }>,
  redoStack: [] as Array<{ board: string; givens: boolean[] }>,
  currentHighlightMode: "none" as HighlightMode,
  currentDifficulty: "custom",
  isGenerating: false,
  isAnimatingSolve: false,
};

// Pub-sub reactive system
type Listener = () => void;
const listeners = new Set<Listener>();

export function subscribe(listener: Listener): () => void {
  listeners.add(listener);
  return () => {
    listeners.delete(listener);
  };
}

export function notify(): void {
  listeners.forEach((listener) => listener());
}

// Trace callbacks to avoid circular dependencies
const clearTraceCallbacks = new Set<() => void>();

export function onClearSolveTrace(cb: () => void): () => void {
  clearTraceCallbacks.add(cb);
  return () => {
    clearTraceCallbacks.delete(cb);
  };
}

export function triggerClearSolveTrace(): void {
  clearTraceCallbacks.forEach((cb) => cb());
}

// Core logic helpers
export function normalizeBoardInput(raw: string): string | null {
  const normalized = raw
    .trim()
    .replace(/\s+/g, "")
    .replace(/_/g, "0")
    .replace(/\./g, "0");

  if (normalized.length !== 81) return null;
  if (!/^[0-9]{81}$/.test(normalized)) return null;
  return normalized;
}

export function isPlacementValid(
  boardStr: string,
  index: number,
  value: string,
): boolean {
  if (index < 0 || index >= 81) return false;

  const row = Math.floor(index / 9);
  const col = index % 9;

  for (let c = 0; c < 9; c++) {
    const idx = row * 9 + c;
    if (idx !== index && boardStr[idx] === value) return false;
  }

  for (let r = 0; r < 9; r++) {
    const idx = r * 9 + col;
    if (idx !== index && boardStr[idx] === value) return false;
  }

  const boxRowStart = Math.floor(row / 3) * 3;
  const boxColStart = Math.floor(col / 3) * 3;
  for (let r = boxRowStart; r < boxRowStart + 3; r++) {
    for (let c = boxColStart; c < boxColStart + 3; c++) {
      const idx = r * 9 + c;
      if (idx !== index && boardStr[idx] === value) return false;
    }
  }

  return true;
}

export function computeInvalidCells(boardStr: string): Set<number> {
  const invalid = new Set<number>();
  for (let i = 0; i < 81; i++) {
    const value = boardStr[i];
    if (value !== "0" && !isPlacementValid(boardStr, i, value)) {
      invalid.add(i);
    }
  }
  return invalid;
}

export function recomputeInvalidCells(): void {
  state.invalidCells = computeInvalidCells(state.currentBoard);
}

export function boardForExport(
  boardStr: string,
  format: "zero" | "dot",
): string {
  return format === "dot" ? boardStr.replace(/0/g, ".") : boardStr;
}

export function syncBoardInput(boardStr: string): void {
  if (!inputBoard) return;
  const format = (selectExportFormat?.value === "dot" ? "dot" : "zero") as
    | "zero"
    | "dot";
  inputBoard.value = boardForExport(boardStr, format);
}

export function saveBoardState(): void {
  try {
    localStorage.setItem(STORAGE_KEYS.board, state.currentBoard);
    localStorage.setItem(
      STORAGE_KEYS.givens,
      state.givenMask.map((isGiven) => (isGiven ? "1" : "0")).join(""),
    );
    localStorage.setItem(STORAGE_KEYS.difficulty, state.currentDifficulty);
  } catch (_err) {
    // Ignore storage failures
  }
}

export function hydrateBoardState(): void {
  try {
    const savedBoard = localStorage.getItem(STORAGE_KEYS.board);
    const savedGivens = localStorage.getItem(STORAGE_KEYS.givens);
    const savedDifficulty = localStorage.getItem(STORAGE_KEYS.difficulty);

    if (savedBoard && /^[0-9]{81}$/.test(savedBoard)) {
      state.currentBoard = savedBoard;
    }

    if (savedGivens && /^[01]{81}$/.test(savedGivens)) {
      state.givenMask = savedGivens.split("").map((ch) => ch === "1");
    } else {
      state.givenMask = state.currentBoard.split("").map((ch) => ch !== "0");
    }

    if (savedDifficulty) {
      state.currentDifficulty = savedDifficulty;
    } else {
      state.currentDifficulty = "custom";
    }
  } catch (_err) {
    // Keep defaults
  }
}

export function pushUndo(): void {
  state.undoStack.push({
    board: state.currentBoard,
    givens: [...state.givenMask],
  });
  if (state.undoStack.length > 50) state.undoStack.shift();
  state.redoStack = [];
}

export function setBoard(
  boardStr: string,
  options?: {
    setAsGiven?: boolean;
    highlightMode?: HighlightMode;
    clearSelection?: boolean;
  },
): void {
  state.currentBoard = boardStr;
  if (options?.setAsGiven) {
    state.givenMask = state.currentBoard.split("").map((ch) => ch !== "0");
  }
  if (options?.clearSelection) {
    state.selectedCell = null;
  }
  state.currentHighlightMode = options?.highlightMode ?? "none";
  recomputeInvalidCells();
  notify();
  syncBoardInput(state.currentBoard);
  saveBoardState();
}

export function updateCell(index: number, value: string): void {
  if (index < 0 || index >= 81) return;
  if (state.givenMask[index]) return;
  if (!/^[0-9]$/.test(value)) return;

  triggerClearSolveTrace();
  pushUndo();

  const chars = state.currentBoard.split("");
  chars[index] = value;
  state.lastPlacedCell = index;
  setBoard(chars.join(""), { highlightMode: "none" });
}

export function undo(): void {
  if (state.isGenerating || state.isAnimatingSolve) return;
  if (state.undoStack.length === 0) return;

  triggerClearSolveTrace();
  const snapshot = state.undoStack.pop()!;
  state.redoStack.push({
    board: state.currentBoard,
    givens: [...state.givenMask],
  });
  state.currentBoard = snapshot.board;
  state.givenMask = snapshot.givens;
  state.currentHighlightMode = "none";
  recomputeInvalidCells();
  notify();
  syncBoardInput(state.currentBoard);
  saveBoardState();
}

export function redo(): void {
  if (state.isGenerating || state.isAnimatingSolve) return;
  if (state.redoStack.length === 0) return;

  triggerClearSolveTrace();
  const snapshot = state.redoStack.pop()!;
  state.undoStack.push({
    board: state.currentBoard,
    givens: [...state.givenMask],
  });
  state.currentBoard = snapshot.board;
  state.givenMask = snapshot.givens;
  state.currentHighlightMode = "none";
  recomputeInvalidCells();
  notify();
  syncBoardInput(state.currentBoard);
  saveBoardState();
}
