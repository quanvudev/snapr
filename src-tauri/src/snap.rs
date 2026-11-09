use crate::config;
use crate::jobs::{self, Job};
use crate::overlay::{Outcome, Overlay, Pick, Rect, Shot};
use crate::recorder::Target;
use crate::recording::{ended, record};
use crate::ring::{self, Busy, Entry, Ring};
use crate::toast::{self, Toast};
use crate::tray::tooltip;
use crate::{Action, State, capture, draw, events, frame, ocr, paddle, panel, pin, redact, transcript, video};
use image::RgbaImage;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::mpsc::Sender;
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager};
use winit::application::ApplicationHandler;
use winit::event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy};
use winit::platform::windows::EventLoopBuilderExtWindows;
use winit::window::WindowId;
use windows::Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize};
use windows_sys::Win32::Foundation::RECT;
use windows_sys::Win32::UI::WindowsAndMessaging::{GetClassNameW, GetCursorPos, GetWindowRect};

const COUNTDOWN: u32 = 3;

const TICK: Duration = Duration::from_secs(1);

const SETTLE: Duration = Duration::from_millis(50);

const MIN_AREA: i32 = 16;

struct Countdown {
    target: Target,
    area: Option<Rect>,
    step: u32,
    next: Instant,
    digit: Option<frame::Digit>,
}

#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Shot,
    Record,
    Text,
}

fn is_desktop(hwnd: isize) -> bool {
    let mut buf = [0u16; 32];
    let len = unsafe { GetClassNameW(hwnd as _, buf.as_mut_ptr(), buf.len() as i32) };
    let name = String::from_utf16_lossy(&buf[..len.max(0) as usize]);
    name == "Progman" || name == "WorkerW"
}

fn cursor_monitor() -> Option<(Target, Rect)> {
    let mut p = unsafe { std::mem::zeroed() };
    unsafe { GetCursorPos(&mut p) };
    let (monitor, rect, _) = frame::monitor_at((p.x, p.y))?;
    Some((Target::Monitor(monitor), rect))
}

fn window_rect(hwnd: isize) -> Rect {
    let mut r = RECT { left: 0, top: 0, right: 0, bottom: 0 };
    unsafe { GetWindowRect(hwnd as _, &mut r) };
    (r.left, r.top, r.right - r.left, r.bottom - r.top)
}

fn region_target(rect: Rect, start: (i32, i32), hwnd: Option<isize>) -> Option<(Target, Rect, Option<Rect>)> {
    let (monitor, (mx, my, mw, mh), _) = frame::monitor_at(start)?;
    let (x0, y0) = (rect.0.max(mx), rect.1.max(my));
    let w = ((rect.0 + rect.2).min(mx + mw) - x0).max(0) & !1;
    let h = ((rect.1 + rect.3).min(my + mh) - y0).max(0) & !1;
    if w < MIN_AREA || h < MIN_AREA {
        return Some(match hwnd.filter(|&hwnd| !is_desktop(hwnd)) {
            Some(hwnd) => (Target::Window(hwnd), window_rect(hwnd), None),
            None => (Target::Monitor(monitor), (mx, my, mw, mh), None),
        });
    }
    let area = Target::Area { monitor, rect: ((x0 - mx) as u32, (y0 - my) as u32, w as u32, h as u32) };
    Some((area, (x0, y0, w, h), Some((x0, y0, w, h))))
}

struct Snap {
    app: AppHandle,
    overlay: Option<Overlay>,
    ring: Option<Ring>,
    toast: Option<Toast>,
    pins: pin::Pins,
    jobs: Sender<Job>,
    mode: Mode,
    style: draw::Style,
    countdown: Option<Countdown>,
    frame: Option<frame::Frame>,
}

impl ApplicationHandler<Action> for Snap {
    fn resumed(&mut self, _event_loop: &ActiveEventLoop) {}

