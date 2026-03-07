import { RefreshCw, FilePlus, FolderPlus, Search, EyeOff, Eye } from "lucide-react";
import { Button } from "@/components/ui/button";
import {
  Tooltip,
  TooltipContent,
  TooltipProvider,
  TooltipTrigger,
} from "@/components/ui/tooltip";

interface FileExplorerToolbarProps {
  showHidden: boolean;
  showSearch: boolean;
  onRefresh: () => void;
  onNewFile: () => void;
  onNewFolder: () => void;
  onToggleSearch: () => void;
  onToggleHidden: () => void;
}

export default function FileExplorerToolbar({
  showHidden,
  showSearch,
  onRefresh,
  onNewFile,
  onNewFolder,
  onToggleSearch,
  onToggleHidden,
}: FileExplorerToolbarProps) {
  return (
    <TooltipProvider delayDuration={300}>
      <div className="flex items-center gap-0.5 px-2 py-1 border-b bg-background">
        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="ghost" size="icon" className="h-6 w-6" onClick={onRefresh}>
              <RefreshCw size={14} />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Refresh</TooltipContent>
        </Tooltip>

        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="ghost" size="icon" className="h-6 w-6" onClick={onNewFile}>
              <FilePlus size={14} />
            </Button>
          </TooltipTrigger>
          <TooltipContent>New File</TooltipContent>
        </Tooltip>

        <Tooltip>
          <TooltipTrigger asChild>
            <Button variant="ghost" size="icon" className="h-6 w-6" onClick={onNewFolder}>
              <FolderPlus size={14} />
            </Button>
          </TooltipTrigger>
          <TooltipContent>New Folder</TooltipContent>
        </Tooltip>

        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant={showSearch ? "secondary" : "ghost"}
              size="icon"
              className="h-6 w-6"
              onClick={onToggleSearch}
            >
              <Search size={14} />
            </Button>
          </TooltipTrigger>
          <TooltipContent>Toggle Search</TooltipContent>
        </Tooltip>

        <div className="flex-1" />

        <Tooltip>
          <TooltipTrigger asChild>
            <Button
              variant={showHidden ? "secondary" : "ghost"}
              size="icon"
              className="h-6 w-6"
              onClick={onToggleHidden}
            >
              {showHidden ? <Eye size={14} /> : <EyeOff size={14} />}
            </Button>
          </TooltipTrigger>
          <TooltipContent>{showHidden ? "Hide Hidden Files" : "Show Hidden Files"}</TooltipContent>
        </Tooltip>
      </div>
    </TooltipProvider>
  );
}
