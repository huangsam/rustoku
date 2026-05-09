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


if __name__ == "__main__":
    unittest.main()
