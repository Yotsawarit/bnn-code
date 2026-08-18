# BNN Code - ฟีเจอร์หลัก (Main Features)

## Overview
Terminal-native AI coding agent powered by Binarized Neural Networks

---

## Features by Category

### 🧠 Code Understanding & AI Assistance
| Feature | Command | Description |
|---------|---------|-------------|
| Natural Language Query | `bnn-code "query"` | ถามคำถามได้แบบภาษาธรรมชาติ |
| Explain Code | `bnn-code explain <file>` | อธิบายโค้ด: วัตถุประสงค์ สถาปัตยกรรม ฟังก์ชันหลัก |
| Refactor Suggestions | `bnn-code refactor <file>` | เสนอการปรับปรุงโค้ด (refactoring) |
| Generate Tests | `bnn-code test <file>` | สร้าง unit tests อัตโนมัติ |
| Fix Bugs | `bnn-code fix [file]` | แก้ไขบั๊ก/ข้อผิดพลาด; สแกนทั้งโปรเจกต์ถ้าไม่ระบุไฟล์ |
| Code Review | `bnn-code review [file]` | Review โค้ดหา bugs, security, performance |
| Auto Documentation | `bnn-code document <file>` | สร้าง docstrings และเอกสารอัตโนมัติ |
| Commit Message | `bnn-code commit` | สร้างข้อความ commit จาก `git diff --cached` |

### 🌐 Multi-language Support
รองรับ **10 ภาษา** ด้วย tree-sitter grammars:
- Python, JavaScript, TypeScript
- Rust, Go, Java
- C++, Ruby, Swift, Kotlin

### 🔍 AST-aware Indexing
- ใช้ **tree-sitter** สำหรับ parsing จริง (ไม่ใช่ regex)
- Smart chunking ตามโครงสร้าง AST (functions, classes, modules)
- รองรับ symbol extraction และ cross-references

### 🖥️ Terminal-native UI
- สร้างด้วย **Ratatui + Crossterm**
- ใช้งานได้ใน terminal ทุกตัว ไม่ต้อง IDE
- Streaming output, syntax highlighting, interactive REPL

### ⚡ BNN Inference (On-device AI)
- **ONNX Runtime** สำหรับ inference
- รองรับ **SSE4.2** (รันบน CPU ทุกตัว)
- โมเดล: CodeBERTa-small และโมเดล BNN อื่นๆ
- Zero external API calls - privacy first

### 💾 Storage & Retrieval
- **SQLite** + **FTS5** full-text search
- Vector similarity search (optional)
- Local-first, ไม่มีข้อมูลออกนอกเครื่อง

### 🔌 Editor Integrations
| Editor | Installation | Key Features |
|--------|--------------|--------------|
| VS Code | `bash scripts/install-vscode.sh` | Index, Search, Query, Inline actions |
| Neovim | `bash scripts/install-neovim.sh` | `<Leader>bn`, `<Leader>bi`, `<Leader>bs` |

### 🛡️ Rogue Detection (Anomaly Detection)
```bash
bnn-code rogue              # รันทั้งหมด
bnn-code rogue -c security  # Security فقط
bnn-code rogue -c code      # Code smells
bnn-code rogue -c ai        # AI output anomalies
bnn-code rogue -c user      # User behavior anomalies
```
| Category | Detects |
|----------|---------|
| Security | Suspicious processes, permission anomalies, network anomalies, SSH keys |
| Code Smells | Hardcoded secrets, TODO/FIXME, unsafe code, nesting depth |
| AI Rogue | Jailbreaks, SQL injection, harmful content, prompt injection |
| User Behavior | Credential access, exfiltration, privilege escalation, dangerous commands |

### 🔢 Mathematical Computation
```bash
bnn-code pi --digits 500     # Chudnovsky algorithm (default)
bnn-code pi --digits 100 --algorithm ramanujan  # Ramanujan algorithm
```
- Arbitrary precision (ใช้ `num-bigint`)
- Chudnovsky & Ramanujan series

---

## Quick Reference

### Commands
```bash
bnn-code [query]              # One-shot query หรือ REPL
bnn-code explain <file>       # อธิบายไฟล์
bnn-code refactor <file>      # เสนอ refactoring
bnn-code test <file>          # สร้าง tests
bnn-code fix [file]           # แก้บั๊ก
bnn-code commit               # Commit message
bnn-code review [file]        # Code review
bnn-code document <file>      # สร้าง docs
bnn-code init                 # Initialize project
bnn-code pi [--digits N]      # คำนวณ π
bnn-code rogue [-c cat]       # Anomaly detection
bnn-code help                 # Help
```

### Options
| Flag | Description |
|------|-------------|
| `-p, --path <PATH>` | Path to codebase (default: `.`) |
| `-m, --model <MODEL>` | BNN model (default: `default`) |
| `-v, --verbose` | Verbose logging |
| `--no-stream` | Disable streaming |
| `-j, --json` | JSON output (rogue) |

---

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

---

## Project Structure
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
├── rogue/       # Anomaly detection
│   ├── security.rs
│   ├── code_smell.rs
│   ├── ai_output.rs
│   ├── user_behavior.rs
│   └── mod.rs
├── math/        # π computation
│   └── pi.rs
├── utils/       # Shared utilities
│   ├── cache.rs
│   └── config.rs
└── main.rs
```

---

## Supported Platforms

| Platform | Binary |
|----------|--------|
| Linux x86_64 (glibc) | `bnn-code-linux-amd64.tar.gz` |
| Linux x86_64 (static) | `bnn-code-linux-amd64-static.tar.gz` |
| macOS x86_64 | `bnn-code-macos-amd64.tar.gz` |
| macOS ARM64 | `bnn-code-macos-arm64.tar.gz` |
| Windows x86_64 | `bnn-code-windows-amd64.exe.zip` |
| Deepin/Debian/Ubuntu | `bnn-code_0.1.1_amd64.deb` |

---

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

---

## License
MIT License — see [LICENSE](LICENSE).

Copyright (c) 2026 Mr. Yotsawarit Pudpong