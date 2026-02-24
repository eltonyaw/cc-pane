import {
  Command, Minus, Square, Copy, X, Sun, Moon,
  PanelLeft, PanelLeftClose, UploadCloud, FolderPlus,
  PanelRight, PanelBottom, Globe, Maximize2,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { useThemeStore, usePanesStore, useBorderlessStore, useSettingsStore, formatKeyCombo } from "@/stores";
import { useWindowControl } from "@/hooks/useWindowControl";
import WorkspaceSwitcher from "@/components/WorkspaceSwitcher";

interface GlobalTopBarProps {
  sidebarCollapsed: boolean;
  onToggleSidebar: () => void;
  onImport: () => void;
  onNew: () => void;
}

export default function GlobalTopBar({ sidebarCollapsed, onToggleSidebar, onImport, onNew }: GlobalTopBarProps) {
  const { t, i18n } = useTranslation();
  const isDark = useThemeStore((s) => s.isDark);
  const toggleTheme = useThemeStore((s) => s.toggleTheme);
  const { closeWindow, minimizeWindow, maximizeWindow, isMaximized, startDrag } = useWindowControl();

  const activePaneId = usePanesStore((s) => s.activePaneId);
  const splitRight = usePanesStore((s) => s.splitRight);
  const splitDown = usePanesStore((s) => s.splitDown);

  const isBorderless = useBorderlessStore((s) => s.isBorderless);
  const exitBorderless = useBorderlessStore((s) => s.exitBorderless);

  const settings = useSettingsStore((s) => s.settings);

  function getShortcut(actionId: string): string {
    const combo = settings?.shortcuts.bindings[actionId];
    return combo ? formatKeyCombo(combo) : "";
  }

  function handleSplitRight() {
    if (activePaneId) splitRight(activePaneId);
  }

  function handleSplitDown() {
    if (activePaneId) splitDown(activePaneId);
  }

  function handleToggleLanguage() {
    const nextLang = i18n.language === "zh-CN" ? "en" : "zh-CN";
    i18n.changeLanguage(nextLang);
    // 同步更新 Settings store
    const store = useSettingsStore.getState();
    if (store.settings) {
      const updated = { ...store.settings, general: { ...store.settings.general, language: nextLang } };
      store.saveSettings(updated).catch(console.error);
    }
  }

  return (
    <div
      className="relative flex items-center h-[38px] px-3 shrink-0 select-none z-10"
      style={{
        background: "var(--app-menubar)",
        borderBottom: "1px solid var(--app-glass-border)",
        backdropFilter: `blur(var(--app-glass-blur))`,
        WebkitBackdropFilter: `blur(var(--app-glass-blur))`,
        boxShadow: "var(--app-glass-highlight)",
      }}
      onMouseDown={(e) => {
        if (e.target === e.currentTarget) startDrag();
      }}
    >
      {/* 顶部高光线 — 增加层次感 */}
      <div
        className="absolute top-0 left-0 right-0 h-px pointer-events-none"
        style={{
          background: isDark
            ? "linear-gradient(90deg, transparent, rgba(255,255,255,0.08) 20%, rgba(255,255,255,0.12) 50%, rgba(255,255,255,0.08) 80%, transparent)"
            : "linear-gradient(90deg, transparent, rgba(255,255,255,0.6) 20%, rgba(255,255,255,0.8) 50%, rgba(255,255,255,0.6) 80%, transparent)",
        }}
      />

      {/* 左侧：品牌区 + Sidebar Toggle + 工作空间切换 + 导入/新建 */}
      <div className="flex items-center gap-2.5 shrink-0 mr-3">
        <div
          className="w-[26px] h-[26px] rounded-[7px] flex items-center justify-center transition-transform hover:scale-105"
          style={{
            background: isDark
              ? "linear-gradient(135deg, rgba(59,130,246,0.8), rgba(99,102,241,0.8))"
              : "linear-gradient(135deg, rgba(255,255,255,0.85), rgba(255,255,255,0.5))",
            border: `1px solid ${isDark ? "rgba(255,255,255,0.12)" : "rgba(255,255,255,0.7)"}`,
            boxShadow: isDark
              ? "0 2px 8px rgba(59,130,246,0.25), inset 0 1px 0 rgba(255,255,255,0.1)"
              : "0 2px 8px rgba(0,0,0,0.08), inset 0 1px 0 rgba(255,255,255,0.5)",
          }}
        >
          <Command
            className="w-[14px] h-[14px]"
            style={{ color: isDark ? "#fff" : "var(--app-accent)" }}
          />
        </div>
        <span
          className="text-[13px] font-semibold tracking-tight"
          style={{ color: "var(--app-text-primary)", opacity: 0.85 }}
        >
          CC-Panes
        </span>

        {/* Sidebar Toggle */}
        <button
          onClick={onToggleSidebar}
          className="w-[26px] h-[26px] flex items-center justify-center rounded-md transition-colors duration-200 text-[var(--app-text-secondary)] hover:bg-[var(--app-hover)]"
          title={sidebarCollapsed ? t("expandSidebar", { ns: "sidebar" }) : t("collapseSidebar", { ns: "sidebar" })}
        >
          {sidebarCollapsed
            ? <PanelLeft className="w-[14px] h-[14px]" />
            : <PanelLeftClose className="w-[14px] h-[14px]" />
          }
        </button>

        {/* 工作空间切换下拉菜单 */}
        <WorkspaceSwitcher
          onExpandSidebar={sidebarCollapsed ? onToggleSidebar : undefined}
        />

        {/* 分隔线 */}
        <div
          className="w-px h-3.5 mx-0.5"
          style={{ background: "var(--app-border)" }}
        />

        {/* 导入按钮 */}
        <Tooltip>
          <TooltipTrigger asChild>
            <button
              onClick={onImport}
              className="w-[26px] h-[26px] flex items-center justify-center rounded-md transition-colors duration-200 text-[var(--app-text-secondary)] hover:bg-[var(--app-hover)]"
            >
              <UploadCloud className="w-[14px] h-[14px]" />
            </button>
          </TooltipTrigger>
          <TooltipContent side="bottom">
            <p>{t("importProject")}</p>
          </TooltipContent>
        </Tooltip>

        {/* 新建按钮 */}
        <Tooltip>
          <TooltipTrigger asChild>
            <button
              onClick={onNew}
              className="w-[26px] h-[26px] flex items-center justify-center rounded-md transition-colors duration-200 text-[var(--app-text-secondary)] hover:bg-[var(--app-hover)]"
            >
              <FolderPlus className="w-[14px] h-[14px]" />
            </button>
          </TooltipTrigger>
          <TooltipContent side="bottom">
            <p>{t("newProject")}</p>
          </TooltipContent>
        </Tooltip>
      </div>

      {/* 中间：拖拽区 */}
      <div
        className="flex-1 h-full cursor-grab"
        style={{ WebkitAppRegion: "drag" } as React.CSSProperties}
        onMouseDown={(e) => { e.preventDefault(); startDrag(); }}
      />

      {/* 右侧：分屏按钮 + 退出无边框 + 语言切换 + 主题切换 + 窗口控件 */}
      <div className="flex items-center shrink-0">
        {/* 分屏按钮组 */}
        <Tooltip>
          <TooltipTrigger asChild>
            <button
              onClick={handleSplitRight}
              className="p-1.5 rounded-md transition-colors duration-200 text-[var(--app-text-secondary)] hover:bg-[var(--app-hover)]"
            >
              <PanelRight className="w-[14px] h-[14px]" />
            </button>
          </TooltipTrigger>
          <TooltipContent side="bottom">
            <p>
              {t("splitRight")}
              {getShortcut("split-right") && (
                <kbd className="ml-1.5 px-1 py-0.5 text-[11px] font-mono bg-white/10 border border-white/15 rounded opacity-80">
                  {getShortcut("split-right")}
                </kbd>
              )}
            </p>
          </TooltipContent>
        </Tooltip>

        <Tooltip>
          <TooltipTrigger asChild>
            <button
              onClick={handleSplitDown}
              className="p-1.5 rounded-md transition-colors duration-200 text-[var(--app-text-secondary)] hover:bg-[var(--app-hover)]"
            >
              <PanelBottom className="w-[14px] h-[14px]" />
            </button>
          </TooltipTrigger>
          <TooltipContent side="bottom">
            <p>
              {t("splitDown")}
              {getShortcut("split-down") && (
                <kbd className="ml-1.5 px-1 py-0.5 text-[11px] font-mono bg-white/10 border border-white/15 rounded opacity-80">
                  {getShortcut("split-down")}
                </kbd>
              )}
            </p>
          </TooltipContent>
        </Tooltip>

        {/* 退出无边框按钮 */}
        {isBorderless && (
          <Tooltip>
            <TooltipTrigger asChild>
              <button
                onClick={() => exitBorderless()}
                className="p-1.5 rounded-md transition-colors duration-200 text-[var(--app-text-secondary)] hover:bg-[var(--app-hover)]"
              >
                <Maximize2 className="w-[14px] h-[14px]" />
              </button>
            </TooltipTrigger>
            <TooltipContent side="bottom">
              <p>{t("exitBorderless")}</p>
            </TooltipContent>
          </Tooltip>
        )}

        {/* 分隔线 */}
        <div
          className="w-px h-3.5 mx-2"
          style={{ background: "var(--app-border)" }}
        />

        {/* 语言切换 */}
        <Tooltip>
          <TooltipTrigger asChild>
            <button
              onClick={handleToggleLanguage}
              className="p-1.5 rounded-md transition-colors duration-200 text-[var(--app-text-secondary)] hover:bg-[var(--app-hover)]"
            >
              <Globe className="w-[14px] h-[14px]" />
            </button>
          </TooltipTrigger>
          <TooltipContent side="bottom">
            <p>{t("switchLanguage")} ({i18n.language === "zh-CN" ? "EN" : "中文"})</p>
          </TooltipContent>
        </Tooltip>

        {/* 主题切换 */}
        <button
          onClick={toggleTheme}
          className={`p-1.5 rounded-md transition-colors duration-200 hover:bg-[var(--app-hover)] ${
            isDark ? "text-amber-400" : "text-[var(--app-text-tertiary)]"
          }`}
          title={isDark ? t("switchToLight", { ns: "dialogs" }) : t("switchToDark", { ns: "dialogs" })}
        >
          {isDark ? <Sun className="w-[14px] h-[14px]" /> : <Moon className="w-[14px] h-[14px]" />}
        </button>

        {/* 分隔线 */}
        <div
          className="w-px h-3.5 mx-2"
          style={{ background: "var(--app-border)" }}
        />

        {/* 窗口控件 */}
        <div className="flex items-center -mr-1">
          <button
            className="w-[34px] h-[28px] flex items-center justify-center rounded-[4px] transition-colors duration-200 text-[var(--app-text-secondary)] hover:bg-[var(--app-hover)]"
            onClick={minimizeWindow}
            title={t("minimize", { ns: "common" })}
          >
            <Minus className="w-[14px] h-[14px]" />
          </button>
          <button
            className="w-[34px] h-[28px] flex items-center justify-center rounded-[4px] transition-colors duration-200 text-[var(--app-text-secondary)] hover:bg-[var(--app-hover)]"
            onClick={maximizeWindow}
            title={isMaximized ? t("restoreWindow", { ns: "common" }) : t("maximize", { ns: "common" })}
          >
            {isMaximized ? <Copy className="w-3 h-3" /> : <Square className="w-3 h-3" />}
          </button>
          <button
            className="w-[34px] h-[28px] flex items-center justify-center rounded-[4px] transition-colors duration-200 text-[var(--app-text-secondary)] hover:bg-red-500/85 hover:text-white"
            onClick={closeWindow}
            title={t("close", { ns: "common" })}
          >
            <X className="w-[14px] h-[14px]" />
          </button>
        </div>
      </div>
    </div>
  );
}
