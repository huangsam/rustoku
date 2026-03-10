import init, {
  solve,
  solve_steps,
  candidates,
  generate,
  check,
} from "./pkg/rustoku_wasm.js";

let isWasmLoaded: boolean = false;
let currentBoard: string = "0".repeat(81);
let givenMask: boolean[] = Array(81).fill(false);
let selectedCell: number | null = null;
let showPencilMarks: boolean = false;
let invalidCells: Set<number> = new Set();

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
const btnClear = document.getElementById("btn-clear") as HTMLButtonElement;
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
const btnModalClose = document.getElementById(
  "btn-modal-close",
) as HTMLButtonElement;

type HighlightMode = "none" | "clue" | "solved";
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
  const invalid = new Set<number>();
  for (let i = 0; i < 81; i++) {
    const value = currentBoard[i];
    if (value !== "0" && !isPlacementValid(currentBoard, i, value)) {
      invalid.add(i);
    }
  }
  invalidCells = invalid;
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
  recomputeInvalidCells();
  renderGrid(currentBoard, options?.highlightMode ?? "none");
  syncBoardInput(currentBoard);
  saveBoardState();
}

function updateCell(index: number, value: string): void {
  if (index < 0 || index >= 81) return;
  if (givenMask[index]) return;
  if (!/^[0-9]$/.test(value)) return;

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

  if (showPencilMarks && isWasmLoaded) {
    const rawCandidates = candidates(currentBoard);
    if (Array.isArray(rawCandidates)) {
      candidateGrid = rawCandidates as number[][][];
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
        renderGrid(currentBoard, "none");
      });
      cell.dataset.bound = "1";
    }

    const val = boardStr[i];
    cell.classList.remove("clue", "solved", "selected", "invalid");

    if (selectedCell === i) {
      cell.classList.add("selected");
    }
    if (invalidCells.has(i)) {
      cell.classList.add("invalid");
    }

    if (val !== "0") {
      cell.innerHTML = `<span>${val}</span>`;

      if (givenMask[i]) {
        cell.classList.add("clue");
      } else if (highlightMode === "solved") {
        cell.classList.add("solved");
      }
    } else {
      if (showPencilMarks && candidateGrid) {
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

    await init();
    isWasmLoaded = true;
    renderGrid(currentBoard);
    console.log("Rustoku WASM loaded!");
  } catch (err) {
    console.error("Failed to load WASM module", err);
    if (grid) {
      grid.innerHTML = `<p style="color:red; grid-column:span 9; text-align:center; padding-top:50%;">Failed to load WASM</p>`;
    }
  }
}

// Helpers
function showInfo(title: string, content: string): void {
  infoTitle.textContent = title;
  infoContent.textContent = content;
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

// Event Listeners — Generate
if (btnGenerate)
  btnGenerate.onclick = () => {
    const difficulty = selectGenDifficulty.value;
    generateAndRender(difficulty);
  };

// Event Listeners — Solve
if (btnSolve)
  btnSolve.onclick = () => {
    if (!isWasmLoaded) return;
    if (currentBoard === "0".repeat(81)) {
      alert("Please generate a board first.");
      return;
    }
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
    // Use all techniques unconditionally
    const result = solve_steps(currentBoard, "expert");
    if (result) {
      let content = `Solved using all available techniques:\n\n`;
      if (result.steps && result.steps.length > 0) {
        result.steps.forEach(
          (
            step: {
              technique: string;
              value: number;
              row?: number;
              col?: number;
              cell?: number;
            },
            idx: number,
          ) => {
            let row = 0;
            let col = 0;

            if (typeof step.row === "number" && typeof step.col === "number") {
              row = step.row + 1;
              col = step.col + 1;
            } else if (typeof step.cell === "number") {
              row = Math.floor(step.cell / 9) + 1;
              col = (step.cell % 9) + 1;
            }

            const cellLabel = row > 0 && col > 0 ? `R${row}C${col}` : "R?C?";
            content += `${idx + 1}. [${step.technique}] ${cellLabel} = ${step.value}\n`;
          },
        );
      } else {
        content += "(No human technique steps recorded)\n";
      }
      content += `\n── Final Board ──\n${formatBoard(result.board)}`;
      showInfo(`Solve Steps`, content);
      setBoard(result.board, { highlightMode: "solved" });
    } else {
      alert(`Could not solve with human techniques.`);
    }
  };

// Event Listeners — Tools
if (btnCandidates)
  btnCandidates.onclick = () => {
    if (!isWasmLoaded) return;
    showPencilMarks = !showPencilMarks;
    btnCandidates.textContent = showPencilMarks
      ? "Hide Candidates"
      : "Candidates";
    renderGrid(currentBoard, "none");
  };

if (btnCheck)
  btnCheck.onclick = () => {
    if (!isWasmLoaded) return;
    const isValid = check(currentBoard);
    if (isValid) {
      showInfo("Validation ✓", "This is a valid, complete Sudoku solution!");
    } else {
      showInfo(
        "Validation ✗",
        "Not a valid solution. Make sure all 81 cells are filled with no duplicates in any row, column, or box.",
      );
    }
  };

if (btnLoadBoard)
  btnLoadBoard.onclick = () => {
    if (!isWasmLoaded) return;
    const parsed = normalizeBoardInput(inputBoard.value);
    if (!parsed) {
      showInfo(
        "Board Load Error",
        "Invalid board string. Use exactly 81 chars with digits 0-9, where 0, ., or _ mean empty cells.",
      );
      return;
    }

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
    const output = boardForExport(currentBoard, format);

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
    syncBoardInput(currentBoard);
  };
}

if (btnClear)
  btnClear.onclick = () => {
    showPencilMarks = false;
    btnCandidates.textContent = "Candidates";
    givenMask = Array(81).fill(false);
    setBoard("0".repeat(81), { clearSelection: true });
    infoPanel.style.display = "none";
  };

if (btnCloseInfo)
  btnCloseInfo.onclick = () => {
    infoPanel.style.display = "none";
  };

document.addEventListener("keydown", (event) => {
  if (selectedCell === null) return;
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
    renderGrid(currentBoard, "none");
    return;
  }
  if (event.key === "ArrowDown") {
    event.preventDefault();
    selectedCell = selectedCell <= 71 ? selectedCell + 9 : selectedCell;
    renderGrid(currentBoard, "none");
    return;
  }
  if (event.key === "ArrowLeft") {
    event.preventDefault();
    selectedCell = selectedCell % 9 === 0 ? selectedCell : selectedCell - 1;
    renderGrid(currentBoard, "none");
    return;
  }
  if (event.key === "ArrowRight") {
    event.preventDefault();
    selectedCell = selectedCell % 9 === 8 ? selectedCell : selectedCell + 1;
    renderGrid(currentBoard, "none");
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
