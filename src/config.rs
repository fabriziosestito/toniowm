use xcb::x;

pub static MOD_KEY: x::ModMask = x::ModMask::N4; // Mod
pub static MOD_KEY_BUT: x::KeyButMask = x::KeyButMask::MOD4;

pub static DRAG_BUTTON: x::ButtonIndex = x::ButtonIndex::N1; // Left Mouse Button
pub static DRAG_BUTTON_MASK: x::KeyButMask = x::KeyButMask::BUTTON1;

pub static SELECT_BUTTON: x::ButtonIndex = x::ButtonIndex::N1; // Left Mouse Button

pub static RESIZE_BUTTON: x::ButtonIndex = x::ButtonIndex::N3; // Right Mouse Button
pub static RESIZE_BUTTON_MASK: x::KeyButMask = x::KeyButMask::BUTTON3;

pub static BORDER_WIDTH: usize = 2;

pub static BORDER_COLOR: u32 = 0xcccccc;
pub static BORDER_COLOR_FOCUS: u32 = 0x00ccff;
