import { useTranslation } from "react-i18next";
import { ListTodo } from "lucide-react";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Badge } from "@/components/ui/badge";
import TodoManager from "@/components/todo/TodoManager";

interface TodoPanelProps {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  scope: string;
  scopeRef: string;
}

export default function TodoPanel({
  open,
  onOpenChange,
  scope,
  scopeRef,
}: TodoPanelProps) {
  const { t } = useTranslation(["sidebar", "common"]);

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="max-w-[80rem] w-[90vw] h-[85vh] flex flex-col overflow-hidden p-0">
        <DialogHeader className="px-6 pt-5 pb-3 border-b shrink-0">
          <DialogTitle className="flex items-center gap-2">
            <ListTodo size={18} />
            {t("todoList")}
            {scope && (
              <Badge variant="outline" className="text-xs font-normal">
                {scope}: {scopeRef}
              </Badge>
            )}
          </DialogTitle>
        </DialogHeader>
        <div className="flex-1 overflow-auto px-6 py-4">
          <TodoManager scope={scope} scopeRef={scopeRef} />
        </div>
      </DialogContent>
    </Dialog>
  );
}
