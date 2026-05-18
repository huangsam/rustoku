import { check } from "./pkg/rustoku_wasm.js";
import {
  state,
  subscribe,
  hydrateBoardState,
  setBoard,
  undo,
  redo,
  updateCell,
  normalizeBoardInput,
  boardForExport,
  syncBoardInput,
} from "./src/state";
import { validThemes, applyTheme } from "./src/theme";
import {
  btnCandidates,
  btnCheck,
  btnErase,
  btnReset,
  btnLoadBoard,
  btnCopyBoard,
  selectExportFormat,
  selectTheme,
  btnInfoHeader,
  btnModalClose,
  projectModal,
  inputBoard,
  infoPanel,
  grid,
} from "./src/elements";
import { clearSolveTrace, getSolveTrace } from "./src/trace";
import { getDisplayedBoard } from "./src/trace-helpers";
import { renderCurrentView, syncCandidatesButton } from "./src/render";
import { showToast } from "./src/toast";
import { triggerConfetti } from "./src/confetti";
import type { ThemeName } from "./src/types";

// Force bundler to include solving & generation modules
import "./src/solver";
import "./src/generator";

// State synchronization loop
subscribe(() => {
  renderCurrentView();
  syncCandidatesButton();
});

// Event Listeners — Tools
if (btnCandidates) {
  btnCandidates.onclick = () => {
    if (!state.isWasmLoaded || state.isGenerating || state.isAnimatingSolve)
      return;
    state.showPencilMarks = !state.showPencilMarks;
    renderCurrentView();
    syncCandidatesButton();
  };
}

if (btnCheck) {
  btnCheck.onclick = () => {
    if (!state.isWasmLoaded || state.isGenerating || state.isAnimatingSolve)
      return;
    const boardToCheck = getDisplayedBoard(getSolveTrace(), state.currentBoard);
    const isValid = check(boardToCheck);
    if (isValid) {
      showToast("Validation successful! You solved it!", "success");
      triggerConfetti();
    } else {
      showToast("Not a valid complete solution yet!", "error");
    }
  };
}

if (btnLoadBoard) {
  btnLoadBoard.onclick = () => {
    if (!state.isWasmLoaded || state.isGenerating || state.isAnimatingSolve)
      return;
    clearSolveTrace();
    const parsed = normalizeBoardInput(inputBoard.value);
    if (!parsed) {
      showToast("Invalid board load string!", "error");
      return;
    }

    state.undoStack = [];
    state.redoStack = [];
    state.currentDifficulty = "custom";
    setBoard(parsed, {
      setAsGiven: true,
      highlightMode: "clue",
      clearSelection: true,
    });
    showToast("Sudoku board loaded successfully!", "success");
  };
}

if (btnCopyBoard) {
  btnCopyBoard.onclick = async () => {
    if (state.isGenerating || state.isAnimatingSolve) return;
    const format = (selectExportFormat.value === "dot" ? "dot" : "zero") as
      | "zero"
      | "dot";
    const output = boardForExport(
      getDisplayedBoard(getSolveTrace(), state.currentBoard),
      format,
    );

    try {
      await navigator.clipboard.writeText(output);
      showToast("Copied board string to clipboard!", "success");
    } catch (_err) {
      showToast("Clipboard copy failed!", "error");
    }
  };
}

if (selectExportFormat) {
  selectExportFormat.onchange = () => {
    syncBoardInput(getDisplayedBoard(getSolveTrace(), state.currentBoard));
  };
}

if (btnErase) {
  btnErase.onclick = () => {
    if (state.isGenerating || state.isAnimatingSolve) return;
    clearSolveTrace();
    state.showPencilMarks = false;
    syncCandidatesButton();
    const chars = state.currentBoard.split("");
    for (let i = 0; i < 81; i++) {
      if (!state.givenMask[i]) {
        chars[i] = "0";
      }
    }
    setBoard(chars.join(""), { clearSelection: true });
    infoPanel.style.display = "none";
    showToast("Erased all user cell entries", "info");
  };
}

if (btnReset) {
  btnReset.onclick = () => {
    if (state.isGenerating || state.isAnimatingSolve) return;
    clearSolveTrace();
    state.showPencilMarks = false;
    syncCandidatesButton();
    state.givenMask = Array(81).fill(false);
    state.undoStack = [];
    state.redoStack = [];
    state.currentDifficulty = "custom";
    setBoard("0".repeat(81), { clearSelection: true });
    infoPanel.style.display = "none";
    showToast("Cleared grid to a fresh blank slate", "info");
  };
}

// Keyboard shortcuts & navigation handler
document.addEventListener("keydown", (event) => {
  if (!state.isWasmLoaded || state.isGenerating || state.isAnimatingSolve)
    return;

  const target = event.target as HTMLElement | null;
  if (
    target &&
    (target.tagName === "INPUT" ||
      target.tagName === "TEXTAREA" ||
      target.tagName === "SELECT")
  ) {
    return;
  }

  // Undo/Redo shortcuts
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
    state.selectedCell = null;
    renderCurrentView();
    return;
  }

  if (state.selectedCell === null) return;

  if (/^[1-9]$/.test(event.key)) {
    updateCell(state.selectedCell, event.key);
    return;
  }

  if (
    event.key === "0" ||
    event.key === "." ||
    event.key === "Backspace" ||
    event.key === "Delete"
  ) {
    updateCell(state.selectedCell, "0");
    return;
  }

  if (event.key === "ArrowUp") {
    event.preventDefault();
    state.selectedCell =
      state.selectedCell >= 9 ? state.selectedCell - 9 : state.selectedCell;
    renderCurrentView();
    return;
  }
  if (event.key === "ArrowDown") {
    event.preventDefault();
    state.selectedCell =
      state.selectedCell <= 71 ? state.selectedCell + 9 : state.selectedCell;
    renderCurrentView();
    return;
  }
  if (event.key === "ArrowLeft") {
    event.preventDefault();
    state.selectedCell =
      state.selectedCell % 9 === 0
        ? state.selectedCell
        : state.selectedCell - 1;
    renderCurrentView();
    return;
  }
  if (event.key === "ArrowRight") {
    event.preventDefault();
    state.selectedCell =
      state.selectedCell % 9 === 8
        ? state.selectedCell
        : state.selectedCell + 1;
    renderCurrentView();
  }
});

// Info Modal controls
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

if (projectModal) {
  projectModal.onclick = (e) => {
    if (e.target === projectModal) {
      projectModal.style.display = "none";
    }
  };
}

// System Boot up
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

    // Bind Number Pad Buttons
    document.querySelectorAll(".numpad-btn").forEach((btn) => {
      const button = btn as HTMLButtonElement;
      button.onclick = () => {
        if (!state.isWasmLoaded || state.isGenerating || state.isAnimatingSolve)
          return;
        if (state.selectedCell === null) {
          showToast("Select a cell first to enter a number", "info");
          return;
        }
        const val = button.getAttribute("data-val");
        if (val) {
          updateCell(state.selectedCell, val);
        }
      };
    });

    // WASM is initialized automatically by the bundler (vite-plugin-wasm)
    state.isWasmLoaded = true;

    // Render initial hydrated state once loaded
    setBoard(state.currentBoard, { clearSelection: true });
    console.log("Rustoku WASM successfully initialized!");
  } catch (err) {
    console.error("Failed to load WASM module", err);
    if (grid) {
      grid.innerHTML = `<p style="color:red; grid-column:span 9; text-align:center; padding-top:50%;">Failed to load WASM</p>`;
    }
  }
}

run();
