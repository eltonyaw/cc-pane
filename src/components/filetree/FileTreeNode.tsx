import { memo, useCallback } from "react";
import {
  ChevronRight, ChevronDown, Folder, FolderOpen,
  File, FileCode, FileText, FileJson, Image, FileType,
  Loader2,
} from "lucide-react";
import type { FileTreeNode as FileTreeNodeType } from "@/types/filesystem";

interface FileTreeNodeProps {
  node: FileTreeNodeType;
  depth: number;
  compact?: boolean;
  rootPath: string;
  selectedFilePath?: string | null;
  gitStatuses?: Record<string, string>;
  onToggle: (path: string) => void;
  onFileClick: (path: string) => void;
  onContextMenu: (e: React.MouseEvent, node: FileTreeNodeType) => void;
}

/** 根据扩展名返回图标 */
function getFileIcon(ext: string | null) {
  if (!ext) return <File size={14} className="shrink-0 text-muted-foreground" />;
  switch (ext.toLowerCase()) {
    case "ts":
    case "tsx":
    case "js":
    case "jsx":
    case "rs":
    case "py":
    case "go":
    case "java":
    case "c":
    case "cpp":
    case "h":
    case "hpp":
    case "rb":
    case "php":
    case "swift":
    case "kt":
    case "dart":
    case "lua":
    case "sh":
    case "bash":
    case "ps1":
    case "bat":
    case "css":
    case "scss":
    case "less":
    case "html":
    case "xml":
    case "sql":
      return <FileCode size={14} className="shrink-0 text-blue-400" />;
    case "md":
    case "txt":
    case "log":
    case "csv":
      return <FileText size={14} className="shrink-0 text-green-400" />;
    case "json":
    case "jsonc":
    case "yaml":
    case "yml":
    case "toml":
    case "ini":
    case "cfg":
      return <FileJson size={14} className="shrink-0 text-yellow-400" />;
    case "png":
    case "jpg":
    case "jpeg":
    case "gif":
    case "svg":
    case "ico":
    case "webp":
    case "bmp":
      return <Image size={14} className="shrink-0 text-purple-400" />;
    default:
      return <FileType size={14} className="shrink-0 text-muted-foreground" />;
  }
}

/** Git 状态颜色映射 */
const GIT_STATUS_COLORS: Record<string, string> = {
  modified: "text-yellow-400",
  added: "text-green-400",
  deleted: "text-red-400 line-through",
  untracked: "text-emerald-300/70",
  renamed: "text-blue-400",
};

export default memo(function FileTreeNode({
  node,
  depth,
  compact,
  rootPath,
  selectedFilePath,
  gitStatuses,
  onToggle,
  onFileClick,
  onContextMenu,
}: FileTreeNodeProps) {
  const handleClick = useCallback(() => {
    if (node.entry.isDir) {
      onToggle(node.entry.path);
    } else {
      onFileClick(node.entry.path);
    }
  }, [node.entry.isDir, node.entry.path, onToggle, onFileClick]);

  const handleContextMenu = useCallback(
    (e: React.MouseEvent) => {
      e.preventDefault();
      onContextMenu(e, node);
    },
    [node, onContextMenu]
  );

  const paddingLeft = compact ? depth * 12 + 4 : depth * 16 + 8;
  const height = compact ? "h-6" : "h-7";
  const isSelected = !node.entry.isDir && node.entry.path === selectedFilePath;
  const gitStatus = gitStatuses?.[node.entry.path];
  const gitColorClass = gitStatus ? GIT_STATUS_COLORS[gitStatus] : undefined;

  return (
    <>
      <div
        className={`flex items-center gap-1 ${height} px-1 cursor-pointer select-none rounded-sm hover:bg-accent/50 transition-colors group ${isSelected ? "bg-accent" : ""}`}
        style={{ paddingLeft }}
        data-file-path={node.entry.path}
        onClick={handleClick}
        onContextMenu={handleContextMenu}
      >
        {/* 展开箭头 / 占位 */}
        {node.entry.isDir ? (
          node.loading ? (
            <Loader2 size={12} className="shrink-0 animate-spin text-muted-foreground" />
          ) : node.expanded ? (
            <ChevronDown size={12} className="shrink-0 text-muted-foreground" />
          ) : (
            <ChevronRight size={12} className="shrink-0 text-muted-foreground" />
          )
        ) : (
          <span className="w-3 shrink-0" />
        )}

        {/* 图标 */}
        {node.entry.isDir ? (
          node.expanded ? (
            <FolderOpen size={14} className="shrink-0 text-amber-400" />
          ) : (
            <Folder size={14} className="shrink-0 text-amber-400" />
          )
        ) : (
          getFileIcon(node.entry.extension)
        )}

        {/* 文件名 */}
        <span className={`truncate text-xs leading-none ${gitColorClass || ""}`}>{node.entry.name}</span>
      </div>

      {/* 递归子节点 */}
      {node.entry.isDir && node.expanded && node.children && (
        <div>
          {node.children.map((child) => (
            <FileTreeNode
              key={child.entry.path}
              node={child}
              depth={depth + 1}
              compact={compact}
              rootPath={rootPath}
              selectedFilePath={selectedFilePath}
              gitStatuses={gitStatuses}
              onToggle={onToggle}
              onFileClick={onFileClick}
              onContextMenu={onContextMenu}
            />
          ))}
        </div>
      )}
    </>
  );
});
