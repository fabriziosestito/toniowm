//! Functions to interact with the ICCCM specification.

use xcb::{x, Xid};

use crate::atoms::Atoms;

/// Get the WM_PROTOCOLS property from a window.
///
/// The WM_PROTOCOLS property (of type ATOM) is a list of atoms.
/// Each atom identifies a communication protocol between the client and the window manager in which the client is willing to participate.
pub fn get_wm_protocols(
    conn: &xcb::Connection,
    atoms: &Atoms,
    window: x::Window,
) -> xcb::Result<Vec<x::Atom>> {
    let cookie = conn.send_request(&x::GetProperty {
        delete: false,
        window,
        property: atoms.wm_protocols,
        r#type: x::ATOM_ATOM,
        long_offset: 0,
        long_length: 124,
    });

    let reply = conn.wait_for_reply(cookie)?;

    Ok(reply.value().to_vec())
}

pub fn send_wm_delete_window(
    conn: &xcb::Connection,
    atoms: &Atoms,
    window: x::Window,
) -> xcb::Result<()> {
    let event = x::ClientMessageEvent::new(
        window,
        atoms.wm_protocols,
        x::ClientMessageData::Data32([
            atoms.wm_delete_window.resource_id(),
            x::CURRENT_TIME,
            0,
            0,
            0,
        ]),
    );

    let cookie = conn.send_request_checked(&x::SendEvent {
        propagate: false,
        destination: x::SendEventDest::Window(window),
        event_mask: x::EventMask::NO_EVENT,
        event: &event,
    });

    conn.check_request(cookie)?;

    Ok(())
}
