import type { Project } from "@/types";

let counter = 0;

function nextId(): string {
  counter += 1;
  return `test-${counter}`;
}

/**
 * 创建测试用 Project 数据
 */
export function createTestProject(overrides?: Partial<Project>): Project {
  const id = nextId();
  return {
    id,
    name: `project-${id}`,
    path: `/tmp/test/${id}`,
    created_at: new Date().toISOString(),
    ...overrides,
  };
}

/**
 * 创建多个测试 Project
 */
export function createTestProjects(count: number): Project[] {
  return Array.from({ length: count }, () => createTestProject());
}

/**
 * 重置计数器（在 beforeEach 中调用）
 */
export function resetTestDataCounter(): void {
  counter = 0;
}
