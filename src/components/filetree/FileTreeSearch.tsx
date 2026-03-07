import { useState, useCallback, useRef, useEffect } from "react";
import { Search, X } from "lucide-react";
import { Input } from "@/components/ui/input";
import type { SearchResult } from "@/types/filesystem";

interface FileTreeSearchProps {
  rootPath: string;
  onSearch: (rootPath: string, query: string) => Promise<SearchResult[]>;
  onFileClick: (filePath: string) => void;
}

export default function FileTreeSearch({
  rootPath,
  onSearch,
  onFileClick,
}: FileTreeSearchProps) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<SearchResult[]>([]);
  const [isSearching, setIsSearching] = useState(false);
  const [showResults, setShowResults] = useState(false);
  const debounceRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  const handleChange = useCallback(
    (e: React.ChangeEvent<HTMLInputElement>) => {
      const value = e.target.value;
      setQuery(value);

      if (debounceRef.current) clearTimeout(debounceRef.current);

      if (!value.trim()) {
        setResults([]);
        setShowResults(false);
        return;
      }

      debounceRef.current = setTimeout(async () => {
        setIsSearching(true);
        try {
          const res = await onSearch(rootPath, value.trim());
          setResults(res);
          setShowResults(true);
        } catch {
          setResults([]);
        } finally {
          setIsSearching(false);
        }
      }, 300);
    },
    [rootPath, onSearch]
  );

  const handleClear = useCallback(() => {
    setQuery("");
    setResults([]);
    setShowResults(false);
  }, []);

  const handleResultClick = useCallback(
    (result: SearchResult) => {
      if (!result.isDir) {
        onFileClick(result.path);
      }
      setShowResults(false);
    },
    [onFileClick]
  );

  // 卸载时清理 debounce timer
  useEffect(() => {
    return () => {
      if (debounceRef.current) clearTimeout(debounceRef.current);
    };
  }, []);

  // 点击外部关闭结果
  const containerRef = useRef<HTMLDivElement>(null);
  useEffect(() => {
    function handleClickOutside(e: MouseEvent) {
      if (containerRef.current && !containerRef.current.contains(e.target as Node)) {
        setShowResults(false);
      }
    }
    document.addEventListener("mousedown", handleClickOutside);
    return () => document.removeEventListener("mousedown", handleClickOutside);
  }, []);

  return (
    <div ref={containerRef} className="relative px-1 py-1">
      <div className="relative">
        <Search size={12} className="absolute left-2 top-1/2 -translate-y-1/2 text-muted-foreground" />
        <Input
          value={query}
          onChange={handleChange}
          placeholder="Search files..."
          className="h-6 text-xs pl-6 pr-6"
          onFocus={() => results.length > 0 && setShowResults(true)}
        />
        {query && (
          <button
            onClick={handleClear}
            className="absolute right-1 top-1/2 -translate-y-1/2 p-0.5 rounded hover:bg-accent"
          >
            <X size={10} />
          </button>
        )}
      </div>

      {/* 搜索结果下拉 */}
      {showResults && results.length > 0 && (
        <div className="absolute z-50 left-1 right-1 mt-1 max-h-48 overflow-y-auto rounded-md border bg-popover shadow-lg">
          {results.map((result) => (
            <div
              key={result.path}
              className="flex items-center gap-1.5 px-2 py-1 text-xs cursor-pointer hover:bg-accent truncate"
              onClick={() => handleResultClick(result)}
            >
              <span className="truncate font-medium">{result.name}</span>
              <span className="truncate text-muted-foreground text-[10px]">
                {result.relPath}
              </span>
            </div>
          ))}
        </div>
      )}

      {showResults && query && results.length === 0 && !isSearching && (
        <div className="absolute z-50 left-1 right-1 mt-1 px-2 py-2 text-xs text-muted-foreground rounded-md border bg-popover shadow-lg">
          No results found
        </div>
      )}
    </div>
  );
}
