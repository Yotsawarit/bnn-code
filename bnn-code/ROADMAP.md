# BNN Code Roadmap

**Last updated: 2026-06-28**

## ✅ Done (v0.1.x)

- [x] CLI with clap (explain, refactor, test, fix, commit, review, document, init, rogue)
- [x] AST-aware codebase indexing (10 languages)
- [x] SQLite FTS5 full-text search
- [x] ONNX Runtime inference engine
- [x] CodeBERTa-small model support
- [x] Model file validation (exists → size → ONNX magic bytes)
- [x] VS Code extension (9 commands + context menu + keybindings)
- [x] Neovim plugin (9 commands + keymaps + floating window output)
- [x] Rogue detection engine (security, code smells, AI output, user behavior)
- [x] Cross-platform builds (Linux, macOS, Windows, ARM64)
- [x] `.deb` package for Debian/Deepin/Ubuntu
- [x] CITATION.cff + llms.txt

## 🚧 In Progress

- [ ] Generative model support (swap from masked LM to SmolLM2/TinyLlama)
- [ ] Token-by-token streaming output in TUI and editor plugins
- [ ] GitHub Actions CI/CD (build, test, clippy, release artifacts)

## 🗓️ Planned (v0.2.0)

### High Priority

- [ ] **Generative text output** — Replace CodeBERTa (masked LM) with a small generative model (SmolLM2-135M) for coherent code explanations, docs, and commit messages
- [ ] **Streaming output** — Wire `StreamHandler` into all CLI commands and editor plugins for real-time token display
- [ ] **CI/CD pipeline** — GitHub Actions: `cargo build`, `cargo test`, `cargo clippy`, auto-publish to crates.io, upload release artifacts

### Medium Priority

- [ ] **`bnn leaderboard` command** — Fetch and display rankings from [The Agentic Leaderboard](https://www.theagenticleaderboard.com)
- [ ] **Self-benchmark mode** — Score BNN Code against leaderboard metrics (reliability, tool selection, iteration, efficiency, mindshare)
- [ ] **Multi-model support** — Select between downloaded models via `--model` flag
- [ ] **Session memory** — Persist conversation context across REPL sessions

### Nice to Have

- [ ] **Language server protocol (LSP)** — Expose code intelligence via LSP for any editor
- [ ] **MCP server support** — Allow BNN Code to act as an MCP tool provider
- [ ] **More languages** — Add tree-sitter grammars for additional languages (PHP, C#, R, etc.)
- [ ] **GUI dashboard** — Web-based dashboard for visualizing codebase metrics and rogue reports
- [ ] **Plugin system** — Allow third-party skills/plugins via WASM or Lua

## 🔍 Good First Issues

Looking to contribute? These are well-scoped tasks ideal for new contributors:

1. **Add platform badges to README** — Easy: add shields.io badges for Linux/macOS/Windows support
2. **Add `bnn leaderboard` subcommand** — Medium: fetch JSON from theagenticleaderboard.com and display rankings
3. **Fix `--model` flag wiring** — Medium: the CLI accepts `--model` but it's not passed to the inference engine
4. **Add integration tests for CLI commands** — Medium: test that `explain`, `fix`, etc. parse correctly end-to-end
5. **Create demo GIF** — Easy: record a terminal session with asciinema and convert to GIF

---

*This roadmap is a living document. Priorities may shift based on community feedback and contributions.*
