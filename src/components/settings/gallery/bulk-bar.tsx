import { Copy, Pin, Trash2, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { useSettingsContext } from "@/components/settings-provider";

type Props = {
  count: number;
  photos: boolean;
  onSelectAll: () => void;
  onDelete: () => void;
  onCopy: () => void;
  onPin: () => void;
  onCancel: () => void;
};

export function BulkBar({ count, photos, onSelectAll, onDelete, onCopy, onPin, onCancel }: Props) {
  const { t } = useSettingsContext();
  return (
    <div className="flex flex-wrap items-center gap-2 rounded-lg border bg-muted/40 px-3 py-2">
      <span className="text-sm font-medium">{t.bulkSelected.replace("{count}", String(count))}</span>
      <div className="ml-auto flex flex-wrap items-center gap-2">
        <Button variant="outline" size="sm" onClick={onSelectAll}>
          {t.bulkSelectAll}
        </Button>
        <Button variant="outline" size="sm" disabled={!photos} onClick={onCopy}>
          <Copy />
          {t.bulkCopy}
        </Button>
        <Button variant="outline" size="sm" disabled={!photos} onClick={onPin}>
          <Pin />
          {t.bulkPin}
        </Button>
        <Button variant="destructive" size="sm" disabled={count === 0} onClick={onDelete}>
          <Trash2 />
          {t.bulkDelete}
        </Button>
        <Button variant="ghost" size="sm" onClick={onCancel}>
          <X />
          {t.bulkCancel}
        </Button>
      </div>
    </div>
  );
}
