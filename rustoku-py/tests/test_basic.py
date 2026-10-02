import rustoku
import unittest


class TestRustoku(unittest.TestCase):
    def test_generate_basic(self):
        puzzle = rustoku.generate("easy")
        self.assertEqual(len(puzzle), 81)
        self.assertTrue(all(c in "0123456789." for c in puzzle))

    def test_generate_advanced(self):
        # Test symmetry and difficulty-first generation
        puzzle = rustoku.generate_advanced(symmetry="rotational180", difficulty="medium")
        self.assertEqual(len(puzzle), 81)

        # Test purely random
        random_puzzle = rustoku.generate_advanced(symmetry="none", difficulty=None)
        self.assertEqual(len(random_puzzle), 81)

    def test_solve(self):
        puzzle = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
        solution = rustoku.solve(puzzle)
        self.assertEqual(len(solution), 81)
        self.assertTrue(rustoku.check(solution))

    def test_invalid_difficulty(self):
        with self.assertRaises(ValueError):
            rustoku.generate("invalid")

    def test_invalid_symmetry(self):
        # Invalid symmetry should fallback to None (as per Rust implementation) or we can test if it raises
        # Based on bind.rs, it fallbacks to Symmetry::None for unknown strings
        puzzle = rustoku.generate_advanced(symmetry="invalid")
        self.assertEqual(len(puzzle), 81)

    def test_solve_steps(self):
        puzzle = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
        result = rustoku.solve_steps(puzzle, "expert")
        self.assertIsNotNone(result)
        self.assertEqual(len(result["board"]), 81)
        self.assertIsInstance(result["steps"], list)
        self.assertGreater(len(result["steps"]), 0)
        first_step = result["steps"][0]
        for key in [
            "type",
            "row",
            "col",
            "value",
            "technique",
            "step_number",
            "candidates_eliminated",
            "related_cell_count",
            "difficulty_point",
        ]:
            self.assertIn(key, first_step)

    def test_solve_steps_unsolvable(self):
        # A valid Sudoku format with no initial duplicate clues, but impossible to complete
        unsolvable = "078002609030008020002000083000000040043090000007300090200001036001840902050003007"
        result = rustoku.solve_steps(unsolvable, "expert")
        self.assertIsNone(result)

    def test_solve_invalid_inputs(self):
        # Invalid length
        with self.assertRaises(ValueError):
            rustoku.solve("too_short")
        with self.assertRaises(ValueError):
            rustoku.solve("1" * 80)
        with self.assertRaises(ValueError):
            rustoku.solve("1" * 82)

        # Invalid characters
        with self.assertRaises(ValueError):
            rustoku.solve("X" + "0" * 80)

        # Duplicate initial clues
        with self.assertRaises(ValueError):
            rustoku.solve("55" + "0" * 79)

    def test_solve_all_invalid_inputs(self):
        with self.assertRaises(ValueError):
            rustoku.solve_all("invalid")

    def test_solve_steps_invalid_difficulty(self):
        puzzle = "53..7....6..195....98....6.8...6...34..8.3..17...2...6.6....28....419..5....8..79"
        with self.assertRaises(ValueError):
            rustoku.solve_steps(puzzle, "unknown_difficulty")

    def test_candidates_invalid_inputs(self):
        with self.assertRaises(ValueError):
            rustoku.candidates("invalid")

    def test_check_invalid_inputs(self):
        with self.assertRaises(ValueError):
            rustoku.check("invalid")

    def test_generate_advanced_invalid_difficulty(self):
        with self.assertRaises(ValueError):
            rustoku.generate_advanced(symmetry="none", difficulty="unknown_difficulty")


if __name__ == "__main__":
    unittest.main()
