# BNN Code — React Web Application

**Terminal-native AI coding agent powered by Binarized Neural Networks**

BNN Code is an interactive, browser-based coding agent and developer workstation rewritten in React and TypeScript. It ports the full capabilities of BNN Code into a modern web application:

- 📟 **Interactive Terminal REPL** — Ratatui & Crossterm-inspired command line console supporting all BNN commands (`explain`, `refactor`, `test`, `fix`, `commit`, `review`, `document`, `pi`, `rogue`, `init`, `stats`).
- 🔍 **Rogue Anomaly Detection Suite** — Full-spectrum security and quality scanner inspecting rogue processes, hardcoded credentials, AST code smells, unsafe AI output (SQL injection, jailbreaks, PII), and anomalous user behavior.
- 🗂️ **AST-aware Indexer & Search** — Semantic code chunking with tree-sitter symbol detection (functions, structs, classes) and fast BM25/FTS keyword retrieval.
- 🥧 **Arbitrary-Precision π Lab** — High-precision computation of π using Chudnovsky and Ramanujan binary splitting formulas with BigInt arithmetic and microsecond benchmarking.
- ⚙️ **Model & Config Hub** — Hyperparameter tuning (temperature, top_k, max_tokens) and live `.bnn/config.json` synchronization.
- 📦 **Editor & Ecosystem Integrations** — Guides and specifications for VS Code extension, Neovim Lua plugin, and Debian packages.

## Running Locally

```bash
npm install
npm run dev
```

App listens on port 3000 (`http://localhost:3000`).

## Build

```bash
npm run build
```

## License

MIT License — Copyright (c) 2026 Mr. Yotsawarit Pudpong
