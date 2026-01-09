# Architecture and interaction guidelines

- **Persistence**: Only `AppState` is serialized/restored (`save`/`new`). Do not serialize `AudioRuntime` or other runtime-only components.
- **Layers**:
  - **UI (App)**: Thin shell. Builds widgets, forwards `UiEvent`s to the controller, and owns repaint policy (immediate, timed). No direct audio/channel access.
  - **Controller**: Handles UI events, updates `AppState`, and consumes audio progress snapshots via its own API. Exposes helpers like `handle_ui_events` and `poll_playback_progress`; does not request repaints itself.
  - **AudioRuntime**: Owns threads/channels, loads audio, plays/pauses/stops, and emits progress snapshots over channels. Accepts an optional initial track on construction. No mutexes/shared state for UI; channels only.
- **Progress updates**: Audio worker sends `PlaybackSnapshot` (pos/duration/is_playing) over a flume channel. Controller polls and updates `AppState`; App schedules repaints (currently 144 FPS while playing).
- **Repaint policy**: App decides when to repaint (e.g., after controller updates or via `request_repaint_after`). Controllers must not call repaint.
- **Event dispatch**: App does not know controller internals. It hands drained `UiEvent`s to `controller.handle_ui_events`, which maps them to controller actions.
- **No mutexes in UI path**: Avoid shared-state locks between audio and UI. Use channels for cross-thread communication.
- **Startup**: Pass restored `last_file` into `AudioRuntime::new` so audio is ready after restore; UI state comes from `AppState` only.

Keep files concise and aligned with this separation. When adding features (seek, progress UI), route data via controller and channels, and let App own repaint timing.
