import { useState } from "react";
import { ImageOff, Video } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { api, type GalleryItem } from "@/lib/api";
import { fileName, isGif, subtitleCodes } from "@/lib/gallery";

const THUMB_FIT = "size-full object-cover";
const ICON_FIT = "size-full object-contain p-4 pb-7";
const MAX_BADGES = 3;

export function Thumb({ item, badges = false }: { item: GalleryItem; badges?: boolean }) {
  const [failed, setFailed] = useState(false);
  const codes = badges ? subtitleCodes(item) : [];
  return (
    <>
      {failed ? (
        <ImageOff className="size-6 text-muted-foreground" />
      ) : (
        <img
          src={api.thumbSrc(item.path)}
          loading="lazy"
          alt={fileName(item.path)}
          className={item.kind === "other" ? ICON_FIT : THUMB_FIT}
          onError={() => setFailed(true)}
        />
      )}
      {item.kind === "video" && <Video className="absolute right-1.5 bottom-1.5 size-4 text-white drop-shadow" />}
      {codes.length > 0 && (
        <div className="absolute bottom-1.5 left-1.5 flex gap-1">
          {codes.slice(0, MAX_BADGES).map((code) => (
            <Badge key={code} variant="secondary" className="h-4 px-1 text-[10px] uppercase shadow">
              {code}
            </Badge>
          ))}
          {codes.length > MAX_BADGES && (
            <Badge variant="secondary" className="h-4 px-1 text-[10px] shadow">
              +{codes.length - MAX_BADGES}
            </Badge>
          )}
        </div>
      )}
      {isGif(item) && <Badge className="absolute right-1.5 bottom-1.5 h-4 px-1.5 text-[10px] shadow">GIF</Badge>}
    </>
  );
}
