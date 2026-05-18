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
