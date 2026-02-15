# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.13.0] - 2026-02-15

### Added
- `--until` flag for limiting the number of solutions found in `solve all` command
- Parallel solution finding using Rayon for improved performance
- Enhanced verbose mode with better solve path visualization
- Enhanced human-readable solving mode with clearer technique explanations
- Improved CSV processing with pass/fail logic for batch operations
- Agentic documentation (AGENTS.md) for project overview and API details
- CONTRIBUTING.md with publishing and contribution guidelines

### Changed
- Refactored units logic into dedicated module for better organization
- Reduced nesting in core and entry logic for improved readability
- Enhanced documentation across all solving techniques
- Improved API for building and manipulating boards
- Adjusted solver to work with DFS (Depth-First Search) approach

### Fixed
- Logic and performance fixes in core solving algorithms
- Various bug fixes and code quality improvements

## [0.12.3] - 2025-11-03

### Changed
- Applied formatting improvements to format.rs
- Adjusted imports and gitignore for better project organization
- Updated Rust syntax for println statements

## [0.12.2] - 2025-06-13

### Added
- Verbose flag to solve commands for enhanced output

## [0.12.1] - 2025-06-11

### Changed
- Updated documentation for format.rs
- Simplified UX for all solutions display
- Improved CLI help text and emoji spacing
- Removed unnecessary padding in output

## [0.12.0] - 2025-06-10

### Added
- Enhanced CLI interface with emojis for better user experience
- Hidden Rustoku board internals from public API

### Changed
- Improved emoji output across the CLI
- Simplified error messages
- Removed author information from CLI derive arguments
- Removed caps lock and trailing periods from output

### Fixed
- Broken end-to-end test

## [0.11.1] - 2025-06-08

### Added
- Public method to access candidates

### Changed
- Removed Default trait from some APIs
- Adjusted API visibility within core module
- Removed redundant .data folder

## [0.11.0] - 2025-06-08

### Added
- Comprehensive unit tests for masks
- Integration tests for each solving technique
- Parametrized tests for better test coverage
- Documentation for propagator and technique trait
- More examples from the web for testing

### Changed
- Limited error API to a single enum for consistency
- Reduced API surface for cleaner library interface
- Simplified formatting APIs
- Refactored core technique handling for better testability
- Renamed candidate_cache to candidates for clarity
- Renamed TechniqueMasks to TechniqueFlags

### Fixed
- Clippy warnings

## [0.10.1] - 2025-06-05

### Changed
- Enhanced library integration with the CLI
- Added missing period in documentation

### Fixed
- Broken link in README

## [0.10.0] - 2025-06-05

### Added
- Solve path tracking with elimination and placement steps
- Hidden pairs solving technique
- Technique insights to solve path
- Candidate elimination tracking
- More test examples and hard puzzles
- Display trait for technique flags
- Code strings for SolveStep enum

### Changed
- Converted SolveStep from struct to enum
- Renamed key variables in various techniques for clarity
- Simplified hidden pairs implementation
- Rearranged technique difficulty ordering
- Changed output format from dots to zeros
- Enhanced README with examples and attribution

### Fixed
- README reference to curated inputs
- Clippy issues

## [0.9.6] - 2025-06-02

### Added
- Display trait to technique flags

### Changed
- Migrated TechniqueFlags to dedicated flags.rs module
- Renamed TechniqueMasks to TechniqueFlags
- Migrated technique display to format.rs
- Enhanced README content

### Fixed
- Extraneous comma in board.rs

## [0.9.5] - 2025-06-02

### Added
- Keywords to Cargo workspace metadata
- Minimum Rust version specification

### Changed
- Consolidated README to focus on key features
- Enhanced call to action in documentation

## [0.9.4] - 2025-06-02

### Changed
- Improved documentation for solve commands
- Adjusted README content
- Removed context about all techniques

### Removed
- Human subcommand from CLI

### Fixed
- Print utility reference in format docs

## [0.9.3] - 2025-06-02

### Added
- Documentation badge to README
- Engaging flair to about text

### Changed
- Improved README content and install instructions
- Added documentation for public struct fields

## [0.9.2] - 2025-06-01

### Fixed
- License details in Cargo.toml files for both crates

## [0.9.1] - 2025-06-01

### Added
- Initial release with core Sudoku solving functionality
- Basic CLI interface
- Support for standard Sudoku solving techniques

[0.13.0]: https://github.com/huangsam/rustoku/compare/rustoku-lib-v0.12.3...rustoku-lib-v0.13.0
[0.12.3]: https://github.com/huangsam/rustoku/compare/rustoku-lib-v0.12.2...rustoku-lib-v0.12.3
[0.12.2]: https://github.com/huangsam/rustoku/compare/rustoku-lib-v0.12.1...rustoku-lib-v0.12.2
[0.12.1]: https://github.com/huangsam/rustoku/compare/rustoku-lib-v0.12.0...rustoku-lib-v0.12.1
[0.12.0]: https://github.com/huangsam/rustoku/compare/rustoku-lib-v0.11.1...rustoku-lib-v0.12.0
[0.11.1]: https://github.com/huangsam/rustoku/compare/rustoku-lib-v0.11.0...rustoku-lib-v0.11.1
[0.11.0]: https://github.com/huangsam/rustoku/compare/rustoku-lib-v0.10.1...rustoku-lib-v0.11.0
[0.10.1]: https://github.com/huangsam/rustoku/compare/rustoku-lib-v0.10.0...rustoku-lib-v0.10.1
[0.10.0]: https://github.com/huangsam/rustoku/compare/rustoku-lib-v0.9.6...rustoku-lib-v0.10.0
[0.9.6]: https://github.com/huangsam/rustoku/compare/rustoku-lib-v0.9.5...rustoku-lib-v0.9.6
[0.9.5]: https://github.com/huangsam/rustoku/compare/rustoku-lib-v0.9.4...rustoku-lib-v0.9.5
[0.9.4]: https://github.com/huangsam/rustoku/compare/rustoku-lib-v0.9.3...rustoku-lib-v0.9.4
[0.9.3]: https://github.com/huangsam/rustoku/compare/rustoku-lib-v0.9.2...rustoku-lib-v0.9.3
[0.9.2]: https://github.com/huangsam/rustoku/compare/rustoku-lib-v0.9.1...rustoku-lib-v0.9.2
[0.9.1]: https://github.com/huangsam/rustoku/releases/tag/rustoku-lib-v0.9.1</content>
