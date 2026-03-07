import { create } from "zustand";
import { persist } from "zustand/middleware";

export type ActivityView = "explorer" | "search" | "sessions";
export type AppViewMode = "panes" | "todo" | "selfchat";

interface ActivityBarState {
  activeView: ActivityView;
  sidebarVisible: boolean;
  appViewMode: AppViewMode;

  toggleView: (view: ActivityView) => void;
  setSidebarVisible: (visible: boolean) => void;
  toggleSidebar: () => void;
  setAppViewMode: (mode: AppViewMode) => void;
  toggleTodoMode: () => void;
  toggleSelfChatMode: () => void;
}

export const useActivityBarStore = create<ActivityBarState>()(
  persist(
    (set, get) => ({
      activeView: "explorer",
      sidebarVisible: true,
      appViewMode: "panes",

      toggleView: (view: ActivityView) => {
        const state = get();
        // 如果当前在非 panes 模式（todo/selfchat）→ 退回 panes 并切到该 view
        if (state.appViewMode !== "panes") {
          set({ appViewMode: "panes", activeView: view, sidebarVisible: true });
          return;
        }
        if (state.activeView === view) {
          // 点击当前视图 → 折叠/展开
          set({ sidebarVisible: !state.sidebarVisible });
        } else {
          // 切换到新视图 → 展开
          set({ activeView: view, sidebarVisible: true });
        }
      },

      setSidebarVisible: (visible: boolean) => set({ sidebarVisible: visible }),

      toggleSidebar: () => set((s) => ({ sidebarVisible: !s.sidebarVisible })),

      setAppViewMode: (mode: AppViewMode) => set({ appViewMode: mode }),

      toggleTodoMode: () =>
        set((s) => ({
          appViewMode: s.appViewMode === "todo" ? "panes" : "todo",
        })),

      toggleSelfChatMode: () =>
        set((s) => ({
          appViewMode: s.appViewMode === "selfchat" ? "panes" : "selfchat",
        })),
    }),
    {
      name: "cc-panes-activity-bar",
      partialize: (state) => ({
        activeView: state.activeView,
        sidebarVisible: state.sidebarVisible,
        // appViewMode 不持久化（每次启动默认回到 panes 模式）
      }),
    }
  )
);
