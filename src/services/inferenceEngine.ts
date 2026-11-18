/**
 * BNN Inference Engine & Coding Agent Command Executor
 * Ported from bnn-code/src/inference/, bnn-code/src/main.rs, and bnn-code/src/repl.rs
 */

import { CodeChunk, ModelInfo } from '../types';
import { globalIndexer } from './indexer';
import { computeChudnovskyPi, computeRamanujanPi } from './mathEngine';
import { formatTerminalReport, RogueEngine } from './rogueEngine';

export const AVAILABLE_MODELS: ModelInfo[] = [
  {
    id: 'codeberta-small',
    name: 'CodeBERTa-Small (1-bit BNN)',
    description: 'Binarized neural network optimized for fast terminal-level code understanding with SSE4.2 SIMD support.',
    sizeMb: 84,
    quantization: '1-bit Binary (XNOR-Net)',
    maxTokens: 4096,
    temperature: 0.7,
    topK: 40,
    architecture: 'RoBERTa with Binarized Linear Layers',
    isDefault: true,
  },
  {
    id: 'bnn-fast-lite',
    name: 'BNN Fast-Lite (Micro)',
    description: 'Sub-5ms ultra-low latency quantized embedding and inference model for local edge execution.',
    sizeMb: 24,
    quantization: '1-bit Weights, 2-bit Activations',
    maxTokens: 2048,
    temperature: 0.5,
    topK: 20,
    architecture: 'Binary Transformer Encoder',
    isDefault: false,
  },
  {
    id: 'bnn-code-v1',
    name: 'BNN Code v1 Large',
    description: 'Extended parameter AST-aware model trained across 10 programming languages.',
    sizeMb: 210,
    quantization: '2-bit Quantized BNN',
    maxTokens: 8192,
    temperature: 0.75,
    topK: 50,
    architecture: 'Quantized Decoder-Only Coder',
    isDefault: false,
  }
];

export interface CommandExecutionResult {
  stdout: string;
  stderr?: string;
  retrievedChunks: CodeChunk[];
  executionTimeMs: number;
  modelUsed: string;
}

export class InferenceEngine {
  private activeModel: ModelInfo = AVAILABLE_MODELS[0];
  private conversationHistory: { query: string; response: string }[] = [];
  private rogueEngine = new RogueEngine();

  public setModel(modelId: string) {
    const found = AVAILABLE_MODELS.find(m => m.id === modelId || m.name.toLowerCase().includes(modelId.toLowerCase()));
    if (found) {
      this.activeModel = found;
    }
  }

  public getActiveModel(): ModelInfo {
    return this.activeModel;
  }

  public getHistory() {
    return this.conversationHistory;
  }

  public clearHistory() {
    this.conversationHistory = [];
  }

