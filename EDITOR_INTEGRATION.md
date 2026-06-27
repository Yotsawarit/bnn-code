# 🔌 BNN Code Editor Integration

BNN Code integrates with your favorite editor, so you can explain, refactor,
and generate code without leaving your workflow.

## 📋 Supported Editors

| Editor | Status | Features |
|--------|--------|----------|
| **VS Code** | ✅ Released | 9 commands, context menu, keybindings, status bar |
| **Neovim** | ✅ Released | 9 commands, floating window, visual mode, keymaps |
| **Vim** | 🔜 Planned | |
| **JetBrains** | 🔜 Planned | |
| **Emacs** | 🔜 Planned | |

---

## 🖥️ VS Code Extension

### Installation

```bash
# Option 1: Install script (recommended)
chmod +x scripts/install-vscode.sh
./scripts/install-vscode.sh

# Option 2: Manual build
cd vscode-extension
npm install
npm run compile
npm run package
code --install-extension bnn-code-0.3.0.vsix

# Option 3: From VS Code Marketplace (coming soon)
# Open Extensions panel → Search "BNN Code" → Install
```

### Features

| Feature | Description |
|---------|-------------|
| **Explain** | Explain selected code in plain English |
| **Refactor** | Suggest refactoring improvements |
| **Test** | Generate unit tests for selected code |
| **Fix** | Fix errors in your code |
| **Commit** | Generate meaningful commit messages |
| **Review** | Review staged changes |
| **Document** | Generate documentation |
| **Query** | Ask any question about your codebase |
| **Terminal** | Open interactive BNN terminal |

### Usage

**Context Menu:** Right-click selected code → BNN → Explain / Refactor / Test / Document

**Keyboard Shortcuts:**

| Shortcut | Action |
|----------|--------|
| `Ctrl+Shift+E` (Cmd+Shift+E) | Explain selection |
| `Ctrl+Shift+R` (Cmd+Shift+R) | Refactor selection |
| `Ctrl+Shift+T` (Cmd+Shift+T) | Generate tests |
| `Ctrl+Shift+B` (Cmd+Shift+B) | Ask question |
| `Ctrl+Shift+\`` (Cmd+Shift+\`) | Open terminal |

**Command Palette:** `Ctrl+Shift+P` → type `BNN:`

### Configuration

Settings → Extensions → BNN Code:

| Setting | Default | Description |
|---------|---------|-------------|
| `bnn.binaryPath` | `bnn` | Path to BNN Code binary |
| `bnn.codebasePath` | `.` | Path to codebase to index |
| `bnn.showOutput` | `true` | Show output panel on response |
| `bnn.streaming` | `true` | Enable streaming output |

---

## 🎮 Neovim Plugin

### Installation

#### Using [lazy.nvim](https://github.com/folke/lazy.nvim)

```lua
{
    'bnn-code/nvim-plugin',
    config = function()
        require('bnn').setup({
            binary_path = "bnn",
            codebase_path = ".",
        })
    end,
}
```

#### Using [packer.nvim](https://github.com/wbthomason/packer.nvim)

```lua
use {
    'bnn-code/nvim-plugin',
    config = function()
        require('bnn').setup()
    end,
}
```

#### Using [vim-plug](https://github.com/junegunn/vim-plug)

```vim
Plug 'bnn-code/nvim-plugin'

lua << EOF
require('bnn').setup()
EOF
```

#### Manual Install

```bash
chmod +x scripts/install-neovim.sh
./scripts/install-neovim.sh
```

### Commands

| Command | Description |
|---------|-------------|
| `:BnnExplain` | Explain current file |
| `:BnnRefactor` | Suggest refactoring |
| `:BnnTest` | Generate tests |
| `:BnnFix` | Fix errors |
| `:BnnCommit` | Generate commit message |
| `:BnnReview` | Review changes |
| `:BnnDocument` | Generate documentation |
| `:BnnQuery {query}` | Ask a question |
| `:BnnTerminal` | Open interactive terminal |

### Keymaps

| Mode | Key | Action |
|------|-----|--------|
| Visual | `<leader>be` | Explain selection |
| Visual | `<leader>br` | Refactor selection |
| Visual | `<leader>bt` | Generate tests |
| Visual | `<leader>bd` | Generate docs |
| Normal | `<leader>bf` | Fix errors |
| Normal | `<leader>bc` | Generate commit message |
| Normal | `<leader>brv` | Review changes |
| Normal | `<leader>bq` | Ask question |
| Normal | `<leader>bT` | Open terminal |

### Configuration

```lua
require('bnn').setup({
    binary_path = "bnn",        -- Path to BNN binary
    codebase_path = ".",        -- Codebase to index
    show_output = true,         -- Show output in floating window
    streaming = true,           -- Enable streaming output
})
```

---

## 🚀 Quick Start

### VS Code

1. Open any code file
2. Select a block of code
3. **Right-click** → **BNN** → **Explain**
4. See the explanation in the output panel

Or press `Ctrl+Shift+B` and type: `explain the authentication flow`

### Neovim

1. Open any code file
2. Select a function with `vib`
3. Press `<leader>be` to explain

Or run `:BnnQuery how does this parser work?`

---

## 📦 Project Structure

```
📁 vscode-extension/
├── package.json          # Extension manifest
├── tsconfig.json         # TypeScript config
├── src/
│   └── extension.ts      # Main extension logic
└── out/                  # Compiled JavaScript

📁 nvim-plugin/
├── lua/
│   └── bnn/
│       └── init.lua      # Main plugin logic
├── plugin/
│   └── bnn.vim           # Plugin entry point
└── doc/
    └── bnn.txt           # Help documentation

📁 scripts/
├── install-vscode.sh     # VS Code extension installer
└── install-neovim.sh     # Neovim plugin installer
```

## 🔧 Development

### VS Code Extension

```bash
cd vscode-extension
npm install
npm run watch        # Watch mode for development
# Press F5 in VS Code to launch Extension Development Host
```

### Neovim Plugin

```bash
# Symlink for development
ln -sf "$(pwd)/nvim-plugin" ~/.local/share/nvim/lazy/bnn.nvim
```

## 📝 License

MIT - See LICENSE file for details.
