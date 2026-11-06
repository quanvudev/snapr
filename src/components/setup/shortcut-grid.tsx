import { Kbd } from "@/components/ui/kbd";
import { useSettingsContext } from "@/components/settings-provider";
import { SLOTS } from "@/lib/hotkeys";
import { cn } from "@/lib/utils";

export const SETUP_SLOTS = (["fullscreen", "window", "area", "record"] as const).map((s) => SLOTS.indexOf(s));

export function ShortcutGrid() {
  const { t, rows } = useSettingsContext();
  return (
    <div className="grid grid-cols-2 gap-2">
      {SETUP_SLOTS.map((i) => {
        const row = rows[i];
        const problem = row?.error ? t[row.error] : row?.in_use ? t.inUse : null;
        const key = row?.combo.split("+").pop();
        return (
          <div
            key={i}
            className={cn(
              "flex items-center justify-between gap-2 rounded-lg border border-white/5 bg-white/5 px-3 py-2.5",
              problem && "border-destructive/50",
            )}
          >
            <div className="min-w-0">
              <div className="truncate">{t[SLOTS[i]]}</div>
              {problem && <div className="truncate text-xs text-destructive">{problem}</div>}
            </div>
            {key && (
              <Kbd className="h-7 min-w-7 rounded-md border border-white/10 bg-white/10 text-sm text-foreground">{key}</Kbd>
            )}
          </div>
        );
      })}
    </div>
  );
}