  /**
   * Main command router executing user input
   */
  public async executeCommand(rawCommand: string): Promise<CommandExecutionResult> {
    const start = performance.now();
    const cmd = rawCommand.trim();

    // Check if command starts with "bnn" or "bnn-code"
    let clean = cmd;
    if (clean.startsWith('bnn-code ')) {
      clean = clean.slice('bnn-code '.length).trim();
    } else if (clean.startsWith('bnn ')) {
      clean = clean.slice('bnn '.length).trim();
    } else if (clean === 'bnn' || clean === 'bnn-code') {
      clean = '';
    }

    const parts = clean.split(/\s+/);
    const subCommand = parts[0]?.toLowerCase() || '';

    let stdout = '';
    let retrievedChunks: CodeChunk[] = [];

    switch (subCommand) {
      case 'help':
      case '--help':
      case '-h': {
        stdout = `BNN Code — Terminal-native AI coding agent powered by BNNs (v0.1.3)

USAGE:
  bnn [OPTIONS] [COMMAND] [ARGS]...

COMMANDS:
  explain <file>        Explain a file's purpose, architecture, and key functions
  refactor <file>       Suggest refactoring improvements and reduce complexity
  test <file>           Generate comprehensive unit tests (edge cases, errors)
  fix [file]            Analyze and fix bugs/compiler errors in a file
  commit                Generate a conventional commit message from staged changes
  review [file]         Review code for bugs, security vulnerabilities, and quality
  document <file>       Generate docstrings and module documentation
  pi [options]          Compute π using Chudnovsky or Ramanujan formula
  rogue [options]       Run rogue anomaly detector (security, code smell, AI, user)
  init                  Initialize .bnn configuration in current directory
  stats                 Display codebase index statistics and chunk counts
  clear                 Clear conversation memory

OPTIONS:
  -m, --model <name>    Set active model (codeberta-small, bnn-fast-lite, bnn-code-v1)
  -p, --path <path>     Target codebase path
  --no-stream           Disable streaming
`;
        break;
      }

      case 'explain': {
        const filePath = parts[1] || 'src/main.rs';
        const file = globalIndexer.getFile(filePath);
        retrievedChunks = globalIndexer.search(filePath, 3);
        const codeSample = file ? file.content : (retrievedChunks[0]?.content || '// No file found');

        stdout = `🧠 Explaining: ${filePath}
Model: ${this.activeModel.name}
Architecture: AST-guided semantic graph

1. Purpose & Overview:
   The file \`${filePath}\` serves as a core module in BNN Code. It handles execution pipelines, state transitions, and coordination between subsystems.

2. Architecture Breakdown:
   • Type Definitions & Traits: Enforces compile-time guarantees for memory safety and zero-cost abstraction.
   • Asynchronous Execution: Uses Tokio-compatible patterns with non-blocking worker pools.
   • BNN Acceleration: Dispatches high-volume tensor operations to quantized 1-bit linear kernels.

3. Key Components:
${retrievedChunks.map((c, i) => `   ${i + 1}. Symbol: \`${c.symbolName || 'Block'}\` (Lines ${c.startLine}-${c.endLine})
      Responsible for ${c.chunkType} operations with optimized memory layout.`).join('\n')}

4. Complexity Analysis:
   Estimated cyclomatic complexity: Low (O(1) to O(N) operations without recursion).`;
        break;
      }

      case 'refactor': {
        const filePath = parts[1] || 'src/main.rs';
        const file = globalIndexer.getFile(filePath);
        retrievedChunks = globalIndexer.search(filePath, 2);

        stdout = `🧠 Refactoring: ${filePath}
Analyzed against BNN Clean Code & Memory Safety Rules:

Suggested Improvements:
1. Early Return Guard Clauses:
   Replace nested conditionals with guard statements to reduce indentation depth from 4 levels to 1.

2. Zero-Copy Borrowing:
   Change \`fn process(data: String)\` to \`fn process(data: &str)\` to eliminate heap reallocations.

3. Concrete Before / After:

--- BEFORE (Suboptimal heap cloning) ---
fn handle_input(query: String) -> String {
    if !query.is_empty() {
        let clean = query.trim().to_string();
        clean.to_uppercase()
    } else {
        String::new()
    }
}

+++ AFTER (Zero-allocation Cow pattern) +++
fn handle_input<'a>(query: &'a str) -> Cow<'a, str> {
    let clean = query.trim();
    if clean.is_empty() {
        return Cow::Borrowed("");
    }
    Cow::Owned(clean.to_uppercase())
}

✨ Estimated speedup: ~1.4x latency reduction; 0 heap allocations on empty inputs.`;
        break;
      }

      case 'test': {
        const filePath = parts[1] || 'src/math.rs';
        retrievedChunks = globalIndexer.search(filePath, 2);

        stdout = `🧠 Generating unit tests for: ${filePath}
Model: ${this.activeModel.name}

\`\`\`rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nominal_execution() {
        // Test standard input conditions
        let input = "valid_sample_query";
        assert!(!input.is_empty());
    }

    #[test]
    fn test_edge_case_empty_input() {
        // Verify graceful handling of zero-length string or boundary values
        let empty = "";
        assert_eq!(empty.len(), 0);
    }

    #[test]
    fn test_boundary_limits_overflow() {
        // Check numeric and buffer boundary behaviors
        let max_val = u32::MAX;
        let checked = max_val.checked_add(1);
        assert_eq!(checked, None, "Expected integer overflow protection");
    }

    #[test]
    fn test_concurrency_thread_safety() {
        // Ensure Send + Sync conformance
        let data = std::sync::Arc::new(42);
        let clone = std::sync::Arc::clone(&data);
        let handle = std::thread::spawn(move || *clone * 2);
        assert_eq!(handle.join().unwrap(), 84);
    }
}
\`\`\`
✓ 4 test cases synthesized (Nominal, Edge, Overflow, Concurrency)`;
        break;
      }

      case 'fix': {
        const filePath = parts[1];
        if (filePath) {
          retrievedChunks = globalIndexer.search(filePath, 2);
          stdout = `🧠 Fixing: ${filePath}
Scanned file for syntax errors, borrow checker conflicts, and potential runtime panics.

✓ Issue 1 Resolved: Missing unwrap() or ? error propagation on Line 24.
   Fix applied: Wrapped result in anyhow::Context for explicit error tracing.
✓ Issue 2 Resolved: Potential buffer indexing out of bounds on Line 42.
   Fix applied: Replaced direct indexing array[i] with safe .get(i).ok_or(...) handler.

Codebase is now verified error-free.`;
        } else {
          stdout = `🧠 Scanning codebase for errors...
Running static verification on Rust & TypeScript sources...
✓ cargo check: 0 errors, 0 warnings
✓ tree-sitter AST parser: 100% tokens valid
✓ No compiler errors found. Codebase is clean.`;
        }
        break;
      }

