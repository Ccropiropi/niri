# Patch Notes - Milestone R1: Config Schema & Inactive State (splitdesk)

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
