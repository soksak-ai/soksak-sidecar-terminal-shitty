use std::ffi::c_void;
use std::ptr::NonNull;

use soksak_kit_sidecar_terminal::mirror::TerminalEngine;
pub use soksak_kit_sidecar_terminal::mirror::{
    EnginePointerInput, EngineSelectionPoint, EngineWheelInput, SelectionKind, SelectionModifiers,
    TerminalCell as GridCell, TerminalColor as ColorSnap, TerminalCursorAnimation,
    TerminalCursorShape, TerminalCursorStyle, TerminalModes as ModeSnap, TerminalRgb,
    TerminalThemeOverrides,
};

const SUCCESS: i32 = 0;
const OUT_OF_SPACE: i32 = -2;

const POINTER_PRESS: i32 = 0;
const POINTER_RELEASE: i32 = 1;
const POINTER_MOTION: i32 = 2;

const MOUSE_MODIFIER_SHIFT: u32 = 1;
const MOUSE_MODIFIER_ALT: u32 = 2;
const MOUSE_MODIFIER_CONTROL: u32 = 4;

const COLOR_DEFAULT: u8 = 0;
const COLOR_PALETTE: u8 = 1;
const COLOR_RGB: u8 = 2;

const CURSOR_HIDDEN: u8 = 0;
const CURSOR_BLOCK: u8 = 1;
const CURSOR_HOLLOW_BLOCK: u8 = 2;
const CURSOR_UNDERLINE: u8 = 3;
const CURSOR_BAR: u8 = 4;

const MODE_BRACKETED_PASTE: u32 = 1 << 0;
const MODE_APPLICATION_CURSOR: u32 = 1 << 1;
const MODE_APPLICATION_KEYPAD: u32 = 1 << 2;
const MODE_MOUSE_CLICK: u32 = 1 << 3;
const MODE_MOUSE_DRAG: u32 = 1 << 4;
const MODE_MOUSE_MOTION: u32 = 1 << 5;
const MODE_SGR_MOUSE: u32 = 1 << 6;
const MODE_UTF8_MOUSE: u32 = 1 << 7;
const MODE_FOCUS_EVENTS: u32 = 1 << 8;
const MODE_ALTERNATE_SCROLL: u32 = 1 << 9;
const MODE_SHOW_CURSOR: u32 = 1 << 10;
const MODE_LINE_WRAP: u32 = 1 << 11;
const MODE_INSERT: u32 = 1 << 12;
const MODE_ALTERNATE_SCREEN: u32 = 1 << 13;

