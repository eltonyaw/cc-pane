import i18n from "@/i18n";

/**
 * 后端错误结构
 */
interface BackendError {
  code: string;
  message: string;
  params?: Record<string, string>;
}

/**
 * 尝试解析后端错误并翻译
 *
 * Tauri invoke 的 catch 块收到的 error 可能是：
 * 1. 结构化的 BackendError JSON 字符串 (新格式)
 * 2. 纯文本错误消息 (旧格式/其他错误)
 *
 * @param error - catch 块中的错误对象
 * @returns 翻译后的用户友好错误消息
 */
export function translateError(error: unknown): string {
  const errorStr = typeof error === "string" ? error : String(error);

  // 尝试解析为 BackendError JSON
  try {
    const parsed: BackendError = JSON.parse(errorStr);
    if (parsed.code) {
      const key = `errors:${parsed.code}`;
      // 动态 key 无法通过 i18next 严格类型检查，使用 exists 判断后再翻译
      if (i18n.exists(key)) {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        return i18n.t(key as any, parsed.params ?? {});
      }
      // 翻译 key 不存在，回退到 message
      return parsed.message;
    }
  } catch {
    // 不是 JSON，使用原始字符串
  }

  return errorStr;
}
