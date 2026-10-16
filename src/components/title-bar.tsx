import { X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { api } from "@/lib/api";

export function TitleBar({ title, label }: { title?: string; label: string }) {
  return (
    <div data-tauri-drag-region className="flex h-12 shrink-0 items-center justify-between pr-2 pl-6 font-semibold">
      {title}
      <Button variant="ghost" size="icon" className="ml-auto" aria-label={label} onClick={api.hide}>
        <X />
      </Button>
    </div>
  );
}