    fn about_to_wait(&mut self, event_loop: &ActiveEventLoop) {
        if self.toast.as_ref().is_some_and(|t| t.expired()) {
            self.toast = None;
        }
        if let Some(ring) = self.ring.as_mut() {
            ring.tick();
        }
        if let Some(entry) = self.ring.take_if(|ring| ring.finished()).and_then(Ring::into_picked) {
            entry.run(&self.app);
        }
        self.pins.tick();
        if let Some(overlay) = self.overlay.as_mut() {
            overlay.tick();
        }
        self.tick_countdown(event_loop);
        let deadline = [
            self.countdown.as_ref().map(|c| c.next),
            self.toast.as_ref().and_then(|t| t.deadline()),
            self.ring.as_ref().map(|r| r.deadline()),
            self.pins.deadline(),
            self.overlay.as_ref().and_then(Overlay::deadline),
        ]
        .into_iter()
        .flatten()
        .min();
        let flow = match deadline {
            Some(deadline) => ControlFlow::WaitUntil(deadline),
            None => ControlFlow::Wait,
        };
        event_loop.set_control_flow(flow);
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, id: WindowId, event: winit::event::WindowEvent) {
        if let Some(ring) = self.ring.as_mut().filter(|r| r.id() == id) {
            match ring.event(event) {
                ring::Outcome::Pending => {}
                ring::Outcome::Cancel => ring.begin_close(None),
                ring::Outcome::Pick(entry) => ring.begin_close(Some(entry)),
            }
            return;
        }
        if let Some(toast) = self.toast.as_mut().filter(|t| t.id() == id) {
            match toast.event(event) {
                toast::Outcome::Pending => {}
                toast::Outcome::Close => self.toast = None,
                toast::Outcome::Pin => {
                    let path = toast.path().to_path_buf();
                    self.toast = None;
                    self.open_pin(event_loop, path);
                }
            }
            return;
        }
        if self.pins.contains(id) {
            match self.pins.event(id, event) {
                pin::Outcome::Pending => {}
                pin::Outcome::Close => self.pins.remove(id),
                pin::Outcome::Copy => match self.pins.image(id).map(capture::copy) {
                    Some(Ok(())) => self.pins.copied(id),
                    Some(Err(e)) => tooltip(&self.app, &format!("Snapr: {e}")),
                    None => {}
                },
                pin::Outcome::Text => {
                    if let Some(image) = self.pins.image(id) {
                        let _ = self.jobs.send(Job::Text(Shot::Ready(image.clone())));
                    }
                }
            }
            return;
        }
        let Some(overlay) = self.overlay.as_mut().filter(|o| o.id() == id) else { return };
        let outcome = overlay.event(event);
        if !matches!(outcome, Outcome::Pending | Outcome::Detect { .. })
            && let Some(style) = overlay.style()
        {
            self.style = style;
        }
        match outcome {
            Outcome::Pending => {}
            Outcome::Detect { token, image } => detect(&self.app, token, image),
            Outcome::Cancel => {
                self.overlay = None;
                if std::mem::replace(&mut self.mode, Mode::Shot) == Mode::Record {
                    self.panel_cancelled();
                }
                self.pins.show_capture();
            }
            Outcome::Copy(img) => {
                self.overlay = None;
                self.mode = Mode::Shot;
                self.pins.show_capture();
                if let Err(e) = capture::copy(&img) {
                    tooltip(&self.app, &format!("Snapr: {e}"));
                }
            }
            Outcome::Pin(img) => {
                self.overlay = None;
                self.mode = Mode::Shot;
                self.pins.show_capture();
                let ocr = self.app.state::<State>().ocr.load(Ordering::Relaxed);
                if let Err(e) = self.pins.add(event_loop, img, ocr) {
                    tooltip(&self.app, &format!("Snapr: {e}"));
                }
            }
            Outcome::Save(shot) => {
                self.overlay = None;
                match (std::mem::replace(&mut self.mode, Mode::Shot), shot) {
                    (Mode::Record, Shot::Window { hwnd, .. }) if is_desktop(hwnd) => match cursor_monitor() {
                        Some((target, bounds)) => self.begin(event_loop, target, bounds, None),
                        None => {
                            self.pins.show_capture();
                            tooltip(&self.app, "Snapr: capture failed");
                        }
                    },
                    (Mode::Record, Shot::Window { hwnd, .. }) => {
                        let camera = !panel::choices(&self.app).1.is_empty();
                        let rect = window_rect(hwnd);
                        let centre = (rect.0 + rect.2 / 2, rect.1 + rect.3 / 2);
                        match camera.then(|| region_target(rect, centre, Some(hwnd))).flatten() {
                            Some((target, bounds, area)) => self.begin(event_loop, target, bounds, area),
                            None => self.begin(event_loop, Target::Window(hwnd), rect, None),
                        }
                    }
                    (Mode::Record, Shot::Region { rect, start, hwnd }) => match region_target(rect, start, hwnd) {
                        Some((target, bounds, area)) => self.begin(event_loop, target, bounds, area),
                        None => {
                            self.pins.show_capture();
                            tooltip(&self.app, "Snapr: capture failed");
                        }
                    },
                    (Mode::Text, shot) => {
                        self.pins.show_capture();
                        let _ = self.jobs.send(Job::Text(shot));
                    }
                    (_, shot) => {
                        self.pins.show_capture();
                        let redact = matches!(shot, Shot::Window { .. });
                        let _ = self.jobs.send(Job::Save(shot, redact));
                    }
                }
            }
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, action: Action) {
        match action {
            Action::Fullscreen => {
                let overlay = self.overlay.is_some();
                self.pins.hide_capture();
                let result = capture::fullscreen();
                if !overlay {
                    self.pins.show_capture();
                }
                match result {
                    Ok(img) => {
                        let _ = self.jobs.send(Job::Save(Shot::Ready(img), true));
                    }
                    Err(e) => tooltip(&self.app, &format!("Snapr: {e}")),
                }
            }
            Action::Pick(pick) => self.pick(event_loop, pick, Mode::Shot, false),
            Action::Text => self.pick(event_loop, Pick::Area, Mode::Text, false),
            Action::Record => {
                self.cancel_countdown();
            }
            Action::RecordPick(mode) => self.record_pick(event_loop, mode),
            Action::CancelCountdown => {
                self.cancel_countdown();
            }
            Action::Captions => {}
            Action::Draw => self.pick(event_loop, Pick::Area, Mode::Shot, true),
            Action::Redactions { token, rects } => {
                if let Some(overlay) = self.overlay.as_mut() {
                    overlay.redactions(token, rects);
                }
            }
            Action::Ring => self.open_ring(event_loop),
            Action::Pin(path) => self.open_pin(event_loop, path),
            Action::PinMany(paths) => self.open_pins(event_loop, paths),
            Action::Saved(img, path) => {
                let _ = self.app.emit_to("main", events::GALLERY_CHANGED, ());
                self.preview(event_loop, &img, path, true);
            }
            Action::Recorded(img, path, has_audio) => {
                self.frame = None;
                self.pins.show_recording();
                ended(&self.app);
                if self.app.state::<State>().discard.swap(false, Ordering::Relaxed) {
                    let _ = std::fs::remove_file(&path);
                    return;
                }
                let (compress, height, quality, transcribe) = {
                    let state = self.app.state::<State>();
                    let cfg = state.cfg.read().unwrap();
                    (cfg.record_compress, cfg.compress_height(), cfg.quality(), cfg.record_transcribe)
                };
                if transcribe && has_audio {
                    self.queue_transcript(&path);
                }
                if compress {
                    video::compress_recording(&self.app, path.clone(), height, quality);
                }
                let _ = self.app.emit_to("main", events::GALLERY_CHANGED, ());
                self.preview(event_loop, &img, path, false);
            }
            Action::RecordFailed(e) => {
                self.app.state::<State>().discard.store(false, Ordering::Relaxed);
                self.frame = None;
                self.pins.show_recording();
                ended(&self.app);
                tooltip(&self.app, &format!("Snapr: {e}"));
            }
        }
    }
}

impl Snap {
    fn queue_transcript(&self, path: &std::path::Path) {
        let Some(path) = path.to_str() else { return };
        let translate = self.app.state::<State>().cfg.read().unwrap().record_translate;
        if let Ok(task) = transcript::TranscribeTask::validated(&self.app, path, transcript::Track::Original) {
            let task = task.then_translate(translate);
            let _ = self.app.state::<State>().queue.add(&self.app, Box::new(task), None);
        }
    }

