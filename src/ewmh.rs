//! Functions to interact with the EWMH specification.

use xcb::x;

use crate::atoms::Atoms;

pub fn get_wm_window_type(
    conn: &xcb::Connection,
    atoms: &Atoms,
    window: x::Window,
) -> xcb::Result<Vec<x::Atom>> {
    let cookie = conn.send_request(&x::GetProperty {
        window,
        delete: false,
        property: atoms.net_wm_window_type,
        r#type: x::ATOM_ATOM,
        long_offset: 0,
        long_length: 1024,
    });
    let reply = conn.wait_for_reply(cookie)?;

    Ok(reply.value().into())
}

// Set the _NET_SUPPORTED property on the root window.
// This is needed to indicate which hints are supported by the window manager.
pub fn set_supported(conn: &xcb::Connection, atoms: &Atoms, root: x::Window) {
    conn.send_request(&x::ChangeProperty {
        mode: x::PropMode::Replace,
        window: root,
        property: atoms.net_supported,
        r#type: x::ATOM_ATOM,
        data: &[
            atoms.net_supported,
            atoms.net_active_window,
            atoms.net_number_of_desktops,
            atoms.net_desktop_names,
            atoms.net_current_desktop,
            atoms.net_wm_window_type,
        ],
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
        r#type: atoms.utf8_string,
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
/// Set the _NET_NUMBER_OF_DESKTOPS property on the root window.
/// This is needed to indicate the number of desktops.
pub fn set_number_of_desktops(conn: &xcb::Connection, atoms: &Atoms, root: x::Window, num: u32) {
    conn.send_request(&x::ChangeProperty {
        mode: x::PropMode::Replace,
        window: root,
        property: atoms.net_number_of_desktops,
        r#type: x::ATOM_CARDINAL,
        data: &[num],
    });
}

/// Set the _NET_DESKTOP_NAMES property on the root window.
/// This is needed to indicate the names of the desktops.
pub fn set_desktop_names(conn: &xcb::Connection, atoms: &Atoms, root: x::Window, names: Vec<&str>) {
    let mut data = names.join("\0").as_bytes().to_vec();
    data.push(b'\0');

    conn.send_request(&x::ChangeProperty {
        mode: x::PropMode::Replace,
        window: root,
        property: atoms.net_desktop_names,
        r#type: atoms.utf8_string,
        data: &data,
    });
}

/// Set the _NET_CURRENT_DESKTOP property on the root window.
/// This is needed to indicate the currently active desktop.
pub fn set_current_desktop(conn: &xcb::Connection, atoms: &Atoms, root: x::Window, num: u32) {
    conn.send_request(&x::ChangeProperty {
        mode: x::PropMode::Replace,
        window: root,
        property: atoms.net_current_desktop,
        r#type: x::ATOM_CARDINAL,
        data: &[num],
    });
}
