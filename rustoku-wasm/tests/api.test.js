import { describe, it, expect } from 'vitest';
import * as rustoku from '../pkg/rustoku_wasm.js';

describe('Rustoku WASM API', () => {
  it('should generate an easy puzzle', () => {
    const puzzle = rustoku.generate('easy');
    expect(puzzle).toHaveLength(81);
    expect(puzzle).toMatch(/^[0-9._]+$/);
  });

  it('should return empty string for invalid difficulty', () => {
    const puzzle = rustoku.generate('invalid');
    expect(puzzle).toBe('');
  });

  it('should solve a puzzle', () => {
    const puzzle = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
    const solution = rustoku.solve(puzzle);
    expect(solution).toHaveLength(81);
    expect(rustoku.check(solution)).toBe(true);
  });

  it('should return empty string for unsolvable puzzle', () => {
    const puzzle = "55..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
    const solution = rustoku.solve(puzzle);
    expect(solution).toBe('');
  });

  it('should find all solutions (smoke test)', () => {
    const puzzle = "4.....8.5.3..........7......2.....6.....8.4......1.......6.3.7.5..2.....1.4......";
    const solutions = rustoku.solve_all(puzzle);
    expect(Array.isArray(solutions)).toBe(true);
    expect(solutions.length).toBeGreaterThanOrEqual(1);
  });

  it('should return candidates grid', () => {
    const puzzle = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
    const grid = rustoku.candidates(puzzle);
    expect(Array.isArray(grid)).toBe(true);
    expect(grid).toHaveLength(9);
    expect(grid[0]).toHaveLength(9);
  });

  it('should handle advanced generation', () => {
    const puzzle = rustoku.generate_advanced('rotational180', 'medium');
    expect(puzzle).toHaveLength(81);
  });

  it('should return step-by-step solution trace', () => {
    const puzzle = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
    const result = rustoku.solve_steps(puzzle, "expert");
    expect(result).not.toBeNull();
    expect(result.board).toHaveLength(81);
    expect(Array.isArray(result.steps)).toBe(true);
    expect(result.steps.length).toBeGreaterThan(0);
    const firstStep = result.steps[0];
    expect(firstStep).toHaveProperty('type');
    expect(firstStep).toHaveProperty('row');
    expect(firstStep).toHaveProperty('col');
    expect(firstStep).toHaveProperty('value');
    expect(firstStep).toHaveProperty('technique');
    expect(firstStep).toHaveProperty('step_number');
  });

  it('should return null for solve_steps with invalid difficulty or unsolvable puzzle', () => {
    const puzzle = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
    expect(rustoku.solve_steps(puzzle, "invalid_difficulty")).toBeNull();

    const unsolvable = "55..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79";
    expect(rustoku.solve_steps(unsolvable, "expert")).toBeNull();
  });
});
