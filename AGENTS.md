# Architecture and interaction guidelines

## Core Architecture (MVC Pattern)

- **Persistence**: Only `AppState` is serialized/restored (`save`/`new`). Do not serialize runtime components (`AudioRuntime`, GPU resources, etc.).
- **Layers**:
  - **UI (App)**: Thin shell. Builds widgets, forwards `UiEvent`s to the controller, and owns repaint policy (immediate, timed). No direct audio/channel access.
  - **Controller**: Handles UI events, updates `AppState`, and consumes audio progress snapshots via its own API. Exposes helpers like `handle_ui_events` and `poll_playback_progress`; does not request repaints itself.
  - **AudioRuntime**: Owns threads/channels, loads audio, plays/pauses/stops, and emits progress snapshots over channels. Accepts an optional initial track on startup. No mutexes/shared state for UI; channels only.
- **Progress updates**: Audio worker sends `PlaybackSnapshot` (pos/duration/is_playing) over a flume channel. Controller polls and updates `AppState`; App schedules repaints (currently 144 FPS while playing).
- **Repaint policy**: App decides when to repaint (e.g., after controller updates or via `request_repaint_after`). Controllers must not call repaint.
- **Event dispatch**: App does not know controller internals. It hands drained `UiEvent`s to `controller.handle_ui_events`, which maps them to controller actions.
- **No mutexes in UI path**: Avoid shared-state locks between audio and UI. Use channels for cross-thread communication.
- **Startup**: Pass restored `last_file` into `AudioRuntime::new` so audio is ready after restore; UI state comes from `AppState` only.

## GPU Rendering System (wgpu Integration)

- **Backend**: Uses wgpu (WebGPU) for cross-platform GPU acceleration instead of glow/OpenGL.
- **Fractal Rendering**: GPU computes fractal mathematics in real-time from FFT audio data.
- **Callback Architecture**: Modular `fractal_callbacks/` system for different fractal types:
  ```
  src/ui/fractal_callbacks/
  ├── mod.rs              # Module exports
  ├── common.rs           # Shared vertex utilities
  └── {fractal_type}/     # Per-fractal implementation
      ├── mod.rs          # CallbackTrait implementation
      └── shader.wgsl     # WGSL shader code
  ```
- **Resource Management**: GPU resources cached in `CallbackResources` to avoid recreation.
- **Scissor Rects**: Proper pixel-based clipping ensures rendering stays within widget bounds.
- **State Integration**: `FractalType` enum in `AppState` for fractal selection persistence.

## Audio-Reactive Fractals

- **FFT Integration**: Audio frequency data flows from `AudioRuntime` → `AppState` → GPU shaders.
- **Real-time Updates**: Fractal parameters (zoom, color, distortion) modulate with audio features.
- **No Computational Threads**: GPU handles fractal computation directly - no separate CPU threads.
- **Shader Parameters**: FFT bins passed as uniform buffers to fractal shaders.

## File Organization

Keep files concise and aligned with this separation. When adding features:
- Route data via controller and channels
- Let App own repaint timing
- Use modular callback architecture for GPU features
- Maintain clear separation between UI, audio, and rendering systems

## Development Guidelines

- **wgpu Backend**: Requires explicit resource management and pipeline creation
- **CallbackTrait Pattern**: Standard interface for GPU rendering callbacks
- **Shader Organization**: One `.wgsl` file per fractal type with matching Rust callback
- **State Management**: UI state in `AppState`, runtime state in respective runtimes
- **Performance**: GPU computation for heavy math, channels for cross-thread communication
