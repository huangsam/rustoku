import { generate_advanced } from "../pkg/rustoku_wasm.js";
import { state, setBoard } from "./state";
import { showToast } from "./toast";
import {
  btnGenerate,
  gridLoader,
  selectGenDifficulty,
  selectGenSymmetry,
} from "./elements";
import { clearSolveTrace } from "./trace";

if (btnGenerate) {
  btnGenerate.onclick = () => {
    if (!state.isWasmLoaded || state.isGenerating || state.isAnimatingSolve)
      return;

    // Show loading overlay
    state.isGenerating = true;
    if (gridLoader) {
      gridLoader.style.display = "flex";
    }
    btnGenerate.disabled = true;

    clearSolveTrace();
    const difficulty = selectGenDifficulty.value;
    const symmetry = selectGenSymmetry.value;

    // Defer execution to allow loader animation to render
    setTimeout(() => {
      try {
        const diffVal = difficulty === "random" ? null : difficulty;
        const boardStr = generate_advanced(symmetry, diffVal as string);

        if (boardStr && boardStr.length === 81) {
          state.undoStack = [];
          state.redoStack = [];
          state.currentDifficulty = difficulty;
          setBoard(boardStr, {
            setAsGiven: true,
            highlightMode: "clue",
            clearSelection: true,
          });
          showToast("Puzzle generated successfully!", "success");
        } else {
          showToast(
            "Generation failed! Try reducing difficulty or changing symmetry.",
            "error",
          );
        }
      } catch (err) {
        console.error(err);
        showToast("Error during generation", "error");
      } finally {
        state.isGenerating = false;
        if (gridLoader) {
          gridLoader.style.display = "none";
        }
        btnGenerate.disabled = false;
      }
    }, 50);
  };
}
