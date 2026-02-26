import { useEffect } from "react";
import { useTranslation } from "react-i18next";
import { Save, X, LayoutTemplate } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import TodoSubtaskList from "./TodoSubtaskList";
import type {
  TodoStatus,
  TodoPriority,
  TodoScope,
  TodoSubtask,
} from "@/types";

export interface TodoEditForm {
  title: string;
  description: string;
  status: TodoStatus;
  priority: TodoPriority;
  scope: TodoScope;
  scopeRef: string;
  tags: string;
  dueDate: string;
  reminderAt: string;
  recurrence: string;
}

interface TodoEditorProps {
  form: TodoEditForm;
  isNew: boolean;
  subtasks: TodoSubtask[];
  onChange: (form: TodoEditForm) => void;
  onSave: () => void;
  onCancel: () => void;
  onToggleSubtask: (id: string) => void;
  onDeleteSubtask: (id: string) => void;
  onAddSubtask: (title: string) => void;
}

function SegmentedControl<T extends string>({
  options,
  value,
  onChange,
}: {
  options: { value: T; label: string }[];
  value: T;
  onChange: (v: T) => void;
}) {
  return (
    <div className="flex p-0.5 bg-muted/50 rounded-lg border border-border/50">
      {options.map((opt) => {
        const isActive = opt.value === value;
        return (
          <button
            key={opt.value}
            onClick={() => onChange(opt.value)}
            className={`flex-1 px-3 py-1.5 text-xs font-medium rounded-md transition-all duration-200
              ${
                isActive
                  ? "bg-background text-foreground shadow-sm ring-1 ring-border/50"
                  : "text-muted-foreground hover:text-foreground hover:bg-muted/80"
              }`}
          >
            {opt.label}
          </button>
        );
      })}
    </div>
  );
}

