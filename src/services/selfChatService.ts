/**
 * Self-Chat 服务
 *
 * 提供应用级上下文收集（工作空间、Todo、可用 Skill），
 * 以及 getAppCwd() 获取 CC-Panes 项目根目录。
 */
import { invoke } from "@tauri-apps/api/core";
import { useWorkspacesStore } from "@/stores";
import { todoService } from "./todoService";

/** 获取应用当前工作目录 */
async function getAppCwd(): Promise<string> {
  return invoke<string>("get_app_cwd");
}

/** 收集应用级上下文并组装 prompt */
async function collectAppContext(): Promise<string> {
  const sections: string[] = [];

  // 1. 工作空间概览
  const workspaces = useWorkspacesStore.getState().workspaces;
  if (workspaces.length > 0) {
    const wsLines = workspaces.map(
      (ws) => `- ${ws.alias || ws.name}（${ws.projects.length} 个项目）`
    );
    sections.push(`## 工作空间 (${workspaces.length} 个)\n${wsLines.join("\n")}`);
  }

  // 2. Todo 统计（全局待办）
  try {
    const result = await todoService.query({
      status: "todo",
      limit: 10,
      offset: 0,
    });
    if (result.items.length > 0) {
      const todoLines = result.items.map(
        (t) => `- [${t.priority}] ${t.title}${t.description ? ` — ${t.description.slice(0, 80)}` : ""}`
      );
      sections.push(
        `## 待办事项 (${result.total} 项)\n${todoLines.join("\n")}` +
          (result.total > 10 ? `\n- ... 还有 ${result.total - 10} 项` : "")
      );
    }
  } catch {
    // 无 todo，跳过
  }

  // 3. 可用 Skill 列表
  sections.push(
    `## 可用 Skill\n` +
    `- /ccbook:workspace — 工作空间管理（CRUD、项目管理）\n` +
    `- /ccbook:start — 启动会话\n` +
    `- /ccbook:check-backend — 后端代码检查\n` +
    `- /ccbook:check-frontend — 前端代码检查\n` +
    `- /ccbook:check-cross-layer — 跨层检查\n` +
    `- /ccbook:check-tauri-bridge — Tauri 桥接一致性检查\n` +
    `- /ccbook:finish-work — 完成工作检查清单\n` +
    `- /ccbook:onboard — 项目导师介绍\n` +
    `- /ccbook:parallel — 多 Agent 并行编排`
  );

  // 4. 系统提示
  const systemHint =
    `你是 CC-Panes 的操控助手。CC-Panes 是一个 Claude Code 多实例分屏管理桌面应用。\n` +
    `你可以使用上面列出的 /ccbook:* skill 来帮助用户管理工作空间、Todo、Plans 等。\n` +
    `请用中文回复。`;

  const contextBlock = sections.length > 0
    ? `${sections.join("\n\n")}\n\n${systemHint}`
    : systemHint;

  return contextBlock + "\n\n请确认你已理解上述应用上下文，然后等待用户的具体问题。";
}

export const selfChatService = {
  getAppCwd,
  collectAppContext,
};
