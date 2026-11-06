import { ImageSegmenter } from "@mediapipe/tasks-vision";
import wasmLoaderPath from "@mediapipe/tasks-vision/vision_wasm_internal.js?url";
import wasmBinaryPath from "@mediapipe/tasks-vision/vision_wasm_internal.wasm?url";
import modelAssetPath from "@/assets/selfie_segmenter.tflite?url";
import type { CameraLook } from "@/lib/api";

const MASK_WIDTH = 256;
const OUTPUT_HEIGHT = 360;
const PERSON_EDGE = [0.35, 0.65] as const;
const CUT_EDGE = [0.12, 0.4] as const;
const FILL_BOOST = 4;
const IMAGE_EDGE_SPAN = 0.4;

export type EffectOptions = CameraLook & { background: ImageBitmap | null };

let shared: Promise<ImageSegmenter> | null = null;

function segmenter() {
  shared ??= (async () => {
    const create = (delegate: "GPU" | "CPU") =>
      ImageSegmenter.createFromOptions(
        { wasmLoaderPath, wasmBinaryPath },
        {
          baseOptions: { modelAssetPath, delegate },
          runningMode: "VIDEO",
          outputCategoryMask: false,
          outputConfidenceMasks: true,
        },
      );
    try {
      return await create("GPU");
    } catch {
      return create("CPU");
    }
  })();
  return shared;
}

function context(canvas: HTMLCanvasElement, read = false) {
  const ctx = canvas.getContext("2d", { willReadFrequently: read });
  if (!ctx) throw new Error("canvas");
  return ctx;
}

function smoothstep([low, high]: readonly [number, number], v: number) {
  const t = Math.min(1, Math.max(0, (v - low) / (high - low)));
  return t * t * (3 - 2 * t);
}

function blurInto(source: HTMLCanvasElement, target: CanvasRenderingContext2D, radius: number, boost: number | null) {
  const { width, height } = target.canvas;
  target.clearRect(0, 0, width, height);
  target.filter = `blur(${radius}px)`;
  target.drawImage(source, 0, 0);
  target.filter = "none";
  const image = target.getImageData(0, 0, width, height);
  const data = image.data;
  for (let i = 3; i < data.length; i += 4) data[i] = boost === null ? 255 : Math.min(255, data[i] * boost);
  target.putImageData(image, 0, 0);
}

function drawCover(ctx: CanvasRenderingContext2D, image: ImageBitmap, w: number, h: number) {
  const scale = Math.max(w / image.width, h / image.height);
  const dw = image.width * scale;
  const dh = image.height * scale;
  ctx.save();
  ctx.translate(w, 0);
  ctx.scale(-1, 1);
  ctx.drawImage(image, (w - dw) / 2, (h - dh) / 2, dw, dh);
  ctx.restore();
}

