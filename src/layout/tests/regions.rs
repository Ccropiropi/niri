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
