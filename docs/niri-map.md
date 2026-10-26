# niri code map — splitdesk change points

Version: **26.4.0** (Cargo.toml:9).
Build: `cargo build --release` (docs/wiki/Getting-Started.md:197).
Run nested: `./target/debug/niri` (verify nested support in wiki).

---

## 1. Workspace-switch animation

**Goal**: find where the vertical workspace-switch offset is calculated so we can add an x component.

### Key types

- `WorkspaceSwitch` enum — `src/layout/monitor.rs:95`
  ```rust
  pub enum WorkspaceSwitch {
      Animation(Animation),
      Gesture(WorkspaceSwitchGesture),
  }
  ```
- `WorkspaceSwitchGesture` — `monitor.rs:101`: tracks `center_idx`, `start_idx`, `current_idx` (fractional).

### Data flow (5 steps)

1. A workspace switch creates `WorkspaceSwitch::Animation` or a gesture populates `WorkspaceSwitchGesture` (`monitor.rs:483`, `monitor.rs:651`).
2. `WorkspaceSwitch::current_idx()` (`monitor.rs:201`) returns a fractional index from the animation value or gesture position.
3. `Monitor::workspace_render_idx()` (`monitor.rs:1400`) calls `switch.current_idx()` at line 1468–1469. Has a special path for synchronized zoom+switch (1403–1465).
4. `Monitor::workspaces_render_geo()` (`monitor.rs:1475`) converts render_idx to a **y-only** offset:
   ```rust
   let first_ws_y = -self.workspace_render_idx() * ws_height_with_gap;  // line 1488
   // ...
   let loc = Point::from((0., y)) + static_offset;                      // line 1494
   ```
   **This is the change point for adding x-axis movement.**
5. `Monitor::render_workspaces()` (`monitor.rs:1679`) iterates the geo rectangles and relocates render elements.

**Confidence: [High]** — read all functions end-to-end.

---

## 2. Window layout (scrolling columns)

**Goal**: understand how the horizontal scrolling column layout is computed.

### Key types

- `ScrollingSpace` — `src/layout/scrolling.rs:~37`: holds `columns: Vec<Column<W>>`, `data: Vec<ColumnData>`, `view_offset: ViewOffset`, `active_column_idx`.
- `Column` — `scrolling.rs:166`: holds tiles, `width: ColumnWidth`, display mode.
- `ColumnData` — `scrolling.rs:108`: lightweight copy of column width for layout math.

### Data flow (4 steps)

1. `column_xs()` (`scrolling.rs:2333`) computes each column's x by accumulating `width + gaps`:
   ```rust
   x += data.width + gaps;  // line 2343
   ```
2. `view_pos()` (`scrolling.rs:2323`) = `column_x(active_column_idx) + view_offset.current()` — the camera's horizontal position.
3. `columns_with_render_positions()` (`scrolling.rs:2406`) subtracts `view_pos()` to produce screen-relative positions:
   ```rust
   let view_off = Point::from((-self.view_pos(), 0.));  // line 2409
   ```
4. `ScrollingSpace::render()` (`scrolling.rs:2961`) iterates columns, renders each tile at `col_pos + tile_off + tile.render_offset()` (line 3002).

**Confidence: [High]** — read all functions.

---

## 3. Rendering, layer-shell surfaces, and blur

**Goal**: find where windows and layer-shell surfaces are drawn, and where blur/background effects are applied.

### Render pipeline (`src/niri.rs`)

1. `Niri::render()` (niri.rs:4443) — entry point. Calls `fill_xray_elements()` then `render_inner()`.
2. `Niri::render_inner()` (niri.rs:4473) — Z-ordered draw (top to bottom):
   pointer → screen transition → exit dialog → config error → lock screen → screenshot UI → hotkey overlay → Alt-Tab → overlay layer → top layer → workspaces + bottom/background layers → shadows → backdrop.
3. Layer-shell: `render_layer_normal()` (niri.rs:4824) and `render_layer_popups()` (niri.rs:4842) iterate `layer_map_for_output()`.
4. Workspaces: `mon.render_workspaces()` called at niri.rs:4648/4689, delegates to `ws.render_scrolling()` / `ws.render_floating()` (workspace.rs:1662/1677).

### Blur and background effects

- `BlurOptions` loaded from config at niri.rs:4395.
- `Blur::render()` (`src/render_helpers/blur.rs:166`) — multi-pass Kawase blur via `blur_down.frag` / `blur_up.frag` shaders.
- `BackgroundEffectState` (niri.rs:317) — Smithay's `ext_background_effect` protocol. Handler at `src/handlers/background_effect.rs:109` reports `Capability::Blur`.
- Xray system: `fill_xray_elements()` (niri.rs:4718) pre-renders background and backdrop layers into effect buffers. These are consumed by the blur compositor during `render_inner()`.

**Confidence: [High]** for render pipeline and blur location. **[Medium]** for exactly how effect buffers feed into per-window blur compositing (the xray/offscreen path is complex; `render_helpers/offscreen.rs` and `effect_buffer.rs` were not fully traced).

---

## Not found / needs verification

- Nested Wayland session support: Claude.md says `./target/debug/niri`; wiki page not checked for caveats.
- Per-window blur application path through offscreen rendering (`render_helpers/offscreen.rs`, `effect_buffer.rs`) — not fully traced. [Low confidence on internals.]

---

## Summary

niri 26.4.0 uses a vertical-only workspace switch offset computed in `workspaces_render_geo()` (monitor.rs:1488–1494) — adding x there gives diagonal transitions. The scrolling column layout lives in `scrolling.rs`, positioning columns via `column_xs()` and a `view_offset` camera. Rendering is orchestrated by `Niri::render_inner()` (niri.rs:4473) with layer-shell surfaces drawn via Smithay's `layer_map_for_output()` and blur handled by a multi-pass Kawase shader with pre-rendered xray effect buffers. All three change points are localized and well-separated.

## Three most important files to read first

1. `src/layout/monitor.rs` — workspace switch animation + render geo
2. `src/layout/scrolling.rs` — column layout + view offset
3. `src/niri.rs` — render pipeline orchestration + blur setup