export function startBlur(video: HTMLVideoElement, canvas: HTMLCanvasElement, options: () => EffectOptions): () => void {
  let alive = true;
  let model: ImageSegmenter | null = null;
  segmenter()
    .then((m) => {
      if (alive) model = m;
    })
    .catch(() => {});

  const make = (read = false) => {
    const c = document.createElement("canvas");
    return { canvas: c, ctx: context(c, read) };
  };
  const small = make();
  const personMask = make();
  const cutMask = make();
  const cut = make();
  const plain = make(true);
  const fill = make(true);
  const person = make();
  const feather = make();
  const wrapSource = make(true);
  const wrap = make();
  const ctx = context(canvas);
  let wrapFor: ImageBitmap | null = null;
  let wrapRadius = 0;
  let personImage: ImageData | null = null;
  let cutImage: ImageData | null = null;
  let previous: Float32Array | null = null;
  let hasMask = false;
  let shown = -1;
  let stamp = 0;
  let handle = 0;

  const segment = () => {
    if (!model || !personImage || !cutImage) return;
    stamp = Math.max(performance.now(), stamp + 1);
    const personData = personImage.data;
    const cutData = cutImage.data;
    const look = options();
    const smoothing = Math.min(0.9, Math.max(0, look.smoothing / 100));
    const low = look.edge / 100;
    const edge = look.background ? ([low, low + IMAGE_EDGE_SPAN] as const) : PERSON_EDGE;
    let updated = false;
    try {
      model.segmentForVideo(small.canvas, stamp, (result) => {
        const confidence = result.confidenceMasks?.[0];
        if (!confidence) return;
        const values = confidence.getAsFloat32Array();
        if (!previous || previous.length !== values.length) previous = Float32Array.from(values);
        for (let i = 0; i < values.length; i++) {
          const v = previous[i] * smoothing + values[i] * (1 - smoothing);
          previous[i] = v;
          personData[i * 4 + 3] = smoothstep(edge, v) * 255;
          cutData[i * 4 + 3] = smoothstep(CUT_EDGE, v) * 255;
        }
        updated = true;
      });
    } catch {
      return;
    }
    if (updated) {
      personMask.ctx.putImageData(personImage, 0, 0);
      cutMask.ctx.putImageData(cutImage, 0, 0);
      hasMask = true;
    }
  };

  const draw = () => {
    if (!alive) return;
    handle = requestAnimationFrame(draw);
    const vw = video.videoWidth;
    const vh = video.videoHeight;
    if (!vw || !vh || video.currentTime === shown) return;
    shown = video.currentTime;

    const h = Math.min(OUTPUT_HEIGHT, vh);
    const w = Math.round((vw * h) / vh);
    if (canvas.width !== w || canvas.height !== h) {
      canvas.width = person.canvas.width = w;
      canvas.height = person.canvas.height = h;
    }
    const mh = Math.max(1, Math.round((MASK_WIDTH * vh) / vw));
    if (small.canvas.width !== MASK_WIDTH || small.canvas.height !== mh) {
      for (const layer of [small, personMask, cutMask, cut, plain, fill, feather, wrapSource, wrap]) {
        layer.canvas.width = MASK_WIDTH;
        layer.canvas.height = mh;
      }
      personImage = personMask.ctx.createImageData(MASK_WIDTH, mh);
      cutImage = cutMask.ctx.createImageData(MASK_WIDTH, mh);
      previous = null;
      hasMask = false;
      wrapFor = null;
    }

    small.ctx.drawImage(video, 0, 0, MASK_WIDTH, mh);
    segment();

    const { strength, background, feather: featherPx, wrap: wrapPercent, wrap_radius } = options();
    const radius = Math.max(1, Math.round((mh * strength) / 100));
    ctx.imageSmoothingQuality = "high";
    if (background && hasMask) {
      drawCover(ctx, background, w, h);
    } else {
      blurInto(small.canvas, plain.ctx, radius, null);
      ctx.drawImage(plain.canvas, 0, 0, w, h);
    }

    if (hasMask) {
      if (!background) {
        cut.ctx.globalCompositeOperation = "copy";
        cut.ctx.drawImage(small.canvas, 0, 0);
        cut.ctx.globalCompositeOperation = "destination-out";
        cut.ctx.drawImage(cutMask.canvas, 0, 0);
        cut.ctx.globalCompositeOperation = "source-over";
        blurInto(cut.canvas, fill.ctx, radius, FILL_BOOST);
        ctx.drawImage(fill.canvas, 0, 0, w, h);
      }

      let edgeMask = personMask.canvas;
      if (background && featherPx > 0) {
        feather.ctx.clearRect(0, 0, MASK_WIDTH, mh);
        feather.ctx.filter = `blur(${featherPx}px)`;
        feather.ctx.drawImage(personMask.canvas, 0, 0);
        feather.ctx.filter = "none";
        edgeMask = feather.canvas;
      }

      person.ctx.globalCompositeOperation = "copy";
      person.ctx.drawImage(video, 0, 0, w, h);
      person.ctx.globalCompositeOperation = "destination-in";
      person.ctx.imageSmoothingQuality = "high";
      person.ctx.drawImage(edgeMask, 0, 0, w, h);
      ctx.drawImage(person.canvas, 0, 0);

      if (background && wrapPercent > 0) {
        if (wrapFor !== background || wrapRadius !== wrap_radius) {
          cut.ctx.globalCompositeOperation = "source-over";
          cut.ctx.clearRect(0, 0, MASK_WIDTH, mh);
          drawCover(cut.ctx, background, MASK_WIDTH, mh);
          blurInto(cut.canvas, wrapSource.ctx, wrap_radius, null);
          wrapFor = background;
          wrapRadius = wrap_radius;
        }
        wrap.ctx.globalCompositeOperation = "copy";
        wrap.ctx.drawImage(wrapSource.canvas, 0, 0);
        wrap.ctx.globalCompositeOperation = "destination-out";
        wrap.ctx.filter = `blur(${wrap_radius}px)`;
        wrap.ctx.drawImage(personMask.canvas, 0, 0);
        wrap.ctx.filter = "none";
        wrap.ctx.globalCompositeOperation = "destination-in";
        wrap.ctx.drawImage(edgeMask, 0, 0);
        wrap.ctx.globalCompositeOperation = "source-over";
        ctx.globalAlpha = wrapPercent / 100;
        ctx.drawImage(wrap.canvas, 0, 0, w, h);
        ctx.globalAlpha = 1;
      }
    }
  };

  handle = requestAnimationFrame(draw);
  return () => {
    alive = false;
    cancelAnimationFrame(handle);
  };
}
