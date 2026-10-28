use std::time::Duration;

use smithay::output::{Mode, Output, PhysicalProperties, Subpixel};
use smithay::utils::{Point, Size};

use super::*;
use crate::animation::Clock;
use crate::layout::Region;

fn make_test_output(w: i32, h: i32) -> Output {
    let output = Output::new(
        "test-output".to_string(),
        PhysicalProperties {
            size: Size::from((w, h)),
            subpixel: Subpixel::Unknown,
            make: String::new(),
            model: String::new(),
            serial_number: String::new(),
        },
    );
    output.change_current_state(
        Some(Mode {
            size: Size::from((w, h)),
            refresh: 60000,
        }),
        None,
        None,
        None,
    );
    output.user_data().insert_if_missing(|| OutputName {
        connector: "test-output".to_string(),
        make: None,
        model: None,
        serial: None,
    });
    output
}

#[test]
fn region_at_disabled_always_right() {
    let mut options = Options::default();
    options.regions.enabled = false;
    options.regions.width_percent = 50.0;
    options.regions.slant_percent = 10.0;

    let mut layout = Layout::<TestWindow>::with_options(Clock::with_time(Duration::ZERO), options);
    let output = make_test_output(1920, 1080);
    layout.add_output(output, None);

    let mon = layout.active_monitor().unwrap();

    assert_eq!(mon.region_at(Point::from((0.0, 0.0))), Region::Right);
    assert_eq!(mon.region_at(Point::from((100.0, 500.0))), Region::Right);
    assert_eq!(mon.region_at(Point::from((950.0, 540.0))), Region::Right);
    assert_eq!(mon.region_at(Point::from((1900.0, 1000.0))), Region::Right);
}

#[test]
fn region_at_vertical_split() {
    let mut options = Options::default();
    options.regions.enabled = true;
    options.regions.width_percent = 50.0;
    options.regions.slant_percent = 0.0;

    let mut layout = Layout::<TestWindow>::with_options(Clock::with_time(Duration::ZERO), options);
    let output = make_test_output(1920, 1080);
    layout.add_output(output, None);

    let mon = layout.active_monitor().unwrap();
    let sep_x = 960.0;

    // Top (y = 0)
    assert_eq!(mon.region_at(Point::from((sep_x - 1.0, 0.0))), Region::Left);
    assert_eq!(mon.region_at(Point::from((sep_x + 1.0, 0.0))), Region::Right);

    // Mid (y = 540)
    assert_eq!(mon.region_at(Point::from((sep_x - 1.0, 540.0))), Region::Left);
    assert_eq!(mon.region_at(Point::from((sep_x + 1.0, 540.0))), Region::Right);

    // Bottom (y = 1080)
    assert_eq!(mon.region_at(Point::from((sep_x - 1.0, 1080.0))), Region::Left);
    assert_eq!(mon.region_at(Point::from((sep_x + 1.0, 1080.0))), Region::Right);
}

#[test]
fn region_at_slanted_split() {
    let mut options = Options::default();
    options.regions.enabled = true;
    options.regions.width_percent = 46.0;
    options.regions.slant_percent = 12.0;

    let mut layout = Layout::<TestWindow>::with_options(Clock::with_time(Duration::ZERO), options);
    let output = make_test_output(1920, 1080);
    layout.add_output(output, None);

    let mon = layout.active_monitor().unwrap();

    // W = 1920, H = 1080
    // top_x = 1920 * 0.46 = 883.2
    // slant = 1920 * 0.12 = 230.4
    // separator_x(y) = 883.2 - 230.4 * (y / 1080)

    // Top: y = 0 => sep_x = 883.2
    let sep_top = 883.2;
    assert_eq!(mon.region_at(Point::from((sep_top - 1.0, 0.0))), Region::Left);
    assert_eq!(mon.region_at(Point::from((sep_top + 1.0, 0.0))), Region::Right);

    // Mid: y = 540 => sep_x = 883.2 - 115.2 = 768.0
    let sep_mid = 768.0;
    assert_eq!(mon.region_at(Point::from((sep_mid - 1.0, 540.0))), Region::Left);
    assert_eq!(mon.region_at(Point::from((sep_mid + 1.0, 540.0))), Region::Right);

    // Bottom: y = 1080 => sep_x = 883.2 - 230.4 = 652.8
    let sep_bot = 652.8;
    assert_eq!(mon.region_at(Point::from((sep_bot - 1.0, 1080.0))), Region::Left);
    assert_eq!(mon.region_at(Point::from((sep_bot + 1.0, 1080.0))), Region::Right);
}

