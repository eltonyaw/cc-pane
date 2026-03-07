import {
  Command, FolderTree, Search, History, Bot, ListTodo, Settings,
} from "lucide-react";
import { useTranslation } from "react-i18next";
import {
  Tooltip,
  TooltipContent,
  TooltipTrigger,
} from "@/components/ui/tooltip";
import { useActivityBarStore, type ActivityView } from "@/stores/useActivityBarStore";
import { useDialogStore, useThemeStore } from "@/stores";

interface ActivityBarIconProps {
  icon: React.ReactNode;
  label: string;
  active: boolean;
  onClick: () => void;
}

function ActivityBarIcon({ icon, label, active, onClick }: ActivityBarIconProps) {
  const isDark = useThemeStore((s) => s.isDark);

  return (
    <Tooltip>
      <TooltipTrigger asChild>
        <button
          className={`relative w-full h-[40px] flex items-center justify-center transition-colors duration-150 ${
            active
              ? (isDark ? "text-white" : "text-[var(--app-text-primary)]")
              : (isDark ? "text-slate-500 hover:text-slate-300" : "text-slate-400 hover:text-slate-600")
          }`}
          style={{
            background: active ? "var(--app-hover)" : undefined,
          }}
          onClick={onClick}
        >
          {/* 左侧高亮条 */}
          {active && (
            <div
              className="absolute left-0 top-[25%] bottom-[25%] w-[2px] rounded-r"
              style={{ background: "var(--app-accent)" }}
            />
          )}
          {icon}
        </button>
      </TooltipTrigger>
      <TooltipContent side="right" sideOffset={8}>
        <p>{label}</p>
      </TooltipContent>
    </Tooltip>
  );
}

export default function ActivityBar() {
  const { t } = useTranslation("sidebar");
  const isDark = useThemeStore((s) => s.isDark);
  const activeView = useActivityBarStore((s) => s.activeView);
  const sidebarVisible = useActivityBarStore((s) => s.sidebarVisible);
  const toggleView = useActivityBarStore((s) => s.toggleView);
  const appViewMode = useActivityBarStore((s) => s.appViewMode);
  const toggleTodoMode = useActivityBarStore((s) => s.toggleTodoMode);
  const toggleSelfChatMode = useActivityBarStore((s) => s.toggleSelfChatMode);
  const openSettings = useDialogStore((s) => s.openSettings);

  const isViewActive = (view: ActivityView) =>
    activeView === view && sidebarVisible;

  const viewItems: { view: ActivityView; icon: React.ReactNode; label: string }[] = [
    { view: "explorer", icon: <FolderTree className="w-[20px] h-[20px]" />, label: t("workspaces") },
    { view: "search", icon: <Search className="w-[20px] h-[20px]" />, label: t("search", { ns: "common", defaultValue: "Search" }) },
    { view: "sessions", icon: <History className="w-[20px] h-[20px]" />, label: t("recentLaunches") },
  ];

  return (
    <div
      className="activity-bar shrink-0 flex flex-col items-center select-none"
      style={{
        width: 48,
        height: "100%",
        background: isDark
          ? "rgba(15, 23, 42, 0.30)"
          : "rgba(255, 255, 255, 0.35)",
        backdropFilter: `blur(var(--app-glass-blur))`,
        WebkitBackdropFilter: `blur(var(--app-glass-blur))`,
      }}
    >
      {/* Logo */}
      <div className="pt-1 pb-1 flex items-center justify-center">
        <div
          className="w-[28px] h-[28px] rounded-[7px] flex items-center justify-center transition-transform hover:scale-105"
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
      </div>

      {/* Separator */}
      <div
        className="w-6 h-px mx-auto my-1"
        style={{ background: "var(--app-border)" }}
      />

      {/* 视图图标 */}
      <div className="flex flex-col w-full gap-0.5">
        {viewItems.map((item) => (
          <ActivityBarIcon
            key={item.view}
            icon={item.icon}
            label={item.label}
            active={isViewActive(item.view)}
            onClick={() => toggleView(item.view)}
          />
        ))}

        {/* Todo (切换全屏 todo 视图模式) */}
        <ActivityBarIcon
          icon={<ListTodo className="w-[20px] h-[20px]" />}
          label={t("todoList")}
          active={appViewMode === "todo"}
          onClick={toggleTodoMode}
        />

        {/* Self-Chat (项目规划助手) */}
        <ActivityBarIcon
          icon={<Bot className="w-[20px] h-[20px]" />}
          label={t("selfChat", { ns: "common", defaultValue: "Self Chat" })}
          active={appViewMode === "selfchat"}
          onClick={toggleSelfChatMode}
        />
      </div>

      {/* 底部设置 */}
      <div className="mt-auto pb-3 w-full">
        <ActivityBarIcon
          icon={<Settings className="w-[20px] h-[20px]" />}
          label={t("settings", { ns: "common", defaultValue: "Settings" })}
          active={false}
          onClick={openSettings}
        />
      </div>
    </div>
  );
}
