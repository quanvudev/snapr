import { X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { api } from "@/lib/api";
import { cn } from "@/lib/utils";

export function TitleBar({
  title,
  label,
  className,
  children,
  actions,
}: {
  title?: string;
  label: string;
  className?: string;
  children?: React.ReactNode;
  actions?: React.ReactNode;
}) {
  return (
    <div data-tauri-drag-region className={cn("flex h-12 shrink-0 items-center pr-2 pl-6 font-semibold", className)}>
      {children}
      {title}
      <div className="ml-auto flex items-center gap-1">
        {actions}
        <Button variant="ghost" size="icon" aria-label={label} onClick={api.hide}>
          <X />
        </Button>
      </div>
    </div>
  );
}
