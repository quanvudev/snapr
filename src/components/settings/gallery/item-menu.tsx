import { ContextMenuItem, ContextMenuSeparator } from "@/components/ui/context-menu";
import { useSettingsContext } from "@/components/settings-provider";
import type { GalleryActions } from "@/hooks/use-gallery";
import type { GalleryItem } from "@/lib/api";
import { canTranslate, isMp4 } from "@/lib/gallery";

export function ItemMenu({ item, actions }: { item: GalleryItem; actions: GalleryActions }) {
  const { t, ocr, translator } = useSettingsContext();
  const hymt = translator.status.ready;
  return (
    <>
      <ContextMenuItem onClick={() => actions.open(item)}>{t.openDefault}</ContextMenuItem>
      <ContextMenuItem onClick={() => actions.reveal(item)}>{t.showInExplorer}</ContextMenuItem>
      {item.kind === "photo" && (
        <>
          <ContextMenuItem onClick={() => actions.pin(item)}>{t.pinImage}</ContextMenuItem>
          <ContextMenuItem onClick={() => actions.copy(item)}>{t.copyImage}</ContextMenuItem>
          {ocr && <ContextMenuItem onClick={() => actions.extractText(item)}>{t.extractText}</ContextMenuItem>}
        </>
      )}
      {isMp4(item) && <ContextMenuItem onClick={() => actions.openGif(item)}>{t.exportGif}</ContextMenuItem>}
      {isMp4(item) && <ContextMenuItem onClick={() => actions.openTrim(item)}>{t.trimVideo}</ContextMenuItem>}
      {isMp4(item) && <ContextMenuItem onClick={() => actions.openCompress(item)}>{t.compressVideo}</ContextMenuItem>}
      {isMp4(item) && <ContextMenuItem onClick={() => actions.transcribe(item)}>{t.transcribe}</ContextMenuItem>}
      {canTranslate(item, hymt) &&
        (hymt ? (
          <ContextMenuItem onClick={() => actions.openTranslate(item)}>{t.translateSubtitles}</ContextMenuItem>
        ) : (
          <ContextMenuItem onClick={() => actions.translate(item)}>{t.translateToEnglish}</ContextMenuItem>
        ))}
      {isMp4(item) && (item.transcript || item.translations.length > 0) && (
        <ContextMenuItem onClick={() => actions.openSubtitleFromTranscript(item)}>{t.exportSubtitles}</ContextMenuItem>
      )}
      <ContextMenuSeparator />
      <ContextMenuItem variant="destructive" onClick={() => actions.remove(item)}>
        {t.deleteItem}
      </ContextMenuItem>
    </>
  );
}
