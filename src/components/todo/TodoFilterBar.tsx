import { useTranslation } from "react-i18next";
import { Search, ListFilter } from "lucide-react";
import { Input } from "@/components/ui/input";
import type { TodoStatus, TodoPriority, TodoScope } from "@/types";

interface TodoFilterBarProps {
  filterStatus: TodoStatus | null;
  filterPriority: TodoPriority | null;
  filterScope: TodoScope | null;
  searchText: string;
  onStatusChange: (status: TodoStatus | null) => void;
  onPriorityChange: (priority: TodoPriority | null) => void;
  onScopeChange: (scope: TodoScope | null) => void;
  onSearchChange: (text: string) => void;
}

function SegmentedControl<T>({
  options,
  value,
  onChange,
}: {
  options: { value: T; label: string }[];
  value: T;
  onChange: (v: T) => void;
}) {
  return (
    <div className="flex bg-muted/40 p-0.5 rounded-md border border-border/50">
      {options.map((opt) => {
        const isActive = opt.value === value;
        return (
          <button
            key={String(opt.label)}
            onClick={() => onChange(opt.value)}
            className={`px-2 py-0.5 text-[10px] font-medium rounded-sm transition-all duration-200
              ${
                isActive
                  ? "bg-background text-foreground shadow-sm"
                  : "text-muted-foreground hover:text-foreground hover:bg-muted/60"
              }`}
          >
            {opt.label}
          </button>
        );
      })}
    </div>
  );
}

export default function TodoFilterBar({
  filterStatus,
  filterPriority,
  filterScope,
  searchText,
  onStatusChange,
  onPriorityChange,
  onScopeChange,
  onSearchChange,
}: TodoFilterBarProps) {
  const { t } = useTranslation("dialogs");

  const STATUS_OPTIONS: { value: TodoStatus | null; label: string }[] = [
    { value: null, label: t("todoAll") },
    { value: "todo", label: t("todoTodo") },
    { value: "in_progress", label: t("todoInProgress") },
    { value: "done", label: t("todoDone") },
  ];

  const PRIORITY_OPTIONS: { value: TodoPriority | null; label: string }[] = [
    { value: null, label: t("todoAll") },
    { value: "high", label: t("todoPriorityHigh") },
    { value: "medium", label: t("todoPriorityMedium") },
    { value: "low", label: t("todoPriorityLow") },
  ];

  const SCOPE_OPTIONS: { value: TodoScope | null; label: string }[] = [
    { value: null, label: t("todoAll") },
    { value: "global", label: t("todoScopeGlobal") },
    { value: "workspace", label: t("todoScopeWorkspaceShort") },
    { value: "project", label: t("todoScopeProject") },
    { value: "external", label: t("todoScopeExternal") },
    { value: "temp_script", label: t("todoScopeScript") },
  ];

  return (
    <div className="px-3 py-2.5 border-b border-border/50 space-y-2.5">
      {/* 搜索框 */}
      <div className="relative group">
        <Search
          size={14}
          className="absolute left-2.5 top-1/2 -translate-y-1/2 text-muted-foreground group-focus-within:text-primary transition-colors"
        />
        <Input
          value={searchText}
          onChange={(e) => onSearchChange(e.target.value)}
          placeholder={t("todoSearchPlaceholder")}
          className="h-8 text-sm pl-8 bg-muted/30 border-transparent focus:bg-background focus:border-primary/50 transition-all"
        />
      </div>

      {/* 筛选器 - 紧凑横向布局 */}
      <div className="flex items-center gap-2 overflow-x-auto pb-0.5">
        <div className="flex items-center text-muted-foreground shrink-0">
          <ListFilter className="w-3 h-3 mr-1" />
        </div>

        <SegmentedControl
          options={STATUS_OPTIONS}
          value={filterStatus}
          onChange={onStatusChange}
        />

        <SegmentedControl
          options={PRIORITY_OPTIONS}
          value={filterPriority}
          onChange={onPriorityChange}
        />
      </div>

      {/* 作用域单独一行（选项较多） */}
      <div className="flex items-center gap-2 overflow-x-auto">
        <SegmentedControl
          options={SCOPE_OPTIONS}
          value={filterScope}
          onChange={onScopeChange}
        />
      </div>
    </div>
  );
}
