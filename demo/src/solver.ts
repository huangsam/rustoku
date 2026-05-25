import { solve, solve_steps, candidates } from "../pkg/rustoku_wasm.js";
import {
  state,
  pushUndo,
  recomputeInvalidCells,
  setBoard,
  notify,
  syncBoardInput,
} from "./state";
import { showToast } from "./toast";
import { clearSolveTrace, showSolveTrace } from "./trace";
import { normalizeCandidateGrid } from "./trace-helpers";
import { renderGrid } from "./render";
import { btnSolve, btnSolveSteps } from "./elements";

export function animateSolve(solvedBoard: string): void {
  if (state.isAnimatingSolve || state.isGenerating) return;
  state.isAnimatingSolve = true;

  // Gather all empty indices that need to be solved
  const indicesToSolve: number[] = [];
  for (let i = 0; i < 81; i++) {
    if (state.currentBoard[i] === "0" && solvedBoard[i] !== "0") {
      indicesToSolve.push(i);
    }
  }

  if (indicesToSolve.length === 0) {
    // Already solved
    setBoard(solvedBoard, { highlightMode: "solved" });
    showToast("Sudoku board solved instantly!", "success");
    state.isAnimatingSolve = false;
    return;
  }

  // Shuffle empty indices to reveal them in a beautifully staggered, random order
  for (let i = indicesToSolve.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [indicesToSolve[i], indicesToSolve[j]] = [
      indicesToSolve[j],
      indicesToSolve[i],
    ];
  }

  // Record starting point in undo stack once before animating
  pushUndo();

  const totalSteps = indicesToSolve.length;
  let currentStepIndex = 0;

  // We reveal a few cells per tick to complete in around 350-500ms
  const cellsPerTick = Math.max(1, Math.round(totalSteps / 25));
  const intervalTime = 20;

  const timer = setInterval(() => {
    if (currentStepIndex >= totalSteps) {
      clearInterval(timer);
      state.isAnimatingSolve = false;
      // Guarantee final state matches solvedBoard completely
      setBoard(solvedBoard, { highlightMode: "solved" });
      showToast("Sudoku board solved!", "success");
      return;
    }

    const boardChars = state.currentBoard.split("");
    for (let c = 0; c < cellsPerTick && currentStepIndex < totalSteps; c++) {
      const idx = indicesToSolve[currentStepIndex];
      boardChars[idx] = solvedBoard[idx];
      currentStepIndex++;
    }

    // Update board in-place step-by-step
    state.currentBoard = boardChars.join("");
    recomputeInvalidCells();
    notify();
    // Render with "solved" highlightMode so the popIn styling is colored correctly
    renderGrid(state.currentBoard, "solved");
    syncBoardInput(state.currentBoard);
  }, intervalTime);
}

// Wire Event Listeners
if (btnSolve) {
  btnSolve.onclick = () => {
    if (!state.isWasmLoaded || state.isGenerating || state.isAnimatingSolve)
      return;
    if (state.currentBoard === "0".repeat(81)) {
      showToast("Please generate or load a board first.", "info");
      return;
    }
    clearSolveTrace();
    const solvedBoard = solve(state.currentBoard);
    if (solvedBoard && solvedBoard.length === 81) {
      animateSolve(solvedBoard);
    } else {
      showToast("Could not solve this board!", "error");
    }
  };
}

if (btnSolveSteps) {
  btnSolveSteps.onclick = () => {
    if (!state.isWasmLoaded || state.isGenerating || state.isAnimatingSolve)
      return;
    if (state.currentBoard === "0".repeat(81)) {
      showToast("Please generate or load a board first.", "info");
      return;
    }
    clearSolveTrace();
    const result = solve_steps(state.currentBoard, "expert");
    if (result) {
      const steps = Array.isArray(result.steps) ? result.steps : [];
      const initialCandidateGrid = normalizeCandidateGrid(
        candidates(state.currentBoard),
      );

      if (steps.length > 0) {
        showSolveTrace(
          state.currentBoard,
          initialCandidateGrid,
          result.board,
          steps,
        );
        showToast("Human-style solve steps loaded!", "success");
      } else {
        if (result.board !== state.currentBoard) {
          pushUndo();
          setBoard(result.board, { highlightMode: "solved" });
        }
        showToast("Solved! (No human steps recorded)", "info");
      }
    } else {
      showToast("Could not solve with human techniques.", "error");
    }
  };
}
