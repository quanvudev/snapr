import { useEffect, useState } from "react";
import { CameraFeed } from "@/components/camera-feed";
import { api, type CameraEffect, type CameraLook } from "@/lib/api";
import "@/App.css";

const params = new URLSearchParams(window.location.search);
const EFFECTS: CameraEffect[] = ["none", "blur", "image"];
const DEFAULT_LOOK: CameraLook = { strength: 4, smoothing: 50, edge: 30, feather: 1, wrap: 50, wrap_radius: 4 };

function initialLook(): CameraLook {
  try {
    return { ...DEFAULT_LOOK, ...JSON.parse(params.get("look") ?? "{}") };
  } catch {
    return DEFAULT_LOOK;
  }
}

export function CameraApp() {
  const [stream, setStream] = useState<MediaStream | null>(null);
  const [effect, setEffect] = useState<CameraEffect>(
    EFFECTS.find((e) => e === params.get("effect")) ?? "none",
  );
  const [image, setImage] = useState(params.get("image") ?? "");
  const [look, setLook] = useState(initialLook);

  useEffect(() => {
    for (const el of [document.documentElement, document.body, document.getElementById("root")]) {
      if (!el) continue;
      el.style.background = "transparent";
      el.style.overflow = "hidden";
    }
    let stream: MediaStream | null = null;
    let current: string | null = null;
    let alive = true;

    const open = async (device: string) => {
      if (device === current) return;
      current = device;
      stream?.getTracks().forEach((track) => track.stop());
      stream = null;
      setStream(null);
      if (!device) return;
      try {
        const next = await navigator.mediaDevices.getUserMedia({ video: { deviceId: { exact: device } } });
        if (!alive || current !== device) {
          next.getTracks().forEach((track) => track.stop());
          return;
        }
        stream = next;
        setStream(next);
      } catch {
        current = null;
      }
    };

    open(params.get("device") ?? "");
    const unlisten = api.onRecordPanel((s) => {
      open(s.camera);
      setEffect(s.effect);
      setImage(s.camera_image);
      setLook(s.look);
    });
    return () => {
      alive = false;
      unlisten.then((f) => f());
      stream?.getTracks().forEach((track) => track.stop());
    };
  }, []);

  return (
    <div data-tauri-drag-region className="h-screen w-screen p-7">
      <div data-tauri-drag-region className="size-full rounded-full drop-shadow-[0_10px_12px_rgba(0,0,0,0.6)]">
        <div
          data-tauri-drag-region
          className="size-full overflow-hidden rounded-full"
          style={{ maskImage: "radial-gradient(circle closest-side, #000 calc(100% - 2px), transparent 100%)" }}
        >
          <CameraFeed stream={stream} effect={effect} options={look} image={image} />
        </div>
      </div>
    </div>
  );
}