    fn panel_cancelled(&self) {
        let handle = self.app.clone();
        let _ = self.app.run_on_main_thread(move || panel::cancelled(&handle));
    }

    fn begin(&mut self, event_loop: &ActiveEventLoop, target: Target, bounds: Rect, area: Option<Rect>) {
        let handle = self.app.clone();
        let _ = self.app.run_on_main_thread(move || panel::picked(&handle, bounds));
        self.pins.hide_recording();
        self.pins.show_capture();
        let state = self.app.state::<State>();
        let countdown = state.cfg.read().unwrap().record_countdown;
        if countdown && let Some(digit) = frame::Digit::open(event_loop, bounds, COUNTDOWN) {
            self.countdown = Some(Countdown { target, area, step: COUNTDOWN, next: Instant::now() + TICK, digit: Some(digit) });
            state.counting.store(true, Ordering::Relaxed);
            let _ = state.stop.set_enabled(true);
            tooltip(&self.app, "Snapr: recording starts…");
            return;
        }
        self.start(event_loop, target, area);
    }

    fn start(&mut self, event_loop: &ActiveEventLoop, target: Target, area: Option<Rect>) {
        if record(&self.app, target) {
            self.frame = area.and_then(|area| frame::Frame::open(event_loop, area));
        } else {
            self.pins.show_recording();
            let _ = self.app.state::<State>().stop.set_enabled(false);
        }
    }

