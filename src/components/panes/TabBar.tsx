import { useState, useRef, useCallback, memo } from "react";
import { X, Plus, PanelRight, PanelBottom, Pin, Pencil } from "lucide-react";
import { useTranslation } from "react-i18next";
import {
  ContextMenu,
  ContextMenuContent,
  ContextMenuItem,
  ContextMenuSeparator,
  ContextMenuTrigger,
} from "@/components/ui/context-menu";
import { useTerminalStatusStore, useThemeStore } from "@/stores";
import StatusIndicator from "@/components/StatusIndicator";
import type { Tab } from "@/types";

interface TabBarProps {
  tabs: Tab[];
  activeId: string;
  onSelect: (tabId: string) => void;
  onClose: (tabId: string) => void;
  onTogglePin: (tabId: string) => void;
  onAdd: () => void;
  onSplitRight: () => void;
  onSplitDown: () => void;
  onFullscreen: (tabId: string) => void;
  onReorder: (fromIndex: number, toIndex: number) => void;
  onRename: (tabId: string, newTitle: string) => void;
  onSplitAndMoveRight: (tabId: string) => void;
  onSplitAndMoveDown: (tabId: string) => void;
}

export default memo(function TabBar({
  tabs,
  activeId,
  onSelect,
  onClose,
  onTogglePin,
  onAdd,
  onSplitRight,
  onSplitDown,
  onFullscreen,
  onReorder,
  onRename,
  onSplitAndMoveRight,
  onSplitAndMoveDown,
}: TabBarProps) {
  const { t } = useTranslation("panes");
  const getStatus = useTerminalStatusStore((s) => s.getStatus);
  const isDark = useThemeStore((s) => s.isDark);

  const [editingTabId, setEditingTabId] = useState<string | null>(null);
  const [editingTitle, setEditingTitle] = useState("");
  const [dragIndex, setDragIndex] = useState<number | null>(null);
  const [dropIndex, setDropIndex] = useState<number | null>(null);
  const editInputRef = useRef<HTMLInputElement>(null);

  // 标签拖拽排序
  function handleDragStart(e: React.DragEvent, index: number) {
    setDragIndex(index);
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = "move";
      e.dataTransfer.setData("text/plain", String(index));
    }
  }

  function handleDragOver(e: React.DragEvent, index: number) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "move";
    setDropIndex(index);
  }

  function handleDrop(e: React.DragEvent, index: number) {
    e.preventDefault();
    if (dragIndex !== null && dragIndex !== index) {
      onReorder(dragIndex, index);
    }
    setDragIndex(null);
    setDropIndex(null);
  }

  function handleDragEnd() {
    setDragIndex(null);
    setDropIndex(null);
  }

  // 标签重命名
  const startRename = useCallback((tab: Tab) => {
    setEditingTabId(tab.id);
    setEditingTitle(tab.title);
    requestAnimationFrame(() => {
      editInputRef.current?.focus();
      editInputRef.current?.select();
    });
  }, []);

  function confirmRename() {
    if (editingTabId && editingTitle.trim()) {
      onRename(editingTabId, editingTitle.trim());
    }
    setEditingTabId(null);
    setEditingTitle("");
  }

  function cancelRename() {
    setEditingTabId(null);
    setEditingTitle("");
  }

  return (
    <div
      className={`flex items-center h-11 px-4 shrink-0 border-b backdrop-blur-xl transition-colors ${
        isDark
          ? 'bg-[#0F1117]/30 border-white/5'
          : 'bg-white/30 border-white/30'
      }`}
    >
      <div
        className="flex items-center gap-2 overflow-x-auto flex-1 h-full pt-1.5"
      >
        {tabs.map((tab, index) => (
          <ContextMenu key={tab.id}>
            <ContextMenuTrigger asChild>
              <div
                className={`relative group flex items-center gap-2.5 px-4 h-9 rounded-t-lg text-sm font-medium transition-all cursor-pointer border-t border-x backdrop-blur-lg ${
                  tab.id === activeId
                    ? isDark
                      ? 'bg-[#0F1117]/60 border-white/5 text-blue-300'
                      : 'bg-white/60 border-white/50 text-slate-800 shadow-sm'
                    : isDark
                      ? 'border-transparent text-slate-500 hover:text-slate-300 hover:bg-white/5'
                      : 'border-transparent text-slate-400 hover:text-slate-600 hover:bg-white/30'
                }
                  ${dragIndex === index ? "opacity-50" : ""}
                  ${dropIndex === index && dragIndex !== index ? "border-[var(--app-accent)] bg-[var(--app-active-bg)]" : ""}
                `}
                draggable
                onClick={() => onSelect(tab.id)}
                onDoubleClick={() => onFullscreen(tab.id)}
                onDragStart={(e) => handleDragStart(e, index)}
                onDragOver={(e) => handleDragOver(e, index)}
                onDragLeave={() => setDropIndex(null)}
                onDrop={(e) => handleDrop(e, index)}
                onDragEnd={handleDragEnd}
              >
                {/* 顶部高亮线 */}
                {tab.id === activeId && (
                  <div className="absolute top-0 left-0 w-full h-[1px] bg-gradient-to-r from-transparent via-blue-400 to-transparent opacity-60" />
                )}
                <StatusIndicator status={getStatus(tab.sessionId)} size={6} />
                {tab.pinned && (
                  <Pin size={12} className="shrink-0 opacity-60 rotate-45" style={{ color: "var(--app-accent)" }} />
                )}
                {editingTabId === tab.id ? (
                  <input
                    ref={editInputRef}
                    value={editingTitle}
                    onChange={(e) => setEditingTitle(e.target.value)}
                    className="max-w-[120px] text-xs font-medium rounded px-1 py-0.5 outline-none"
                    style={{
                      background: "var(--app-content)",
                      border: "1px solid var(--app-accent)",
                      color: "var(--app-text-primary)",
                    }}
                    onBlur={confirmRename}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") confirmRename();
                      else if (e.key === "Escape") cancelRename();
                    }}
                    onClick={(e) => e.stopPropagation()}
                  />
                ) : (
                  <span
                    className="max-w-[120px] truncate"
                    onDoubleClick={(e) => {
                      e.stopPropagation();
                      startRename(tab);
                    }}
                  >
                    {tab.title}
                  </span>
                )}
                {!tab.pinned && (
                  <button
                    className={`opacity-0 group-hover:opacity-100 p-0.5 rounded-full transition-all ${isDark ? 'hover:bg-white/10' : 'hover:bg-slate-200'}`}
                    onClick={(e) => {
                      e.stopPropagation();
                      onClose(tab.id);
                    }}
                  >
                    <X className="w-3 h-3" />
                  </button>
                )}
              </div>
            </ContextMenuTrigger>
            <ContextMenuContent className="w-48">
              <ContextMenuItem onClick={() => startRename(tab)}>
                <Pencil size={14} className="mr-2" /> {t("renameTab")}
              </ContextMenuItem>
              <ContextMenuItem onClick={() => onTogglePin(tab.id)}>
                {tab.pinned ? t("unpinTab") : t("pinTab")}
              </ContextMenuItem>
              <ContextMenuSeparator />
              <ContextMenuItem onClick={onSplitRight}>
                <PanelRight size={14} className="mr-2" /> {t("splitRight")}
              </ContextMenuItem>
              <ContextMenuItem onClick={onSplitDown}>
                <PanelBottom size={14} className="mr-2" /> {t("splitDown")}
              </ContextMenuItem>
              {tabs.length > 1 && (
                <>
                  <ContextMenuItem onClick={() => onSplitAndMoveRight(tab.id)}>
                    <PanelRight size={14} className="mr-2" /> {t("splitAndMoveRight")}
                  </ContextMenuItem>
                  <ContextMenuItem onClick={() => onSplitAndMoveDown(tab.id)}>
                    <PanelBottom size={14} className="mr-2" /> {t("splitAndMoveDown")}
                  </ContextMenuItem>
                </>
              )}
              {!tab.pinned && (
                <>
                  <ContextMenuSeparator />
                  <ContextMenuItem className="text-destructive" onClick={() => onClose(tab.id)}>
                    {t("closeTab")}
                  </ContextMenuItem>
                </>
              )}
            </ContextMenuContent>
          </ContextMenu>
        ))}
        <button
          className={`p-2 mb-1 rounded-lg transition-colors ${
            isDark
              ? 'text-slate-400 hover:bg-white/10 hover:text-slate-200'
              : 'text-slate-500 hover:bg-white/40 hover:text-slate-800'
          }`}
          onClick={onAdd}
        >
          <Plus className="w-4 h-4" />
        </button>
      </div>
    </div>
  );
});
