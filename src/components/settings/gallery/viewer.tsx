import { useEffect } from "react";
import { ArrowLeft, ChevronLeft, ChevronRight } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ContextMenu, ContextMenuContent, ContextMenuTrigger } from "@/components/ui/context-menu";
import { useSettingsContext } from "@/components/settings-provider";
import type { GalleryActions } from "@/hooks/use-gallery";
import { api, type GalleryItem } from "@/lib/api";
import { fileName } from "@/lib/gallery";
import { ItemMenu } from "./item-menu";
import { ViewerStrip } from "./viewer-strip";

type Props = {
  item: GalleryItem;
  index: number;
  count: number;
  items: GalleryItem[];
  actions: GalleryActions;
  paused: boolean;
  onStep: (delta: number) => void;
  onSelect: (item: GalleryItem) => void;
  onClose: () => void;
};

const MEDIA = "h-[calc(100vh-13rem)] max-w-full object-contain";

export function Viewer({ item, index, count, items, actions, paused, onStep, onSelect, onClose }: Props) {
  const { t } = useSettingsContext();

  useEffect(() => {
    if (paused) return;
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "ArrowLeft") onStep(-1);
      else if (e.key === "ArrowRight") onStep(1);
      else if (e.key === "Escape") onClose();
      else return;
      e.preventDefault();
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  });

  const src = api.mediaSrc(item.path);

  return (
    <div className="space-y-3">
      <div className="flex items-center gap-2">
        <Button variant="ghost" onClick={onClose}>
          <ArrowLeft />
          {t.back}
        </Button>
        <span className="min-w-0 flex-1 truncate text-center text-muted-foreground">{fileName(item.path)}</span>
        <Button variant="ghost" size="icon" title={t.previous} disabled={index === 0} onClick={() => onStep(-1)}>
          <ChevronLeft />
        </Button>
        <Button variant="ghost" size="icon" title={t.next} disabled={index === count - 1} onClick={() => onStep(1)}>
          <ChevronRight />
        </Button>
      </div>
      <ContextMenu>
        <ContextMenuTrigger className="flex justify-center">
          {item.kind === "video" ? (
            <video key={src} src={src} controls autoPlay className={MEDIA} />
          ) : (
            <img key={src} src={src} alt={fileName(item.path)} className={MEDIA} />
          )}
        </ContextMenuTrigger>
        <ContextMenuContent>
          <ItemMenu item={item} actions={actions} />
        </ContextMenuContent>
      </ContextMenu>
      <ViewerStrip items={items} activePath={item.path} onSelect={onSelect} />
    </div>
  );
}
