import { useEffect, useRef } from "react";
import { Button } from "@/components/ui/button";
import { ScrollArea as ScrollAreaPrimitive } from "@base-ui/react/scroll-area";
import { ScrollBar } from "@/components/ui/scroll-area";
import type { GalleryItem } from "@/lib/api";
import { fileName } from "@/lib/gallery";
import { Thumb } from "./thumb";

const TILE = "relative size-16 shrink-0 overflow-hidden bg-muted p-0";
const TILE_ACTIVE = "ring-2 ring-primary";

type Props = { items: GalleryItem[]; activePath: string; onSelect: (item: GalleryItem) => void };

export function ViewerStrip({ items, activePath, onSelect }: Props) {
  const active = useRef<HTMLButtonElement>(null);

  useEffect(() => {
    active.current?.scrollIntoView({ inline: "center", block: "nearest" });
  }, [activePath]);

  return (
    <ScrollAreaPrimitive.Root className="relative w-full">
      <ScrollAreaPrimitive.Viewport
        className="size-full rounded-[inherit] outline-none"
        onWheel={(e) => {
          e.currentTarget.scrollLeft += e.deltaY;
        }}
      >
        <div className="flex gap-2 p-1">
          {items.map((item) => {
            const isActive = item.path === activePath;
            return (
              <Button
                key={item.path}
                ref={isActive ? active : undefined}
                variant="ghost"
                title={fileName(item.path)}
                className={`${TILE} ${isActive ? TILE_ACTIVE : ""}`}
                onClick={() => onSelect(item)}
              >
                <Thumb item={item} />
              </Button>
            );
          })}
        </div>
      </ScrollAreaPrimitive.Viewport>
      <ScrollBar orientation="horizontal" />
    </ScrollAreaPrimitive.Root>
  );
}