    fn tick_countdown(&mut self, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let Some(countdown) = self.countdown.as_mut().filter(|c| now >= c.next) else { return };
        if countdown.step > 1 {
            countdown.step -= 1;
            countdown.next = now + TICK;
            if let Some(digit) = &countdown.digit {
                digit.show(countdown.step);
            }
        } else if countdown.digit.take().is_some() {
            countdown.next = now + SETTLE;
        } else if let Some(Countdown { target, area, .. }) = self.countdown.take() {
            self.app.state::<State>().counting.store(false, Ordering::Relaxed);
            self.start(event_loop, target, area);
        }
    }

    fn cancel_countdown(&mut self) -> bool {
        if self.countdown.take().is_none() {
            return false;
        }
        self.app.state::<State>().counting.store(false, Ordering::Relaxed);
        self.pins.show_recording();
        ended(&self.app);
        true
    }

    fn record_pick(&mut self, event_loop: &ActiveEventLoop, mode: config::RecordMode) {
        if self.overlay.is_some() {
            self.panel_cancelled();
            return;
        }
        let pick = match mode {
            config::RecordMode::Fullscreen => {
                match cursor_monitor() {
                    Some((target, bounds)) => self.begin(event_loop, target, bounds, None),
                    None => {
                        tooltip(&self.app, "Snapr: capture failed");
                        self.panel_cancelled();
                    }
                }
                return;
            }
            config::RecordMode::Window => Pick::Window,
            config::RecordMode::Area => Pick::Area,
            config::RecordMode::Captions => {
                self.panel_cancelled();
                return;
            }
        };
        self.pick(event_loop, pick, Mode::Record, false);
        if self.mode != Mode::Record {
            self.panel_cancelled();
        }
    }

    fn pick(&mut self, event_loop: &ActiveEventLoop, pick: Pick, mode: Mode, draw: bool) {
        if self.overlay.is_some() {
            return;
        }
        self.toast = None;
        self.pins.hide_capture();
        let state = self.app.state::<State>();
        let (vi, detectable) = {
            let cfg = state.cfg.read().unwrap();
            self.style.redact = cfg.redact_style();
            let ocr = state.ocr.load(Ordering::Relaxed) || paddle::for_redaction(&self.app, &cfg).is_some();
            let ready = ocr && !redact::Rules::from(&cfg.redact).is_empty();
            (cfg.language == "vi", ready)
        };
        let record = mode == Mode::Record && matches!(pick, Pick::Area);
        self.overlay = Overlay::open(event_loop, pick, record, draw.then_some(self.style), vi, detectable);
        self.mode = if self.overlay.is_some() { mode } else { Mode::Shot };
        if self.overlay.is_none() {
            self.pins.show_capture();
            tooltip(&self.app, "Snapr: capture failed");
        }
    }

