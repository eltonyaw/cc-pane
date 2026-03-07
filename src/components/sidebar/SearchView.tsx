import { useState, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { Search, File, Loader2 } from "lucide-react";
import { useThemeStore, useWorkspacesStore } from "@/stores";
import { invoke } from "@tauri-apps/api/core";

interface SearchResult {
  path: string;
  name: string;
  projectName: string;
}

export default function SearchView() {
  const { t } = useTranslation("common");
  const isDark = useThemeStore((s) => s.isDark);
  const workspaces = useWorkspacesStore((s) => s.workspaces);

  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [searching, setSearching] = useState(false);

  const handleSearch = useCallback(async (q: string) => {
    setQuery(q);
    if (q.trim().length < 2) {
      setResults([]);
      return;
    }

    setSearching(true);
    try {
      // 搜索所有项目目录下的文件
      const allResults: SearchResult[] = [];
      for (const ws of workspaces) {
        for (const project of ws.projects) {
          try {
            const files = await invoke<string[]>("search_files", {
              dirPath: project.path,
              pattern: q.trim(),
              maxResults: 10,
            });
            for (const f of files) {
              const name = f.split(/[/\\]/).pop() || f;
              allResults.push({
                path: f,
                name,
                projectName: project.alias || project.path.split(/[/\\]/).pop() || project.path,
              });
            }
          } catch {
            // 忽略单个项目搜索错误
          }
        }
      }
      setResults(allResults.slice(0, 30));
    } catch (e) {
      console.error("Search error:", e);
    } finally {
      setSearching(false);
    }
  }, [workspaces]);

  return (
    <div className="flex flex-col h-full">
      {/* 视图标题栏 */}
      <div className="flex items-center justify-between px-4 py-2 shrink-0">
        <span
          className="text-[11px] font-bold tracking-wider"
          style={{ color: "var(--app-text-secondary)" }}
        >
          SEARCH
        </span>
      </div>

      {/* 搜索输入框 */}
      <div className="px-3 pb-2 shrink-0">
        <div
          className="flex items-center gap-2 px-2.5 py-1.5 rounded-lg"
          style={{
            background: isDark ? "rgba(255,255,255,0.06)" : "rgba(0,0,0,0.04)",
            border: `1px solid ${isDark ? "rgba(255,255,255,0.08)" : "rgba(0,0,0,0.08)"}`,
          }}
        >
          <Search className="w-3.5 h-3.5 shrink-0" style={{ color: "var(--app-text-tertiary)" }} />
          <input
            type="text"
            value={query}
            onChange={(e) => handleSearch(e.target.value)}
            placeholder={t("search")}
            className="flex-1 bg-transparent text-[12px] outline-none placeholder:text-[var(--app-text-tertiary)]"
            style={{ color: "var(--app-text-primary)" }}
          />
          {searching && <Loader2 className="w-3 h-3 animate-spin" style={{ color: "var(--app-text-tertiary)" }} />}
        </div>
      </div>

      {/* 搜索结果 */}
      <div className="flex-1 overflow-y-auto px-3">
        {results.length > 0 ? (
          <div className="space-y-0.5">
            {results.map((r, idx) => (
              <div
                key={`${r.path}-${idx}`}
                className={`flex items-center gap-2 px-2 py-1.5 rounded-lg transition-colors cursor-default ${
                  isDark
                    ? 'hover:bg-white/5 text-slate-300'
                    : 'hover:bg-white/40 text-slate-600'
                }`}
              >
                <File className="w-3.5 h-3.5 shrink-0" style={{ color: "var(--app-text-tertiary)" }} />
                <div className="min-w-0 flex-1">
                  <span className="text-[12px] truncate block">{r.name}</span>
                  <span className={`text-[10px] truncate block ${isDark ? 'text-slate-600' : 'text-slate-400'}`}>
                    {r.projectName}
                  </span>
                </div>
              </div>
            ))}
          </div>
        ) : query.length >= 2 && !searching ? (
          <p className={`text-xs text-center mt-8 ${isDark ? 'text-slate-600' : 'text-slate-400'}`}>
            No results found
          </p>
        ) : (
          <p className={`text-xs text-center mt-8 ${isDark ? 'text-slate-600' : 'text-slate-400'}`}>
            Search across all projects
          </p>
        )}
      </div>
    </div>
  );
}