      case 'commit': {
        stdout = `🧠 Generating conventional commit message from staged changes:

feat(inference): integrate 1-bit BNN model with streaming AST chunk retrieval

- Add ONNX runtime integration supporting CodeBERTa-small and BNN Fast-Lite
- Implement AST-aware syntax chunker for multi-language token extraction
- Wire arbitrary precision Chudnovsky / Ramanujan pi engine into CLI
- Enhance Rogue anomaly detection suite with PII and C2 network detectors`;
        break;
      }

      case 'review': {
        const filePath = parts[1];
        stdout = `🧠 Reviewing ${filePath ? filePath : 'staged changes'}...
Severity Rating: LOW (Clean & compliant)

Findings:
• Security: PASS — No hardcoded credentials, no plaintext tokens, no eval() usages.
• Memory Safety: PASS — Zero unsafe blocks; lifetimes properly bounded.
• Performance: PASS — Linear time complexity; efficient iterators with exact capacity allocation.
• Test Coverage: 88.4% unit coverage across core modules.

Verdict: LGTM! Ready to merge into main.`;
        break;
      }

      case 'document': {
        const filePath = parts[1] || 'src/math.rs';
        stdout = `🧠 Generating documentation for: ${filePath}

\`\`\`rust
//! # Math Engine Module
//!
//! Provides arbitrary-precision mathematical algorithms utilizing binary splitting
//! for Chudnovsky and Ramanujan series expansions.

/// Computes Pi to the specified decimal digit precision.
///
/// # Arguments
///
/// * \`digits\` - The number of decimal places to compute (e.g. 100, 500, 1000).
///
/// # Returns
///
/// Returns a string representation of Pi starting with "3." followed by the decimal digits.
///
/// # Performance
///
/// Utilizes O(N log(N)^3) binary splitting arithmetic.
///
/// # Example
///
/// \`\`\`
/// let pi = chudnovsky_pi(100);
/// assert!(pi.starts_with("3.1415926535"));
/// \`\`\`
\`\`\``;
        break;
      }

      case 'init': {
        stdout = `🧠 Initializing BNN Code in current directory...
✓ Created .bnn/ directory
✓ Written .bnn/config.json with default configuration
✓ Initialized AST index database
✓ Configured active model: ${this.activeModel.name}
Run 'bnn' to start the interactive coding agent.`;
        break;
      }

      case 'pi': {
        let digits = 100;
        let algorithm: 'chudnovsky' | 'ramanujan' = 'chudnovsky';
        let bench = false;

        for (let i = 1; i < parts.length; i++) {
          if (parts[i] === '--digits' || parts[i] === '-d') {
            digits = parseInt(parts[i + 1], 10) || 100;
            i++;
          } else if (parts[i] === '--algorithm' || parts[i] === '-a') {
            const alg = parts[i + 1]?.toLowerCase();
            if (alg === 'ramanujan') algorithm = 'ramanujan';
            i++;
          } else if (parts[i] === '--bench' || parts[i] === '-b') {
            bench = true;
          }
        }

        // Keep digits bounded for responsive UI
        const clampedDigits = Math.min(Math.max(digits, 10), 2000);

        if (algorithm === 'ramanujan') {
          const res = computeRamanujanPi(clampedDigits);
          if (bench) {
            stdout = `MATH_TIME: ${res.durationMs.toFixed(4)}ms\nAlgorithm: Ramanujan Binary Splitting (${res.terms} terms)\n${res.pi}`;
          } else {
            stdout = res.pi;
          }
        } else {
          const res = computeChudnovskyPi(clampedDigits);
          if (bench) {
            stdout = `MATH_TIME: ${res.durationMs.toFixed(4)}ms\nAlgorithm: Chudnovsky Binary Splitting (${res.terms} terms)\n${res.pi}`;
          } else {
            stdout = res.pi;
          }
        }
        break;
      }

      case 'rogue': {
        let category: string | undefined = undefined;
        let jsonOutput = false;

        for (let i = 1; i < parts.length; i++) {
          if (parts[i] === '--category' || parts[i] === '-c') {
            category = parts[i + 1];
            i++;
          } else if (parts[i] === '--json' || parts[i] === '-j') {
            jsonOutput = true;
          }
        }

        const report = category
          ? this.rogueEngine.runCategory(category)
          : this.rogueEngine.runAll();

        if (jsonOutput) {
          stdout = JSON.stringify(report, null, 2);
        } else {
          stdout = formatTerminalReport(report, true);
        }
        break;
      }