#[test]
fn region_at_negative_slant() {
    let mut options = Options::default();
    options.regions.enabled = true;
    options.regions.width_percent = 40.0;
    options.regions.slant_percent = -10.0;

    let mut layout = Layout::<TestWindow>::with_options(Clock::with_time(Duration::ZERO), options);
    let output = make_test_output(1000, 1000);
    layout.add_output(output, None);

    let mon = layout.active_monitor().unwrap();

    // W = 1000, H = 1000
    // top_x = 400.0, slant = -100.0
    // sep_x(y) = 400.0 - (-100.0) * (y / 1000) = 400.0 + 100.0 * (y / 1000)

    // Top: y = 0 => sep_x = 400.0
    assert_eq!(mon.region_at(Point::from((399.0, 0.0))), Region::Left);
    assert_eq!(mon.region_at(Point::from((401.0, 0.0))), Region::Right);

    // Bottom: y = 1000 => sep_x = 500.0
    assert_eq!(mon.region_at(Point::from((499.0, 1000.0))), Region::Left);
    assert_eq!(mon.region_at(Point::from((501.0, 1000.0))), Region::Right);
}

#[test]
fn focus_gating_and_switching() {
    let mut options = Options::default();
    options.regions.enabled = true;
    options.regions.width_percent = 50.0;

    let mut layout = Layout::<TestWindow>::with_options(Clock::with_time(Duration::ZERO), options);
    let output = make_test_output(1920, 1080);
    layout.add_output(output, None);

    // Add a window into the layout
    let win = TestWindow::new(TestWindowParams::new(1));
    layout.add_window(win, AddWindowTarget::Auto, None, None, false, false, ActivateWindow::Yes);

    let mon = layout.active_monitor().unwrap();
    // Default region should be Right
    assert_eq!(mon.focused_region, Region::Right);

    // With focused_region = Right, active_window returns the right workspace's active window
    assert!(mon.active_window().is_some());
    assert_eq!(mon.active_window().unwrap().id(), &1);

    // Switch focused region to Left
    mon.focused_region = Region::Left;
    // With focused_region = Left and left_region empty, active_window returns None (gated focus!)
    assert!(mon.active_window().is_none());

    // Switch back to Right
    mon.focused_region = Region::Right;
    assert!(mon.active_window().is_some());
    assert_eq!(mon.active_window().unwrap().id(), &1);
}

#[test]
fn focus_gating_when_disabled_stock_behavior() {
    let mut options = Options::default();
    options.regions.enabled = false;

    let mut layout = Layout::<TestWindow>::with_options(Clock::with_time(Duration::ZERO), options);
    let output = make_test_output(1920, 1080);
    layout.add_output(output, None);

    let win = TestWindow::new(TestWindowParams::new(1));
    layout.add_window(win, AddWindowTarget::Auto, None, None, false, false, ActivateWindow::Yes);

    let mon = layout.active_monitor().unwrap();
    // Force focused_region = Left, but regions.enabled = false
    mon.focused_region = Region::Left;

    // Must return the workspace's active window anyway (100% stock behavior)
    assert!(mon.active_window().is_some());
    assert_eq!(mon.active_window().unwrap().id(), &1);
}

