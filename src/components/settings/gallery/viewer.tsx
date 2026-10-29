import { useEffect, useLayoutEffect, useRef, useState } from "react";
import { ArrowLeft, ChevronLeft, ChevronRight, Copy, ExternalLink, FolderOpen, Pin, ScanText, Trash2 } from "lucide-react";
import { Button } from "@/components/ui/button";
import { ContextMenu, ContextMenuContent, ContextMenuTrigger } from "@/components/ui/context-menu";
import { Separator } from "@/components/ui/separator";
import { useSettingsContext } from "@/components/settings-provider";
import type { GalleryActions, TextResult } from "@/hooks/use-gallery";
import { api, type GalleryItem } from "@/lib/api";
import { fileName, formatDuration, formatSize } from "@/lib/gallery";
import { ItemMenu } from "./item-menu";
import { TextPane } from "./text-pane";
import { ViewerStrip } from "./viewer-strip";
import { ZoomImage } from "./zoom-image";

type Props = {
  item: GalleryItem;
  index: number;
  count: number;
  items: GalleryItem[];
  actions: GalleryActions;
  paused: boolean;
  text: TextResult | null;
  onToggleText: () => void;
  onCloseText: () => void;
  onStep: (delta: number) => void;
  onSelect: (item: GalleryItem) => void;
  origin: DOMRect | null;
  onClose: () => void;
};

type Meta = { w: number; h: number; duration?: number };

export function Viewer(props: Props) {
  const { item, index, count, items, actions, paused, text, onToggleText, onCloseText, onStep, onSelect, origin, onClose } =
    props;
  const { t, ocr, cfg } = useSettingsContext();
  const [loaded, setLoaded] = useState<Meta & { path: string }>();
  const meta = loaded?.path === item.path ? loaded : null;
  const setMeta = (next: Meta) => setLoaded({ ...next, path: item.path });
  const photo = item.kind === "photo";
  const stage = useRef<HTMLDivElement>(null);

  useLayoutEffect(() => {
    const el = stage.current;
    if (!el || !origin) return;
    const rect = el.getBoundingClientRect();
    const x = origin.left + origin.width / 2 - rect.left;
    const y = origin.top + origin.height / 2 - rect.top;
    el.style.transformOrigin = `${x}px ${y}px`;
  }, []);

  useEffect(() => {
    if (paused) return;
    const onKey = (e: KeyboardEvent) => {
      const ctrl = e.ctrlKey || e.metaKey;
      if (e.key === "ArrowLeft") onStep(-1);
      else if (e.key === "ArrowRight") onStep(1);
      else if (e.key === "Escape") (text ? onCloseText : onClose)();
      else if (e.key === "Delete") actions.remove(item);
      else if (photo && ctrl && e.key.toLowerCase() === "c" && !window.getSelection()?.toString()) actions.copy(item);
      else if (photo && !ctrl && e.key.toLowerCase() === "p") actions.pin(item);
      else return;
      e.preventDefault();
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  });

  const src = api.mediaSrc(item.path);
  const details = [
    meta && `${meta.w}×${meta.h}`,
    meta?.duration !== undefined && formatDuration(meta.duration),
    formatSize(item.size),
    new Intl.DateTimeFormat(cfg.language, { dateStyle: "medium", timeStyle: "short" }).format(item.date_ms),
  ].filter(Boolean);

  return (
    <div className="flex min-h-0 flex-1 flex-col gap-3 p-4">
      <div className="flex animate-in items-center gap-1 duration-200 fade-in-0 slide-in-from-top-2">
        <Button variant="ghost" onClick={onClose}>
          <ArrowLeft />
          {t.back}
        </Button>
        <div className="grid min-w-0 flex-1 text-center leading-tight">
          <span className="truncate">{fileName(item.path)}</span>
          <span className="truncate text-xs text-muted-foreground">{details.join(" · ")}</span>
        </div>
        {photo && (
          <>
            <Button variant="ghost" size="icon" title={`${t.copyImage} (Ctrl+C)`} onClick={() => actions.copy(item)}>
              <Copy />
            </Button>
            <Button variant="ghost" size="icon" title={`${t.pinImage} (P)`} onClick={() => actions.pin(item)}>
              <Pin />
            </Button>
          </>
        )}
        <Button variant="ghost" size="icon" title={t.openDefault} onClick={() => actions.open(item)}>
          <ExternalLink />
        </Button>
        <Button variant="ghost" size="icon" title={t.showInExplorer} onClick={() => actions.reveal(item)}>
          <FolderOpen />
        </Button>
        <Button variant="ghost" size="icon" title={`${t.deleteItem} (Del)`} onClick={() => actions.remove(item)}>
          <Trash2 />
        </Button>
        {ocr && photo && (
          <Button variant={text ? "secondary" : "ghost"} size="icon" title={t.extractText} onClick={onToggleText}>
            <ScanText />
          </Button>
        )}
        <Separator orientation="vertical" className="mx-1 data-vertical:h-5 data-vertical:self-center" />
        <Button variant="ghost" size="icon" title={t.previous} disabled={index === 0} onClick={() => onStep(-1)}>
          <ChevronLeft />
        </Button>
        <Button variant="ghost" size="icon" title={t.next} disabled={index === count - 1} onClick={() => onStep(1)}>
          <ChevronRight />
        </Button>
      </div>
      <div
        ref={stage}
        className="flex min-h-0 flex-1 animate-in gap-3 duration-200 ease-out fade-in-0 zoom-in-50 motion-reduce:animate-none"
      >
        <ContextMenu>
          <ContextMenuTrigger className="relative min-w-0 flex-1 overflow-hidden rounded-lg">
            {item.kind === "video" ? (
              <video
                key={src}
                src={src}
                controls
                autoPlay
                className="absolute inset-0 size-full animate-in object-contain duration-200 fade-in-0"
                onLoadedMetadata={(e) =>
                  setMeta({ w: e.currentTarget.videoWidth, h: e.currentTarget.videoHeight, duration: e.currentTarget.duration })
                }
              />
            ) : (
              <ZoomImage key={src} src={src} alt={fileName(item.path)} onSize={setMeta} />
            )}
          </ContextMenuTrigger>
          <ContextMenuContent>
            <ItemMenu item={item} actions={actions} />
          </ContextMenuContent>
        </ContextMenu>
        {text && <TextPane text={text} onClose={onCloseText} />}
      </div>
      <div className="animate-in duration-200 fade-in-0 slide-in-from-bottom-2">
        <ViewerStrip items={items} activePath={item.path} onSelect={onSelect} />
      </div>
    </div>
  );
}
