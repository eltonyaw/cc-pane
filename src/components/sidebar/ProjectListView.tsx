import { useCallback } from "react";
import { useTranslation } from "react-i18next";
import { openPath } from "@tauri-apps/plugin-opener";
import { toast } from "sonner";
import {
  Folder, ChevronRight, Trash2, Plus, Pencil, Clock,
  FolderOpen, Terminal, GitBranch, Copy,
} from "lucide-react";
import {
  ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger,
  ContextMenuSeparator, ContextMenuSub, ContextMenuSubTrigger, ContextMenuSubContent,
} from "@/components/ui/context-menu";
import { useProvidersStore, useThemeStore, useDialogStore } from "@/stores";
import { FileTree } from "@/components/filetree";
import { getProjectName } from "@/utils";
import type { Workspace, WorkspaceProject } from "@/types";

interface ProjectListViewProps {
  projects: WorkspaceProject[];
  ws: Workspace;
  expandedProjectId: string | null;
  gitBranches: Record<string, string | null>;
  onExpandProject: (projectId: string) => void;
  onOpenTerminal: (path: string, workspaceName?: string, providerId?: string, workspacePath?: string, launchClaude?: boolean) => void;
  onRemoveProject: (ws: Workspace, project: WorkspaceProject) => void;
  onSetProjectAlias: (ws: Workspace, project: WorkspaceProject) => void;
  onImportProject: (ws: Workspace) => void;
  onOpenWorktreeManager: (project: WorkspaceProject, ws: Workspace) => void;
}

function getRelativePath(projectPath: string, wsPath?: string | null): string {
  const normalize = (p: string) => p.replace(/\\/g, "/").replace(/\/+$/, "");
  if (wsPath) {
    const normBase = normalize(wsPath);
    const normFull = normalize(projectPath);
    if (normFull.startsWith(normBase + "/")) {
      return normFull.slice(normBase.length + 1);
    }
  }
  const parts = projectPath.replace(/\\/g, "/").split("/").filter(Boolean);
  return parts.pop() || projectPath;
}

