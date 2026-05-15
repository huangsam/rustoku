import {
  solve,
  solve_steps,
  candidates,
  generate_advanced,
  check,
} from "./pkg/rustoku_wasm.js";

let isWasmLoaded: boolean = false;
let currentBoard: string = "0".repeat(81);
let givenMask: boolean[] = Array(81).fill(false);
let selectedCell: number | null = null;
let showPencilMarks: boolean = false;
let invalidCells: Set<number> = new Set();
let undoStack: Array<{ board: string; givens: boolean[] }> = [];
let redoStack: Array<{ board: string; givens: boolean[] }> = [];
let currentHighlightMode: HighlightMode = "none";

const STORAGE_KEYS = {
  board: "rustoku-board",
  givens: "rustoku-givens",
};

const grid = document.getElementById("sudoku-grid") as HTMLDivElement;
const selectGenDifficulty = document.getElementById(
  "select-gen-difficulty",
) as HTMLSelectElement;
const btnGenerate = document.getElementById(
  "btn-generate",
) as HTMLButtonElement;
const btnSolve = document.getElementById("btn-solve") as HTMLButtonElement;
const btnSolveSteps = document.getElementById(
  "btn-solve-steps",
) as HTMLButtonElement;
const btnCandidates = document.getElementById(
  "btn-candidates",
) as HTMLButtonElement;
const btnCheck = document.getElementById("btn-check") as HTMLButtonElement;
const btnErase = document.getElementById("btn-erase") as HTMLButtonElement;
const btnReset = document.getElementById("btn-reset") as HTMLButtonElement;
const inputBoard = document.getElementById("input-board") as HTMLInputElement;
const btnLoadBoard = document.getElementById(
  "btn-load-board",
) as HTMLButtonElement;
const btnCopyBoard = document.getElementById(
  "btn-copy-board",
) as HTMLButtonElement;
const selectExportFormat = document.getElementById(
  "select-export-format",
) as HTMLSelectElement;
const infoPanel = document.getElementById("info-panel") as HTMLDivElement;
const infoTitle = document.getElementById("info-title") as HTMLHeadingElement;
const solveTracePanel = document.getElementById(
  "solve-trace-panel",
) as HTMLDivElement;
const solveTraceStepCount = document.getElementById(
  "solve-trace-step-count",
) as HTMLSpanElement;
const solveTraceStatus = document.getElementById(
  "solve-trace-status",
) as HTMLSpanElement;
const solveTraceTechnique = document.getElementById(
  "solve-trace-technique",
) as HTMLParagraphElement;
const solveTracePlacement = document.getElementById(
  "solve-trace-placement",
) as HTMLParagraphElement;
const solveTraceDetail = document.getElementById(
  "solve-trace-detail",
) as HTMLParagraphElement;
const solveTraceChanges = document.getElementById(
  "solve-trace-changes",
) as HTMLDivElement;
const btnTracePrev = document.getElementById(
  "btn-trace-prev",
) as HTMLButtonElement;
const btnTracePlay = document.getElementById(
  "btn-trace-play",
) as HTMLButtonElement;
const btnTraceNext = document.getElementById(
  "btn-trace-next",
) as HTMLButtonElement;
const btnTraceNextPlacement = document.getElementById(
  "btn-trace-next-placement",
) as HTMLButtonElement;
const btnTraceNextElimination = document.getElementById(
  "btn-trace-next-elimination",
) as HTMLButtonElement;
const infoContent = document.getElementById("info-content") as HTMLPreElement;
const btnCloseInfo = document.getElementById(
  "btn-close-info",
) as HTMLButtonElement;
const selectTheme = document.getElementById(
  "select-theme",
) as HTMLSelectElement;
const btnInfoHeader = document.getElementById(
  "btn-info-header",
) as HTMLButtonElement;
const projectModal = document.getElementById("project-modal") as HTMLDivElement;
const selectGenSymmetry = document.getElementById(
  "select-gen-symmetry",
) as HTMLSelectElement;

const btnModalClose = document.getElementById(
  "btn-modal-close",
) as HTMLButtonElement;

