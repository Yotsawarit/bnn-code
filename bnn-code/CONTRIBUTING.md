# Contributing to BNN Code

First off, thank you for considering contributing to BNN Code! 🎉

## Code of Conduct

This project adheres to the [Rust Code of Conduct](https://www.rust-lang.org/policies/code-of-conduct). By participating, you are expected to uphold this code.

## How Can I Contribute?

### 🐛 Reporting Bugs

1. Check the [issue tracker](https://github.com/Yotsawarit/bnn-code/issues) for existing reports
2. If none exists, [open a new issue](https://github.com/Yotsawarit/bnn-code/issues/new)
3. Include:
   - Your OS and Rust version (`rustc --version`)
   - Steps to reproduce
   - Expected vs actual behavior
   - Full error output (if any)

### 💡 Suggesting Features

1. [Open a feature request](https://github.com/Yotsawarit/bnn-code/issues/new)
2. Describe the problem you're solving, not just the solution
3. Include examples of how the feature would work

### 🛠️ Pull Requests

1. **Fork** the repo and create your branch from `main`
2. **Test** your changes: `cargo test`
3. **Lint** your code: `cargo clippy`
4. **Format** your code: `cargo fmt`
5. **Write tests** for new functionality
6. **Update documentation** (README, `llms.txt`, or inline docs)
7. Open the PR with a clear title and description

### 📝 Style Guide

- Follow standard Rust idioms and [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `anyhow::Result` for fallible functions
- Use `tracing` for logging (not `println!` or `eprintln!`)
- Prefer `Option`/`Result` over panics
- Document public APIs with doc comments (`///`)

### 🧪 Testing

- Unit tests go next to the code they test (in-module `#[cfg(test)]`)
- Integration tests go in `tests/`
- Run the full suite: `cargo test`
- New features should include tests

### 📋 Good First Issues

Look for issues tagged [`good first issue`](https://github.com/Yotsawarit/bnn-code/labels/good%20first%20issue) on GitHub. These are smaller, well-scoped tasks ideal for new contributors.

## Development Setup

```bash
# Clone your fork
git clone https://github.com/your-username/bnn-code.git
cd bnn-code

# Build
cargo build

# Download a model
bash scripts/download_model.sh --auto

# Run tests
cargo test

# Run the CLI
./target/debug/bnn-code --help
```

## Project Structure

See [README.md](README.md#project-structure) for the full directory layout.

## Questions?

Open a [discussion](https://github.com/Yotsawarit/bnn-code/discussions) or reach out to the maintainer.

Thank you for helping make BNN Code better! 🚀
