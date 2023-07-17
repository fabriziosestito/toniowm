use xcb::x;

pub static MOD_KEY: x::ModMask = x::ModMask::N4; // Mod
pub static MOD_KEY_BUT: x::KeyButMask = x::KeyButMask::MOD4;

pub static DRAG_BUTTON: x::ButtonIndex = x::ButtonIndex::N1; // Left Mouse Button
pub static DRAG_BUTTON_MASK: x::KeyButMask = x::KeyButMask::BUTTON1;

pub static SELECT_BUTTON: x::ButtonIndex = x::ButtonIndex::N1; // Left Mouse Button

pub static RESIZE_BUTTON: x::ButtonIndex = x::ButtonIndex::N3; // Right Mouse Button
pub static RESIZE_BUTTON_MASK: x::KeyButMask = x::KeyButMask::BUTTON3;

pub struct Config {
    pub border_width: u32,
    pub border_color: u32,
    pub border_color_focus: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            border_width: 0,
            border_color: 0xcccccc,
            border_color_focus: 0x00ccff,
        }
    }
}
