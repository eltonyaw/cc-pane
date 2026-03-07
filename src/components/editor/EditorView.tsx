import { useEffect, useRef, useState, useCallback } from "react";
import Editor, { type OnMount } from "@monaco-editor/react";
import type { editor as MonacoEditor } from "monaco-editor";
import { toast } from "sonner";
import { filesystemService } from "@/services/filesystemService";
import { usePanesStore } from "@/stores";
import { useThemeStore } from "@/stores/useThemeStore";
import EditorToolbar from "./EditorToolbar";
import MarkdownPreview from "./MarkdownPreview";

/** 文件扩展名 → Monaco 语言 ID */
const EXTENSION_LANGUAGE_MAP: Record<string, string> = {
  ts: "typescript",
  tsx: "typescriptreact",
  js: "javascript",
  jsx: "javascriptreact",
  json: "json",
  jsonc: "json",
  md: "markdown",
  mdx: "markdown",
  css: "css",
  scss: "scss",
  less: "less",
  html: "html",
  htm: "html",
  xml: "xml",
  svg: "xml",
  yaml: "yaml",
  yml: "yaml",
  toml: "ini",
  rs: "rust",
  py: "python",
  go: "go",
  java: "java",
  c: "c",
  cpp: "cpp",
  h: "cpp",
  hpp: "cpp",
  cs: "csharp",
  rb: "ruby",
  php: "php",
  sh: "shell",
  bash: "shell",
  zsh: "shell",
  ps1: "powershell",
  sql: "sql",
  graphql: "graphql",
  dockerfile: "dockerfile",
  makefile: "makefile",
  lua: "lua",
  swift: "swift",
  kt: "kotlin",
  dart: "dart",
  r: "r",
};

function getLanguageFromPath(filePath: string): string {
  const ext = filePath.split(".").pop()?.toLowerCase() || "";
  const name = filePath.split(/[/\\]/).pop()?.toLowerCase() || "";
  if (name === "dockerfile") return "dockerfile";
  if (name === "makefile" || name === "gnumakefile") return "makefile";
  return EXTENSION_LANGUAGE_MAP[ext] || "plaintext";
}

type PreviewMode = "edit" | "preview" | "split";

/** 从未知 error 中提取可读消息 */
function getErrorMessage(err: unknown): string {
  if (typeof err === "string") return err;
  if (err instanceof Error) return err.message;
  if (err && typeof err === "object" && "message" in err) return String((err as { message: unknown }).message);
  return String(err);
}

const MAX_FILE_SIZE = 5 * 1024 * 1024; // 5MB

interface EditorViewProps {
  filePath: string;
  projectPath: string;
  tabId: string;
  paneId: string;
}