#[test]
fn left_region_initialization_and_layout_bounding() {
    let mut options = Options::default();
    options.regions.enabled = true;
    options.regions.width_percent = 50.0;
    options.regions.gap = 16.0;

    let mut layout = Layout::<TestWindow>::with_options(Clock::with_time(Duration::ZERO), options);
    let output = make_test_output(1920, 1080);
    layout.add_output(output, None);

    let mon = layout.active_monitor().unwrap();
    assert!(mon.left_region.is_some());

    let left = mon.left_region.as_ref().unwrap();
    assert_eq!(left.view_size().w, 960.0);
    assert_eq!(left.view_size().h, 1080.0);
    assert_eq!(left.working_area().loc.x, 0.0);
    assert_eq!(left.working_area().size.w, 960.0);

    let right_ws = mon.active_workspace_ref();
    assert_eq!(right_ws.scrolling().working_area().loc.x, 976.0);
    assert_eq!(right_ws.scrolling().working_area().size.w, 944.0);

    // Assert zero overlap between left region and right workspace working areas
    let left_end = left.working_area().loc.x + left.working_area().size.w;
    let right_start = right_ws.scrolling().working_area().loc.x;
    assert!(left_end <= right_start);
}

#[test]
fn window_transfer_between_regions() {
    let mut options = Options::default();
    options.regions.enabled = true;
    options.regions.width_percent = 50.0;

    let mut layout = Layout::<TestWindow>::with_options(Clock::with_time(Duration::ZERO), options);
    let output = make_test_output(1920, 1080);
    layout.add_output(output, None);

    // Open a window in right workspace
    let win = TestWindow::new(TestWindowParams::new(1));
    layout.add_window(win, AddWindowTarget::Auto, None, None, false, false, ActivateWindow::Yes);

    let mon = layout.active_monitor().unwrap();
    assert!(mon.active_workspace_ref().has_window(&1));
    assert!(!mon.left_region.as_ref().unwrap().tiles().any(|t| t.window().id() == &1));

    // Move window from right workspace to left region
    layout.move_window_to_region_left();

    let mon = layout.active_monitor().unwrap();
    assert!(!mon.active_workspace_ref().has_window(&1));
    assert!(mon.left_region.as_ref().unwrap().tiles().any(|t| t.window().id() == &1));
    assert_eq!(mon.focused_region, Region::Left);
    assert_eq!(mon.active_window().map(|w| *w.id()), Some(1));

    // Move window back to right workspace
    layout.move_window_to_region_right();

    let mon = layout.active_monitor().unwrap();
    assert!(mon.active_workspace_ref().has_window(&1));
    assert!(!mon.left_region.as_ref().unwrap().tiles().any(|t| t.window().id() == &1));
    assert_eq!(mon.focused_region, Region::Right);
    assert_eq!(mon.active_window().map(|w| *w.id()), Some(1));
}

#[test]
fn workspace_switch_invariance() {
    let mut options = Options::default();
    options.regions.enabled = true;
    options.regions.width_percent = 50.0;

    let mut layout = Layout::<TestWindow>::with_options(Clock::with_time(Duration::ZERO), options);
    let output = make_test_output(1920, 1080);
    layout.add_output(output, None);

    // Add window 1 and move to left region
    let win1 = TestWindow::new(TestWindowParams::new(1));
    layout.add_window(win1, AddWindowTarget::Auto, None, None, false, false, ActivateWindow::Yes);
    layout.move_window_to_region_left();

    // Add window 2 to right workspace
    let win2 = TestWindow::new(TestWindowParams::new(2));
    layout.add_window(win2, AddWindowTarget::Auto, None, None, false, false, ActivateWindow::Yes);

    // Add workspace 1 and switch to it
    let mon = layout.active_monitor_mut().unwrap();
    mon.add_workspace_bottom();
    layout.switch_workspace(1);

    // Add window 3 on workspace 1
    let win3 = TestWindow::new(TestWindowParams::new(3));
    layout.add_window(win3, AddWindowTarget::Auto, None, None, false, false, ActivateWindow::Yes);

    let mon = layout.active_monitor().unwrap();
    assert_eq!(mon.active_workspace_idx, 1);
    // Window 1 is still in left region
    assert!(mon.left_region.as_ref().unwrap().tiles().any(|t| t.window().id() == &1));
    // Window 3 is on active workspace 1
    assert!(mon.active_workspace_ref().has_window(&3));
    // Window 2 is on workspace 0
    assert!(mon.workspaces[0].has_window(&2));

    // Switch back to workspace 0
    layout.switch_workspace(0);

    let mon = layout.active_monitor().unwrap();
    assert_eq!(mon.active_workspace_idx, 0);
    // Left region window 1 remains completely untouched!
    assert!(mon.left_region.as_ref().unwrap().tiles().any(|t| t.window().id() == &1));
    assert!(mon.active_workspace_ref().has_window(&2));

    // Verify invariants pass
    mon.verify_invariants();
}

