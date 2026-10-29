import { AppWindow } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useSettingsContext } from "@/components/settings-provider";
import type { BuiltinId } from "@/lib/api";
import { BUILTINS } from "@/lib/ring";
import { Glyph } from "./ring-icon";
import type { RingDrag } from "./use-ring-drag";

type Props = {
  removed: BuiltinId[];
  full: boolean;
  drag: RingDrag;
  onAddBuiltin: (id: BuiltinId) => void;
  onAddApp: () => void;
};

export function RingAvailable({ removed, full, drag, onAddBuiltin, onAddApp }: Props) {
  const { t } = useSettingsContext();

  return (
    <div className="flex flex-col gap-2">
      <h3 className="text-xs font-medium text-muted-foreground">{t.ringAddToRing}</h3>
      <div className="flex flex-wrap gap-2">
        {removed.map((id) => {
          return (
            <Button
              key={id}
              variant="outline"
              size="sm"
              disabled={full}
              draggable={!full}
              onClick={() => onAddBuiltin(id)}
              onDragStart={drag.start({ kind: "builtin", id })}
              onDragEnd={drag.end}
            >
              <Glyph char={BUILTINS[id].glyph} />
              {t[BUILTINS[id].name]}
            </Button>
          );
        })}
        <Button
          variant="outline"
          size="sm"
          disabled={full}
          draggable={!full}
          onClick={onAddApp}
          onDragStart={drag.start({ kind: "app" })}
          onDragEnd={drag.end}
        >
          <AppWindow />
          {t.ringAddApp}
        </Button>
      </div>
      {full && <p className="text-xs text-muted-foreground">{t.ringMax}</p>}
    </div>
  );
}
