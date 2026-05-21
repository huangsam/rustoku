import type { CandidateGrid, SolveTraceStep, SolveTraceState } from "./types";
import {
  infoPanel,
  infoTitle,
  solveTracePanel,
  solveTraceStepCount,
  solveTraceStatus,
  solveTraceTechnique,
  solveTracePlacement,
  solveTraceDetail,
  solveTraceChanges,
  btnTracePrev,
  btnTracePlay,
  btnTraceNext,
  btnTraceNextPlacement,
  btnTraceNextElimination,
  btnCloseInfo,
} from "./elements";
import {
  getTraceCellIndex,
  getCurrentTraceStep,
  isPlacementStep,
  findNextTraceStepIndex,
  formatCellLabel,
  titleCaseTechnique,
  formatDigitList,
} from "./trace-helpers";
import { notify, onClearSolveTrace } from "./state";

// Module-level SolveTraceState
let solveTrace: SolveTraceState | null = null;

const TECHNIQUE_INFO: Record<
  string,
  { desc: string; difficulty: "easy" | "medium" | "hard" | "expert" }
> = {
  "Naked Singles": {
    desc: "Only one candidate number is possible for this cell. Therefore, that number must be placed here.",
    difficulty: "easy",
  },
  "Hidden Singles": {
    desc: "This number can only go in one cell within its row, column, or box. Even if other candidates are possible in this cell, this number must be placed here.",
    difficulty: "easy",
  },
  "Naked Pairs": {
    desc: "Two cells in the same house contain exactly the same two candidates. These candidates can be eliminated from all other cells in that house.",
    difficulty: "medium",
  },
  "Hidden Pairs": {
    desc: "Two candidates can only go in the same two cells within a house. All other candidates can be eliminated from those two cells.",
    difficulty: "medium",
  },
  "Locked Candidates": {
    desc: "A candidate is restricted to a single row or column within a 3x3 box (or vice versa), allowing eliminations outside that constraint.",
    difficulty: "medium",
  },
  "Naked Triples": {
    desc: "Three cells in the same house contain a subset of the same three candidates. These candidates can be eliminated from all other cells in that house.",
    difficulty: "medium",
  },
  "Hidden Triples": {
    desc: "Three candidates can only go in the same three cells within a house. All other candidates can be eliminated from those three cells.",
    difficulty: "medium",
  },
  "X-Wing": {
    desc: "A candidate is restricted to the same two cells in two parallel rows (or columns). It can be eliminated from all other cells in those columns/rows.",
    difficulty: "hard",
  },
  "Naked Quads": {
    desc: "Four cells in the same house contain a subset of the same four candidates. These candidates can be eliminated from all other cells in that house.",
    difficulty: "hard",
  },
  "Hidden Quads": {
    desc: "Four candidates can only go in the same four cells within a house. All other candidates can be eliminated from those four cells.",
    difficulty: "hard",
  },
  Swordfish: {
    desc: "A candidate is restricted to at most three cells in three rows (or columns) that align in three columns (or rows). The candidate is eliminated elsewhere.",
    difficulty: "hard",
  },
  Jellyfish: {
    desc: "A candidate is restricted to at most four cells in four rows (or columns) that align in four columns (or rows). The candidate is eliminated elsewhere.",
    difficulty: "hard",
  },
  Skyscraper: {
    desc: "Two columns (or rows) have exactly two cells containing a candidate. Two of these cells align, allowing eliminations from cells seeing the other two.",
    difficulty: "hard",
  },
  "W-Wing": {
    desc: "Two bi-value cells contain the same candidates. They are linked through a third house, allowing eliminations from cells seeing both.",
    difficulty: "expert",
  },
  "XY-Wing": {
    desc: "A pivot cell and two pincers share candidates. Any cell that sees both pincers cannot contain their shared candidate, allowing its elimination.",
    difficulty: "expert",
  },
  "XYZ-Wing": {
    desc: "A pivot cell with three candidates and two pincers. Any cell that sees all three of these cells cannot contain the shared candidate, allowing its elimination.",
    difficulty: "expert",
  },
  "Alternating Inference Chain": {
    desc: "A chain of strong and weak links between candidate cells. If the chain forms a cycle or a contradiction, candidates can be eliminated.",
    difficulty: "expert",
  },
};

export function getSolveTrace(): SolveTraceState | null {
  return solveTrace;
}

export function stopSolveTracePlayback(): void {
  const trace = solveTrace;
  if (!trace) return;

  if (trace.playbackTimer !== null) {
    window.clearTimeout(trace.playbackTimer);
    trace.playbackTimer = null;
  }

  trace.isPlaying = false;
}

export function clearSolveTrace(): void {
  if (!solveTrace) return;

  stopSolveTracePlayback();
  solveTrace = null;
  solveTracePanel.hidden = true;
  notify();
}

// Hook into state clear triggers (e.g. from updateCell, undo, redo)
onClearSolveTrace(() => {
  clearSolveTrace();
});

