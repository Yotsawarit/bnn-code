# BNN Code

**Terminal-native AI coding agent powered by Binarized Neural Networks**

[![Crates.io](https://img.shields.io/crates/v/bnn-code)](https://crates.io/crates/bnn-code)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![CI/CD](https://github.com/Yotsawarit/bnn-code/actions/workflows/release.yml/badge.svg)](https://github.com/Yotsawarit/bnn-code/actions)

BNN Code is a terminal-based AI coding assistant that helps you understand, refactor, and test your codebase using Binarized Neural Networks (BNNs). It runs entirely in your terminal with zero IDE lock-in.

## Features

- **Code Understanding** — Index your codebase and ask questions in natural language
- **Multi-language Support** — Python, JavaScript, TypeScript, Rust, Go, Java, C++, Ruby, Swift, Kotlin
- **AST-aware Indexing** — Smart chunking using tree-sitter grammars
- **Terminal-native UI** — Built with Ratatui + Crossterm, no IDE required
- **BNN Inference** — On-device AI with ONNX Runtime (SSE4.2 compatible)
- **SQLite-backed** — Fast local search and retrieval
- **Editor Integrations** — VS Code extension + Neovim plugin included

## Installation

### From GitHub Releases (recommended)

Download the latest binary for your platform from the [Releases page](https://github.com/Yotsawarit/bnn-code/releases):

```bash
# Linux (glibc)
tar -xzf bnn-code-linux-amd64.tar.gz
sudo mv bnn-code /usr/local/bin/

# macOS (Intel)
tar -xzf bnn-code-macos-amd64.tar.gz
sudo mv bnn-code /usr/local/bin/

# macOS (Apple Silicon)
tar -xzf bnn-code-macos-arm64.tar.gz
sudo mv bnn-code /usr/local/bin/

# Windows
unzip bnn-code-windows-amd64.exe.zip
# Move bnn-code.exe to a directory in your PATH
```

### From crates.io

```bash
cargo install bnn-code
```

### Deepin / Debian / Ubuntu

Download the `.deb` package from the [Releases page](https://github.com/Yotsawarit/bnn-code/releases):

```bash
sudo dpkg -i bnn-code_0.1.1_amd64.deb
```

### Build from Source

```bash
git clone https://github.com/Yotsawarit/bnn-code.git
cd bnn-code
cargo build --release
./target/release/bnn-code --help
```

## Quick Start

```bash
# Index your codebase
bnn-code index --path /path/to/your/project

# Search for code patterns
bnn-code search "function that handles authentication"

# Start interactive TUI
bnn-code

# Get help
bnn-code --help
```

## Editor Integrations

BNN Code comes with first-class editor support.

### VS Code

1. Install the extension from VS Code Marketplace (coming soon) or install manually:

```bash
bash scripts/install-vscode.sh
```

2. Open command palette (`Ctrl+Shift+P`) and run:
   - `BNN: Index Workspace`
   - `BNN: Search Code`
   - `BNN: Query with AI`

### Neovim

```bash
bash scripts/install-neovim.sh
```

Then use the following keymaps:
- `<Leader>bn` — Open BNN Code
- `<Leader>bi` — Index workspace
- `<Leader>bs` — Search code
- See `:help bnn-code` for full documentation

## Download Models

```bash
# Download CodeBERTa-small model (recommended for most users)
bash scripts/download_model.sh

# Or use the Python script for more options
python3 scripts/download_model.py --model codeberta-small
```

## Commands

| Command | Description |
|---------|-------------|
| `bnn-code index` | Index a codebase |
| `bnn-code search` | Search indexed code |
| `bnn-code query` | Query with AI inference |
| `bnn-code` | Launch interactive TUI |
| `bnn-code --help` | Show help |

## Architecture

```
┌─────────────────────────────────────┐
│           CLI (clap)                │
├─────────────────────────────────────┤
│  ┌─────────┐  ┌──────────┐         │
│  │ Indexer │  │Retrieval │         │
│  │ (AST)   │  │ (SQLite) │         │
│  └────┬────┘  └────┬─────┘         │
│       │            │               │
│  ┌────▼────────────▼─────┐         │
│  │   Inference Engine    │         │
│  │  (ONNX Runtime + BNN) │         │
│  └───────────────────────┘         │
│  ┌───────────────────────┐         │
│  │   Terminal UI (TUI)   │         │
│  │  (Ratatui + Crossterm)│         │
│  └───────────────────────┘         │
├─────────────────────────────────────┤
│  VS Code Extension │ Neovim Plugin │
└─────────────────────────────────────┘
```

## Supported Platforms

| Platform | Binary |
|----------|--------|
| Linux x86_64 (glibc) | `bnn-code-linux-amd64.tar.gz` |
| Linux x86_64 (static) | `bnn-code-linux-amd64-static.tar.gz` |
| macOS x86_64 | `bnn-code-macos-amd64.tar.gz` |
| macOS ARM64 | `bnn-code-macos-arm64.tar.gz` |
| Windows x86_64 | `bnn-code-windows-amd64.exe.zip` |
| Deepin/Debian/Ubuntu | `bnn-code_0.1.1_amd64.deb` |

## Development

```bash
# Check
cargo check

# Run tests
cargo test

# Build release
cargo build --release

# Build static binary
RUSTFLAGS="-C target-feature=+crt-static" cargo build --release
```

### Project Structure

```
src/
├── cli/         # CLI argument parsing (clap)
├── indexer/     # Code indexing with tree-sitter AST
│   ├── chunker.rs
│   ├── database.rs
│   └── parser.rs
├── inference/   # ONNX Runtime + BNN inference
│   └── model.rs
├── retrieval/   # SQLite search and retrieval
│   └── search.rs
├── ui/          # Terminal UI (Ratatui)
│   ├── mod.rs
│   ├── streaming.rs
│   └── terminal.rs
├── utils/       # Shared utilities
│   ├── cache.rs
│   └── config.rs
└── main.rs
```

## License

MIT License — see [LICENSE](LICENSE).

Copyright (c) 2026 Mr. Yotsawarit Pudpong
