import { useTranslation } from "react-i18next";
import {
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogFooter,
} from "@/components/ui/dialog";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import ScanImportDialog from "@/components/ScanImportDialog";
import GitCloneDialog from "@/components/GitCloneDialog";
import type { ScannedRepo } from "@/services/workspaceService";

interface DialogFieldProps {
  open: boolean;
  setOpen: (v: boolean) => void;
}

interface TextDialogProps extends DialogFieldProps {
  title: string;
  placeholder: string;
  value: string;
  setValue: (v: string) => void;
  onConfirm: () => void;
}

function TextInputDialog({ open, setOpen, title, placeholder, value, setValue, onConfirm }: TextDialogProps) {
  const { t } = useTranslation("dialogs");
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent className="sm:max-w-sm">
        <DialogHeader><DialogTitle>{title}</DialogTitle></DialogHeader>
        <div className="py-4">
          <Input
            value={value}
            onChange={(e) => setValue(e.target.value)}
            placeholder={placeholder}
            onKeyDown={(e) => e.key === "Enter" && onConfirm()}
          />
        </div>
        <DialogFooter>
          <Button variant="secondary" onClick={() => setOpen(false)}>{t("cancel", { ns: "common" })}</Button>
          <Button onClick={onConfirm}>{t("confirm", { ns: "common" })}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

interface ConfirmDialogProps extends DialogFieldProps {
  title: string;
  description: string;
  onConfirm: () => void;
  variant?: "default" | "destructive";
}

function ConfirmDialog({ open, setOpen, title, description, onConfirm, variant = "default" }: ConfirmDialogProps) {
  const { t } = useTranslation("dialogs");
  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogContent className="sm:max-w-sm">
        <DialogHeader><DialogTitle>{title}</DialogTitle></DialogHeader>
        <p className="text-sm text-muted-foreground py-2">{description}</p>
        <DialogFooter>
          <Button variant="secondary" onClick={() => setOpen(false)}>{t("cancel", { ns: "common" })}</Button>
          <Button variant={variant === "destructive" ? "destructive" : "default"} onClick={onConfirm}>{t("confirm", { ns: "common" })}</Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}

export interface ConfirmDialogState {
  open: boolean;
  setOpen: (v: boolean) => void;
  title: string;
  description: string;
  onConfirm: () => void;
  variant?: "default" | "destructive";
}

export interface WorkspaceDialogsProps {
  newWorkspace: {
    open: boolean;
    setOpen: (v: boolean) => void;
    name: string;
    setName: (v: string) => void;
    onConfirm: () => void;
  };
  renameWorkspace: {
    open: boolean;
    setOpen: (v: boolean) => void;
    name: string;
    setName: (v: string) => void;
    onConfirm: () => void;
  };
  projectAlias: {
    open: boolean;
    setOpen: (v: boolean) => void;
    value: string;
    setValue: (v: string) => void;
    onConfirm: () => void;
  };
  workspaceAlias: {
    open: boolean;
    setOpen: (v: boolean) => void;
    value: string;
    setValue: (v: string) => void;
    onConfirm: () => void;
  };
  scan: {
    open: boolean;
    setOpen: (v: boolean) => void;
    results: ScannedRepo[];
    onConfirm: (paths: string[]) => void;
  };
  gitClone: {
    open: boolean;
    setOpen: (v: boolean) => void;
    workspaceName: string;
    onCloned: (path: string) => void;
  };
  confirm: ConfirmDialogState;
}

export default function WorkspaceDialogs(props: WorkspaceDialogsProps) {
  const { t } = useTranslation("dialogs");
  const { newWorkspace, renameWorkspace, projectAlias, workspaceAlias, scan, gitClone, confirm } = props;

  return (
    <>
      <ConfirmDialog
        open={confirm.open}
        setOpen={confirm.setOpen}
        title={confirm.title}
        description={confirm.description}
        onConfirm={confirm.onConfirm}
        variant={confirm.variant}
      />

      <TextInputDialog
        open={newWorkspace.open}
        setOpen={newWorkspace.setOpen}
        title={t("newWorkspace")}
        placeholder={t("workspaceNamePlaceholder")}
        value={newWorkspace.name}
        setValue={newWorkspace.setName}
        onConfirm={newWorkspace.onConfirm}
      />

      <TextInputDialog
        open={renameWorkspace.open}
        setOpen={renameWorkspace.setOpen}
        title={t("renameWorkspace")}
        placeholder={t("newNamePlaceholder")}
        value={renameWorkspace.name}
        setValue={renameWorkspace.setName}
        onConfirm={renameWorkspace.onConfirm}
      />

      <TextInputDialog
        open={projectAlias.open}
        setOpen={projectAlias.setOpen}
        title={t("setProjectAlias")}
        placeholder={t("projectAliasPlaceholder")}
        value={projectAlias.value}
        setValue={projectAlias.setValue}
        onConfirm={projectAlias.onConfirm}
      />

      <TextInputDialog
        open={workspaceAlias.open}
        setOpen={workspaceAlias.setOpen}
        title={t("setWorkspaceAlias")}
        placeholder={t("workspaceAliasPlaceholder")}
        value={workspaceAlias.value}
        setValue={workspaceAlias.setValue}
        onConfirm={workspaceAlias.onConfirm}
      />

      <ScanImportDialog
        open={scan.open}
        onOpenChange={scan.setOpen}
        repos={scan.results}
        onConfirm={scan.onConfirm}
      />

      <GitCloneDialog
        open={gitClone.open}
        onOpenChange={gitClone.setOpen}
        workspaceName={gitClone.workspaceName}
        onCloned={gitClone.onCloned}
      />
    </>
  );
}
