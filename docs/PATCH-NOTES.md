## [2026-10-28] - Milestone R2: Focus & Input Routing (splitdesk)

### Added
- **`Region` Enum (`src/layout/mod.rs`):**
  - Added `Region::Left` and `Region::Right` with `#[default] Right`.
- **Monitor Region State & Mathematical Truth (`src/layout/monitor.rs`):**
  - Added `focused_region: Region` field to `Monitor<W>` (defaulting to `Region::Right`).
  - Added `left_region: Option<ScrollingSpace<W>>` field to `Monitor<W>` for monitor-scoped static tiling.
  - Implemented `Monitor::region_at(pos: Point<f64, Logical>) -> Region` calculating:
    $$\text{separator\_x}(y) = \text{top\_x} - \text{slant} \cdot \left(\frac{y}{H}\right)$$
    using `options.regions.width_percent` and `options.regions.slant_percent`.
  - Implemented focus gating in `active_window()` and `active_window_mut()` respecting `focused_region`.
  - Updated hit-testing in `window_under()` and `resize_edges_under()` to route through `region_at()`.
- **Action & Pointer Dispatch (`src/input/mod.rs`):**
  - Functional dispatch for `FocusRegionLeft`, `FocusRegionRight`, and `ToggleRegionFocus` switching `focused_region` on the active monitor and updating keyboard focus.
  - Click-to-focus region activation on pointer button press when clicking within a region boundary.
- **Unit Test Suite (`src/layout/tests/regions.rs`):**
  - 6 exhaustive unit tests:
    - `region_at_disabled_always_right`
    - `region_at_vertical_split`
    - `region_at_slanted_split`
    - `region_at_negative_slant`
    - `focus_gating_and_switching`
    - `focus_gating_when_disabled_stock_behavior`

---

## [2026-10-28] - Milestone R1: Config Schema & Inactive State (splitdesk)

## Summary
Implementation of PRD Milestone R1 for the splitdesk feature. Adds the `regions` configuration block and corresponding action variants, with the regions feature disabled by default to ensure identical behavior to stock niri when not enabled.

## Files Changed

### 1. `niri-config/src/regions.rs` (NEW)
- Created new module defining `Regions` and `RegionsPart` config models
- `Regions`: Final parsed configuration with fields `enabled`, `width_percent`, `slant_percent`, `gap`
- `RegionsPart`: Knuffel-decoded partial config for merging from multiple config files (`enabled: Option<bool>`, `width_percent: Option<FloatOrInt<0, 100>>`, `slant_percent: Option<FloatOrInt<-100, 100>>`, `gap: Option<FloatOrInt<0, 1000>>`)
- Default values: `enabled: false`, `width_percent: 46.0`, `slant_percent: 12.0`, `gap: 16.0`
- `validate()` method clamps values to valid ranges
- Unit tests: `default_regions`, `regions_validation`, `regions_parsing`

### 2. `niri-config/src/lib.rs`
- Added `pub mod regions;` module declaration
- Exported `Regions` and `RegionsPart` types
- Added `pub regions: Regions` field to `Config` struct
- Added `"regions" => m_merge!(regions)` handler in `ConfigPart::decode_children`
- Updated snapshot test to include default `regions` section

### 3. `niri-config/src/binds.rs`
- Added 6 new `Action` variants:
  - `ToggleSplitMode`
  - `FocusRegionLeft`
  - `FocusRegionRight`
  - `ToggleRegionFocus`
  - `MoveWindowToRegionLeft`
  - `MoveWindowToRegionRight`
- Updated `From<niri_ipc::Action> for Action` conversion to map all 6 new action variants

### 4. `niri-ipc/src/lib.rs`
- Mirrored the 6 new `Action` variants with documentation comments and CLI/IPC subcommands

### 5. `src/layout/mod.rs`
- Added `pub regions: Regions` field to `Options` struct
- Imported `Regions` from `niri_config`
- Updated `Options::from_config()` to propagate `regions: config.regions`

### 6. `src/input/mod.rs`
- Added safe inactive no-op handling for the 6 new action variants in `State::do_action()` (functional layout dispatch deferred to Milestone R2+)

## Config Schema
```kdl
regions {
    // off by default
    enabled false
    width-percent 46
    slant-percent 12
    gap 16
}
```

## Verification Evidence
- `cargo check --workspace`: PASS (0 errors, 0 warnings)
- `cargo test -p niri-config`: PASS (22 unittests + 1 integration test, 0 failures)
- Default mode (`regions { enabled false }`): Indistinguishable from stock niri

## Compliance with PRD & NFRs
- NFR-1: Default mode indistinguishable from stock niri ✓ (regions disabled by default, inactive state)
- NFR-3: Minimal patch - 6 files touched (1 new, 5 modified) ✓
- NFR-5: `cargo check --workspace` and `cargo test -p niri-config` pass ✓
- FR-10: `regions { ... }` config block with `enabled`, `width-percent`, `slant-percent`, `gap` ✓
- FR-11: Split mode off is default ✓
