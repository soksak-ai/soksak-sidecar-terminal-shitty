use soksak_kit_sidecar_terminal::mirror::{
    EngineWheelInput, EngineWheelRoute, SelectionModifiers, TerminalEngine,
};
use soksak_sidecar_terminal_shitty::engine::Engine;

fn wheel(horizontal: i32, vertical: i32, route: EngineWheelRoute) -> EngineWheelInput {
    EngineWheelInput {
        row: 2,
        col: 1,
        horizontal,
        vertical,
        modifiers: SelectionModifiers::default(),
        route,
    }
}

#[test]
fn live_encoder_owns_sgr_wheel_axes_repetition_position_and_modifiers() {
    let mut engine = Engine::new(120, 40);
    engine.feed(b"\x1b[?1000h\x1b[?1006h");

    assert_eq!(
        TerminalEngine::wheel_input(&mut engine, wheel(0, -2, EngineWheelRoute::MouseReport))
            .unwrap(),
        b"\x1b[<64;2;3M\x1b[<64;2;3M",
    );
    assert_eq!(
        TerminalEngine::wheel_input(&mut engine, wheel(-1, 1, EngineWheelRoute::MouseReport))
            .unwrap(),
        b"\x1b[<65;2;3M\x1b[<66;2;3M",
    );
    let mut right = wheel(2, 0, EngineWheelRoute::MouseReport);
    right.modifiers = SelectionModifiers {
        shift: true,
        alt: true,
        control: true,
        meta: true,
    };
    assert_eq!(
        TerminalEngine::wheel_input(&mut engine, right).unwrap(),
        b"\x1b[<95;2;3M\x1b[<95;2;3M",
    );
}

#[test]
fn live_encoder_owns_legacy_utf8_and_urxvt_wheel_protocols() {
    let mut engine = Engine::new(240, 120);
    engine.feed(b"\x1b[?1000h");
    let mut legacy = wheel(0, -1, EngineWheelRoute::MouseReport);
    legacy.modifiers = SelectionModifiers {
        shift: true,
        alt: true,
        control: true,
        meta: false,
    };
    assert_eq!(
        TerminalEngine::wheel_input(&mut engine, legacy).unwrap(),
        [0x1b, b'[', b'M', 124, 34, 35],
    );

    engine.feed(b"\x1b[?1005h");
    let mut utf8 = wheel(0, -1, EngineWheelRoute::MouseReport);
    utf8.col = 100;
    assert_eq!(
        TerminalEngine::wheel_input(&mut engine, utf8).unwrap(),
        [0x1b, b'[', b'M', 96, 0xc2, 0x85, 35],
    );

    engine.feed(b"\x1b[?1005l\x1b[?1015h");
    assert_eq!(
        TerminalEngine::wheel_input(&mut engine, wheel(0, -1, EngineWheelRoute::MouseReport))
            .unwrap(),
        b"\x1b[96;2;3M",
    );
}

#[test]
fn alternate_screen_mode_1007_owns_both_axes_and_live_cursor_encoding() {
    let mut engine = Engine::new(80, 24);
    engine.feed(b"\x1b[?1049h\x1b[?1007h");

    assert_eq!(
        TerminalEngine::wheel_input(&mut engine, wheel(2, -2, EngineWheelRoute::AlternateScroll),)
            .unwrap(),
        b"\x1b[A\x1b[A\x1b[C\x1b[C",
    );

    engine.feed(b"\x1b[?1h");
    assert_eq!(
        TerminalEngine::wheel_input(&mut engine, wheel(-1, 1, EngineWheelRoute::AlternateScroll),)
            .unwrap(),
        b"\x1bOB\x1bOD",
    );
}

#[test]
fn stale_wheel_routes_are_refused_instead_of_reinterpreted() {
    let mut engine = Engine::new(80, 24);
    engine.feed(b"\x1b[?1000h\x1b[?1006h\x1b[?1000l");
    let mouse_error =
        TerminalEngine::wheel_input(&mut engine, wheel(0, -1, EngineWheelRoute::MouseReport))
            .unwrap_err();
    assert!(
        mouse_error.starts_with("WHEEL_MODE_CHANGED:"),
        "{mouse_error}"
    );

    engine.feed(b"\x1b[?1049h\x1b[?1007h\x1b[?1007l");
    let alternate_error =
        TerminalEngine::wheel_input(&mut engine, wheel(0, -1, EngineWheelRoute::AlternateScroll))
            .unwrap_err();
    assert!(
        alternate_error.starts_with("WHEEL_MODE_CHANGED:"),
        "{alternate_error}"
    );

    engine.feed(b"\x1b[?1007h\x1b[?1000h");
    let precedence_error =
        TerminalEngine::wheel_input(&mut engine, wheel(0, -1, EngineWheelRoute::AlternateScroll))
            .unwrap_err();
    assert!(
        precedence_error.starts_with("WHEEL_MODE_CHANGED:"),
        "{precedence_error}"
    );
}

#[test]
#[ignore = "RED: the public terminal-mode contract has no DEC 9/1001 tracking variant"]
fn red_x10_and_highlight_modes_can_generate_a_mouse_report_route() {
    for mode in [9, 1001] {
        let mut engine = Engine::new(80, 24);
        engine.feed(format!("\x1b[?{mode}h").as_bytes());
        let modes = TerminalEngine::modes(&engine);
        assert!(
            modes.mouse_click || modes.mouse_drag || modes.mouse_motion,
            "DEC {mode} is live in the engine but absent from the public route-generation modes",
        );
    }
}