export default function EditorView({
  filePath,
  projectPath: _projectPath,
  tabId,
  paneId,
}: EditorViewProps) {
  void _projectPath; // 保留 prop 供未来使用（如路径沙箱验证）
  const editorRef = useRef<MonacoEditor.IStandaloneCodeEditor | null>(null);
  const [content, setContent] = useState<string>("");
  const [originalContent, setOriginalContent] = useState<string>("");
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [readOnly, setReadOnly] = useState(false);
  const [readOnlyReason, setReadOnlyReason] = useState<"path" | "encoding" | null>(null);
  const [previewMode, setPreviewMode] = useState<PreviewMode>("edit");
  const lastSavedMtime = useRef<string | null>(null);

  const isDark = useThemeStore((s) => s.isDark);
  const setTabDirty = usePanesStore((s) => s.setTabDirty);

  const language = getLanguageFromPath(filePath);
  const isMarkdown = language === "markdown";
  const dirty = content !== originalContent;

  // 同步 dirty 状态到 tab
  useEffect(() => {
    setTabDirty(paneId, tabId, dirty);
  }, [dirty, paneId, tabId, setTabDirty]);

  // 检测只读路径
  const isReadOnlyPath = useCallback((p: string) => {
    const normalized = p.replace(/\\/g, "/").toLowerCase();
    return (
      normalized.includes("/node_modules/") ||
      normalized.includes("/.git/") ||
      normalized.includes("/target/")
    );
  }, []);

  // 加载文件
  useEffect(() => {
    let cancelled = false;
    setLoading(true);
    setError(null);

    (async () => {
      try {
        // 先检查文件大小
        const info = await filesystemService.getEntryInfo(filePath);
        if (info.size > MAX_FILE_SIZE) {
          setError(`File too large (${(info.size / 1024 / 1024).toFixed(1)}MB). Maximum is 5MB.`);
          setLoading(false);
          return;
        }

        const result = await filesystemService.readFile(filePath);
        if (cancelled) return;

        if (result.encoding === "binary") {
          setError("Binary file — cannot preview");
          setLoading(false);
          return;
        }

        setContent(result.content);
        setOriginalContent(result.content);
        const pathReadOnly = isReadOnlyPath(filePath);
        const encodingReadOnly = result.encoding !== "utf-8";
        setReadOnly(pathReadOnly || encodingReadOnly);
        setReadOnlyReason(pathReadOnly ? "path" : encodingReadOnly ? "encoding" : null);

        // 记录 mtime 用于冲突检测
        const entryInfo = await filesystemService.getEntryInfo(filePath);
        lastSavedMtime.current = entryInfo.modified;
      } catch (err) {
        if (!cancelled) {
          setError(`Failed to load file: ${getErrorMessage(err)}`);
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();

    return () => {
      cancelled = true;
    };
  }, [filePath, isReadOnlyPath]);

  // 保存
  const handleSave = useCallback(async () => {
    if (readOnly || !dirty) return;

    try {
      // 冲突检测：比对 mtime
      if (lastSavedMtime.current) {
        const current = await filesystemService.getEntryInfo(filePath);
        if (current.modified && current.modified !== lastSavedMtime.current) {
          const overwrite = window.confirm(
            "File has been modified externally. Overwrite?"
          );
          if (!overwrite) return;
        }
      }

      await filesystemService.writeFile(filePath, content);
      setOriginalContent(content);

      // 更新 mtime
      const info = await filesystemService.getEntryInfo(filePath);
      lastSavedMtime.current = info.modified;

      toast.success("File saved");
    } catch (err) {
      toast.error(`Save failed: ${getErrorMessage(err)}`);
    }
  }, [filePath, content, readOnly, dirty]);

  // Monaco mount
  const handleEditorMount: OnMount = useCallback(
    (editor) => {
      editorRef.current = editor;

      // Ctrl+S 快捷键
      editor.addAction({
        id: "save-file",
        label: "Save File",
        keybindings: [
          // eslint-disable-next-line no-bitwise
          2048 | 49, // monaco.KeyMod.CtrlCmd | monaco.KeyCode.KeyS
        ],
        run: () => {
          handleSave();
        },
      });
    },
    [handleSave]
  );

  // 编辑器内容变化
  const handleEditorChange = useCallback(
    (value: string | undefined) => {
      if (value !== undefined) {
        setContent(value);
      }
    },
    []
  );

  // 工具栏操作
  const handleUndo = useCallback(() => {
    editorRef.current?.trigger("toolbar", "undo", null);
  }, []);

  const handleRedo = useCallback(() => {
    editorRef.current?.trigger("toolbar", "redo", null);
  }, []);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full text-sm text-muted-foreground">
        Loading file...
      </div>
    );
  }

  if (error) {
    return (
      <div className="flex items-center justify-center h-full text-sm text-muted-foreground">
        {error}
      </div>
    );
  }

  const showEditor = previewMode === "edit" || previewMode === "split" || !isMarkdown;
  const showPreview = isMarkdown && (previewMode === "preview" || previewMode === "split");

  return (
    <div className="flex flex-col h-full overflow-hidden">
      <EditorToolbar
        language={language}
        dirty={dirty}
        readOnly={readOnly}
        isMarkdown={isMarkdown}
        previewMode={previewMode}
        onSave={handleSave}
        onUndo={handleUndo}
        onRedo={handleRedo}
        onPreviewModeChange={setPreviewMode}
      />

      <div className="flex-1 flex overflow-hidden">
        {showEditor && (
          <div className={showPreview ? "w-1/2 border-r" : "flex-1"}>
            <Editor
              height="100%"
              language={language}
              value={content}
              theme={isDark ? "vs-dark" : "vs"}
              onChange={handleEditorChange}
              onMount={handleEditorMount}
              options={{
                readOnly,
                minimap: { enabled: false },
                fontSize: 13,
                lineNumbers: "on",
                wordWrap: "on",
                scrollBeyondLastLine: false,
                automaticLayout: true,
                tabSize: 2,
                renderWhitespace: "selection",
                bracketPairColorization: { enabled: true },
              }}
            />
          </div>
        )}

        {showPreview && (
          <div className={showEditor ? "w-1/2" : "flex-1"}>
            <MarkdownPreview content={content} />
          </div>
        )}
      </div>

      {readOnly && (
        <div className="px-2 py-0.5 text-[10px] text-amber-500 bg-amber-500/5 border-t">
          {readOnlyReason === "encoding"
            ? "Read-only (non UTF-8 encoding)"
            : "Read-only (protected path)"}
        </div>
      )}
    </div>
  );
}