export default function ProjectListView({
  projects, ws, expandedProjectId, gitBranches,
  onExpandProject, onOpenTerminal, onRemoveProject, onSetProjectAlias,
  onImportProject, onOpenWorktreeManager,
}: ProjectListViewProps) {
  const { t } = useTranslation(["sidebar", "common"]);
  const isDark = useThemeStore((s) => s.isDark);
  const providerList = useProvidersStore((s) => s.providers);
  const onOpenHistory = useDialogStore((s) => s.openLocalHistory);

  const handleRevealFolder = useCallback(async (path: string) => {
    try {
      await openPath(path);
    } catch (e) {
      toast.error(t("openFolderFailed", { error: e }));
    }
  }, [t]);

  const handleCopyPath = useCallback(async (path: string) => {
    try {
      await navigator.clipboard.writeText(path);
      toast.success(t("copiedToClipboard"));
    } catch (e) {
      toast.error(t("copyFailed", { error: e }));
    }
  }, [t]);

  return (
    <div className="pl-4 pr-1 pb-2 flex flex-col gap-0.5">
      {projects.map((project) => (
        <div key={project.id}>
          <ContextMenu>
            <ContextMenuTrigger asChild>
              <div
                className={`flex items-center gap-1.5 px-2 py-1.5 cursor-pointer rounded-lg transition-all ${
                  expandedProjectId === project.id
                    ? isDark
                      ? 'bg-white/5 text-slate-200'
                      : 'bg-white/40 text-slate-800 shadow-sm'
                    : isDark
                      ? 'text-slate-400 hover:bg-white/5 hover:text-slate-200'
                      : 'text-slate-500 hover:bg-white/30 hover:text-slate-800'
                }`}
                onClick={() => onExpandProject(project.id)}
                onDoubleClick={() => onOpenTerminal(project.path, ws.name, ws.providerId)}
              >
                <ChevronRight className={`w-3 h-3 shrink-0 transition-transform ${expandedProjectId === project.id ? 'rotate-90' : ''}`} />
                <Folder size={14} className="shrink-0" style={{ color: "var(--app-accent)" }} />
                <span className="flex-1 text-xs truncate">{project.alias || getProjectName(project.path)}</span>
                {gitBranches[project.path] && (
                  <span className="text-[10px] px-1 rounded shrink-0" style={{ color: "var(--app-accent)", background: "var(--app-active-bg)" }}>
                    {gitBranches[project.path]}
                  </span>
                )}
              </div>
            </ContextMenuTrigger>
            <ContextMenuContent className="w-48">
              <ContextMenuItem onClick={() => onOpenTerminal(project.path, ws.name, ws.providerId)}>
                <Terminal /> {t("openTerminal")}
              </ContextMenuItem>
              <ContextMenuSub>
                <ContextMenuSubTrigger>
                  <Terminal /> {t("openClaudeCode")}
                </ContextMenuSubTrigger>
                <ContextMenuSubContent className="w-48">
                  <ContextMenuItem onClick={() => onOpenTerminal(project.path, ws.name, ws.providerId, ws.path, true)}>
                    {t("useWorkspaceProvider")}
                    {ws.providerId && providerList.find(p => p.id === ws.providerId) && (
                      <span className="ml-auto text-[10px] opacity-60">
                        {providerList.find(p => p.id === ws.providerId)?.name}
                      </span>
                    )}
                  </ContextMenuItem>
                  {providerList.length > 0 && <ContextMenuSeparator />}
                  {providerList.map((p) => (
                    <ContextMenuItem
                      key={p.id}
                      onClick={() => onOpenTerminal(project.path, ws.name, p.id, ws.path, true)}
                    >
                      {p.name}
                    </ContextMenuItem>
                  ))}
                </ContextMenuSubContent>
              </ContextMenuSub>
              <ContextMenuItem onClick={() => handleRevealFolder(project.path)}>
                <FolderOpen /> {t("openFolder")}
              </ContextMenuItem>
              <ContextMenuSub>
                <ContextMenuSubTrigger>
                  <Copy /> {t("copyPath")}
                </ContextMenuSubTrigger>
                <ContextMenuSubContent>
                  <ContextMenuItem onClick={() => handleCopyPath(project.path)}>
                    {t("absolutePath")}
                  </ContextMenuItem>
                  <ContextMenuItem onClick={() => handleCopyPath(getRelativePath(project.path, ws.path))}>
                    {t("relativePath")}
                  </ContextMenuItem>
                </ContextMenuSubContent>
              </ContextMenuSub>
              <ContextMenuSeparator />
              <ContextMenuItem onClick={() => onOpenHistory(project.path)}>
                <Clock /> {t("fileHistory")}
              </ContextMenuItem>
              <ContextMenuItem onClick={() => onOpenWorktreeManager(project, ws)}>
                <GitBranch /> {t("worktreeManager")}
              </ContextMenuItem>
              <ContextMenuSeparator />
              <ContextMenuItem onClick={() => onSetProjectAlias(ws, project)}>
                <Pencil /> {t("setAlias")}
              </ContextMenuItem>
              <ContextMenuItem variant="destructive" onClick={() => onRemoveProject(ws, project)}>
                <Trash2 /> {t("removeProject")}
              </ContextMenuItem>
            </ContextMenuContent>
          </ContextMenu>

          {/* 文件树 */}
          {expandedProjectId === project.id && (
            <div className="ml-4 py-1 max-h-64 overflow-y-auto">
              <FileTree
                rootPath={project.path}
                compact={true}
                showSearch={true}
                onOpenTerminal={(path) => onOpenTerminal(path, ws.name, ws.providerId)}
              />
            </div>
          )}
        </div>
      ))}

      {/* 导入项目按钮 */}
      <div
        className={`flex items-center justify-center gap-1 p-1.5 mt-1 text-[11px] rounded-lg cursor-pointer transition-all border border-dashed group ${
          isDark
            ? 'border-white/10 text-slate-400 hover:border-blue-500/50 hover:text-blue-300 hover:bg-blue-500/10'
            : 'border-slate-400/30 text-slate-500 hover:border-blue-400 hover:text-blue-600 hover:bg-blue-50/50'
        }`}
        onClick={() => onImportProject(ws)}
      >
        <Plus size={12} className="transition-transform group-hover:rotate-90" />
        <span>{t("importProject")}</span>
      </div>
    </div>
  );
}