type HighlightMode = "none" | "clue" | "solved";
type CandidateGrid = number[][][];
type CandidateChange = {
  row: number;
  col: number;
  before: number[];
  after: number[];
  removed: number[];
  added: number[];
};
type SolveTraceStep = {
  type?: "placement" | "elimination";
  technique: string;
  value: number;
  row?: number;
  col?: number;
  cell?: number;
  step_number?: number;
  candidates_eliminated?: number;
  related_cell_count?: number;
  difficulty_point?: number;
  candidate_changes?: CandidateChange[];
};
type SolveTraceState = {
  initialBoard: string;
  initialCandidateGrid: CandidateGrid;
  solvedBoard: string;
  steps: SolveTraceStep[];
  currentStep: number;
  isPlaying: boolean;
  playbackTimer: number | null;
};
type ThemeName =
  | "midnight"
  | "paper"
  | "arcade"
  | "blueprint"
  | "ember"
  | "forest"
  | "mono";
const validThemes: ThemeName[] = [
  "midnight",
  "paper",
  "arcade",
  "blueprint",
  "ember",
  "forest",
  "mono",
];
let solveTrace: SolveTraceState | null = null;

// Undo/redo helpers
function pushUndo(): void {
  undoStack.push({ board: currentBoard, givens: [...givenMask] });
  if (undoStack.length > 50) undoStack.shift();
  redoStack = [];
}

function undo(): void {
  if (undoStack.length === 0) return;
  clearSolveTrace();
  const snapshot = undoStack.pop()!;
  redoStack.push({ board: currentBoard, givens: [...givenMask] });
  currentBoard = snapshot.board;
  givenMask = snapshot.givens;
  currentHighlightMode = "none";
  recomputeInvalidCells();
  renderCurrentView();
  syncBoardInput(currentBoard);
  saveBoardState();
}

function redo(): void {
  if (redoStack.length === 0) return;
  clearSolveTrace();
  const snapshot = redoStack.pop()!;
  undoStack.push({ board: currentBoard, givens: [...givenMask] });
  currentBoard = snapshot.board;
  givenMask = snapshot.givens;
  currentHighlightMode = "none";
  recomputeInvalidCells();
  renderCurrentView();
  syncBoardInput(currentBoard);
  saveBoardState();
}

function applyTheme(theme: ThemeName): void {
  document.documentElement.setAttribute("data-theme", theme);
  localStorage.setItem("rustoku-theme", theme);
}

function normalizeBoardInput(raw: string): string | null {
  const normalized = raw
    .trim()
    .replace(/\s+/g, "")
    .replace(/_/g, "0")
    .replace(/\./g, "0");

  if (normalized.length !== 81) return null;
  if (!/^[0-9]{81}$/.test(normalized)) return null;
  return normalized;
}

function computeInvalidCells(boardStr: string): Set<number> {
  const invalid = new Set<number>();
  for (let i = 0; i < 81; i++) {
    const value = boardStr[i];
    if (value !== "0" && !isPlacementValid(boardStr, i, value)) {
      invalid.add(i);
    }
  }
  return invalid;
}

function getTraceCellIndex(step: SolveTraceStep): number | null {
  if (typeof step.cell === "number") {
    return step.cell >= 0 && step.cell < 81 ? step.cell : null;
  }

  if (typeof step.row === "number" && typeof step.col === "number") {
    const index = step.row * 9 + step.col;
    return index >= 0 && index < 81 ? index : null;
  }

  return null;
}

function formatCellLabel(index: number | null): string {
  if (index === null) return "R?C?";
  return `R${Math.floor(index / 9) + 1}C${(index % 9) + 1}`;
}

function titleCaseTechnique(technique: string): string {
  return technique
    .replace(/[_-]+/g, " ")
    .replace(/\s+/g, " ")
    .trim()
    .replace(/\b\w/g, (letter) => letter.toUpperCase());
}

function isPlacementStep(step: SolveTraceStep): boolean {
  return step.type !== "elimination";
}

function createEmptyCandidateGrid(): CandidateGrid {
  return Array.from({ length: 9 }, () => Array.from({ length: 9 }, () => []));
}