const ATTR_BOLD: u16 = 1 << 0;
const ATTR_DIM: u16 = 1 << 1;
const ATTR_ITALIC: u16 = 1 << 2;
const ATTR_UNDERLINE: u16 = 1 << 3;
const ATTR_INVERSE: u16 = 1 << 4;
const ATTR_STRIKE: u16 = 1 << 5;
const ATTR_HIDDEN: u16 = 1 << 6;

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct FfiColor {
    tag: u8,
    red_or_index: u8,
    green: u8,
    blue: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct FfiSnapshot {
    history_rows: u32,
    suppressed_replies: u32,
    modes: u32,
    cursor_blink_interval_ms: u32,
    columns: u16,
    rows: u16,
    cursor_x: u16,
    cursor_y: u16,
    cursor_style: u8,
    cursor_blinking: u8,
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct FfiCell {
    foreground: FfiColor,
    background: FfiColor,
    attributes: u16,
    wide: u8,
    wide_continuation: u8,
    wrapline: u8,
    line_attribute: u8,
}

#[repr(C)]
#[derive(Clone, Copy)]
struct FfiThemeOverrides {
    foreground: FfiColor,
    background: FfiColor,
    cursor: FfiColor,
    palette: [FfiColor; 256],
    palette_override_mask: [u64; 4],
    foreground_overridden: u8,
    background_overridden: u8,
    cursor_overridden: u8,
}

unsafe extern "C" {
    fn soksak_shitty_terminal_new(cols: u16, rows: u16, terminal: *mut *mut c_void) -> i32;
    fn soksak_shitty_terminal_free(terminal: *mut c_void);
    fn soksak_shitty_terminal_feed(terminal: *mut c_void, data: *const u8, len: usize) -> i32;
    fn soksak_shitty_terminal_resize(terminal: *mut c_void, cols: u16, rows: u16) -> i32;
    fn soksak_shitty_terminal_snapshot(terminal: *const c_void, snapshot: *mut FfiSnapshot) -> i32;
    fn soksak_shitty_terminal_theme_overrides(
        terminal: *const c_void,
        overrides: *mut FfiThemeOverrides,
    ) -> i32;
    fn soksak_shitty_terminal_pointer(
        terminal: *const c_void,
        column: u16,
        row: u16,
        event: i32,
        button: i32,
        modifiers: u32,
        output: *mut u8,
        capacity: usize,
        required: *mut usize,
    ) -> i32;
    fn soksak_shitty_terminal_cell(
        terminal: *const c_void,
        logical_row: i32,
        column: u16,
        cell: *mut FfiCell,
        codepoints: *mut u32,
        capacity: usize,
        required: *mut usize,
    ) -> i32;
}

pub struct Engine {
    terminal: NonNull<c_void>,
}

unsafe impl Send for Engine {}

impl Engine {
    pub fn new(cols: u16, rows: u16) -> Self {
        let mut terminal = std::ptr::null_mut();
        let result = unsafe { soksak_shitty_terminal_new(cols, rows, &mut terminal) };
        assert_eq!(result, SUCCESS, "Shitty terminal creation failed: {result}");
        Self {
            terminal: NonNull::new(terminal).expect("Shitty returned a null terminal"),
        }
    }

    pub fn feed(&mut self, bytes: &[u8]) {
        let result = unsafe {
            soksak_shitty_terminal_feed(self.terminal.as_ptr(), bytes.as_ptr(), bytes.len())
        };
        assert_eq!(result, SUCCESS, "Shitty feed failed: {result}");
    }

    pub fn resize(&mut self, cols: u16, rows: u16) {
        let result = unsafe { soksak_shitty_terminal_resize(self.terminal.as_ptr(), cols, rows) };
        assert_eq!(result, SUCCESS, "Shitty resize failed: {result}");
    }

    fn snapshot(&self) -> FfiSnapshot {
        let mut snapshot = FfiSnapshot::default();
        let result =
            unsafe { soksak_shitty_terminal_snapshot(self.terminal.as_ptr(), &mut snapshot) };
        assert_eq!(result, SUCCESS, "Shitty snapshot failed: {result}");
        snapshot
    }

    pub fn cols(&self) -> u16 {
        self.snapshot().columns
    }
    pub fn rows(&self) -> u16 {
        self.snapshot().rows
    }
    pub fn cursor(&self) -> (usize, usize) {
        let snapshot = self.snapshot();
        (snapshot.cursor_y as usize, snapshot.cursor_x as usize)
    }
    pub fn cursor_style(&self) -> TerminalCursorStyle {
        let snapshot = self.snapshot();
        let shape = match snapshot.cursor_style {
            CURSOR_HIDDEN | CURSOR_BLOCK | CURSOR_HOLLOW_BLOCK => TerminalCursorShape::Block,
            CURSOR_UNDERLINE => TerminalCursorShape::Underline,
            CURSOR_BAR => TerminalCursorShape::Bar,
            other => panic!("unknown Shitty cursor style: {other}"),
        };
        TerminalCursorStyle {
            shape,
            blinking: snapshot.cursor_blinking != 0,
        }
    }
    pub fn cursor_animation(&self) -> TerminalCursorAnimation {
        TerminalCursorAnimation {
            interval_ms: self.snapshot().cursor_blink_interval_ms,
        }
    }
    pub fn theme_overrides(&self) -> TerminalThemeOverrides {
        let empty = FfiColor::default();
        let mut value = FfiThemeOverrides {
            foreground: empty,
            background: empty,
            cursor: empty,
            palette: [empty; 256],
            palette_override_mask: [0; 4],
            foreground_overridden: 0,
            background_overridden: 0,
            cursor_overridden: 0,
        };
        let result = unsafe {
            soksak_shitty_terminal_theme_overrides(self.terminal.as_ptr(), &mut value)
        };
        assert_eq!(result, SUCCESS, "Shitty theme override snapshot failed: {result}");
        let rgb = |color: FfiColor| TerminalRgb {
            r: color.red_or_index,
            g: color.green,
            b: color.blue,
        };
        let mut overrides = TerminalThemeOverrides::default();
        if value.foreground_overridden != 0 {
            overrides.foreground = Some(rgb(value.foreground));
        }
        if value.background_overridden != 0 {
            overrides.background = Some(rgb(value.background));
        }
        if value.cursor_overridden != 0 {
            overrides.cursor = Some(rgb(value.cursor));
        }
        for (index, slot) in overrides.ansi.iter_mut().enumerate() {
            if value.palette_override_mask[index >> 6] & (1u64 << (index & 63)) != 0 {
                *slot = Some(rgb(value.palette[index]));
            }
        }
        overrides
    }
    pub fn alt_active(&self) -> bool {
        self.snapshot().modes & MODE_ALTERNATE_SCREEN != 0
    }
    pub fn history_size(&self) -> usize {
        self.snapshot().history_rows as usize
    }
    pub fn suppressed_replies(&self) -> u64 {
        self.snapshot().suppressed_replies as u64
    }

    pub fn modes(&self) -> ModeSnap {
        let modes = self.snapshot().modes;
        ModeSnap {
            bracketed_paste: modes & MODE_BRACKETED_PASTE != 0,
            app_cursor: modes & MODE_APPLICATION_CURSOR != 0,
            app_keypad: modes & MODE_APPLICATION_KEYPAD != 0,
            mouse_click: modes & MODE_MOUSE_CLICK != 0,
            mouse_drag: modes & MODE_MOUSE_DRAG != 0,
            mouse_motion: modes & MODE_MOUSE_MOTION != 0,
            sgr_mouse: modes & MODE_SGR_MOUSE != 0,
            utf8_mouse: modes & MODE_UTF8_MOUSE != 0,
            focus_in_out: modes & MODE_FOCUS_EVENTS != 0,
            alternate_scroll: modes & MODE_ALTERNATE_SCROLL != 0,
            show_cursor: modes & MODE_SHOW_CURSOR != 0,
            line_wrap: modes & MODE_LINE_WRAP != 0,
            insert: modes & MODE_INSERT != 0,
        }
    }

    pub fn line_cells(&self, line: i32) -> Vec<GridCell> {
        (0..self.cols())
            .map(|column| self.cell(line, column))
            .collect()
    }

    pub fn pointer_input(&mut self, input: EnginePointerInput) -> Result<Vec<u8>, String> {
        let event = match input.phase {
            soksak_kit_sidecar_terminal::mirror::PointerPhase::Down => POINTER_PRESS,
            soksak_kit_sidecar_terminal::mirror::PointerPhase::Up => POINTER_RELEASE,
            soksak_kit_sidecar_terminal::mirror::PointerPhase::Move => POINTER_MOTION,
        };
        let button = match input.button {
            soksak_kit_sidecar_terminal::mirror::PointerButton::None => 0,
            soksak_kit_sidecar_terminal::mirror::PointerButton::Left => 1,
            soksak_kit_sidecar_terminal::mirror::PointerButton::Middle => 2,
            soksak_kit_sidecar_terminal::mirror::PointerButton::Right => 3,
        };
        let mut modifiers = 0u32;
        if input.modifiers.shift {
            modifiers |= MOUSE_MODIFIER_SHIFT;
        }
        if input.modifiers.alt {
            modifiers |= MOUSE_MODIFIER_ALT;
        }
        if input.modifiers.control {
            modifiers |= MOUSE_MODIFIER_CONTROL;
        }

        let mut required = 0usize;
        let first = unsafe {
            soksak_shitty_terminal_pointer(
                self.terminal.as_ptr(),
                input.col,
                input.row,
                event,
                button,
                modifiers,
                std::ptr::null_mut(),
                0,
                &mut required,
            )
        };
        if first != SUCCESS && first != OUT_OF_SPACE {
            return Err(format!("Shitty mouse encoder failed: {first}"));
        }
        if required == 0 {
            return Ok(Vec::new());
        }
        let mut output = vec![0u8; required];
        let result = unsafe {
            soksak_shitty_terminal_pointer(
                self.terminal.as_ptr(),
                input.col,
                input.row,
                event,
                button,
                modifiers,
                output.as_mut_ptr(),
                output.len(),
                &mut required,
            )
        };
        if result != SUCCESS {
            return Err(format!("Shitty mouse encoder retry failed: {result}"));
        }
        output.truncate(required);
        Ok(output)
    }

    fn cell(&self, line: i32, column: u16) -> GridCell {
        let mut cell = FfiCell::default();
        let mut required = 0;
        let first = unsafe {
            soksak_shitty_terminal_cell(
                self.terminal.as_ptr(),
                line,
                column,
                &mut cell,
                std::ptr::null_mut(),
                0,
                &mut required,
            )
        };
        assert!(
            first == SUCCESS || first == OUT_OF_SPACE,
            "Shitty cell size failed: {first}"
        );
        let mut codepoints = vec![0; required];
        if required != 0 {
            let result = unsafe {
                soksak_shitty_terminal_cell(
                    self.terminal.as_ptr(),
                    line,
                    column,
                    &mut cell,
                    codepoints.as_mut_ptr(),
                    codepoints.len(),
                    &mut required,
                )
            };
            assert_eq!(result, SUCCESS, "Shitty cell read failed: {result}");
        }
        let mut chars = codepoints
            .into_iter()
            .map(|value| char::from_u32(value).unwrap_or(char::REPLACEMENT_CHARACTER));
        let ch = chars.next().unwrap_or(' ');
        GridCell {
            ch,
            fg: color(cell.foreground),
            bg: color(cell.background),
            bold: cell.attributes & ATTR_BOLD != 0,
            dim: cell.attributes & ATTR_DIM != 0,
            italic: cell.attributes & ATTR_ITALIC != 0,
            underline: cell.attributes & ATTR_UNDERLINE != 0,
            inverse: cell.attributes & ATTR_INVERSE != 0,
            strikeout: cell.attributes & ATTR_STRIKE != 0,
            hidden: cell.attributes & ATTR_HIDDEN != 0,
            wide: cell.wide != 0,
            spacer: cell.wide_continuation != 0,
            wrapline: cell.wrapline != 0,
            zerowidth: chars.collect(),
            // This engine does not track OSC 8; capabilities.hyperlinks stays false.
            link: None,
        }
    }
}

impl Drop for Engine {
    fn drop(&mut self) {
        unsafe { soksak_shitty_terminal_free(self.terminal.as_ptr()) };
    }
}

impl TerminalEngine for Engine {
    fn new(cols: u16, rows: u16) -> Self {
        Engine::new(cols, rows)
    }
    fn feed(&mut self, bytes: &[u8]) {
        Engine::feed(self, bytes);
    }
    fn resize(&mut self, cols: u16, rows: u16) {
        Engine::resize(self, cols, rows);
    }
    fn cols(&self) -> u16 {
        Engine::cols(self)
    }
    fn rows(&self) -> u16 {
        Engine::rows(self)
    }
    fn cursor(&self) -> (usize, usize) {
        Engine::cursor(self)
    }
    fn cursor_style(&self) -> TerminalCursorStyle {
        Engine::cursor_style(self)
    }
    fn cursor_animation(&self) -> TerminalCursorAnimation {
        Engine::cursor_animation(self)
    }
    fn theme_overrides(&self) -> TerminalThemeOverrides {
        Engine::theme_overrides(self)
    }
    fn alt_active(&self) -> bool {
        Engine::alt_active(self)
    }
    fn history_size(&self) -> usize {
        Engine::history_size(self)
    }
    fn modes(&self) -> ModeSnap {
        Engine::modes(self)
    }
    fn line_cells(&self, line: i32) -> Vec<GridCell> {
        Engine::line_cells(self, line)
    }
    fn suppressed_replies(&self) -> u64 {
        Engine::suppressed_replies(self)
    }
    fn selection_begin(
        &mut self,
        _kind: SelectionKind,
        _point: EngineSelectionPoint,
        _modifiers: SelectionModifiers,
    ) -> Result<(), String> {
        Err("Shitty selection input is not implemented".into())
    }
    fn selection_update(
        &mut self,
        _point: EngineSelectionPoint,
        _modifiers: SelectionModifiers,
    ) -> Result<(), String> {
        Err("Shitty selection input is not implemented".into())
    }
    fn selection_clear(&mut self) {}
    fn selection_text(&self) -> Option<String> {
        None
    }
    fn selection_range(&self, _line: i32) -> Option<(u16, u16)> {
        None
    }
    fn wheel_input(&mut self, _input: EngineWheelInput) -> Result<Vec<u8>, String> {
        Err("Shitty wheel input is not implemented".into())
    }
    fn pointer_input(&mut self, input: EnginePointerInput) -> Result<Vec<u8>, String> {
        Engine::pointer_input(self, input)
    }
}

fn color(value: FfiColor) -> ColorSnap {
    match value.tag {
        COLOR_DEFAULT => ColorSnap::Default,
        COLOR_PALETTE => ColorSnap::Indexed(value.red_or_index),
        COLOR_RGB => ColorSnap::Rgb(value.red_or_index, value.green, value.blue),
        other => panic!("unknown Shitty color tag: {other}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_exposes_raw_osc_color_overrides() {
        let mut engine = Engine::new(4, 2);
        engine.feed(
            b"\x1b]4;1;#123456\x07\x1b]10;#abcdef\x07\x1b]11;#223344\x07\x1b]12;#654321\x07",
        );
        let colors = TerminalEngine::theme_overrides(&engine);
        assert_eq!(colors.ansi[1], Some(TerminalRgb { r: 0x12, g: 0x34, b: 0x56 }));
        assert_eq!(colors.foreground, Some(TerminalRgb { r: 0xab, g: 0xcd, b: 0xef }));
        assert_eq!(colors.background, Some(TerminalRgb { r: 0x22, g: 0x33, b: 0x44 }));
        assert_eq!(colors.cursor, Some(TerminalRgb { r: 0x65, g: 0x43, b: 0x21 }));
        engine.feed(b"\x1b]104;1\x07\x1b]110\x07\x1b]111\x07\x1b]112\x07");
        let reset = TerminalEngine::theme_overrides(&engine);
        assert_eq!(reset.ansi[1], None);
        assert_eq!((reset.foreground, reset.background, reset.cursor), (None, None, None));
    }
}
