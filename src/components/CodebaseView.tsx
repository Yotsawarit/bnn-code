import React, { useState } from 'react';
import { 
  FolderTree, 
  FileCode, 
  Search, 
  Plus, 
  Trash2, 
  Layers, 
  RefreshCw, 
  Check, 
  Copy, 
  FileCheck,
  Code
} from 'lucide-react';
import { CodeChunk, CodeFile } from '../types';
import { globalIndexer } from '../services/indexer';

export const CodebaseView: React.FC = () => {
  const [files, setFiles] = useState<CodeFile[]>(() => globalIndexer.getFiles());
  const [selectedFile, setSelectedFile] = useState<CodeFile>(() => files[0]);
  const [searchQuery, setSearchQuery] = useState('');
  const [searchResults, setSearchResults] = useState<CodeChunk[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [chunks, setChunks] = useState<CodeChunk[]>(() => globalIndexer.getChunks());
  const [copiedCode, setCopiedCode] = useState(false);
  
  // New file modal
  const [showNewFileModal, setShowNewFileModal] = useState(false);
  const [newFilePath, setNewFilePath] = useState('src/auth.rs');
  const [newFileContent, setNewFileContent] = useState(`pub struct AuthSession {
    pub token: String,
    pub user_id: u64,
}

impl AuthSession {
    pub fn new(token: &str, user_id: u64) -> Self {
        Self {
            token: token.to_string(),
            user_id,
        }
    }

    pub fn validate(&self) -> bool {
        !self.token.is_empty()
    }
}`);

  const handleSearch = (q: string) => {
    setSearchQuery(q);
    if (!q.trim()) {
      setIsSearching(false);
      setSearchResults([]);
      return;
    }
    setIsSearching(true);
    const results = globalIndexer.search(q, 10);
    setSearchResults(results);
  };

  const handleSelectFile = (file: CodeFile) => {
    setSelectedFile(file);
    setIsSearching(false);
    setSearchQuery('');
  };

  const handleCreateFile = () => {
    if (!newFilePath.trim()) return;
    const ext = newFilePath.split('.').pop() || 'rs';
    const lang = ext === 'rs' ? 'rust' : ext === 'ts' || ext === 'tsx' ? 'typescript' : 'javascript';

    const newFile: CodeFile = {
      path: newFilePath.trim(),
      language: lang,
      content: newFileContent,
      sizeBytes: new Blob([newFileContent]).size,
    };

    globalIndexer.addOrUpdateFile(newFile);
    setFiles(globalIndexer.getFiles());
    setChunks(globalIndexer.getChunks());
    setSelectedFile(newFile);
    setShowNewFileModal(false);
  };

  const handleReindex = () => {
    const count = globalIndexer.index();
    setChunks(globalIndexer.getChunks());
    setFiles(globalIndexer.getFiles());
  };

  const handleCopyCode = () => {
    navigator.clipboard.writeText(selectedFile.content);
    setCopiedCode(true);
    setTimeout(() => setCopiedCode(false), 2000);
  };

  // Filter chunks belonging to selected file
  const fileChunks = chunks.filter(c => c.filePath === selectedFile.path);

  return (
    <div className="flex-1 flex flex-col h-full max-w-7xl mx-auto w-full p-4 overflow-hidden space-y-3 font-mono">
      {/* Top Bar: Search & Index Controls */}
      <div className="flex flex-col sm:flex-row items-center justify-between gap-3 bg-[#0d1117] p-3 rounded-xl border border-slate-800">
        <div className="relative flex-1 w-full">
          <Search className="w-4 h-4 text-slate-400 absolute left-3 top-1/2 -translate-y-1/2" />
          <input
            type="text"
            value={searchQuery}
            onChange={(e) => handleSearch(e.target.value)}
            placeholder="Search symbols, functions, keywords in indexed codebase..."
            className="w-full bg-slate-900 border border-slate-700/80 rounded-lg pl-9 pr-3 py-1.5 text-xs text-white placeholder-slate-500 focus:outline-none focus:border-cyan-500"
          />
        </div>

        <div className="flex items-center gap-2 w-full sm:w-auto justify-end">
          <button
            onClick={handleReindex}
            className="px-3 py-1.5 rounded-lg bg-slate-900 hover:bg-slate-800 border border-slate-700 text-slate-300 text-xs flex items-center gap-1.5 transition-colors cursor-pointer"
            title="Re-run AST Chunking"
          >
            <RefreshCw className="w-3.5 h-3.5 text-cyan-400" />
            <span>Re-index ({chunks.length} chunks)</span>
          </button>

          <button
            onClick={() => setShowNewFileModal(true)}
            className="px-3 py-1.5 rounded-lg bg-cyan-600 hover:bg-cyan-500 text-white text-xs flex items-center gap-1.5 transition-colors shadow-sm cursor-pointer"
          >
            <Plus className="w-3.5 h-3.5" />
            <span>Add File</span>
          </button>
        </div>
      </div>

      {/* Main 3-Column Workstation Layout */}
      <div className="flex-1 grid grid-cols-1 md:grid-cols-12 gap-3 min-h-0 overflow-hidden">
        {/* Left Column: File Explorer (3 cols) */}
        <div className="md:col-span-3 rounded-xl border border-slate-800 bg-[#0d1117] p-3 flex flex-col gap-2 overflow-hidden">
          <div className="flex items-center justify-between pb-2 border-b border-slate-800">
            <span className="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
              <FolderTree className="w-3.5 h-3.5 text-blue-400" />
              Codebase Files
            </span>
            <span className="text-[10px] text-slate-500">{files.length} indexed</span>
          </div>

          <div className="flex-1 overflow-y-auto space-y-1">
            {files.map((f) => (
              <button
                key={f.path}
                onClick={() => handleSelectFile(f)}
                className={`w-full text-left px-2.5 py-1.5 rounded-lg text-xs flex items-center justify-between transition-colors cursor-pointer ${
                  selectedFile.path === f.path && !isSearching
                    ? 'bg-cyan-950/60 text-cyan-300 border border-cyan-800/80 font-semibold'
                    : 'text-slate-300 hover:bg-slate-800/60 border border-transparent'
                }`}
              >
                <div className="flex items-center gap-2 truncate">
                  <FileCode className={`w-3.5 h-3.5 shrink-0 ${
                    f.path.endsWith('.rs') ? 'text-orange-400' :
                    f.path.endsWith('.json') ? 'text-yellow-400' : 'text-blue-400'
                  }`} />
                  <span className="truncate">{f.path}</span>
                </div>
                <span className="text-[10px] text-slate-500 shrink-0">
                  {(f.sizeBytes / 1024).toFixed(1)}k
                </span>
              </button>
            ))}
          </div>
        </div>

        {/* Center Column: File Viewer or Search Results (6 cols) */}
        <div className="md:col-span-6 rounded-xl border border-slate-800 bg-[#0d1117] flex flex-col overflow-hidden">
          {/* Header */}
          <div className="bg-slate-900/90 border-b border-slate-800 px-3 py-2 flex items-center justify-between">
            <div className="flex items-center gap-2 truncate">
              <Code className="w-4 h-4 text-cyan-400 shrink-0" />
              <span className="text-xs font-semibold text-white truncate">
                {isSearching ? `Search Results for "${searchQuery}" (${searchResults.length})` : selectedFile.path}
              </span>
            </div>

            {!isSearching && (
              <button
                onClick={handleCopyCode}
                className="p-1 rounded bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs flex items-center gap-1 transition-colors"
                title="Copy file content"
              >
                {copiedCode ? <Check className="w-3.5 h-3.5 text-emerald-400" /> : <Copy className="w-3.5 h-3.5" />}
              </button>
            )}
          </div>

          {/* Content Area */}
          <div className="flex-1 overflow-y-auto p-3 text-xs leading-relaxed bg-[#090d16]">
            {isSearching ? (
              <div className="space-y-3">
                {searchResults.length === 0 ? (
                  <div className="p-8 text-center text-slate-500">
                    No matching AST chunks found. Try another term like "pi", "main", "chudnovsky", "engine".
                  </div>
                ) : (
                  searchResults.map((res, i) => (
                    <div key={i} className="p-3 rounded-lg bg-slate-900 border border-slate-800 space-y-1">
                      <div className="flex items-center justify-between text-cyan-300 font-semibold">
                        <span>{res.filePath} (L{res.startLine}-{res.endLine})</span>
                        <span className="text-[10px] px-1.5 py-0.5 rounded bg-cyan-950 border border-cyan-800 text-cyan-400">
                          Score: {res.score}
                        </span>
                      </div>
                      {res.symbolName && (
                        <div className="text-[11px] text-purple-300">
                          Symbol: <span className="text-white font-bold">{res.symbolName}</span> ({res.chunkType})
                        </div>
                      )}
                      <pre className="text-slate-300 bg-slate-950 p-2 rounded border border-slate-800/80 overflow-x-auto text-[11px]">
                        {res.content}
                      </pre>
                    </div>
                  ))
                )}
              </div>
            ) : (
              <div className="flex">
                {/* Line numbers */}
                <div className="pr-3 select-none text-slate-600 text-right border-r border-slate-800 font-mono text-[11px]">
                  {selectedFile.content.split('\n').map((_, idx) => (
                    <div key={idx}>{idx + 1}</div>
                  ))}
                </div>
                {/* Code body */}
                <pre className="pl-3 flex-1 overflow-x-auto text-slate-200 font-mono text-[11px]">
                  {selectedFile.content}
                </pre>
              </div>
            )}
          </div>
        </div>

        {/* Right Column: AST Chunks Breakdown (3 cols) */}
        <div className="md:col-span-3 rounded-xl border border-slate-800 bg-[#0d1117] p-3 flex flex-col gap-2 overflow-hidden">
          <div className="flex items-center justify-between pb-2 border-b border-slate-800">
            <span className="text-xs font-semibold text-slate-300 flex items-center gap-1.5">
              <Layers className="w-3.5 h-3.5 text-purple-400" />
              AST Chunks ({fileChunks.length})
            </span>
            <span className="text-[10px] text-slate-500">Tree-sitter</span>
          </div>

          <div className="flex-1 overflow-y-auto space-y-2">
            {fileChunks.length === 0 ? (
              <div className="p-4 text-center text-slate-500 text-xs">
                No individual AST chunks extracted for this file.
              </div>
            ) : (
              fileChunks.map((chunk, i) => (
                <div key={i} className="p-2.5 rounded-lg bg-slate-900/80 border border-slate-800 space-y-1 text-xs">
                  <div className="flex items-center justify-between">
                    <span className="font-semibold text-purple-300 truncate">
                      {chunk.symbolName || `Chunk #${i + 1}`}
                    </span>
                    <span className="text-[10px] px-1.5 py-0.2 rounded bg-purple-950 border border-purple-800 text-purple-300 uppercase">
                      {chunk.chunkType}
                    </span>
                  </div>
                  <div className="text-[10px] text-slate-400">
                    Lines {chunk.startLine} to {chunk.endLine} ({chunk.content.split('\n').length} lines)
                  </div>
                  <pre className="text-[10px] text-slate-400 bg-slate-950 p-1.5 rounded border border-slate-800/40 line-clamp-2 overflow-hidden">
                    {chunk.content}
                  </pre>
                </div>
              ))
            )}
          </div>
        </div>
      </div>

      {/* New File Modal */}
      {showNewFileModal && (
        <div className="fixed inset-0 bg-black/70 backdrop-blur-sm z-50 flex items-center justify-center p-4">
          <div className="bg-slate-900 border border-slate-700 rounded-xl max-w-lg w-full p-4 space-y-3 font-mono">
            <div className="flex items-center justify-between border-b border-slate-800 pb-2">
              <h3 className="text-sm font-semibold text-white flex items-center gap-1.5">
                <Plus className="w-4 h-4 text-cyan-400" />
                Add Code File to Index
              </h3>
              <button
                onClick={() => setShowNewFileModal(false)}
                className="text-slate-400 hover:text-white text-xs cursor-pointer"
              >
                ✕
              </button>
            </div>

            <div>
              <label className="block text-xs text-slate-400 mb-1">File Path</label>
              <input
                type="text"
                value={newFilePath}
                onChange={(e) => setNewFilePath(e.target.value)}
                placeholder="src/utils/helper.rs"
                className="w-full bg-slate-950 border border-slate-800 rounded-lg px-3 py-1.5 text-xs text-white focus:outline-none focus:border-cyan-500"
              />
            </div>

            <div>
              <label className="block text-xs text-slate-400 mb-1">Source Code</label>
              <textarea
                value={newFileContent}
                onChange={(e) => setNewFileContent(e.target.value)}
                rows={8}
                className="w-full bg-slate-950 border border-slate-800 rounded-lg p-2.5 text-xs text-slate-200 font-mono focus:outline-none focus:border-cyan-500 resize-none"
              />
            </div>

            <div className="flex justify-end gap-2 pt-2 border-t border-slate-800">
              <button
                onClick={() => setShowNewFileModal(false)}
                className="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-slate-300 text-xs"
              >
                Cancel
              </button>
              <button
                onClick={handleCreateFile}
                className="px-3.5 py-1.5 rounded-lg bg-cyan-600 hover:bg-cyan-500 text-white text-xs font-semibold"
              >
                Index File
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  );
};
