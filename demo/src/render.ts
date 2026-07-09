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
  const currentStepObj = solveTrace ? getCurrentTraceStep(solveTrace) : null;
  const traceFocusIndex =
    solveTrace && solveTrace.currentStep >= 0
      ? getTraceCellIndex(solveTrace.steps[solveTrace.currentStep])
      : null;
  const traceAffectedIndices = getTraceAffectedIndices(currentStepObj);
  const traceCandidateGrid =
    solveTrace && solveTrace.currentStep >= 0
      ? buildTraceCandidateGrid(
          solveTrace.initialCandidateGrid,
          solveTrace.steps,
          solveTrace.currentStep,
        )
      : null;
  const prevCandidateGrid =
    solveTrace && solveTrace.currentStep >= 0
      ? buildTraceCandidateGrid(
          solveTrace.initialCandidateGrid,
          solveTrace.steps,
          solveTrace.currentStep - 1,
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
        if (state.selectedCell === i) return;
        state.selectedCell = i;
        notify();
      });
      cell.addEventListener("focus", () => {
        if (state.isGenerating || state.isAnimatingSolve) return;
        if (state.selectedCell === i) return;
        state.selectedCell = i;
        notify();
      });
      cell.dataset.bound = "1";
    }

    const val = boardStr[i];

    // Compute status states
    let isClue = false;
    let isSolved = false;
    const isSelected = state.selectedCell === i;
    const isInvalid = invalidSet.has(i);
    let isRelated = false;
    let isSameDigit = false;
    const isTraceAffected = traceAffectedIndices.has(i);
    const isTraceFocus = traceFocusIndex === i;
    let isTraceRelated = false;
    let isJustPlaced = false;

    // Cell relationships & same-digit highlights
    if (solveTrace && currentStepObj) {
      // Trace mode highlighting
      if (traceFocusIndex !== null) {
        const focusRow = Math.floor(traceFocusIndex / 9);
        const focusCol = traceFocusIndex % 9;
        const focusBox =
          Math.floor(focusRow / 3) * 3 + Math.floor(focusCol / 3);

        const cellRow = Math.floor(i / 9);
        const cellCol = i % 9;
        const cellBox = Math.floor(cellRow / 3) * 3 + Math.floor(cellCol / 3);

        if (
          i !== traceFocusIndex &&
          (cellRow === focusRow || cellCol === focusCol || cellBox === focusBox)
        ) {
          isTraceRelated = true;
        }
      }

      // Highlight the active digit of interest across the board
      const interestDigit = currentStepObj.value;
      if (interestDigit >= 1 && interestDigit <= 9) {
        const interestValStr = String(interestDigit);
        if (val === interestValStr && i !== traceFocusIndex) {
          isSameDigit = true;
        }
      }
    } else if (state.selectedCell !== null) {
      // Normal play highlighting
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
        isRelated = true;
      }

      const selectedVal = boardStr[state.selectedCell];
      if (
        selectedVal !== "0" &&
        val === selectedVal &&
        i !== state.selectedCell
      ) {
        isSameDigit = true;
      }
    }

    if (val !== "0") {
      if (state.givenMask[i]) {
        isClue = true;
      } else if (highlightMode === "solved") {
        isSolved = true;
      } else if (state.lastPlacedCell === i) {
        isJustPlaced = true;
      }
    }

    // Toggle CSS classes instead of removing/re-adding unconditionally to prevent rendering flicker
    cell.classList.toggle("clue", isClue);
    cell.classList.toggle("solved", isSolved);
    cell.classList.toggle("selected", isSelected);
    cell.classList.toggle("invalid", isInvalid);
    cell.classList.toggle("related", isRelated);
    cell.classList.toggle("same-digit", isSameDigit);
    cell.classList.toggle("trace-affected", isTraceAffected);
    cell.classList.toggle("trace-focus", isTraceFocus);
    cell.classList.toggle("trace-related", isTraceRelated);
    cell.classList.toggle("just-placed", isJustPlaced);

    let isNewValue = false;
    if (val !== "0") {
      const currentSpan = cell.querySelector("span");
      const currentVal = currentSpan ? currentSpan.textContent : null;
      isNewValue =
        currentVal !== val || Boolean(cell.querySelector(".pencil-grid"));
      if (isNewValue) {
        cell.innerHTML = `<span>${val}</span>`;
      }
    } else {
      if ((state.showPencilMarks || Boolean(solveTrace)) && candidateGrid) {
        const row = Math.floor(i / 9);
        const col = i % 9;

        let cellCandidates: number[] = [];
        let removedCandidates: number[] = [];
        let addedCandidates: number[] = [];

        if (solveTrace && prevCandidateGrid && currentStepObj) {
          cellCandidates = prevCandidateGrid[row]?.[col] || [];
          const change = currentStepObj.candidate_changes?.find(
            (c) => c.row === row && c.col === col,
          );
          if (change) {
            removedCandidates = change.removed || [];
            addedCandidates = change.added || [];
          }
        } else {
          cellCandidates = candidateGrid[row]?.[col] || [];
        }

        const marks = Array.from({ length: 9 }, (_, n) => {
          const digit = n + 1;
          if (removedCandidates.includes(digit)) {
            return `<span class="pmark visible pmark-removed">${digit}</span>`;
          } else if (addedCandidates.includes(digit)) {
            return `<span class="pmark visible pmark-added">${digit}</span>`;
          } else {
            const visible = cellCandidates.includes(digit);
            return `<span class="pmark${visible ? " visible" : ""}">${digit}</span>`;
          }
        }).join("");
        const newHTML = `<div class="pencil-grid">${marks}</div>`;
        if (cell.innerHTML !== newHTML) {
          cell.innerHTML = newHTML;
        }
      } else {
        if (cell.innerHTML !== "") {
          cell.innerHTML = "";
        }
      }
    }

    cell.classList.toggle("pop-in", isNewValue && !state.givenMask[i]);
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
