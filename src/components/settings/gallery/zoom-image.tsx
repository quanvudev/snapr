import { useEffect, useRef, useState } from "react";
import type { PointerEvent, WheelEvent } from "react";
import { Minus, Plus, RotateCcw } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useSettingsContext } from "@/components/settings-provider";
import { cn } from "@/lib/utils";

type Size = { w: number; h: number };
type Point = { x: number; y: number };

const STEP = 1.25;
const MAX_SCALE = 8;
const ORIGIN: Point = { x: 0, y: 0 };

type Props = { src: string; alt: string; onSize: (size: Size) => void };

export function ZoomImage({ src, alt, onSize }: Props) {
  const { t } = useSettingsContext();
  const box = useRef<HTMLDivElement>(null);
  const drag = useRef<Point | null>(null);
  const [area, setArea] = useState<Size>({ w: 0, h: 0 });
  const [natural, setNatural] = useState<Size | null>(null);
  const [zoom, setZoom] = useState(1);
  const [offset, setOffset] = useState<Point>(ORIGIN);
  const [dragging, setDragging] = useState(false);

  useEffect(() => {
    const el = box.current;
    if (!el) return;
    const observer = new ResizeObserver(([entry]) =>
      setArea({ w: entry.contentRect.width, h: entry.contentRect.height }),
    );
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  const fit = natural && area.w && area.h ? Math.min(1, area.w / natural.w, area.h / natural.h) : 1;
  const maxZoom = Math.max(1, MAX_SCALE / fit);
  const scale = fit * zoom;
  const width = (natural?.w ?? 0) * scale;
  const height = (natural?.h ?? 0) * scale;

  const clamp = (next: Point, w: number, h: number): Point => {
    const limitX = Math.max(0, (w - area.w) / 2);
    const limitY = Math.max(0, (h - area.h) / 2);
    return {
      x: Math.min(limitX, Math.max(-limitX, next.x)),
      y: Math.min(limitY, Math.max(-limitY, next.y)),
    };
  };

  const zoomTo = (target: number, at: Point = { x: area.w / 2, y: area.h / 2 }) => {
    if (!natural) return;
    const next = Math.min(maxZoom, Math.max(1, target));
    const ratio = next / zoom;
    const dx = at.x - area.w / 2 - offset.x;
    const dy = at.y - area.h / 2 - offset.y;
    const moved = { x: offset.x - dx * (ratio - 1), y: offset.y - dy * (ratio - 1) };
    setZoom(next);
    setOffset(next === 1 ? ORIGIN : clamp(moved, natural.w * fit * next, natural.h * fit * next));
  };

  const local = (e: { clientX: number; clientY: number }): Point => {
    const rect = box.current!.getBoundingClientRect();
    return { x: e.clientX - rect.left, y: e.clientY - rect.top };
  };

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.ctrlKey || e.altKey || e.metaKey) return;
      if (e.key === "+" || e.key === "=") zoomTo(zoom * STEP);
      else if (e.key === "-") zoomTo(zoom / STEP);
      else if (e.key === "0") zoomTo(1);
      else return;
      e.preventDefault();
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  });

  const wheel = (e: WheelEvent<HTMLDivElement>) => zoomTo(e.deltaY < 0 ? zoom * STEP : zoom / STEP, local(e));

  const down = (e: PointerEvent<HTMLDivElement>) => {
    if (e.button !== 0 || zoom === 1) return;
    e.currentTarget.setPointerCapture(e.pointerId);
    drag.current = { x: e.clientX, y: e.clientY };
    setDragging(true);
  };

  const move = (e: PointerEvent<HTMLDivElement>) => {
    const last = drag.current;
    if (!last) return;
    drag.current = { x: e.clientX, y: e.clientY };
    setOffset((prev) => clamp({ x: prev.x + e.clientX - last.x, y: prev.y + e.clientY - last.y }, width, height));
  };

  const up = () => {
    drag.current = null;
    setDragging(false);
  };

  return (
    <div className="absolute inset-0">
      <div
        ref={box}
        className={cn(
          "absolute inset-0 overflow-hidden select-none",
          zoom > 1 && (dragging ? "cursor-grabbing" : "cursor-grab"),
        )}
        onWheel={wheel}
        onPointerDown={down}
        onPointerMove={move}
        onPointerUp={up}
        onPointerCancel={up}
        onDoubleClick={(e) => zoomTo(zoom > 1 ? 1 : Math.max(STEP * STEP, 1 / fit), local(e))}
      >
        <img
          src={src}
          alt={alt}
          draggable={false}
          className={cn("absolute max-w-none transition-opacity duration-200", natural ? "opacity-100" : "opacity-0")}
          style={{
            width,
            height,
            left: (area.w - width) / 2 + offset.x,
            top: (area.h - height) / 2 + offset.y,
          }}
          onLoad={(e) => {
            const size = { w: e.currentTarget.naturalWidth, h: e.currentTarget.naturalHeight };
            setNatural(size);
            onSize(size);
          }}
        />
      </div>
      {natural && (
        <div className="absolute right-2 bottom-2 flex items-center gap-1 rounded-lg border bg-popover/90 p-1 text-xs text-popover-foreground shadow-sm">
          <Button
            variant="ghost"
            size="icon-xs"
            title={`${t.zoomOut} (-)`}
            disabled={zoom <= 1}
            onClick={() => zoomTo(zoom / STEP)}
          >
            <Minus />
          </Button>
          <span className="w-11 text-center tabular-nums">{Math.round(scale * 100)}%</span>
          <Button
            variant="ghost"
            size="icon-xs"
            title={`${t.zoomIn} (+)`}
            disabled={zoom >= maxZoom}
            onClick={() => zoomTo(zoom * STEP)}
          >
            <Plus />
          </Button>
          <Button
            variant="ghost"
            size="icon-xs"
            title={`${t.zoomReset} (0)`}
            disabled={zoom === 1}
            onClick={() => zoomTo(1)}
          >
            <RotateCcw />
          </Button>
        </div>
      )}
    </div>
  );
}
