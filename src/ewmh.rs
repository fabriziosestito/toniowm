//! Functions to interact with the EWMH specification.

use xcb::x;

use crate::atoms::Atoms;

// Set the _NET_SUPPORTED property on the root window.
// This is needed to indicate which hints are supported by the window manager.
pub fn set_supported(conn: &xcb::Connection, atoms: &Atoms, root: x::Window) {
    conn.send_request(&x::ChangeProperty {
        mode: x::PropMode::Replace,
        window: root,
        property: atoms.net_supported,
        r#type: x::ATOM_ATOM,
        data: &[atoms.net_supported, atoms.net_active_window],
    });
}
/// Set the _NET_SUPPORTING_WM_CHECK property on the root and child windows.
/// This is needed to indicate that a compliant window manager is active.
pub fn set_supporting_wm_check(
    conn: &xcb::Connection,
    atoms: &Atoms,
    root: x::Window,
    child: x::Window,
) {
    conn.send_request(&x::ChangeProperty {
        mode: x::PropMode::Replace,
        window: child,
        property: atoms.net_supporting_wm_check,
        r#type: x::ATOM_WINDOW,
        data: &[child],
    });

    conn.send_request(&x::ChangeProperty {
        mode: x::PropMode::Replace,
        window: root,
        property: atoms.net_supporting_wm_check,
        r#type: x::ATOM_WINDOW,
        data: &[child],
    });
}

/// Set the _NET_WM_NAME property on the child window.
/// This is needed to indicate the name of the window manager.
pub fn set_wm_name(conn: &xcb::Connection, atoms: &Atoms, child: x::Window, wm_name: &str) {
    conn.send_request(&x::ChangeProperty {
        mode: x::PropMode::Replace,
        window: child,
        property: atoms.net_wm_name,
        r#type: x::ATOM_STRING,
        data: wm_name.as_bytes(),
    });
}

/// Set the _NET_ACTIVE_WINDOW property on the root window.
/// This is needed to indicate the currently active window.
pub fn set_active_window(
    conn: &xcb::Connection,
    atoms: &Atoms,
    root: x::Window,
    window: x::Window,
) {
    conn.send_request(&x::ChangeProperty {
        mode: x::PropMode::Replace,
        window: root,
        property: atoms.net_active_window,
        r#type: x::ATOM_WINDOW,
        data: &[window],
    });
}
