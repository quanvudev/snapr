import { useState, type DragEvent } from "react";
import type { BuiltinId, RingItem } from "@/lib/api";
import { insertItem, MAX_RING_ITEMS, moveItem, nearestSlot, removeItem, slotCount, wheelRadius } from "@/lib/ring";

export type DragSource = { kind: "slot"; index: number } | { kind: "builtin"; id: BuiltinId } | { kind: "app" };

export function useRingDrag(
  items: RingItem[],
  onChange: (next: RingItem[]) => void,
  onSelect: (index: number | null) => void,
  pickApp: () => Promise<RingItem | null>,
) {
  const [source, setSource] = useState<DragSource | null>(null);
  const slots = slotCount(items.length);
  const [over, setOver] = useState<number | null>(null);

  const end = () => {
    setSource(null);
    setOver(null);
  };

  const pointer = (e: DragEvent) => {
    const rect = e.currentTarget.getBoundingClientRect();
    const x = e.clientX - (rect.left + rect.width / 2);
    const y = e.clientY - (rect.top + rect.height / 2);
    return { x, y, outside: Math.hypot(x, y) > wheelRadius(items.length) };
  };

  const remove = (src: DragSource | null) => {
    if (src?.kind !== "slot") return;
    onChange(removeItem(items, src.index));
    onSelect(null);
  };

  const dropOnCircle = async (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    const src = source;
    const { x, y, outside } = pointer(e);
    end();
    if (!src) return;
    if (outside) return remove(src);
    const at = nearestSlot(x, y, slots);
    if (src.kind === "slot") {
      const to = Math.min(at, items.length - 1);
      onChange(moveItem(items, src.index, to));
      return onSelect(to);
    }
    if (items.length >= MAX_RING_ITEMS) return;
    const item: RingItem | null = src.kind === "builtin" ? { kind: "builtin", id: src.id } : await pickApp();
    if (!item) return;
    onChange(insertItem(items, item, at));
    onSelect(at);
  };

  const start = (next: DragSource) => (e: DragEvent) => {
    e.dataTransfer.effectAllowed = "move";
    e.dataTransfer.setData("text/plain", "ring");
    setSource(next);
  };

  return {
    source,
    over,
    removing: source?.kind === "slot",
    start,
    end,
    circle: {
      onDragOver: (e: DragEvent) => {
        if (!source) return;
        e.preventDefault();
        const { x, y, outside } = pointer(e);
        setOver(outside ? null : nearestSlot(x, y, slots));
      },
      onDragLeave: (e: DragEvent) => {
        if (!e.currentTarget.contains(e.relatedTarget as Node | null)) setOver(null);
      },
      onDrop: dropOnCircle,
    },
    bin: {
      onDragOver: (e: DragEvent) => e.preventDefault(),
      onDrop: (e: DragEvent) => {
        e.preventDefault();
        e.stopPropagation();
        const src = source;
        end();
        remove(src);
      },
    },
    outside: {
      onDragOver: (e: DragEvent) => {
        if (source?.kind === "slot") e.preventDefault();
      },
      onDrop: (e: DragEvent) => {
        if (source?.kind !== "slot") return;
        e.preventDefault();
        const src = source;
        end();
        remove(src);
      },
    },
  };
}

export type RingDrag = ReturnType<typeof useRingDrag>;
