import { state, notify, computeInvalidCells, syncBoardInput } from "./state";
import {
  grid,
  statDifficulty,
  statGivens,
  statProgress,
  btnCandidates,
  btnUndo,
  btnRedo,
} from "./elements";
import {
  getTraceCellIndex,
  getTraceAffectedIndices,
  buildTraceCandidateGrid,
  normalizeCandidateGrid,
  getCurrentTraceStep,
  getDisplayedBoard,
} from "./trace-helpers";
import { getSolveTrace } from "./trace";
import { candidates } from "../pkg/rustoku_wasm.js";
import type { HighlightMode } from "./types";

export function syncCandidatesButton(): void {
  if (!btnCandidates) {
    return;
  }

  const solveTrace = getSolveTrace();
  if (solveTrace) {
    btnCandidates.textContent = "Trace Candidates";
    btnCandidates.disabled = true;
    return;
  }

  btnCandidates.disabled = false;
  btnCandidates.textContent = state.showPencilMarks
    ? "Hide Candidates"
    : "Candidates";
}

export function updateHistoryButtons(): void {
  const solveTrace = getSolveTrace();
  const isDisabled =
    state.isGenerating || state.isAnimatingSolve || Boolean(solveTrace);

  if (btnUndo) {
    btnUndo.disabled = isDisabled || state.undoStack.length === 0;
  }
  if (btnRedo) {
    btnRedo.disabled = isDisabled || state.redoStack.length === 0;
  }
}

export function updateNumpadCounts(boardStr: string): void {
  const counts = Array(10).fill(0);
  for (let i = 0; i < 81; i++) {
    const val = parseInt(boardStr[i], 10);
    if (val >= 1 && val <= 9) {
      counts[val]++;
    }
  }

  const selectedVal =
    state.selectedCell !== null
      ? parseInt(boardStr[state.selectedCell], 10)
      : 0;

  for (let digit = 1; digit <= 9; digit++) {
    const btn = document.querySelector(
      `.numpad-btn[data-val="${digit}"]`,
    ) as HTMLButtonElement;
    if (!btn) continue;

    const count = counts[digit];
    const isCompleted = count === 9;
    const isActive = selectedVal === digit;

    if (isCompleted) {
      btn.classList.add("completed");
    } else {
      btn.classList.remove("completed");
    }

    if (isActive) {
      btn.classList.add("active");
    } else {
      btn.classList.remove("active");
    }

    const countDisplay = isCompleted ? "✓" : `${count}/9`;
    btn.innerHTML = `
      <div class="numpad-btn-inner">
        <span class="num">${digit}</span>
        <span class="count">${countDisplay}</span>
      </div>
    `;
  }
}

export function updateStats(): void {
  if (!statDifficulty || !statGivens || !statProgress) return;

  // 1. Difficulty
  statDifficulty.textContent =
    state.currentDifficulty.charAt(0).toUpperCase() +
    state.currentDifficulty.slice(1);
  statDifficulty.className = "stat-badge";
  if (
    ["easy", "medium", "hard", "expert"].includes(
      state.currentDifficulty.toLowerCase(),
    )
  ) {
    statDifficulty.classList.add(state.currentDifficulty.toLowerCase());
  }

  // 2. Givens
  const cluesCount = state.givenMask.filter(Boolean).length;
  statGivens.textContent = String(cluesCount);

  // 3. Progress
  const filledCount = state.currentBoard
    .split("")
    .filter((ch) => ch !== "0").length;
  statProgress.textContent = `${filledCount}/81`;
}

export function renderGrid(
  boardStr: string,
  highlightMode: "none" | "clue" | "solved" = "none",
): void {
  if (!grid) return;

  // Dynamically update stats bar on every render
  updateStats();
  updateNumpadCounts(boardStr);
  updateHistoryButtons();

  const cells = grid.querySelectorAll(".cell");
  const needsCreate = cells.length === 0;
  const solveTrace = getSolveTrace();
  let candidateGrid: number[][][] | null = null;

  const invalidSet =
    boardStr === state.currentBoard
      ? state.invalidCells
      : computeInvalidCells(boardStr);
  const traceFocusIndex =
    solveTrace && solveTrace.currentStep >= 0
      ? getTraceCellIndex(solveTrace.steps[solveTrace.currentStep])
      : null;
  const traceAffectedIndices = getTraceAffectedIndices(
    getCurrentTraceStep(solveTrace),
  );
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
  } else if (state.showPencilMarks && state.isWasmLoaded) {
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
        if (state.isGenerating || state.isAnimatingSolve) return;
        state.selectedCell = i;
        notify();
      });
      cell.addEventListener("focus", () => {
        if (state.isGenerating || state.isAnimatingSolve) return;
        state.selectedCell = i;
        notify();
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
      "just-placed",
      "pop-in",
    );

    if (state.selectedCell !== null) {
      const selectedRow = Math.floor(state.selectedCell / 9);
      const selectedCol = state.selectedCell % 9;
      const selectedBox =
        Math.floor(selectedRow / 3) * 3 + Math.floor(selectedCol / 3);

      const cellRow = Math.floor(i / 9);
      const cellCol = i % 9;
      const cellBox = Math.floor(cellRow / 3) * 3 + Math.floor(cellCol / 3);

      if (
        i !== state.selectedCell &&
        (cellRow === selectedRow ||
          cellCol === selectedCol ||
          cellBox === selectedBox)
      ) {
        cell.classList.add("related");
      }

      const selectedVal = boardStr[state.selectedCell];
      if (
        selectedVal !== "0" &&
        val === selectedVal &&
        i !== state.selectedCell
      ) {
        cell.classList.add("same-digit");
      }
    }

    if (state.selectedCell === i) {
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
      const currentSpan = cell.querySelector("span");
      const currentVal = currentSpan ? currentSpan.textContent : null;
      const isNewValue =
        currentVal !== val || cell.querySelector(".pencil-grid");
      if (isNewValue) {
        cell.innerHTML = `<span>${val}</span>`;
        if (!state.givenMask[i]) {
          cell.classList.add("pop-in");
        }
      }

      if (state.givenMask[i]) {
        cell.classList.add("clue");
      } else if (highlightMode === "solved") {
        cell.classList.add("solved");
      } else if (state.lastPlacedCell === i) {
        cell.classList.add("just-placed");
      }
    } else {
      if ((state.showPencilMarks || Boolean(solveTrace)) && candidateGrid) {
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

  // Clear transient flag
  state.lastPlacedCell = null;
}

export function renderCurrentView(): void {
  const solveTrace = getSolveTrace();
  if (solveTrace) {
    const replayBoard = getDisplayedBoard(solveTrace, state.currentBoard);
    const mode: HighlightMode =
      solveTrace.currentStep >= solveTrace.steps.length - 1 ? "solved" : "none";
    renderGrid(replayBoard, mode);
    syncBoardInput(replayBoard);
    return;
  }

  renderGrid(state.currentBoard, state.currentHighlightMode);
  syncBoardInput(state.currentBoard);
}