export default function TodoEditor({
  form,
  isNew,
  subtasks,
  onChange,
  onSave,
  onCancel,
  onToggleSubtask,
  onDeleteSubtask,
  onAddSubtask,
}: TodoEditorProps) {
  const { t } = useTranslation("dialogs");

  const STATUS_OPTIONS: { value: TodoStatus; label: string }[] = [
    { value: "todo", label: t("todoTodo") },
    { value: "in_progress", label: t("todoInProgress") },
    { value: "done", label: t("todoDone") },
  ];

  const PRIORITY_OPTIONS: { value: TodoPriority; label: string }[] = [
    { value: "high", label: t("todoPriorityHigh") },
    { value: "medium", label: t("todoPriorityMedium") },
    { value: "low", label: t("todoPriorityLow") },
  ];

  const SCOPE_OPTIONS: { value: TodoScope; label: string }[] = [
    { value: "global", label: t("todoScopeGlobal") },
    { value: "workspace", label: t("todoScopeWorkspace") },
    { value: "project", label: t("todoScopeProject") },
    { value: "external", label: t("todoScopeExternal") },
    { value: "temp_script", label: t("todoScopeScript") },
  ];

  // Ctrl+S 保存
  useEffect(() => {
    function handleKeydown(e: KeyboardEvent) {
      if ((e.ctrlKey || e.metaKey) && e.key === "s") {
        e.preventDefault();
        onSave();
      }
    }
    document.addEventListener("keydown", handleKeydown);
    return () => document.removeEventListener("keydown", handleKeydown);
  }, [onSave]);

  const needsScopeRef = form.scope === "workspace" || form.scope === "project";

  return (
    <div className="flex flex-col h-full">
      {/* 头部工具栏 */}
      <header className="flex items-center justify-between px-5 py-3 border-b border-border/50 bg-background/95 backdrop-blur">
        <div className="flex items-center gap-2 text-muted-foreground">
          <LayoutTemplate className="w-4 h-4" />
          <span className="text-xs font-medium">
            {isNew ? t("todoNewTask") : t("todoDetail")}
          </span>
        </div>
        <div className="flex items-center gap-1.5">
          <Button
            size="sm"
            variant="ghost"
            onClick={onCancel}
            className="h-7 text-xs"
          >
            <X size={14} className="mr-1" /> {t("cancel", { ns: "common" })}
          </Button>
          <Button
            size="sm"
            onClick={onSave}
            disabled={!form.title.trim()}
            className="h-7 text-xs"
          >
            <Save size={14} className="mr-1" /> {isNew ? t("create", { ns: "common" }) : t("save", { ns: "common" })}
          </Button>
        </div>
      </header>

      {/* 滚动内容区 */}
      <div className="flex-1 overflow-y-auto">
        {/* 标题 - 沉浸式大输入框 */}
        <div className="px-5 pt-5 pb-3">
          <input
            type="text"
            value={form.title}
            onChange={(e) => onChange({ ...form, title: e.target.value })}
            placeholder={t("todoTitlePlaceholder")}
            className="w-full text-lg font-semibold bg-transparent border-none placeholder:text-muted-foreground/40 focus:outline-none"
          />
        </div>

        {/* 属性区域 */}
        <div className="px-5 pb-5 space-y-4">
          {/* 状态 + 优先级 分段控制器 */}
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1.5">
              <label className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
                {t("todoStatus")}
              </label>
              <SegmentedControl
                options={STATUS_OPTIONS}
                value={form.status}
                onChange={(v) => onChange({ ...form, status: v })}
              />
            </div>
            <div className="space-y-1.5">
              <label className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
                {t("todoPriority")}
              </label>
              <SegmentedControl
                options={PRIORITY_OPTIONS}
                value={form.priority}
                onChange={(v) => onChange({ ...form, priority: v })}
              />
            </div>
          </div>

          {/* 作用域 + 到期日 */}
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1.5">
              <label className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
                {t("todoScope")}
              </label>
              <SegmentedControl
                options={SCOPE_OPTIONS}
                value={form.scope}
                onChange={(v) => onChange({ ...form, scope: v })}
              />
            </div>
            <div className="space-y-1.5">
              <label className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
                {t("todoDueDate")}
              </label>
              <Input
                type="date"
                value={form.dueDate ? form.dueDate.split("T")[0] : ""}
                onChange={(e) =>
                  onChange({
                    ...form,
                    dueDate: e.target.value
                      ? new Date(e.target.value).toISOString()
                      : "",
                  })
                }
                className="h-8 text-xs bg-muted/30 border-border/50 focus:bg-background"
              />
            </div>
          </div>

          {/* 提醒 + 重复 */}
          <div className="grid grid-cols-2 gap-4">
            <div className="space-y-1.5">
              <label className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
                {t("todoReminderAt")}
              </label>
              <Input
                type="datetime-local"
                value={form.reminderAt ? form.reminderAt.slice(0, 16) : ""}
                onChange={(e) =>
                  onChange({
                    ...form,
                    reminderAt: e.target.value
                      ? new Date(e.target.value).toISOString()
                      : "",
                  })
                }
                className="h-8 text-xs bg-muted/30 border-border/50 focus:bg-background"
              />
            </div>
            <div className="space-y-1.5">
              <label className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
                {t("todoRecurrence")}
              </label>
              <select
                value={form.recurrence || ""}
                onChange={(e) =>
                  onChange({ ...form, recurrence: e.target.value })
                }
                className="h-8 w-full text-xs rounded-md bg-muted/30 border border-border/50 px-2 focus:bg-background focus:border-primary/50 focus:ring-2 focus:ring-primary/20 transition-all outline-none"
              >
                <option value="">{t("todoRecurrenceNone")}</option>
                <option value="daily">{t("todoRecurrenceDaily")}</option>
                <option value="weekly">{t("todoRecurrenceWeekly")}</option>
                <option value="monthly">{t("todoRecurrenceMonthly")}</option>
              </select>
            </div>
          </div>

          {/* scopeRef */}
          {needsScopeRef && (
            <div className="space-y-1.5">
              <label className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
                {t("todoScopeRef")}
              </label>
              <Input
                value={form.scopeRef}
                onChange={(e) => onChange({ ...form, scopeRef: e.target.value })}
                placeholder={t("todoScopeRefPlaceholder")}
                className="h-8 text-xs bg-muted/30 border-border/50 focus:bg-background"
              />
            </div>
          )}

          {/* 标签 */}
          <div className="space-y-1.5">
            <label className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
              {t("todoTags")}
            </label>
            <Input
              value={form.tags}
              onChange={(e) => onChange({ ...form, tags: e.target.value })}
              placeholder={t("todoTagsPlaceholder")}
              className="h-8 text-xs bg-muted/30 border-border/50 focus:bg-background"
            />
          </div>

          {/* 描述 */}
          <div className="space-y-1.5">
            <label className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
              {t("todoDescription")}
            </label>
            <textarea
              value={form.description}
              onChange={(e) =>
                onChange({ ...form, description: e.target.value })
              }
              className="w-full min-h-[120px] p-3 rounded-lg bg-muted/30 border border-border/50 text-sm font-mono
                         focus:bg-background focus:border-primary/50 focus:ring-2 focus:ring-primary/20
                         transition-all outline-none resize-y placeholder:text-muted-foreground/40"
              placeholder={t("todoDescPlaceholder")}
              spellCheck={false}
            />
          </div>

          {/* 子任务 - 仅编辑模式显示 */}
          {!isNew && (
            <div className="space-y-1.5">
              <label className="text-[10px] font-semibold text-muted-foreground uppercase tracking-wider">
                {t("todoSubtasks")}
              </label>
              <TodoSubtaskList
                subtasks={subtasks}
                onToggle={onToggleSubtask}
                onDelete={onDeleteSubtask}
                onAdd={onAddSubtask}
              />
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
