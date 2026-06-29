# Changelog

All notable changes to BNN Code are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.3] — 2026-06-28

### Added
- **4 new CLI commands**: `fix`, `commit`, `review`, `document` with full inference engine integration
- **`run_inference_on_file()`** — Shared helper that reads a file, runs BNN inference, and returns the response
- **`run_inference_on_codebase()`** — Codebase-wide analysis: indexes, retrieves context, and runs inference
- **`run_inference()`** — Direct prompt inference (no retrieval), used by `commit` and `review` diff modes
- **Model auto-discovery** — `BnnInference::new()` finds any `.onnx` file in `models/`, not just `model.onnx`
- **Model validation** — Three-layer check (exists → size → ONNX magic byte) with actionable error messages
- **`CITATION.cff`** — Machine-readable citation metadata with leaderboard and dependency references
- **`llms.txt`** — LLM-optimized project index following the llms.txt specification
- **`CONTRIBUTING.md`** — Contribution guidelines for community contributors
- **`ROADMAP.md`** — Public roadmap with planned features and good-first-issue suggestions
- **`CHANGELOG.md`** — Release history (this file)
- **Platform badges** — Linux, macOS, Windows support badges in README
- **Benchmark section** — Performance benchmarks table in README
- **Editor plugin fixes** — Neovim and VS Code now pass filename for `fix` command

### Changed
- **`Fix.file`** is now optional (`Option<String>`) — works with both `bnn fix` and `bnn fix <file>`
- **`Review.file`** is now optional (`Option<String>`) — works with both `bnn review` and `bnn review <file>`
- **`Explain`, `Refactor`, `Test`** — Wired from stubs to real inference engine calls via `run_inference_on_file()`
- **CLI version** updated from `0.1.0` to `0.1.3`
- **Model loading** — Now compatible with both `download_model.py` (saves `model.onnx`) and `download_model.sh` (saves named files)
- **README** — Expanded about section, updated commands table, fixed project structure, added references
- **`inference/bnn.rs`** — Integrated validation from `model.rs` draft, added tracing logs
- **`lib.rs`** — Exported `inference` module for integration tests
- **Integration tests** — Updated version check and parser symbol count

### Removed
- **Nested `bnn-code/bnn-code/` duplicate crate** — Was a full standalone copy with own `.git/`
- **Orphan `model.rs`** — Draft PR file that never compiled (wrong `ort` API); validation logic ported to `bnn.rs`
- **0-byte model placeholders** — Removed empty `.onnx` stubs that blocked downloads

### Fixed
- **Corrupted `indexer/mod.rs`** — Fixed the `database()` accessor method that had duplicate syntax
- **Model path mismatch** — Shell script saves `codeberta-small.onnx` but code expected `model.onnx`
- **Editor plugin `fix` command** — Both Neovim and VS Code now pass the file path
- **Graceful error handling** — Clear messages for missing models, empty git diffs, and missing files

## [0.1.2] — 2026-06-20

### Added
- `rogue` subcommand with security, code smell, AI output, and user behavior detectors
- VS Code extension (9 commands, context menu, keybindings)
- Neovim plugin (9 commands, configurable keymaps, floating window output)
- `.deb` package builder (`build-deb.sh`)
- `EDITOR_INTEGRATION.md` documentation
- Cross-platform release builds (Linux, macOS, Windows, ARM64)

### Changed
- Upgraded to clap 4.x for CLI parsing
- Improved codebase indexing performance

### Fixed
- Tokenizer path resolution in ONNX inference
- SQLite FTS5 search edge cases with special characters

## [0.1.1] — 2026-06-15

### Added
- AST-aware codebase indexer with tree-sitter (10 languages)
- SQLite-backed storage with FTS5 full-text search
- ONNX Runtime inference with CodeBERTa-small model
- Model download scripts (shell + Python/HuggingFace)
- Terminal UI with Ratatui + Crossterm
- Project initialization command (`init`)
- CI/CD via GitHub Actions

### Fixed
- Cross-compilation targets for macOS ARM64
- Static binary builds for Linux

## [0.1.0] — 2026-06-10

### Added
- Initial release
- Basic CLI structure with clap
- Codebase scanning and chunking
- ONNX integration prototype
- REPL mode for interactive queries