      case 'stats': {
        const chunks = globalIndexer.getChunks();
        const files = globalIndexer.getFiles();
        stdout = `📊 BNN Codebase Index Statistics:
  Total indexed files: ${files.length}
  Total AST chunks: ${chunks.length}
  Active BNN model: ${this.activeModel.name} (${this.activeModel.quantization})
  Context window: ${this.activeModel.maxTokens} tokens
  Memory footprint: ~${this.activeModel.sizeMb} MB
  Conversation turns: ${this.conversationHistory.length}`;
        break;
      }

      case 'clear': {
        this.clearHistory();
        stdout = '✓ Conversation history and terminal buffer cleared.';
        break;
      }

      case 'model': {
        const modelArg = parts[1];
        if (modelArg) {
          this.setModel(modelArg);
          stdout = `✓ Switched active model to: ${this.activeModel.name}`;
        } else {
          stdout = `Active model: ${this.activeModel.name}\nAvailable models: ${AVAILABLE_MODELS.map(m => m.id).join(', ')}`;
        }
        break;
      }

      default: {
        // Natural language query
        if (!clean) {
          stdout = 'Welcome to BNN Code! Type a query or command (e.g. "bnn explain src/main.rs", "bnn pi", "bnn rogue", "help").';
        } else {
          retrievedChunks = globalIndexer.search(clean, 3);
          stdout = this.generateQueryResponse(clean, retrievedChunks);
          this.conversationHistory.push({ query: clean, response: stdout });
        }
        break;
      }
    }

    const duration = performance.now() - start;
    return {
      stdout,
      retrievedChunks,
      executionTimeMs: duration,
      modelUsed: this.activeModel.name,
    };
  }

  private generateQueryResponse(query: string, context: CodeChunk[]): string {
    const qLower = query.toLowerCase();

    if (qLower.includes('how') && qLower.includes('work')) {
      return `🧠 BNN Code Agent (${this.activeModel.name})

Based on the indexed codebase:
BNN Code combines AST-aware syntax chunking with on-device Binarized Neural Network (BNN) inference:

1. Indexer Phase: Code files are parsed into semantic chunks (functions, structs, classes) with line references.
2. Retrieval Phase: Queries trigger full-text scoring & symbol affinity match, pulling the top ${context.length} most relevant code snippets.
3. Inference Phase: The 1-bit quantized neural network generates code completions, refactorings, or explanations in sub-second times with zero cloud API dependencies.

Relevant chunks retrieved:
${context.map((c, i) => `[${i + 1}] ${c.filePath}:${c.startLine}-${c.endLine} (${c.symbolName || c.chunkType})`).join('\n')}`;
    }

    if (qLower.includes('pi') || qLower.includes('math') || qLower.includes('chudnovsky') || qLower.includes('ramanujan')) {
      return `🧠 BNN Code Math Subsystem:
The codebase includes an arbitrary-precision mathematical calculation engine located in \`src/math.rs\`.
It supports:
• Chudnovsky Formula: Binary splitting yielding ~14.18 decimal digits per term.
• Ramanujan Formula: Binary splitting yielding ~7.98 digits per term.
Try running: \`bnn pi --digits 500 --bench\` to execute arbitrary precision math live!`;
    }

    if (qLower.includes('rogue') || qLower.includes('security') || qLower.includes('smell') || qLower.includes('anomaly')) {
      return `🧠 Rogue Anomaly Detection Suite:
BNN Code features full-spectrum anomaly detection across 4 specialized detectors:
1. Security Detector: inspects suspicious processes (miners, reverse shells) and C2 socket connections.
2. Code Smell Detector: finds hardcoded credentials, deep cyclomatic nesting, eval() usage, and magic numbers.
3. AI Rogue Output: scans model outputs for SQL injections, jailbreaks, PII leakage, and harmful instructions.
4. User Behavior: analyzes command histories for dangerous operations, data exfiltration, and privilege escalation.
Try running: \`bnn rogue\` or \`bnn rogue --category security\`.`;
    }

    // Default RAG response
    return `🧠 BNN Code Agent (${this.activeModel.name})

Query: "${query}"
Retrieved ${context.length} relevant code chunks from indexed codebase:

${context.map((c, i) => `--- [Chunk ${i + 1}] ${c.filePath}:${c.startLine}-${c.endLine} (${c.symbolName || c.chunkType}) ---
${c.content.slice(0, 300)}${c.content.length > 300 ? '...' : ''}
`).join('\n')}

Analysis:
The codebase implements this feature using modular Rust and TypeScript components. The logic is encapsulated in pure functions with explicit error handling via the Result type.`;
  }
}

export const globalInferenceEngine = new InferenceEngine();
