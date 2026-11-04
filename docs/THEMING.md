# Theme adapter

Rust/Tauri retained per architecture decision. No Quickshell dependency, hooks, shell imports or system writes. Preferences and application menus retain their contracts. `read_theme` returns a cached complete snapshot, adding `generation`; `theme-changed` carries the same shape. The frontend subscribes before reading, accepts strictly increasing generations and cleans up its subscription.

## Versioned inputs

Verified official [rc3 resolver](https://github.com/omacom/omarchy/blob/v4.5.0-rc3/bin/omarchy-theme-color), [theming](https://github.com/omacom/omarchy/blob/v4.5.0-rc3/docs/theming.md), and [activation](https://github.com/omacom/omarchy/blob/v4.5.0-rc3/bin/omarchy-theme-set). Rule: explicit `mode`, legacy `theme_type`, adjacent `light.mode`, sum of RGB background bytes >382 => light, otherwise dark. Explicit invalid modes are rejected. This is upstream's byte threshold, not WCAG luminance. Fixtures copy Tokyo Night colors from exact v4.0.6 and v4.5.0-rc3 tags; neither constitutes native runtime qualification.

Canonical accent/selection/muted win; accent falls back to blue/color4, background/foreground to bg/fg then color0/color7, selection to selection_background/color8/color0/background, muted to color8/dark_foreground/dark_fg/foreground, red to color1. Application surfaces and readable foreground are derived locally; this is not a full shell.toml interpreter. Selection ink is calculated separately. Overlay and its ink are semantic tokens so hero and menu components contain no literal colors.

## Paths and failure handling

Priority: XDG_STATE_HOME/omarchy/current/theme/colors.toml (default HOME/.local/state), then the literal HOME/.local/state path used by upstream when XDG differs, then XDG_CONFIG_HOME/omarchy/current/theme/colors.toml (default HOME/.config), a legacy application fallback. First existing valid source wins; an invalid active source retains the last valid palette rather than jumping to stale lower-priority colors. At startup without a valid source, a complete application fallback is returned. File reads are bounded to 64 KiB. Missing background or foreground means incomplete; optional roles receive defaults and warnings. Two immediate reads and marker checks detect concurrent replacement without sleeping on IPC/UI threads. A valid partial write with both required colors is not distinguishable from a deliberately minimal theme.

## Observation

`notify` 8.2.1 (locked) is the maintained cross-platform filesystem watcher abstraction, using inotify on Linux; std has no filesystem notification API. One background worker owns reads and publication ordering. Nonrecursive watches cover theme/current/omarchy parents, or their nearest existing ancestors, and are recalculated after every notification. Creation of absent parents and atomic theme-directory replacement therefore rearm observation. Capacity-one channel coalesces notifications, quiet debounce 80 ms bounded to 500 ms; every signal rereads current filesystem state rather than applying event payloads. Queue loss/overflow signals likewise resync. A 30-second resync bounds missed notifications, suspend recovery and watcher failure. Watch installation failure appears in snapshot warnings; backend errors are logged to stderr. Failed watches are retried on the next cycle. No 500-ms browser polling or 150-ms read sleep remains.

Test subprocess: `python tests/theme-watch.py` after `npm run release`; isolated HOME/XDG directories, no native window and no Steam actions. `--theme-watch-probe` publishes real worker JSON snapshots to stdout and requests cooperative shutdown on a stdin line or EOF. The harness asserts clean exit within one second (worker cleanup budget: 500 ms). An optional binary path is accepted: `python tests/theme-watch.py /absolute/path/to/steam-omarchy`. It is a diagnostic path, not a second watcher implementation.

## Worker lifecycle

The managed `Worker` owns its JoinHandle. A release/acquire stop flag and the capacity-one wake channel interrupt both the 30-second receive and debounce (a full queue already contains a wake). Returning from the worker drops the watcher before signalling completion. Tauri ExitRequested requests stop without waiting; Exit waits at most 500 ms for completion, joining only after completion. Drop requests stop without blocking. A stalled filesystem/backend or callback can exceed the budget: shutdown reports failure and process exit reclaims remaining resources; this is cooperative cancellation, not forced thread termination. Cancelling a Tauri exit request does not restart the theme worker.

No frontend subscription, generation ordering, ACL or last-valid transaction contract changed. Native frontend event delivery and normal-window shutdown remain NOT TESTED; the no-window probe exercises the production worker lifecycle only.
