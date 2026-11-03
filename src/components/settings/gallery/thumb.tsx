import { useState } from "react";
import { ImageOff, Video } from "lucide-react";
import { Badge } from "@/components/ui/badge";
import { api, type GalleryItem } from "@/lib/api";
import { fileName, isGif } from "@/lib/gallery";

const THUMB_FIT = "size-full object-cover";
const ICON_FIT = "size-full object-contain p-4 pb-7";

export function Thumb({ item }: { item: GalleryItem }) {
  const [failed, setFailed] = useState(false);
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
      {isGif(item) && <Badge className="absolute right-1.5 bottom-1.5 h-4 px-1.5 text-[10px] shadow">GIF</Badge>}
    </>
  );
}
