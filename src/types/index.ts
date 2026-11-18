export type ChunkType = 'function' | 'struct' | 'class' | 'module' | 'impl' | 'interface' | 'constant' | 'other';

export interface CodeChunk {
  id: string;
  filePath: string;
  content: string;
  startLine: number;
  endLine: number;
  symbolName?: string;
  chunkType: ChunkType;
  score?: number;
}

export interface CodeFile {
  path: string;
  language: string;
  content: string;
  sizeBytes: number;
  lastModified?: string;
}

export type Severity = 'Critical' | 'High' | 'Medium' | 'Low' | 'Info';

export type RogueCategory = 'security' | 'code_smell' | 'ai_rogue' | 'user_behavior';

export interface Finding {
  id: string;
  severity: Severity;
  category: RogueCategory;
  title: string;
  description: string;
  confidence: number;
  location?: string;
  recommendation: string;
  timestamp?: string;
}

export interface RogueReport {
  totalFindings: number;
  bySeverity: Record<Severity, number>;
  byCategory: Record<string, number>;
  findings: Finding[];
  timestamp: string;
}

export interface ModelInfo {
  id: string;
  name: string;
  description: string;
  sizeMb: number;
  quantization: string;
  maxTokens: number;
  temperature: number;
  topK: number;
  architecture: string;
  isDefault?: boolean;
}

export interface BnnConfig {
  version: string;
  model: {
    path: string;
    max_tokens: number;
    temperature: number;
    top_k: number;
  };
  indexing: {
    max_file_size_kb: number;
    exclude_dirs: string[];
    include_extensions: string[];
  };
  ui: {
    theme: 'dark' | 'matrix' | 'cyberpunk' | 'slate';
    show_context: boolean;
    stream_output: boolean;
  };
}

export interface TerminalLog {
  id: string;
  timestamp: string;
  command: string;
  output: string;
  isError?: boolean;
  isStreaming?: boolean;
  category?: 'command' | 'response' | 'info' | 'system';
}

export interface PiBenchmarkResult {
  digits: number;
  algorithm: 'chudnovsky' | 'ramanujan';
  piString: string;
  durationMs: number;
  termsCalculated: number;
  memoryEstimatedKb: number;
  isAccurate: boolean;
  timestamp: string;
}
