export { formatRelativeTime, formatFullTime, formatSize } from "./format";
export { getFileName, getDirName, getProjectName } from "./path";

/** Tauri IPC 桥接是否已注入（在 Tauri webview 内运行时为 true） */
export function isTauriReady(): boolean {
  return typeof window !== "undefined" && window.__TAURI_INTERNALS__ !== undefined;
}

/**
 * 等待 Tauri IPC 桥接就绪（最多等待 5 秒）。
 * 在 HMR 热重载时 IPC 可能短暂不可用，此函数会轮询等待。
 */
export function waitForTauri(timeoutMs = 5000): Promise<boolean> {
  if (isTauriReady()) return Promise.resolve(true);
  return new Promise((resolve) => {
    const start = Date.now();
    const check = () => {
      if (isTauriReady()) {
        resolve(true);
      } else if (Date.now() - start > timeoutMs) {
        resolve(false);
      } else {
        setTimeout(check, 50);
      }
    };
    check();
  });
}
