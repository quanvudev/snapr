import { useEffect, useRef, useState } from "react";
import { Pause, Play, Trash2, TriangleAlert } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Separator } from "@/components/ui/separator";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { api, type PanelState } from "@/lib/api";
import type { Strings } from "@/lib/i18n";

function elapsed(s: PanelState, now: number) {
  if (s.started_ms == null) return 0;
  const paused = s.paused_total_ms + (s.paused_at_ms != null ? now - s.paused_at_ms : 0);
  return Math.max(0, now - s.started_ms - paused);
}

function format(ms: number) {
  const total = Math.floor(ms / 1000);
  const minutes = Math.floor(total / 60);
  const seconds = String(total % 60).padStart(2, "0");
  return minutes >= 60 ? `${Math.floor(minutes / 60)}:${String(minutes % 60).padStart(2, "0")}:${seconds}` : `${minutes}:${seconds}`;
}

function Action({ label, onClick, children, className }: { label: string; onClick: () => void; children: React.ReactNode; className?: string }) {
  return (
    <Tooltip>
      <TooltipTrigger
        render={<Button variant="ghost" size="icon" className={`rounded-full ${className ?? ""}`} aria-label={label} onClick={onClick} />}
      >
        {children}
      </TooltipTrigger>
      <TooltipContent side="bottom">{label}</TooltipContent>
    </Tooltip>
  );
}

export function RecordingPanel({ state, t }: { state: PanelState; t: Strings }) {
  const [now, setNow] = useState(Date.now());
  const [confirm, setConfirm] = useState(false);
  const timer = useRef<number | undefined>(undefined);
  const paused = state.paused_at_ms != null;

  useEffect(() => {
    const id = setInterval(() => setNow(Date.now()), 250);
    return () => {
      clearInterval(id);
      window.clearTimeout(timer.current);
    };
  }, []);

  const discard = () => {
    if (confirm) {
      api.recordDiscard();
      return;
    }
    setConfirm(true);
    window.clearTimeout(timer.current);
    timer.current = window.setTimeout(() => setConfirm(false), 3000);
  };

  return (
    <div className="px-4 pt-1 pb-10">
      <div
        data-tauri-drag-region
        className="flex h-14 w-fit items-center gap-1 rounded-full border border-foreground/10 bg-popover px-3 text-popover-foreground shadow-[0_6px_20px_rgba(0,0,0,0.25)]"
      >
        <Action label={t.recStop} onClick={() => api.recordStop()}>
          <span className="size-4 rounded-[3px] bg-[#ff623e]" />
        </Action>
        <span
          data-tauri-drag-region
          className={`min-w-12 pr-3 text-base font-semibold tabular-nums ${paused ? "animate-pulse text-muted-foreground" : ""}`}
        >
          {format(elapsed(state, now))}
        </span>
        {state.warning === "mic_lost" ? (
          <Tooltip>
            <TooltipTrigger render={<span className="flex size-6 items-center justify-center text-destructive" />}>
              <TriangleAlert className="size-4" />
            </TooltipTrigger>
            <TooltipContent side="bottom">{t.recMicLost}</TooltipContent>
          </Tooltip>
        ) : null}
        <Separator orientation="vertical" className="mx-1 h-7 data-vertical:self-center" />
        <Action label={paused ? t.recResume : t.recPause} onClick={() => (paused ? api.recordResume() : api.recordPause())}>
          {paused ? <Play className="size-5 fill-current" /> : <Pause className="size-5 fill-current" />}
        </Action>
        <Action
          label={confirm ? t.recDiscardConfirm : t.recDiscard}
          onClick={discard}
          className={confirm ? "bg-destructive/15 text-destructive hover:bg-destructive/20 hover:text-destructive" : ""}
        >
          <Trash2 className="size-5" />
        </Action>
      </div>
    </div>
  );
}
