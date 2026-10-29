import { useEffect, useRef } from "react";
import { Copy, Pin, Trash2 } from "lucide-react";
import type { MouseEvent, PointerEvent } from "react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { ContextMenu, ContextMenuContent, ContextMenuTrigger } from "@/components/ui/context-menu";
import { useSettingsContext } from "@/components/settings-provider";
import type { GalleryActions } from "@/hooks/use-gallery";
import type { GalleryItem as Item } from "@/lib/api";
import { fileName, isViewable } from "@/lib/gallery";
import { cn } from "@/lib/utils";
import { ItemMenu } from "./item-menu";
import { Thumb } from "./thumb";

type Props = {
  item: Item;
  actions: GalleryActions;
  onOpen: (from: DOMRect) => void;
  selectMode: boolean;
  selected: boolean;
  onSelect: () => void;
  onRange: () => void;
  onLongPress: () => void;
};

export function GalleryItem({ item, actions, onOpen, selectMode, selected, onSelect, onRange, onLongPress }: Props) {
  const { t } = useSettingsContext();
  const timer = useRef<number | null>(null);
  const long = useRef(false);
  const origin = useRef<{ x: number; y: number } | null>(null);

  const cancel = () => {
    if (timer.current !== null) {
      window.clearTimeout(timer.current);
      timer.current = null;
    }
  };

  useEffect(() => () => cancel(), []);

  const down = (event: PointerEvent<HTMLButtonElement>) => {
    if (event.button !== 0) return;
    long.current = false;
    origin.current = { x: event.clientX, y: event.clientY };
    cancel();
    timer.current = window.setTimeout(() => {
      timer.current = null;
      long.current = true;
      onLongPress();
    }, 500);
  };

  const move = (event: PointerEvent<HTMLButtonElement>) => {
    const start = origin.current;
    if (!start) return;
    if (Math.abs(event.clientX - start.x) > 4 || Math.abs(event.clientY - start.y) > 4) cancel();
  };

  const up = () => {
    cancel();
    origin.current = null;
  };

  const click = (event: MouseEvent<HTMLButtonElement>) => {
    if (long.current) {
      long.current = false;
      event.preventDefault();
      event.stopPropagation();
      return;
    }
    if (event.shiftKey) {
      onRange();
      return;
    }
    if (selectMode || event.ctrlKey || event.metaKey) {
      onSelect();
      return;
    }
    if (isViewable(item)) onOpen(event.currentTarget.getBoundingClientRect());
  };

  const photo = item.kind === "photo";

  return (
    <ContextMenu>
      <div className="group/tile relative">
        <ContextMenuTrigger
          render={
            <Button
              variant="ghost"
              title={fileName(item.path)}
              className={cn(
                "relative aspect-video h-auto w-full overflow-hidden bg-muted p-0",
                selected && "ring-2 ring-primary ring-offset-2 ring-offset-background",
              )}
              onClick={click}
              onPointerDown={down}
              onPointerMove={move}
              onPointerUp={up}
              onPointerLeave={up}
              onPointerCancel={up}
            />
          }
        >
          <Thumb item={item} />
          {!isViewable(item) && (
            <span className="absolute inset-x-0 bottom-0 truncate bg-background/80 px-1.5 py-0.5 text-xs">
              {fileName(item.path)}
            </span>
          )}
        </ContextMenuTrigger>
        <span
          className={cn(
            "absolute top-1.5 left-1.5 z-10 flex rounded-md bg-background/90 p-1 shadow-sm transition-opacity",
            selectMode || selected ? "opacity-100" : "opacity-0 group-hover/tile:opacity-100",
          )}
        >
          <Checkbox checked={selected} onCheckedChange={onSelect} />
        </span>
        {!selectMode && (
          <span className="absolute top-1.5 right-1.5 z-10 flex gap-1 opacity-0 transition-opacity group-hover/tile:opacity-100">
            {photo && (
              <>
                <Button variant="secondary" size="icon-xs" title={t.copyImage} onClick={() => actions.copy(item)}>
                  <Copy />
                </Button>
                <Button variant="secondary" size="icon-xs" title={t.pinImage} onClick={() => actions.pin(item)}>
                  <Pin />
                </Button>
              </>
            )}
            <Button variant="secondary" size="icon-xs" title={t.deleteItem} onClick={() => actions.remove(item)}>
              <Trash2 />
            </Button>
          </span>
        )}
      </div>
      <ContextMenuContent>
        <ItemMenu item={item} actions={actions} />
      </ContextMenuContent>
    </ContextMenu>
  );
}
