import { useEffect, useRef, useState } from "react";
import {
  AppWindow,
  ChevronUp,
  Circle,
  Mic,
  MicOff,
  Monitor,
  SquareDashed,
  Video,
  VideoOff,
  Volume2,
  VolumeX,
  XIcon,
} from "lucide-react";
import { Button } from "@/components/ui/button";
import { ScrollArea } from "@/components/ui/scroll-area";
import { Separator } from "@/components/ui/separator";
import {
  DropdownMenu,
  DropdownMenuCheckboxItem,
  DropdownMenuContent,
  DropdownMenuGroup,
  DropdownMenuLabel,
  DropdownMenuRadioGroup,
  DropdownMenuRadioItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { CameraFeed } from "@/components/camera-feed";
import { api, type CameraEffect, type CameraLook, type InputDevice, type PanelState, type RecordMode } from "@/lib/api";
import type { Strings } from "@/lib/i18n";

type Camera = { id: string; name: string };

function useCameras() {
  const [cameras, setCameras] = useState<Camera[]>([]);
  const [error, setError] = useState(false);
  const [loaded, setLoaded] = useState(false);

  useEffect(() => {
    let alive = true;
    (async () => {
      try {
        const all = await navigator.mediaDevices.enumerateDevices();
        const found = all.filter((d) => d.kind === "videoinput");
        if (found.length > 0 && found.every((d) => !d.label)) {
          const stream = await navigator.mediaDevices.getUserMedia({
            video: true,
          });
          stream.getTracks().forEach((track) => track.stop());
        }
        const named = (await navigator.mediaDevices.enumerateDevices()).filter((d) => d.kind === "videoinput");
        if (!alive) return;
        setCameras(named.map((d) => ({ id: d.deviceId, name: d.label || d.deviceId })));
        setError(false);
      } catch {
        if (!alive) return;
        setCameras([]);
        setError(true);
      }
      if (alive) setLoaded(true);
    })();
    return () => {
      alive = false;
    };
  }, []);

  return { cameras, error, loaded };
}

function Preview({
  device,
  effect,
  options,
  image,
  onError,
}: {
  device: string;
  effect: CameraEffect;
  options: CameraLook;
  image: string;
  onError: () => void;
}) {
  const [stream, setStream] = useState<MediaStream | null>(null);

  useEffect(() => {
    let alive = true;
    let stream: MediaStream | null = null;
    navigator.mediaDevices
      .getUserMedia({ video: { deviceId: { exact: device } } })
      .then((next) => {
        if (!alive) {
          next.getTracks().forEach((track) => track.stop());
          return;
        }
        stream = next;
        setStream(next);
      })
      .catch(() => alive && onError());
    return () => {
      alive = false;
      stream?.getTracks().forEach((track) => track.stop());
      setStream(null);
    };
  }, [device]);

  return <CameraFeed stream={stream} effect={effect} options={options} image={image} />;
}

export function SetupPanel({ state, level, t }: { state: PanelState; level: number; t: Strings }) {
  const [mic, setMic] = useState(state.mic);
  const [camera, setCamera] = useState(state.camera);
  const [speaker, setSpeaker] = useState(state.speaker);
  const [denoise, setDenoise] = useState(state.denoise);
  const [effect, setEffect] = useState<CameraEffect>(state.effect);
  const [mode, setMode] = useState<RecordMode>(state.mode);
  const [inputs, setInputs] = useState<InputDevice[]>([]);
  const [outputs, setOutputs] = useState<InputDevice[]>([]);
  const { cameras, error, loaded } = useCameras();
  const [failed, setFailed] = useState(false);
  const picked = useRef(false);
  const first = useRef(true);
  const starting = useRef(false);
  const [busy, setBusy] = useState(false);
  const [open, setOpen] = useState<"mode" | "audio" | "camera" | null>(null);

  useEffect(() => {
    const close = () => setOpen(null);
    window.addEventListener("blur", close);
    return () => window.removeEventListener("blur", close);
  }, []);

  const start = () => {
    if (starting.current) return;
    starting.current = true;
    setBusy(true);
    api.recordStart().catch(() => {
      starting.current = false;
      setBusy(false);
    });
  };

  useEffect(() => {
    if (!loaded || picked.current) return;
    picked.current = true;
    if (error || cameras.length === 0) setCamera("");
    else if (!cameras.some((c) => c.id === camera)) setCamera(cameras[0].id);
  }, [loaded, error, cameras, camera]);

  useEffect(() => {
    api
      .recordInputs()
      .then(setInputs)
      .catch(() => setInputs([]));
    api
      .recordOutputs()
      .then(setOutputs)
      .catch(() => setOutputs([]));
  }, []);

  useEffect(() => {
    if (first.current) {
      first.current = false;
      return;
    }
    api.recordSelect(mic, camera, speaker, mode, denoise, effect);
  }, [mic, camera, speaker, mode, denoise, effect]);

  useEffect(() => {
    setFailed(false);
  }, [camera]);

  useEffect(() => {
    const onKey = (e: KeyboardEvent) => {
      if (e.defaultPrevented) return;
      if (e.key === "Escape") api.recordClose();
      else if (e.key === "Enter") {
        const target = e.target as HTMLElement | null;
        if (target?.closest("button, input, select, [role=combobox], [role=listbox], [role=switch]")) return;
        start();
      }
    };
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  const micName = (id: string) =>
    id === "" ? t.recMicNone : id === "default" ? t.recMicDefault : (inputs.find((d) => d.id === id)?.name ?? id);
  const speakerName = (id: string) =>
    id === ""
      ? t.recSpeakerNone
      : id === "default"
        ? t.recSpeakerDefault
        : (outputs.find((d) => d.id === id)?.name ?? id);
  const cameraName = (id: string) =>
    id === "" ? t.recCameraOff : (cameras.find((c) => c.id === id)?.name ?? t.recCamera);
  const modes = [
    { value: "fullscreen", label: t.recModeFullscreen, Icon: Monitor },
    { value: "window", label: t.recModeWindow, Icon: AppWindow },
    { value: "area", label: t.recModeArea, Icon: SquareDashed },
  ] as const;
  const current = modes.find((m) => m.value === mode) ?? modes[0];
  const ring = (1 + Math.min(1, Math.sqrt(level)) * 3).toFixed(1);
  const blocked = error || failed;
  const live = camera !== "" && !failed;

  return (
    <div className="flex flex-col items-center gap-2 p-4 text-foreground">
      <div
        data-tauri-drag-region
        className={`relative flex size-52 items-center justify-center rounded-full drop-shadow-[0_10px_12px_rgba(0,0,0,0.6)] ${live ? "" : "bg-muted"}`}
      >
        {live ? (
          <div
            data-tauri-drag-region
            className="size-full overflow-hidden rounded-full"
            style={{ maskImage: "radial-gradient(circle closest-side, #000 calc(100% - 2px), transparent 100%)" }}
          >
            <Preview
              device={camera}
              effect={effect}
              options={state.look}
              image={state.camera_image}
              onError={() => setFailed(true)}
            />
          </div>
        ) : (
          <div
            data-tauri-drag-region
            className="flex flex-col items-center gap-2 px-8 text-center text-muted-foreground"
          >
            <VideoOff data-tauri-drag-region className="size-8" />
            <span data-tauri-drag-region className="text-xs">
              {blocked ? t.recCameraBlocked : cameras.length === 0 && loaded ? t.recNoCamera : t.recCameraOff}
            </span>
            {blocked ? (
              <Button size="xs" variant="outline" onClick={() => api.openCameraSettings()}>
                {t.recCameraSettings}
              </Button>
            ) : null}
          </div>
        )}
      </div>

      <div className="flex gap-2 items-center rounded-full border border-foreground/20 bg-popover p-2 text-popover-foreground shadow-[0_4px_14px_rgba(0,0,0,0.45)]">
        <DropdownMenu open={open === "mode"} onOpenChange={(next) => setOpen(next ? "mode" : null)}>
          <Tooltip>
            <TooltipTrigger
              render={
                <DropdownMenuTrigger
                  render={
                    <Button variant="ghost" className="h-9 gap-1 rounded-full" aria-label={current.label} />
                  }
                />
              }
            >
              <current.Icon />
              <ChevronUp className="size-3 text-muted-foreground" />
            </TooltipTrigger>
            <TooltipContent side="top">{current.label}</TooltipContent>
          </Tooltip>
          <DropdownMenuContent side="top" className="w-auto min-w-44">
            <DropdownMenuRadioGroup value={mode} onValueChange={(v) => setMode(v as RecordMode)}>
              {modes.map(({ value, label, Icon }) => (
                <DropdownMenuRadioItem key={value} value={value} closeOnClick>
                  <Icon />
                  {label}
                </DropdownMenuRadioItem>
              ))}
            </DropdownMenuRadioGroup>
          </DropdownMenuContent>
        </DropdownMenu>

        <Separator orientation="vertical" className="h-6 data-vertical:self-center" />

        <Tooltip>
          <TooltipTrigger
            render={
              <Button
                variant="ghost"
                size="icon"
                className="size-8 rounded-full"
                aria-label={t.recStart}
                disabled={busy}
                onClick={start}
              />
            }
          >
            <Circle className="size-5 fill-[#ff623e] text-[#ff623e]" />
          </TooltipTrigger>
          <TooltipContent side="top">{t.recStart}</TooltipContent>
        </Tooltip>

        <Tooltip>
          <TooltipTrigger
            render={
              <Button
                variant="ghost"
                size="icon"
                className="size-8 rounded-full"
                aria-label={t.recClose}
                onClick={() => api.recordClose()}
              />
            }
          >
            <XIcon />
          </TooltipTrigger>
          <TooltipContent side="top">{t.recClose}</TooltipContent>
        </Tooltip>


        <Separator orientation="vertical" className="h-6 data-vertical:self-center" />


        <DropdownMenu open={open === "audio"} onOpenChange={(next) => setOpen(next ? "audio" : null)}>
          <Tooltip>
            <TooltipTrigger
              render={
                <DropdownMenuTrigger
                  render={
                    <Button
                      variant="ghost"
                      className="h-9 gap-1 rounded-full transition-[box-shadow] duration-75"
                      aria-label={t.recAudio}
                      style={{
                        boxShadow: mic === "" ? undefined : `0 0 0 ${ring}px rgb(34 197 94 / 0.75)`,
                      }}
                    />
                  }
                />
              }
            >
              {mic === "" ? <MicOff className="text-destructive" /> : <Mic />}
              {speaker === "" ? <VolumeX className="text-destructive" /> : <Volume2 />}
              <ChevronUp className="size-3 text-muted-foreground" />
            </TooltipTrigger>
            <TooltipContent side="top">
              {inputs.length === 0 ? t.recMicMissing : micName(mic)} · {speakerName(speaker)}
            </TooltipContent>
          </Tooltip>
          <DropdownMenuContent side="top" className="w-[calc(100vw-2rem)] overflow-hidden p-0">
            <ScrollArea className="[&>[data-slot=scroll-area-viewport]]:max-h-[calc(var(--available-height)-0.5rem)]">
              <div className="p-1">
                <DropdownMenuGroup>
                  <DropdownMenuLabel>{t.recMic}</DropdownMenuLabel>
                  <DropdownMenuRadioGroup value={mic} onValueChange={(v) => setMic(v as string)}>
                    <DropdownMenuRadioItem value="">{t.recMicNone}</DropdownMenuRadioItem>
                    <DropdownMenuRadioItem value="default">{t.recMicDefault}</DropdownMenuRadioItem>
                    {inputs.map((d) => (
                      <DropdownMenuRadioItem key={d.id} value={d.id}>
                        <span className="truncate">{d.name}</span>
                      </DropdownMenuRadioItem>
                    ))}
                  </DropdownMenuRadioGroup>
                  <DropdownMenuCheckboxItem
                    checked={denoise}
                    disabled={mic === ""}
                    onCheckedChange={(v) => setDenoise(v)}
                  >
                    {t.recDenoise}
                  </DropdownMenuCheckboxItem>
                </DropdownMenuGroup>
                <DropdownMenuSeparator />
                <DropdownMenuGroup>
                  <DropdownMenuLabel>{t.recSpeaker}</DropdownMenuLabel>
                  <DropdownMenuRadioGroup value={speaker} onValueChange={(v) => setSpeaker(v as string)}>
                    <DropdownMenuRadioItem value="">{t.recSpeakerNone}</DropdownMenuRadioItem>
                    <DropdownMenuRadioItem value="default">{t.recSpeakerDefault}</DropdownMenuRadioItem>
                    {outputs.map((d) => (
                      <DropdownMenuRadioItem key={d.id} value={d.id}>
                        <span className="truncate">{d.name}</span>
                      </DropdownMenuRadioItem>
                    ))}
                  </DropdownMenuRadioGroup>
                </DropdownMenuGroup>
              </div>
            </ScrollArea>
          </DropdownMenuContent>
        </DropdownMenu>

        <DropdownMenu open={open === "camera"} onOpenChange={(next) => setOpen(next ? "camera" : null)}>
          <Tooltip>
            <TooltipTrigger
              render={
                <DropdownMenuTrigger
                  render={
                    <Button variant="ghost" className="h-9 gap-1 rounded-full" aria-label={cameraName(camera)} />
                  }
                />
              }
            >
              {camera === "" ? <VideoOff className="text-destructive" /> : <Video />}
              <ChevronUp className="size-3 text-muted-foreground" />
            </TooltipTrigger>
            <TooltipContent side="top">{cameraName(camera)}</TooltipContent>
          </Tooltip>
          <DropdownMenuContent side="top" className="w-[calc(100vw-2rem)]">
            <DropdownMenuRadioGroup value={camera} onValueChange={(v) => setCamera(v as string)}>
              <DropdownMenuRadioItem value="" closeOnClick>
                {t.recCameraOff}
              </DropdownMenuRadioItem>
              {cameras.map((c) => (
                <DropdownMenuRadioItem key={c.id} value={c.id} closeOnClick>
                  <span className="truncate">{c.name}</span>
                </DropdownMenuRadioItem>
              ))}
            </DropdownMenuRadioGroup>
            <DropdownMenuSeparator />
            <DropdownMenuGroup>
              <DropdownMenuLabel>{t.recBackground}</DropdownMenuLabel>
              <DropdownMenuRadioGroup value={effect} onValueChange={(v) => setEffect(v as CameraEffect)}>
                <DropdownMenuRadioItem value="none" disabled={camera === ""}>
                  {t.recBackgroundNone}
                </DropdownMenuRadioItem>
                <DropdownMenuRadioItem value="blur" disabled={camera === ""}>
                  {t.recBlur}
                </DropdownMenuRadioItem>
                <DropdownMenuRadioItem value="image" disabled={camera === "" || !state.camera_image}>
                  {t.recBackgroundImage}
                </DropdownMenuRadioItem>
              </DropdownMenuRadioGroup>
            </DropdownMenuGroup>
          </DropdownMenuContent>
        </DropdownMenu>
      </div>
    </div>
  );
}
