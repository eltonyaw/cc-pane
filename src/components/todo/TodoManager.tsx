import { useState, useEffect, useCallback } from "react";
import { useTranslation } from "react-i18next";
import { toast } from "sonner";
import { Plus, ListTodo, Trash2, LayoutList, Loader2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { useTodoStore } from "@/stores";
import TodoFilterBar from "./TodoFilterBar";
import TodoListItem from "./TodoListItem";
import TodoEditor from "./TodoEditor";
import type { TodoEditForm } from "./TodoEditor";
import type {
  TodoItem,
  TodoStatus,
  TodoScope,
  CreateTodoRequest,
  UpdateTodoRequest,
} from "@/types";

interface TodoManagerProps {
  /** Tab 的 scope 值（如 "workspace" / "project" / ""） */
  scope: string;
  /** Tab 的 scopeRef 值（如工作空间名或项目路径） */
  scopeRef: string;
}

/** 状态循环：todo → in_progress → done → todo */
function nextStatus(current: TodoStatus): TodoStatus {
  const cycle: TodoStatus[] = ["todo", "in_progress", "done"];
  const idx = cycle.indexOf(current);
  return cycle[(idx + 1) % cycle.length];
}

export default function TodoManager({ scope, scopeRef }: TodoManagerProps) {
  const { t } = useTranslation("dialogs");
  const { t: tNotify } = useTranslation("notifications");

  const todos = useTodoStore((s) => s.todos);
  const total = useTodoStore((s) => s.total);
  const loading = useTodoStore((s) => s.loading);
  const selectedTodo = useTodoStore((s) => s.selectedTodo);
  const filterStatus = useTodoStore((s) => s.filterStatus);
  const filterScope = useTodoStore((s) => s.filterScope);
  const filterPriority = useTodoStore((s) => s.filterPriority);
  const searchText = useTodoStore((s) => s.searchText);
  const loadList = useTodoStore((s) => s.loadList);
  const create = useTodoStore((s) => s.create);
  const update = useTodoStore((s) => s.update);
  const remove = useTodoStore((s) => s.remove);
  const select = useTodoStore((s) => s.select);
  const setFilterStatus = useTodoStore((s) => s.setFilterStatus);
  const setFilterScope = useTodoStore((s) => s.setFilterScope);
  const setFilterPriority = useTodoStore((s) => s.setFilterPriority);
  const setSearchText = useTodoStore((s) => s.setSearchText);
  const setContext = useTodoStore((s) => s.setContext);
  const reset = useTodoStore((s) => s.reset);
  const addSubtask = useTodoStore((s) => s.addSubtask);
  const toggleSubtask = useTodoStore((s) => s.toggleSubtask);
  const deleteSubtask = useTodoStore((s) => s.deleteSubtask);

  const [isCreating, setIsCreating] = useState(false);
  const [editForm, setEditForm] = useState<TodoEditForm>({
    title: "",
    description: "",
    status: "todo",
    priority: "medium",
    scope: "global",
    scopeRef: "",
    tags: "",
    dueDate: "",
  });

  // 初始化：设置上下文并加载
  useEffect(() => {
    const validScope = scope as TodoScope | undefined;
    if (validScope && scopeRef) {
      setContext(validScope, scopeRef);
    }
    loadList();
    return () => reset();
  }, [scope, scopeRef, setContext, loadList, reset]);

  // 搜索去抖
  useEffect(() => {
    const timer = setTimeout(() => {
      loadList();
    }, 300);
    return () => clearTimeout(timer);
  }, [searchText, loadList]);

  // 选中时填充编辑表单
  useEffect(() => {
    if (selectedTodo) {
      setEditForm({
        title: selectedTodo.title,
        description: selectedTodo.description ?? "",
        status: selectedTodo.status,
        priority: selectedTodo.priority,
        scope: selectedTodo.scope,
        scopeRef: selectedTodo.scopeRef ?? "",
        tags: selectedTodo.tags.join(", "),
        dueDate: selectedTodo.dueDate ?? "",
      });
      setIsCreating(false);
    }
  }, [selectedTodo]);

  const handleNew = useCallback(() => {
    select(null);
    setIsCreating(true);
    setEditForm({
      title: "",
      description: "",
      status: "todo",
      priority: "medium",
      scope: "global",
      scopeRef: "",
      tags: "",
      dueDate: "",
    });
  }, [select]);

  const handleSave = useCallback(async () => {
    if (!editForm.title.trim()) {
      toast.error(tNotify("titleRequired"));
      return;
    }

    const tags = editForm.tags
      .split(",")
      .map((t) => t.trim())
      .filter(Boolean);

    try {
      if (isCreating) {
        const request: CreateTodoRequest = {
          title: editForm.title.trim(),
          description: editForm.description || undefined,
          status: editForm.status,
          priority: editForm.priority,
          scope: editForm.scope,
          scopeRef: editForm.scopeRef || undefined,
          tags: tags.length > 0 ? tags : undefined,
          dueDate: editForm.dueDate || undefined,
        };
        await create(request);
        setIsCreating(false);
        toast.success(tNotify("todoCreated"));
      } else if (selectedTodo) {
        const request: UpdateTodoRequest = {
          title: editForm.title.trim(),
          description: editForm.description,
          status: editForm.status,
          priority: editForm.priority,
          scope: editForm.scope,
          scopeRef: editForm.scopeRef || undefined,
          tags,
          dueDate: editForm.dueDate || undefined,
        };
        await update(selectedTodo.id, request);
        toast.success(tNotify("todoUpdated"));
      }
    } catch (e) {
      toast.error(tNotify("operationFailed", { error: String(e) }));
    }
  }, [editForm, isCreating, selectedTodo, create, update]);

  const handleDelete = useCallback(
    async (id: string) => {
      try {
        await remove(id);
        toast.success(tNotify("todoDeleted"));
      } catch (e) {
        toast.error(tNotify("operationFailed", { error: String(e) }));
      }
    },
    [remove]
  );

  const handleCancel = useCallback(() => {
    setIsCreating(false);
    select(null);
  }, [select]);

  const handleToggleStatus = useCallback(
    async (todo: TodoItem) => {
      try {
        await update(todo.id, { status: nextStatus(todo.status) });
      } catch (e) {
        toast.error(tNotify("operationFailed", { error: String(e) }));
      }
    },
    [update]
  );

  const handleAddSubtask = useCallback(
    async (title: string) => {
      if (!selectedTodo) return;
      try {
        await addSubtask(selectedTodo.id, title);
      } catch (e) {
        toast.error(tNotify("operationFailed", { error: String(e) }));
      }
    },
    [selectedTodo, addSubtask]
  );

  const handleToggleSubtask = useCallback(
    async (subtaskId: string) => {
      try {
        await toggleSubtask(subtaskId);
      } catch (e) {
        toast.error(tNotify("operationFailed", { error: String(e) }));
      }
    },
    [toggleSubtask]
  );

  const handleDeleteSubtask = useCallback(
    async (subtaskId: string) => {
      try {
        await deleteSubtask(subtaskId);
      } catch (e) {
        toast.error(tNotify("operationFailed", { error: String(e) }));
      }
    },
    [deleteSubtask]
  );

  const showEditor = isCreating || selectedTodo;

  return (
    <div className="flex h-full">
      {/* 左侧列表面板 */}
      <aside className="w-[340px] flex-shrink-0 border-r border-border bg-background/95 backdrop-blur shadow-[1px_0_8px_rgba(0,0,0,0.02)] flex flex-col z-10">
        {/* 头部 */}
        <div className="px-3 py-2.5 border-b border-border/50">
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <ListTodo size={16} className="text-primary/60" />
              <span className="text-sm font-semibold">TodoList</span>
              <Badge variant="secondary" className="text-[10px] h-5 px-1.5">
                {total}
              </Badge>
            </div>
            <Button
              size="icon"
              variant="ghost"
              className="h-7 w-7 hover:bg-primary/10 hover:text-primary transition-colors"
              onClick={handleNew}
            >
              <Plus size={14} />
            </Button>
          </div>
        </div>

        {/* 筛选栏 */}
        <TodoFilterBar
          filterStatus={filterStatus}
          filterPriority={filterPriority}
          filterScope={filterScope}
          searchText={searchText}
          onStatusChange={setFilterStatus}
          onPriorityChange={setFilterPriority}
          onScopeChange={setFilterScope}
          onSearchChange={setSearchText}
        />

        {/* 列表 */}
        <div className="flex-1 overflow-y-auto py-1">
          {loading && (
            <div className="flex items-center justify-center gap-2 py-8 text-sm text-muted-foreground">
              <Loader2 size={16} className="animate-spin" />
              <span>{t("loading", { ns: "common" })}</span>
            </div>
          )}

          {!loading && todos.length === 0 && (
            <div className="text-center py-12 text-muted-foreground">
              <ListTodo size={28} className="mx-auto mb-3 opacity-40" />
              <p className="text-xs">{t("noTasks")}</p>
              <p className="text-xs mt-1">{t("clickToCreate")}</p>
            </div>
          )}

          {todos.map((todo) => (
            <div key={todo.id} className="group relative">
              <TodoListItem
                todo={todo}
                isSelected={selectedTodo?.id === todo.id}
                onSelect={() => select(todo)}
                onToggleStatus={() => handleToggleStatus(todo)}
              />
              <div className="absolute right-2 top-2 hidden group-hover:flex">
                <Button
                  size="icon"
                  variant="ghost"
                  className="h-6 w-6 text-muted-foreground hover:text-destructive hover:bg-destructive/10 transition-colors"
                  onClick={(e) => {
                    e.stopPropagation();
                    handleDelete(todo.id);
                  }}
                >
                  <Trash2 size={12} />
                </Button>
              </div>
            </div>
          ))}
        </div>
      </aside>

      {/* 右侧编辑器 */}
      <main className="flex-1 overflow-hidden bg-muted/5">
        {showEditor ? (
          <TodoEditor
            form={editForm}
            isNew={isCreating}
            subtasks={selectedTodo?.subtasks ?? []}
            onChange={setEditForm}
            onSave={handleSave}
            onCancel={handleCancel}
            onToggleSubtask={handleToggleSubtask}
            onDeleteSubtask={handleDeleteSubtask}
            onAddSubtask={handleAddSubtask}
          />
        ) : (
          <div className="flex items-center justify-center h-full text-muted-foreground">
            <div className="text-center">
              <LayoutList size={32} className="mx-auto mb-3 opacity-40" />
              <p className="text-sm">{t("selectOrCreateTask")}</p>
              <p className="text-xs mt-1 text-muted-foreground/60">
                {t("todoDesc")}
              </p>
            </div>
          </div>
        )}
      </main>
    </div>
  );
}
