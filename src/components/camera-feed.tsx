import { useEffect, useRef, useState } from "react";
import { api, type CameraEffect, type CameraLook } from "@/lib/api";

const FEED = "pointer-events-none size-full scale-y-[1.04] -scale-x-[1.04] object-cover";
const BACKGROUND_HEIGHT = 720;

function useBackground(path: string, active: boolean) {
  const [background, setBackground] = useState<ImageBitmap | null>(null);

  useEffect(() => {
    if (!active || !path) return;
    let alive = true;
    let bitmap: ImageBitmap | null = null;
    api
      .cameraBackground(path)
      .then((bytes) => createImageBitmap(new Blob([bytes]), { resizeHeight: BACKGROUND_HEIGHT, resizeQuality: "high" }))
      .then((next) => {
        if (!alive) return next.close();
        bitmap = next;
        setBackground(next);
      })
      .catch(() => {});
    return () => {
      alive = false;
      bitmap?.close();
      setBackground(null);
    };
  }, [path, active]);

  return background;
}

export function CameraFeed({
  stream,
  effect,
  options,
  image,
}: {
  stream: MediaStream | null;
  effect: CameraEffect;
  options: CameraLook;
  image: string;
}) {
  const video = useRef<HTMLVideoElement>(null);
  const canvas = useRef<HTMLCanvasElement>(null);
  const processed = effect !== "none";
  const background = useBackground(image, effect === "image");
  const live = useRef({ ...options, background });
  live.current = { ...options, background };

  useEffect(() => {
    if (video.current) video.current.srcObject = stream;
  }, [stream]);

  useEffect(() => {
    const v = video.current;
    const c = canvas.current;
    if (!processed || !v || !c) return;
    let stop: (() => void) | null = null;
    let alive = true;
    import("@/lib/background-blur").then(({ startBlur }) => {
      if (alive) stop = startBlur(v, c, () => live.current);
    });
    return () => {
      alive = false;
      stop?.();
    };
  }, [processed]);

  return (
    <>
      <video ref={video} autoPlay muted playsInline className={processed ? "hidden" : FEED} />
      {processed ? <canvas ref={canvas} className={FEED} /> : null}
    </>
  );
}