function normalizeCandidateGrid(raw: unknown): CandidateGrid {
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

function cloneCandidateGrid(grid: CandidateGrid): CandidateGrid {
  return grid.map((row) => row.map((cell) => [...cell]));
}

function getCurrentTraceStep(): SolveTraceStep | null {
  if (!solveTrace) {
    return null;
  }

  return solveTrace.steps[solveTrace.currentStep] ?? null;
}

function formatDigitList(values: number[]): string {
  return values.join(", ");
}

function findNextTraceStepIndex(
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

function getTraceAffectedIndices(step: SolveTraceStep | null): Set<number> {
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

function renderTraceChanges(step: SolveTraceStep | null): void {
  if (!step) {
    solveTraceChanges.innerHTML = "";
    return;
  }

  const candidateChanges = step.candidate_changes ?? [];
  if (candidateChanges.length === 0) {
    solveTraceChanges.innerHTML =
      '<div class="solve-trace-change"><span>No candidate changes recorded for this step.</span></div>';
    return;
  }

  const items = candidateChanges
    .slice(0, 5)
    .map((change) => {
      const cellLabel = formatCellLabel(change.row * 9 + change.col);
      const removedText =
        change.removed.length > 0
          ? `removed ${formatDigitList(change.removed)}`
          : null;
      const remainingText =
        change.after.length > 0 ? `now ${formatDigitList(change.after)}` : null;
      const addedText =
        change.added.length > 0
          ? `added ${formatDigitList(change.added)}`
          : null;
      const parts = [removedText, addedText, remainingText].filter(
        (part): part is string => Boolean(part),
      );

      return `<div class="solve-trace-change"><strong>${cellLabel}</strong><span>${parts.join(" • ")}</span></div>`;
    })
    .join("");

  const overflow =
    candidateChanges.length > 5
      ? `<div class="solve-trace-change"><span>+${candidateChanges.length - 5} more affected cell${candidateChanges.length - 5 === 1 ? "" : "s"}</span></div>`
      : "";

  solveTraceChanges.innerHTML = items + overflow;
}

function buildTraceBoard(
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

function buildTraceCandidateGrid(
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

function getDisplayedBoard(): string {
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

function syncCandidatesButton(): void {
  if (!btnCandidates) {
    return;
  }

  if (solveTrace) {
    btnCandidates.textContent = "Trace Candidates";
    btnCandidates.disabled = true;
    return;
  }

  btnCandidates.disabled = false;
  btnCandidates.textContent = showPencilMarks
    ? "Hide Candidates"
    : "Candidates";
}

function stopSolveTracePlayback(): void {
  const trace = solveTrace;
  if (!trace) return;

  if (trace.playbackTimer !== null) {
    window.clearTimeout(trace.playbackTimer);
    trace.playbackTimer = null;
  }

  trace.isPlaying = false;
}

function renderCurrentView(): void {
  if (solveTrace) {
    const replayBoard = getDisplayedBoard();
    const mode: HighlightMode =
      solveTrace.currentStep >= solveTrace.steps.length - 1 ? "solved" : "none";
    renderGrid(replayBoard, mode);
    syncBoardInput(replayBoard);
    return;
  }

  renderGrid(currentBoard, currentHighlightMode);
  syncBoardInput(currentBoard);
}

function clearSolveTrace(options?: { restoreBoard?: boolean }): void {
  if (!solveTrace) return;

  stopSolveTracePlayback();
  solveTrace = null;
  solveTracePanel.hidden = true;
  infoContent.hidden = false;
  syncCandidatesButton();

  if (options?.restoreBoard) {
    renderCurrentView();
  }
}

function renderSolveTracePanel(): void {
  if (!solveTrace) return;

  const totalSteps = solveTrace.steps.length;
  const currentStep = getCurrentTraceStep();
  if (!currentStep) return;
  const currentCell = currentStep ? getTraceCellIndex(currentStep) : null;
  const stepNumber = solveTrace.currentStep + 1;
  const isComplete = solveTrace.currentStep >= totalSteps - 1;
  const eliminated = currentStep.candidates_eliminated ?? 0;
  const relatedCells = currentStep.related_cell_count ?? 0;
  const isPlacement = isPlacementStep(currentStep);
  const nextPlacementIndex = findNextTraceStepIndex(
    solveTrace.currentStep,
    isPlacementStep,
  );
  const nextEliminationIndex = findNextTraceStepIndex(
    solveTrace.currentStep,
    (step) => !isPlacementStep(step),
  );

  infoTitle.textContent = "Solve Steps";
  infoPanel.style.display = "block";
  solveTracePanel.hidden = false;
  infoContent.hidden = true;

  solveTraceStepCount.textContent = `Step ${stepNumber} of ${totalSteps}`;
  solveTraceStatus.textContent = isComplete
    ? "Solved board"
    : solveTrace.isPlaying
      ? "Auto-playing"
      : "Manual review";
  solveTraceTechnique.textContent = titleCaseTechnique(currentStep.technique);
  solveTracePlacement.textContent = isPlacement
    ? `${formatCellLabel(currentCell)} = ${currentStep.value}`
    : `${formatCellLabel(currentCell)} eliminate ${currentStep.value}`;
  solveTraceDetail.textContent = isComplete
    ? "Final step reached. The board now matches the solved state."
    : isPlacement
      ? "Placement step. Use Prev and Next to inspect each digit, or Play to animate the remaining trace."
      : `Elimination step. Removed ${eliminated} candidate${eliminated === 1 ? "" : "s"} across ${relatedCells} related cell${relatedCells === 1 ? "" : "s"}. The board digits do not change on this step.`;
  renderTraceChanges(currentStep);

  btnTracePrev.disabled = solveTrace.currentStep <= 0;
  btnTraceNext.disabled = solveTrace.currentStep >= totalSteps - 1;
  btnTracePlay.textContent = solveTrace.isPlaying ? "Pause" : "Play";
  btnTraceNextPlacement.disabled = nextPlacementIndex === null;
  btnTraceNextElimination.disabled = nextEliminationIndex === null;
}

function scheduleSolveTracePlayback(): void {
  if (!solveTrace || !solveTrace.isPlaying) return;

  if (solveTrace.currentStep >= solveTrace.steps.length - 1) {
    stopSolveTracePlayback();
    renderSolveTracePanel();
    return;
  }

  solveTrace.playbackTimer = window.setTimeout(() => {
    if (!solveTrace || !solveTrace.isPlaying) return;
    solveTrace.currentStep += 1;
    renderSolveTracePanel();
    renderCurrentView();
    scheduleSolveTracePlayback();
  }, 700);
}

function toggleSolveTracePlayback(): void {
  if (!solveTrace) return;

  if (solveTrace.isPlaying) {
    stopSolveTracePlayback();
    renderSolveTracePanel();
    return;
  }

  if (solveTrace.currentStep >= solveTrace.steps.length - 1) {
    solveTrace.currentStep = 0;
  }

  solveTrace.isPlaying = true;
  renderSolveTracePanel();
  renderCurrentView();
  scheduleSolveTracePlayback();
}

function showSolveTrace(
  initialBoard: string,
  initialCandidateGrid: CandidateGrid,
  solvedBoard: string,
  steps: SolveTraceStep[],
): void {
  solveTrace = {
    initialBoard,
    initialCandidateGrid,
    solvedBoard,
    steps,
    currentStep: 0,
    isPlaying: false,
    playbackTimer: null,
  };

  syncCandidatesButton();
  renderSolveTracePanel();
  renderCurrentView();
  infoPanel.scrollIntoView({ behavior: "smooth" });
}

function boardForExport(boardStr: string, format: "zero" | "dot"): string {
  return format === "dot" ? boardStr.replace(/0/g, ".") : boardStr;
}

function syncBoardInput(boardStr: string): void {
  if (!inputBoard) return;
  const format = (selectExportFormat?.value === "dot" ? "dot" : "zero") as
    | "zero"
    | "dot";
  inputBoard.value = boardForExport(boardStr, format);
}

function saveBoardState(): void {
  try {
    localStorage.setItem(STORAGE_KEYS.board, currentBoard);
    localStorage.setItem(
      STORAGE_KEYS.givens,
      givenMask.map((isGiven) => (isGiven ? "1" : "0")).join(""),
    );
  } catch (_err) {
    // Ignore storage write failures in restricted contexts.
  }
}

function hydrateBoardState(): void {
  try {
    const savedBoard = localStorage.getItem(STORAGE_KEYS.board);
    const savedGivens = localStorage.getItem(STORAGE_KEYS.givens);

    if (savedBoard && /^[0-9]{81}$/.test(savedBoard)) {
      currentBoard = savedBoard;
    }

    if (savedGivens && /^[01]{81}$/.test(savedGivens)) {
      givenMask = savedGivens.split("").map((ch) => ch === "1");
    } else {
      givenMask = currentBoard.split("").map((ch) => ch !== "0");
    }
  } catch (_err) {
    // Ignore storage read failures and keep defaults.
  }
}

function isPlacementValid(
  boardStr: string,
  index: number,
  value: string,
): boolean {
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

function recomputeInvalidCells(): void {
  invalidCells = computeInvalidCells(currentBoard);
}

function setBoard(
  boardStr: string,
  options?: {
    setAsGiven?: boolean;
    highlightMode?: HighlightMode;
    clearSelection?: boolean;
  },
): void {
  currentBoard = boardStr;
  if (options?.setAsGiven) {
    givenMask = currentBoard.split("").map((ch) => ch !== "0");
  }
  if (options?.clearSelection) {
    selectedCell = null;
  }
  currentHighlightMode = options?.highlightMode ?? "none";
  recomputeInvalidCells();
  renderCurrentView();
  syncBoardInput(currentBoard);
  saveBoardState();
}

function updateCell(index: number, value: string): void {
  if (index < 0 || index >= 81) return;
  if (givenMask[index]) return;
  if (!/^[0-9]$/.test(value)) return;

  clearSolveTrace();

  pushUndo();

  const chars = currentBoard.split("");
  chars[index] = value;
  setBoard(chars.join(""), { highlightMode: "none" });
}

// Initialize the grid UI
function renderGrid(
  boardStr: string,
  highlightMode: HighlightMode = "none",
): void {
  if (!grid) return;

  const cells = grid.querySelectorAll(".cell");
  const needsCreate = cells.length === 0;
  let candidateGrid: number[][][] | null = null;
  const invalidSet =
    boardStr === currentBoard ? invalidCells : computeInvalidCells(boardStr);
  const traceFocusIndex =
    solveTrace && solveTrace.currentStep >= 0
      ? getTraceCellIndex(solveTrace.steps[solveTrace.currentStep])
      : null;
  const traceAffectedIndices = getTraceAffectedIndices(getCurrentTraceStep());
  const traceCandidateGrid =
    solveTrace && solveTrace.currentStep >= 0
      ? buildTraceCandidateGrid(
          solveTrace.initialCandidateGrid,
          solveTrace.steps,
          solveTrace.currentStep,
        )
      : null;

  if (traceCandidateGrid) {
    candidateGrid = traceCandidateGrid;
  } else if (showPencilMarks && isWasmLoaded) {
    const rawCandidates = candidates(boardStr);
    if (Array.isArray(rawCandidates)) {
      candidateGrid = normalizeCandidateGrid(rawCandidates);
    }
  }

  for (let i = 0; i < 81; i++) {
    let cell: HTMLDivElement;
    if (needsCreate) {
      cell = document.createElement("div");
      cell.className = "cell";
      grid.appendChild(cell);
    } else {
      cell = cells[i] as HTMLDivElement;
    }

    cell.dataset.index = String(i);
    if (cell.dataset.bound !== "1") {
      cell.tabIndex = 0;
      cell.addEventListener("click", () => {
        selectedCell = i;
        renderCurrentView();
      });
      cell.dataset.bound = "1";
    }

    const val = boardStr[i];
    cell.classList.remove(
      "clue",
      "solved",
      "selected",
      "invalid",
      "related",
      "same-digit",
      "trace-affected",
      "trace-focus",
    );

    if (selectedCell !== null) {
      const selectedRow = Math.floor(selectedCell / 9);
      const selectedCol = selectedCell % 9;
      const selectedBox =
        Math.floor(selectedRow / 3) * 3 + Math.floor(selectedCol / 3);

      const cellRow = Math.floor(i / 9);
      const cellCol = i % 9;
      const cellBox = Math.floor(cellRow / 3) * 3 + Math.floor(cellCol / 3);

      if (
        i !== selectedCell &&
        (cellRow === selectedRow ||
          cellCol === selectedCol ||
          cellBox === selectedBox)
      ) {
        cell.classList.add("related");
      }

      const selectedVal = boardStr[selectedCell];
      if (selectedVal !== "0" && val === selectedVal && i !== selectedCell) {
        cell.classList.add("same-digit");
      }
    }

    if (selectedCell === i) {
      cell.classList.add("selected");
    }
    if (invalidSet.has(i)) {
      cell.classList.add("invalid");
    }

    if (traceAffectedIndices.has(i)) {
      cell.classList.add("trace-affected");
    }

    if (traceFocusIndex === i) {
      cell.classList.add("trace-focus");
    }

    if (val !== "0") {
      cell.innerHTML = `<span>${val}</span>`;

      if (givenMask[i]) {
        cell.classList.add("clue");
      } else if (highlightMode === "solved") {
        cell.classList.add("solved");
      }
    } else {
      if ((showPencilMarks || Boolean(solveTrace)) && candidateGrid) {
        const row = Math.floor(i / 9);
        const col = i % 9;
        const cellCandidates = Array.isArray(candidateGrid[row]?.[col])
          ? candidateGrid[row][col]
          : [];
        const marks = Array.from({ length: 9 }, (_, n) => {
          const digit = n + 1;
          const visible = cellCandidates.includes(digit);
          return `<span class="pmark${visible ? " visible" : ""}">${digit}</span>`;
        }).join("");
        cell.innerHTML = `<div class="pencil-grid">${marks}</div>`;
      } else {
        cell.innerHTML = "";
      }
    }
  }
}

async function run(): Promise<void> {
  try {
    hydrateBoardState();

    const savedTheme = localStorage.getItem("rustoku-theme");
    const initialTheme: ThemeName =
      savedTheme && validThemes.includes(savedTheme as ThemeName)
        ? (savedTheme as ThemeName)
        : "midnight";
    applyTheme(initialTheme);
    if (selectTheme) {
      selectTheme.value = initialTheme;
      selectTheme.onchange = () => applyTheme(selectTheme.value as ThemeName);
    }

    // Render hydrated board immediately to avoid flash while WASM initializes.
    setBoard(currentBoard, { clearSelection: true });
    syncCandidatesButton();

    // WASM is initialized automatically by the bundler (vite-plugin-wasm)
    isWasmLoaded = true;
    renderCurrentView();
    console.log("Rustoku WASM loaded!");
  } catch (err) {
    console.error("Failed to load WASM module", err);
    if (grid) {
      grid.innerHTML = `<p style="color:red; grid-column:span 9; text-align:center; padding-top:50%;">Failed to load WASM</p>`;
    }
  }
}

// Helpers
function showInfo(
  title: string,
  content: string,
  options?: { preserveTrace?: boolean },
): void {
  if (!options?.preserveTrace) {
    clearSolveTrace({ restoreBoard: true });
  }
  infoTitle.textContent = title;
  infoContent.textContent = content;
  infoContent.hidden = false;
  solveTracePanel.hidden = options?.preserveTrace
    ? solveTracePanel.hidden
    : true;
  infoPanel.style.display = "block";
  infoPanel.scrollIntoView({ behavior: "smooth" });
}

function formatBoard(boardStr: string): string {
  let result = "";
  for (let i = 0; i < 9; i++) {
    if (i % 3 === 0 && i !== 0) result += "------+-------+------\n";
    for (let j = 0; j < 9; j++) {
      if (j % 3 === 0 && j !== 0) result += "| ";
      result += boardStr[i * 9 + j] + " ";
    }
    result += "\n";
  }
  return result;
}

/*
// Legacy helper preserved for reference
function generateAndRender(difficulty: string): void {
  if (!isWasmLoaded) return;
  const boardStr = generate(difficulty);
  if (boardStr && boardStr.length === 81) {
    setBoard(boardStr, {
      setAsGiven: true,
      highlightMode: "clue",
      clearSelection: true,
    });
  }
}
*/

// Event Listeners — Generate

if (btnGenerate)
  btnGenerate.onclick = () => {
    if (!isWasmLoaded) return;
    clearSolveTrace();
    const difficulty = selectGenDifficulty.value;
    const symmetry = selectGenSymmetry.value;

    // Use advanced generation if available
    const diffVal = difficulty === "random" ? null : difficulty;
    const boardStr = generate_advanced(symmetry, diffVal as string);

    if (boardStr && boardStr.length === 81) {
      undoStack = [];
      redoStack = [];
      setBoard(boardStr, {
        setAsGiven: true,
        highlightMode: "clue",
        clearSelection: true,
      });
    } else {
      alert("Generation failed! Try reducing difficulty or changing symmetry.");
    }
  };

// Event Listeners — Solve
if (btnSolve)
  btnSolve.onclick = () => {
    if (!isWasmLoaded) return;
    if (currentBoard === "0".repeat(81)) {
      alert("Please generate a board first.");
      return;
    }
    clearSolveTrace();
    const solvedBoard = solve(currentBoard);
    if (solvedBoard && solvedBoard.length === 81) {
      setBoard(solvedBoard, { highlightMode: "solved" });
    } else {
      alert("Could not solve this board!");
    }
  };

if (btnSolveSteps)
  btnSolveSteps.onclick = () => {
    if (!isWasmLoaded) return;
    if (currentBoard === "0".repeat(81)) {
      alert("Please generate a board first.");
      return;
    }
    clearSolveTrace();
    const result = solve_steps(currentBoard, "expert");
    if (result) {
      const steps = Array.isArray(result.steps)
        ? (result.steps as SolveTraceStep[])
        : [];
      const initialCandidateGrid = normalizeCandidateGrid(
        candidates(currentBoard),
      );

      if (steps.length > 0) {
        showSolveTrace(currentBoard, initialCandidateGrid, result.board, steps);
      } else {
        showInfo(
          "Solve Steps",
          `(No human technique steps recorded)\n\n── Final Board ──\n${formatBoard(result.board)}`,
        );
        setBoard(result.board, { highlightMode: "solved" });
      }
    } else {
      alert(`Could not solve with human techniques.`);
    }
  };

// Event Listeners — Tools
if (btnCandidates)
  btnCandidates.onclick = () => {
    if (!isWasmLoaded) return;
    showPencilMarks = !showPencilMarks;
    syncCandidatesButton();
    renderCurrentView();
  };

if (btnCheck)
  btnCheck.onclick = () => {
    if (!isWasmLoaded) return;
    const boardToCheck = getDisplayedBoard();
    const isValid = check(boardToCheck);
    if (isValid) {
      showInfo("Validation ✓", "This is a valid, complete Sudoku solution!", {
        preserveTrace: Boolean(solveTrace),
      });
    } else {
      showInfo(
        "Validation ✗",
        "Not a valid solution. Make sure all 81 cells are filled with no duplicates in any row, column, or box.",
        { preserveTrace: Boolean(solveTrace) },
      );
    }
  };

if (btnLoadBoard)
  btnLoadBoard.onclick = () => {
    if (!isWasmLoaded) return;
    clearSolveTrace();
    const parsed = normalizeBoardInput(inputBoard.value);
    if (!parsed) {
      showInfo(
        "Board Load Error",
        "Invalid board string. Use exactly 81 chars with digits 0-9, where 0, ., or _ mean empty cells.",
      );
      return;
    }

    undoStack = [];
    redoStack = [];
    setBoard(parsed, {
      setAsGiven: true,
      highlightMode: "clue",
      clearSelection: true,
    });
    showInfo("Board Loaded", "Board loaded successfully.");
  };

if (btnCopyBoard)
  btnCopyBoard.onclick = async () => {
    const format = (selectExportFormat.value === "dot" ? "dot" : "zero") as
      | "zero"
      | "dot";
    const output = boardForExport(getDisplayedBoard(), format);

    try {
      await navigator.clipboard.writeText(output);
      showInfo(
        "Board Copied",
        `Copied 81-char board as ${format === "dot" ? "dot" : "zero"} format.`,
      );
    } catch (_err) {
      showInfo(
        "Copy Failed",
        "Clipboard write failed in this browser context. You can still copy directly from the Board String field.",
      );
    }
  };

if (selectExportFormat) {
  selectExportFormat.onchange = () => {
    syncBoardInput(getDisplayedBoard());
  };
}

if (btnErase)
  btnErase.onclick = () => {
    clearSolveTrace();
    showPencilMarks = false;
    syncCandidatesButton();
    const chars = currentBoard.split("");
    for (let i = 0; i < 81; i++) {
      if (!givenMask[i]) {
        chars[i] = "0";
      }
    }
    setBoard(chars.join(""), { clearSelection: true });
    infoPanel.style.display = "none";
  };

if (btnReset)
  btnReset.onclick = () => {
    clearSolveTrace();
    showPencilMarks = false;
    syncCandidatesButton();
    givenMask = Array(81).fill(false);
    undoStack = [];
    redoStack = [];
    setBoard("0".repeat(81), { clearSelection: true });
    infoPanel.style.display = "none";
  };

if (btnCloseInfo)
  btnCloseInfo.onclick = () => {
    stopSolveTracePlayback();
    if (solveTrace) {
      renderSolveTracePanel();
    }
    infoPanel.style.display = "none";
  };

if (btnTracePrev)
  btnTracePrev.onclick = () => {
    if (!solveTrace) return;
    stopSolveTracePlayback();
    solveTrace.currentStep = Math.max(0, solveTrace.currentStep - 1);
    renderSolveTracePanel();
    renderCurrentView();
  };

if (btnTraceNext)
  btnTraceNext.onclick = () => {
    if (!solveTrace) return;
    stopSolveTracePlayback();
    solveTrace.currentStep = Math.min(
      solveTrace.steps.length - 1,
      solveTrace.currentStep + 1,
    );
    renderSolveTracePanel();
    renderCurrentView();
  };

if (btnTracePlay)
  btnTracePlay.onclick = () => {
    toggleSolveTracePlayback();
  };

if (btnTraceNextPlacement)
  btnTraceNextPlacement.onclick = () => {
    if (!solveTrace) return;
    const nextIndex = findNextTraceStepIndex(
      solveTrace.currentStep,
      isPlacementStep,
    );
    if (nextIndex === null) return;
    stopSolveTracePlayback();
    solveTrace.currentStep = nextIndex;
    renderSolveTracePanel();
    renderCurrentView();
  };

if (btnTraceNextElimination)
  btnTraceNextElimination.onclick = () => {
    if (!solveTrace) return;
    const nextIndex = findNextTraceStepIndex(
      solveTrace.currentStep,
      (step) => !isPlacementStep(step),
    );
    if (nextIndex === null) return;
    stopSolveTracePlayback();
    solveTrace.currentStep = nextIndex;
    renderSolveTracePanel();
    renderCurrentView();
  };

document.addEventListener("keydown", (event) => {
  if (!isWasmLoaded) return;

  const target = event.target as HTMLElement | null;
  if (
    target &&
    (target.tagName === "INPUT" ||
      target.tagName === "TEXTAREA" ||
      target.tagName === "SELECT")
  ) {
    return;
  }

  // Undo/Redo shortcuts (work even without selectedCell)
  if (
    (event.ctrlKey || event.metaKey) &&
    event.key === "z" &&
    !event.shiftKey
  ) {
    event.preventDefault();
    undo();
    return;
  }
  if (
    ((event.ctrlKey || event.metaKey) &&
      (event.key === "Z" || (event.shiftKey && event.key === "z"))) ||
    (event.ctrlKey && event.key === "y")
  ) {
    event.preventDefault();
    redo();
    return;
  }

  // Escape to deselect
  if (event.key === "Escape") {
    selectedCell = null;
    renderCurrentView();
    return;
  }

  if (selectedCell === null) return;

  if (/^[1-9]$/.test(event.key)) {
    updateCell(selectedCell, event.key);
    return;
  }

  if (
    event.key === "0" ||
    event.key === "." ||
    event.key === "Backspace" ||
    event.key === "Delete"
  ) {
    updateCell(selectedCell, "0");
    return;
  }

  if (event.key === "ArrowUp") {
    event.preventDefault();
    selectedCell = selectedCell >= 9 ? selectedCell - 9 : selectedCell;
    renderCurrentView();
    return;
  }
  if (event.key === "ArrowDown") {
    event.preventDefault();
    selectedCell = selectedCell <= 71 ? selectedCell + 9 : selectedCell;
    renderCurrentView();
    return;
  }
  if (event.key === "ArrowLeft") {
    event.preventDefault();
    selectedCell = selectedCell % 9 === 0 ? selectedCell : selectedCell - 1;
    renderCurrentView();
    return;
  }
  if (event.key === "ArrowRight") {
    event.preventDefault();
    selectedCell = selectedCell % 9 === 8 ? selectedCell : selectedCell + 1;
    renderCurrentView();
  }
});

// Modal handlers
if (btnInfoHeader) {
  btnInfoHeader.onclick = () => {
    projectModal.style.display = "flex";
  };
}

if (btnModalClose) {
  btnModalClose.onclick = () => {
    projectModal.style.display = "none";
  };
}

// Close modal when clicking outside
if (projectModal) {
  projectModal.onclick = (e) => {
    if (e.target === projectModal) {
      projectModal.style.display = "none";
    }
  };
}

// Boot up
run();
