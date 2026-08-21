use std::ffi::c_void;
use std::ptr::NonNull;

use soksak_kit_sidecar_terminal::mirror::TerminalEngine;
pub use soksak_kit_sidecar_terminal::mirror::{
    TerminalCell as GridCell, TerminalColor as ColorSnap, TerminalModes as ModeSnap,
};

const SUCCESS: i32 = 0;
const OUT_OF_SPACE: i32 = -2;

const COLOR_DEFAULT: u8 = 0;
const COLOR_PALETTE: u8 = 1;
const COLOR_RGB: u8 = 2;

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
    columns: u16,
    rows: u16,
    cursor_x: u16,
    cursor_y: u16,
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

unsafe extern "C" {
    fn soksak_shitty_terminal_new(cols: u16, rows: u16, terminal: *mut *mut c_void) -> i32;
    fn soksak_shitty_terminal_free(terminal: *mut c_void);
    fn soksak_shitty_terminal_feed(terminal: *mut c_void, data: *const u8, len: usize) -> i32;
    fn soksak_shitty_terminal_resize(terminal: *mut c_void, cols: u16, rows: u16) -> i32;
    fn soksak_shitty_terminal_snapshot(terminal: *const c_void, snapshot: *mut FfiSnapshot) -> i32;
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
}

fn color(value: FfiColor) -> ColorSnap {
    match value.tag {
        COLOR_DEFAULT => ColorSnap::Default,
        COLOR_PALETTE => ColorSnap::Indexed(value.red_or_index),
        COLOR_RGB => ColorSnap::Rgb(value.red_or_index, value.green, value.blue),
        other => panic!("unknown Shitty color tag: {other}"),
    }
}