    fn open_ring(&mut self, event_loop: &ActiveEventLoop) {
        if let Some(ring) = self.ring.as_mut() {
            ring.begin_close(None);
            return;
        }
        if self.overlay.is_some() {
            return;
        }
        self.toast = None;
        let state = self.app.state::<State>();
        let (text, vi, items, enabled, keys, idle) = {
            let cfg = state.cfg.read().unwrap();
            let idle = (cfg.ring.idle_close > 0).then(|| Duration::from_secs(cfg.ring.idle_close.into()));
            (cfg.text_capture && state.ocr.load(Ordering::Relaxed), cfg.language == "vi", cfg.ring_items(), cfg.ring.enabled, cfg.ring.keys, idle)
        };
        if !enabled {
            return;
        }
        let busy = Busy { recording: state.recording.lock().unwrap().is_some(), captioning: crate::captions::session::running(&self.app) };
        let counting = state.counting.load(Ordering::Relaxed);
        let entries: Vec<Entry> = items.iter().map(Entry::from).filter(|entry| entry.available(text, busy, counting)).collect();
        if entries.is_empty() {
            return;
        }
        self.ring = Ring::open(event_loop, entries, busy, vi, keys, idle);
        if self.ring.is_none() {
            tooltip(&self.app, "Snapr: ring failed");
        }
    }

    fn preview(&mut self, event_loop: &ActiveEventLoop, img: &RgbaImage, path: PathBuf, pinnable: bool) {
        self.toast = None;
        let secs = self.app.state::<State>().cfg.read().unwrap().preview_seconds;
        if secs > 0 && self.overlay.is_none() {
            self.toast = Toast::open(event_loop, img, path, secs, pinnable);
        }
    }

    fn open_pin(&mut self, event_loop: &ActiveEventLoop, path: PathBuf) {
        let image = match image::open(&path) {
            Ok(image) => image.to_rgba8(),
            Err(e) => {
                tooltip(&self.app, &format!("Snapr: {e}"));
                return;
            }
        };
        let ocr = self.app.state::<State>().ocr.load(Ordering::Relaxed);
        if let Err(e) = self.pins.add(event_loop, image, ocr) {
            tooltip(&self.app, &format!("Snapr: {e}"));
        }
    }

    fn open_pins(&mut self, event_loop: &ActiveEventLoop, paths: Vec<PathBuf>) {
        let ocr = self.app.state::<State>().ocr.load(Ordering::Relaxed);
        for (index, path) in paths.into_iter().enumerate() {
            let image = match image::open(&path) {
                Ok(image) => image.to_rgba8(),
                Err(e) => {
                    tooltip(&self.app, &format!("Snapr: {e}"));
                    continue;
                }
            };
            let offset = ((index as i32) * 24, (index as i32) * 24);
            if let Err(e) = self.pins.add_at(event_loop, image, offset, ocr) {
                tooltip(&self.app, &format!("Snapr: {e}"));
            }
        }
    }
}

fn detect(app: &AppHandle, token: u64, image: RgbaImage) {
    let app = app.clone();
    std::thread::spawn(move || {
        let _ = unsafe { RoInitialize(RO_INIT_MULTITHREADED) };
        let state = app.state::<State>();
        let (rules, model, prefer) = {
            let cfg = state.cfg.read().unwrap();
            (redact::Rules::from(&cfg.redact), paddle::for_redaction(&app, &cfg), cfg.redact_paddle())
        };
        let rects = if rules.is_empty() {
            Vec::new()
        } else {
            ocr::words(&image, model.as_ref(), prefer).map(|lines| redact::find(&lines, &rules, image.dimensions())).unwrap_or_default()
        };
        let _ = state.proxy.lock().unwrap().send_event(Action::Redactions { token, rects });
    });
}

pub(crate) fn spawn_capture(app: AppHandle) -> EventLoopProxy<Action> {
    let (tx, rx) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        let event_loop = EventLoop::<Action>::with_user_event()
            .with_any_thread(true)
            .build()
            .expect("capture event loop");
        let proxy = event_loop.create_proxy();
        let _ = tx.send(proxy.clone());
        let jobs = jobs::spawn(app.clone(), proxy);
        let _ = event_loop.run_app(&mut Snap {
            app,
            overlay: None,
            ring: None,
            toast: None,
            pins: pin::Pins::default(),
            jobs,
            mode: Mode::Shot,
            style: draw::Style::default(),
            countdown: None,
            frame: None,
        });
    });
    rx.recv().expect("capture event loop")
}