export function renderTraceChanges(step: SolveTraceStep | null): void {
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

export function renderSolveTracePanel(): void {
  if (!solveTrace) return;

  const totalSteps = solveTrace.steps.length;
  const currentStep = getCurrentTraceStep(solveTrace);
  if (!currentStep) return;
  const currentCell = currentStep ? getTraceCellIndex(currentStep) : null;
  const stepNumber = solveTrace.currentStep + 1;
  const isComplete = solveTrace.currentStep >= totalSteps - 1;
  const eliminated = currentStep.candidates_eliminated ?? 0;
  const relatedCells = currentStep.related_cell_count ?? 0;
  const isPlacement = isPlacementStep(currentStep);
  const nextPlacementIndex = findNextTraceStepIndex(
    solveTrace,
    solveTrace.currentStep,
    isPlacementStep,
  );
  const nextEliminationIndex = findNextTraceStepIndex(
    solveTrace,
    solveTrace.currentStep,
    (step) => !isPlacementStep(step),
  );

  infoTitle.textContent = "Solve Steps";
  infoPanel.style.display = "block";
  solveTracePanel.hidden = false;

  solveTraceStepCount.textContent = `Step ${stepNumber} of ${totalSteps}`;
  solveTraceStatus.textContent = isComplete
    ? "Solved board"
    : solveTrace.isPlaying
      ? "Auto-playing"
      : "Manual review";

  const techName = titleCaseTechnique(currentStep.technique);
  const techInfo = TECHNIQUE_INFO[techName] || {
    desc: isPlacement ? "Placement step." : "Elimination step.",
    difficulty: "easy",
  };

  solveTraceTechnique.innerHTML = `
    <span class="tech-badge difficulty-${techInfo.difficulty}">${techInfo.difficulty.toUpperCase()}</span>
    <span class="tech-name">${techName}</span>
  `;

  solveTracePlacement.innerHTML = isPlacement
    ? `Place <span class="trace-value-highlight">${currentStep.value}</span> in <strong>${formatCellLabel(currentCell)}</strong>`
    : `Eliminate candidate <span class="trace-value-highlight elim">${currentStep.value}</span> in <strong>${formatCellLabel(currentCell)}</strong>`;

  let detailHtml: string;
  if (isComplete) {
    detailHtml = "Final step reached. The board now matches the solved state.";
  } else {
    detailHtml = `<div class="tech-explanation">${techInfo.desc}</div>`;
    if (isPlacement) {
      detailHtml += `<div class="tech-action-detail">Use the controls below to inspect candidates, or play the remaining trace.</div>`;
    } else {
      detailHtml += `<div class="tech-action-detail">Removed ${eliminated} candidate${eliminated === 1 ? "" : "s"} across ${relatedCells} related cell${relatedCells === 1 ? "" : "s"}. The digits do not change.</div>`;
    }
  }
  solveTraceDetail.innerHTML = detailHtml;

  renderTraceChanges(currentStep);

  btnTracePrev.disabled = solveTrace.currentStep <= 0;
  btnTraceNext.disabled = solveTrace.currentStep >= totalSteps - 1;
  btnTracePlay.textContent = solveTrace.isPlaying ? "Pause" : "Play";
  btnTraceNextPlacement.disabled = nextPlacementIndex === null;
  btnTraceNextElimination.disabled = nextEliminationIndex === null;
}

export function scheduleSolveTracePlayback(): void {
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
    notify();
    scheduleSolveTracePlayback();
  }, 700);
}

export function toggleSolveTracePlayback(): void {
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
  notify();
  scheduleSolveTracePlayback();
}

export function showSolveTrace(
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

  renderSolveTracePanel();
  notify();
  infoPanel.scrollIntoView({ behavior: "smooth" });
}

// Bind Button Listeners
if (btnCloseInfo) {
  btnCloseInfo.onclick = () => {
    stopSolveTracePlayback();
    if (solveTrace) {
      renderSolveTracePanel();
    }
    infoPanel.style.display = "none";
  };
}

if (btnTracePrev) {
  btnTracePrev.onclick = () => {
    if (!solveTrace) return;
    stopSolveTracePlayback();
    solveTrace.currentStep = Math.max(0, solveTrace.currentStep - 1);
    renderSolveTracePanel();
    notify();
  };
}

if (btnTraceNext) {
  btnTraceNext.onclick = () => {
    if (!solveTrace) return;
    stopSolveTracePlayback();
    solveTrace.currentStep = Math.min(
      solveTrace.steps.length - 1,
      solveTrace.currentStep + 1,
    );
    renderSolveTracePanel();
    notify();
  };
}

if (btnTracePlay) {
  btnTracePlay.onclick = () => {
    toggleSolveTracePlayback();
  };
}

if (btnTraceNextPlacement) {
  btnTraceNextPlacement.onclick = () => {
    if (!solveTrace) return;
    const nextIndex = findNextTraceStepIndex(
      solveTrace,
      solveTrace.currentStep,
      isPlacementStep,
    );
    if (nextIndex === null) return;
    stopSolveTracePlayback();
    solveTrace.currentStep = nextIndex;
    renderSolveTracePanel();
    notify();
  };
}

if (btnTraceNextElimination) {
  btnTraceNextElimination.onclick = () => {
    if (!solveTrace) return;
    const nextIndex = findNextTraceStepIndex(
      solveTrace,
      solveTrace.currentStep,
      (step) => !isPlacementStep(step),
    );
    if (nextIndex === null) return;
    stopSolveTracePlayback();
    solveTrace.currentStep = nextIndex;
    renderSolveTracePanel();
    notify();
  };
}
