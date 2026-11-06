import { Captions, Clapperboard, Copy, Eraser, FileDown, ImageUpscale, Pin, Sparkles, Trash2, X } from "lucide-react";
import { Button } from "@/components/ui/button";
import { DropdownMenu, DropdownMenuContent, DropdownMenuItem, DropdownMenuSeparator, DropdownMenuTrigger } from "@/components/ui/dropdown-menu";
import { useSettingsContext } from "@/components/settings-provider";
import type { UpscaleScale } from "@/lib/api";
import { UPSCALE_SCALES } from "@/lib/upscale";

type Props = {
  count: number;
  photos: boolean;
  videos: boolean;
  onSelectAll: () => void;
  onDelete: () => void;
  onCopy: () => void;
  onPin: () => void;
  onExportGif: () => void;
  onCompress: () => void;
  onTranscribe: () => void;
  onRemoveBackground: () => void;
  onUpscale: (scale: UpscaleScale) => void;
  onCancel: () => void;
};

export function BulkBar({ count, photos, videos, onSelectAll, onDelete, onCopy, onPin, onExportGif, onCompress, onTranscribe, onRemoveBackground, onUpscale, onCancel }: Props) {
  const { t } = useSettingsContext();
  return (
    <div className="absolute inset-x-0 bottom-4 z-30 mx-auto flex w-fit max-w-[calc(100%-3rem)] flex-wrap items-center gap-2 rounded-xl border bg-popover px-3 py-2 text-popover-foreground shadow-lg">
      <span className="pr-2 text-sm font-medium">{t.bulkSelected.replace("{count}", String(count))}</span>
      <div className="flex flex-wrap items-center gap-2">
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
        <Button variant="outline" size="sm" disabled={!videos} onClick={onExportGif}>
          <Clapperboard />
          {t.exportGif}
        </Button>
        <Button variant="outline" size="sm" disabled={!videos} onClick={onCompress}>
          <FileDown />
          {t.compressVideo}
        </Button>
        <Button variant="outline" size="sm" disabled={!videos} onClick={onTranscribe}>
          <Captions />
          {t.transcribe}
        </Button>
        <DropdownMenu>
          <DropdownMenuTrigger render={<Button variant="outline" size="sm" disabled={!photos} />}>
            <Sparkles />
            {t.aiTools}
          </DropdownMenuTrigger>
          <DropdownMenuContent side="top" className="w-auto">
            <DropdownMenuItem onClick={onRemoveBackground}>
              <Eraser />
              {t.removeBackground}
            </DropdownMenuItem>
            <DropdownMenuSeparator />
            {UPSCALE_SCALES.map((scale) => (
              <DropdownMenuItem key={scale} onClick={() => onUpscale(scale)}>
                <ImageUpscale />
                {t.upscale} ×{scale}
              </DropdownMenuItem>
            ))}
          </DropdownMenuContent>
        </DropdownMenu>
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