#[test]
fn toggle_split_mode_roundtrip_evacuation() {
    let mut options = Options::default();
    options.regions.enabled = true;
    options.regions.width_percent = 50.0;
    options.regions.gap = 16.0;

    let mut layout = Layout::<TestWindow>::with_options(Clock::with_time(Duration::ZERO), options);
    let output = make_test_output(1920, 1080);
    layout.add_output(output, None);

    // Add window 1 to right workspace and move to left region
    let win1 = TestWindow::new(TestWindowParams::new(1));
    layout.add_window(win1, AddWindowTarget::Auto, None, None, false, false, ActivateWindow::Yes);
    layout.move_window_to_region_left();

    // Add window 2 to right workspace
    let win2 = TestWindow::new(TestWindowParams::new(2));
    layout.add_window(win2, AddWindowTarget::Auto, None, None, false, false, ActivateWindow::Yes);

    // Add window 3 to right workspace and move to left region
    let win3 = TestWindow::new(TestWindowParams::new(3));
    layout.add_window(win3, AddWindowTarget::Auto, None, None, false, false, ActivateWindow::Yes);
    layout.move_window_to_region_left();

    // Verify initial layout state
    {
        let mon = layout.active_monitor().unwrap();
        assert!(mon.is_split_mode_active());
        assert!(mon.left_region.is_some());
        assert_eq!(mon.windows().count(), 3);
        let left = mon.left_region.as_ref().unwrap();
        assert!(left.tiles().any(|t| t.window().id() == &1));
        assert!(left.tiles().any(|t| t.window().id() == &3));
        assert!(mon.active_workspace_ref().has_window(&2));
        assert_eq!(mon.focused_region, Region::Left);
    }

    // Toggle split mode OFF
    let active = layout.toggle_split_mode();
    assert!(!active);

    {
        let mon = layout.active_monitor().unwrap();
        assert!(!mon.is_split_mode_active());
        assert!(mon.left_region.is_none());
        assert_eq!(mon.focused_region, Region::Right);

        // 100% zero window loss: all 3 windows safely evacuated to active workspace
        assert_eq!(mon.windows().count(), 3);
        let active_ws = mon.active_workspace_ref();
        assert!(active_ws.has_window(&1));
        assert!(active_ws.has_window(&2));
        assert!(active_ws.has_window(&3));

        // Restored normal working area dimensions on workspaces
        assert_eq!(active_ws.scrolling().working_area().loc.x, 0.0);
        assert_eq!(active_ws.scrolling().working_area().size.w, 1920.0);

        mon.verify_invariants();
    }

    layout.verify_invariants();
}

