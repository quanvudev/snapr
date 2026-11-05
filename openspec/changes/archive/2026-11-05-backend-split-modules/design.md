# Design

## Context

See proposal.md for the motivation. Current coupling that shapes the split:

- `crate::State` is imported by `gallery`, `paddle`, `queue`, `settings`, `transcript`, `update`, `video`, `whisper`, `panel` and `ring`; `crate::Action` by `gallery`, `mouse`, `panel`, `recorder` and `ring`.
- Functions in `lib.rs` called from other modules: `tooltip` (`panel`, `ring`), `apply` (`paddle`, `settings`), `trigger` (`panel`, `ring`), `stop_recording` and `pause_recording` (`panel`), `show_settings` (`settings`, `ring`, `update`).
- `paddle.rs` exports `Remote`, `Fetch`, `agent`, `clean` and `part` to `whisper.rs`, `onnx.rs` and `update.rs`; `disk_bytes` is used only by `paddle.rs` but is not Paddle-specific.
- Emit sites: `emit_to("main", ...)` in `lib.rs`, `queue.rs`, `paddle.rs`, `whisper.rs`, `update.rs`, `settings`/`show_settings`; `app.emit("record-panel", ..)` (broadcast) and `emit_to(PANEL, "record-level", ..)` in `panel.rs`.
- Two threads: the Tauri main thread and the winit capture thread. Several `lib.rs` functions are called from both (for example `ended` from the capture thread, `stop_recording` from the main thread), and some hop threads (`close_panel` uses `run_on_main_thread`).
- There are no tests; `cargo clippy` and a manual smoke test are the only checks.

## Goals / Non-Goals

**Goals:**
- Every moved function keeps its body unchanged apart from paths (`crate::x` to `crate::module::x`) and visibility.
- After the change, `lib.rs` contains only shared types (`Action`, `State`) and app wiring.
- Each step builds and runs, so the work can be committed and bisected step by step.

**Non-Goals:**
- No renames of functions, types or fields beyond what a move requires; no new traits, no signature changes, no reordering of locks or thread hops.
- No change to `State`'s fields or to `Action`'s variants (P3 and P4 do that).
- No `pub use` re-exports from `lib.rs` to preserve old paths.

## Decisions

**`Action` and `State` stay in `lib.rs`.** They are the shared contract between the two threads and are imported by most modules as `crate::Action` / `crate::State`. Moving them would churn about fifteen import lines for no structural gain, and P3 reshapes `State` anyway. Alternative considered: an `app.rs` holding both; rejected for this step for the same reason.

**`recording.rs` is separate from `snap.rs`.** `record`, `ended`, `close_panel`, `stop_recording`, `pause_recording` and `stop_after_limit` operate on `State.recording` / `State.capture` and are called from the main thread (panel commands, tray) as well as from `Snap`. Putting them in `snap.rs` would make main-thread code depend on the event-loop module. A separate module also marks out exactly what P4's `Recording` object will absorb. Alternative: everything recording-related inside `snap.rs`, as first sketched; rejected because of that dependency.

**`trigger` goes to `hotkeys.rs`.** It is the dispatch for hotkeys and the ring (the Record branch calls `recording::stop_recording` and `panel::toggle`, everything else forwards the `Action`). It sits next to `apply`, which registers the closures that call it. `ocr_available` also lives here since `apply` is its main caller; `show_settings` imports it.

**`show_settings` and `show_splash` move into `settings.rs`.** They manage the `main` settings window and `settings.rs` already owns its commands (`open_settings` calls `show_settings`). Alternative: a `window.rs`; rejected as one more tiny module.

**`jobs.rs` exposes a spawn function.** `jobs::spawn(app, proxy) -> Sender<Job>` starts the worker thread (MTA init, `BackgroundCache`, the `for job in queue` loop) and is called from `spawn_capture` at the same point as today, so the worker keeps its current start order and thread. `Job`, `work` and `read_text` move with it.

**`snap.rs` owns the capture thread.** `Snap`, its `ApplicationHandler` impl, `Mode`, `Countdown`, the `COUNTDOWN` / `TICK` / `SETTLE` / `MIN_AREA` constants, the target helpers and `spawn_capture`. `Snap`'s fields stay private; `lib.rs` calls only `snap::spawn_capture`.

**`tray.rs` owns the tray.** `tray_menu`, `build_tray` (with its menu event closure), `tooltip` and `refresh`. `State` keeps holding the `MenuItem`s; `setup` still builds them before `app.manage` and calls `tray::build_tray` after it, in the same order.

**`fetch.rs` takes the generic download code.** `Remote`, `Fetch` (`get`, `stream`, `file`), `agent`, `clean`, `part` and `disk_bytes` move as they are, `pub(crate)`. `paddle.rs` keeps its model table, dictionary parsing and commands. The literal `"cancelled"` returned by `Fetch` stays as is (unifying it with `queue::CANCELLED` is a behaviour question for P2).

**`events.rs` holds plain `&str` constants**, one per event name: `GALLERY_CHANGED`, `PADDLE_PROGRESS`, `PADDLE_DONE`, `WHISPER_PROGRESS`, `WHISPER_DONE`, `QUEUE`, `QUEUE_ENDED`, `QUEUE_HISTORY`, `SETTINGS_OPEN`, `UPDATE_STATUS`, `RECORD_PANEL`, `RECORD_LEVEL`. Each emit site keeps its current target (`emit_to("main", ..)`, the broadcast `emit` for `record-panel`, `emit_to(PANEL, ..)` for `record-level`); only the name becomes a constant. Window labels are not part of this step. Alternative: an `Event` enum with `as_str`; rejected as more machinery than twelve names need.

**Visibility.** Moved items become `pub(crate)` only when another module calls them; everything else stays private.

## Risks / Trade-offs

- [A moved function ends up called on a different thread] → bodies and call sites are moved verbatim; `close_panel`, `trigger` and the panel calls keep their `run_on_main_thread` hops, and the smoke test covers recording start, stop, pause, discard and countdown cancel from the hotkey, panel, ring and tray.
- [An event name is mistyped while replacing a literal] → after the change, `grep` finds no event-name string literal outside `events.rs`, and every constant's value is checked against the current literal and the frontend `listen` calls.
- [Import churn hides an accidental logic edit in review] → one commit per module move, each building cleanly, so each diff is mostly moved lines.
- [`CLAUDE.md` drifts from the new layout] → a dedicated task updates every path and function location it names in the Architecture section.
- [Conflicts with other in-flight backend work] → the tree is clean and no other changes are open; land this before starting P2.

## Migration Plan

No data or config migration. Land as a series of commits on `master` (one per moved module, then events, then docs), each passing `cargo clippy`. Rollback is a `git revert` of the affected commits. No release is needed for this change on its own.
