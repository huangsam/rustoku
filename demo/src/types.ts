export type HighlightMode = "none" | "clue" | "solved";

export type CandidateGrid = number[][][];

export interface CandidateChange {
  row: number;
  col: number;
  before: number[];
  after: number[];
  removed: number[];
  added: number[];
}

export interface SolveTraceStep {
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
}

export interface SolveTraceState {
  initialBoard: string;
  initialCandidateGrid: CandidateGrid;
  solvedBoard: string;
  steps: SolveTraceStep[];
  currentStep: number;
  isPlaying: boolean;
  playbackTimer: number | null;
}

export type ThemeName =
  | "midnight"
  | "paper"
  | "arcade"
  | "blueprint"
  | "ember"
  | "forest"
  | "mono";