#[test]
fn toggle_split_mode_back_on() {
    let mut options = Options::default();
    options.regions.enabled = true;
    options.regions.width_percent = 50.0;
    options.regions.gap = 16.0;

    let mut layout = Layout::<TestWindow>::with_options(Clock::with_time(Duration::ZERO), options);
    let output = make_test_output(1920, 1080);
    layout.add_output(output, None);

    // Add window 1 and move to left region
    let win1 = TestWindow::new(TestWindowParams::new(1));
    layout.add_window(win1, AddWindowTarget::Auto, None, None, false, false, ActivateWindow::Yes);
    layout.move_window_to_region_left();

    // Toggle split mode OFF
    let active = layout.toggle_split_mode();
    assert!(!active);
    {
        let mon = layout.active_monitor().unwrap();
        assert!(!mon.is_split_mode_active());
        assert!(mon.left_region.is_none());
        assert!(mon.active_workspace_ref().has_window(&1));
    }

    // Toggle split mode BACK ON
    let active = layout.toggle_split_mode();
    assert!(active);

    {
        let mon = layout.active_monitor().unwrap();
        assert!(mon.is_split_mode_active());
        assert!(mon.left_region.is_some());

        // Left region recreated with bounded layout
        let left = mon.left_region.as_ref().unwrap();
        assert_eq!(left.view_size().w, 960.0);
        assert_eq!(left.view_size().h, 1080.0);
        assert_eq!(left.working_area().loc.x, 0.0);
        assert_eq!(left.working_area().size.w, 960.0);

        // Right workspace bounded to the right
        let right_ws = mon.active_workspace_ref();
        assert_eq!(right_ws.scrolling().working_area().loc.x, 976.0);
        assert_eq!(right_ws.scrolling().working_area().size.w, 944.0);

        // Window 1 retained on active workspace
        assert!(right_ws.has_window(&1));
        assert_eq!(mon.windows().count(), 1);

        mon.verify_invariants();
    }

    layout.verify_invariants();

    // Verify Monitor::toggle_split_mode directly
    let mon = layout.active_monitor_mut().unwrap();
    let state = mon.toggle_split_mode();
    assert!(!state);
    assert!(!mon.is_split_mode_active());
    assert!(mon.left_region.is_none());
    assert_eq!(mon.active_workspace_ref().scrolling().working_area().size.w, 1920.0);

    let state = mon.toggle_split_mode();
    assert!(state);
    assert!(mon.is_split_mode_active());
    assert!(mon.left_region.is_some());
    assert_eq!(mon.left_region.as_ref().unwrap().working_area().size.w, 960.0);
    assert_eq!(mon.active_workspace_ref().scrolling().working_area().size.w, 944.0);
    mon.verify_invariants();
}

#[test]
fn ipc_telemetry_regions_query_and_events() {
    let mut options = Options::default();
    options.regions.enabled = false;
    options.regions.width_percent = 50.0;

    let mut layout = Layout::<TestWindow>::with_options(Clock::with_time(Duration::ZERO), options);
    let output = make_test_output(1920, 1080);
    layout.add_output(output, None);

    // Initial state: disabled
    assert!(!layout.is_split_mode_active());
    assert_eq!(layout.focused_region(), None);

    let win1 = TestWindow::new(TestWindowParams::new(1));
    layout.add_window(win1, AddWindowTarget::Auto, None, None, false, false, ActivateWindow::Yes);
    assert_eq!(layout.region_of_window(&1), None);

    // Toggle split mode on
    assert!(layout.toggle_split_mode());
    assert!(layout.is_split_mode_active());
    assert_eq!(layout.focused_region(), Some(Region::Right));
    assert_eq!(layout.region_of_window(&1), Some(Region::Right));

    // Move window 1 to left region
    layout.move_window_to_region_left();
    assert_eq!(layout.focused_region(), Some(Region::Left));
    assert_eq!(layout.region_of_window(&1), Some(Region::Left));

    // Add window 2 to right workspace
    let win2 = TestWindow::new(TestWindowParams::new(2));
    layout.add_window(win2, AddWindowTarget::Auto, None, None, false, false, ActivateWindow::Yes);
    assert_eq!(layout.region_of_window(&2), Some(Region::Right));

    // Verify with_windows yields both windows (left and right)
    let mut visited = Vec::new();
    layout.with_windows(|win, _out, ws_id, _layout| {
        visited.push((*win.id(), ws_id.is_some()));
    });
    // Window 1 is in left_region (ws_id is None)
    assert!(visited.iter().any(|(id, has_ws)| *id == 1 && !*has_ws));
    // Window 2 is in right workspace (ws_id is Some)
    assert!(visited.iter().any(|(id, has_ws)| *id == 2 && *has_ws));

    // Toggle off
    assert!(!layout.toggle_split_mode());
    assert!(!layout.is_split_mode_active());
    assert_eq!(layout.focused_region(), None);
    assert_eq!(layout.region_of_window(&1), None);
    assert_eq!(layout.region_of_window(&2), None);
}
