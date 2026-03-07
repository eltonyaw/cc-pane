import { Minus, Square, Copy, X } from "lucide-react";
import { useTranslation } from "react-i18next";
import { useThemeStore, useBorderlessStore } from "@/stores";
import { useWindowControl } from "@/hooks/useWindowControl";

interface TitleBarProps {
  workspaceName?: string;
}

export default function TitleBar({ workspaceName }: TitleBarProps) {
  const { t } = useTranslation("common");
  const isDark = useThemeStore((s) => s.isDark);
  const isBorderless = useBorderlessStore((s) => s.isBorderless);
  const { closeWindow, minimizeWindow, maximizeWindow, isMaximized, startDrag } = useWindowControl();

  // 无边框模式时隐藏标题栏
  if (isBorderless) return null;

  return (
    <div
      className="relative flex items-center h-[32px] px-3 shrink-0 select-none z-10"
      style={{
        background: "var(--app-menubar)",
        borderBottom: "1px solid var(--app-glass-border)",
        backdropFilter: `blur(var(--app-glass-blur))`,
        WebkitBackdropFilter: `blur(var(--app-glass-blur))`,
      }}
    >
      {/* 顶部高光线 */}
      <div
        className="absolute top-0 left-0 right-0 h-px pointer-events-none"
        style={{
          background: isDark
            ? "linear-gradient(90deg, transparent, rgba(255,255,255,0.08) 20%, rgba(255,255,255,0.12) 50%, rgba(255,255,255,0.08) 80%, transparent)"
            : "linear-gradient(90deg, transparent, rgba(255,255,255,0.6) 20%, rgba(255,255,255,0.8) 50%, rgba(255,255,255,0.6) 80%, transparent)",
        }}
      />

      {/* 左侧：工作空间名 */}
      <div className="flex items-center gap-2 shrink-0 min-w-0">
        <span
          className="text-[12px] font-medium truncate max-w-[200px]"
          style={{ color: "var(--app-text-secondary)" }}
        >
          {workspaceName || "CC-Panes"}
        </span>
      </div>

      {/* 中间：拖拽区 */}
      <div
        className="flex-1 h-full cursor-grab"
        style={{ WebkitAppRegion: "drag" } as React.CSSProperties}
        onMouseDown={(e) => { e.preventDefault(); startDrag(); }}
      />

      {/* 右侧：窗口控件 */}
      <div className="flex items-center -mr-1 shrink-0">
        <button
          className="w-[34px] h-[28px] flex items-center justify-center rounded-[4px] transition-colors duration-200 text-[var(--app-text-secondary)] hover:bg-[var(--app-hover)]"
          onClick={minimizeWindow}
          title={t("minimize")}
        >
          <Minus className="w-[13px] h-[13px]" />
        </button>
        <button
          className="w-[34px] h-[28px] flex items-center justify-center rounded-[4px] transition-colors duration-200 text-[var(--app-text-secondary)] hover:bg-[var(--app-hover)]"
          onClick={maximizeWindow}
          title={isMaximized ? t("restoreWindow") : t("maximize")}
        >
          {isMaximized ? <Copy className="w-3 h-3" /> : <Square className="w-3 h-3" />}
        </button>
        <button
          className="w-[34px] h-[28px] flex items-center justify-center rounded-[4px] transition-colors duration-200 text-[var(--app-text-secondary)] hover:bg-red-500/85 hover:text-white"
          onClick={closeWindow}
          title={t("close")}
        >
          <X className="w-[13px] h-[13px]" />
        </button>
      </div>
    </div>
  );
}
