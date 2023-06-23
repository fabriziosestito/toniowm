use indexmap::IndexMap;
use thiserror::Error;
use xcb::{x, Xid, XidNew};

use crate::{
    commands::{Direction, Selector},
    vector::Vector2D,
};

const MIN_CLIENT_SIZE: Vector2D = Vector2D { x: 32, y: 32 };

#[derive(Error, Debug)]
pub enum Error {
    #[error("Client not found")]
    ClientNotFound,
    #[error("Client already exists")]
    ClientAlreadyExists,
}

#[derive(Clone, Copy, Debug)]
/// A client is everything we know by a window
pub struct Client {
    /// The window id
    window: x::Window,
    /// The position of the window
    pos: Vector2D,
    /// The size of the window
    size: Vector2D,
}

impl Client {
    pub fn window(&self) -> x::Window {
        self.window
    }

    pub fn pos(&self) -> Vector2D {
        self.pos
    }

    pub fn size(&self) -> Vector2D {
        self.size
    }
}

pub struct State {
    /// The root window
    pub root: x::Window,
    /// The window manager window
    pub child: x::Window,
    /// The list of clients managed by the window manager
    clients: IndexMap<x::Window, Client>,
    /// The currently focused window
    focused: Option<x::Window>,
    /// The last focused window
    last_focused: Option<x::Window>,
    /// The start position of the cursor when dragging a window.
    /// This is used to calculate the new position of the window.
    pub drag_start_pos: Vector2D,
    /// The start position of the frame when dragging a window
    /// This is used to calculate the new position of the window.
    pub drag_start_frame_pos: Vector2D,
}

impl Default for State {
    fn default() -> Self {
        Self {
            root: x::Window::none(),
            child: x::Window::none(),
            clients: IndexMap::new(),
            focused: Default::default(),
            last_focused: Default::default(),
            drag_start_pos: Default::default(),
            drag_start_frame_pos: Default::default(),
        }
    }
}

impl State {
    pub fn add_client(
        &mut self,
        window: x::Window,
        pos: Vector2D,
        size: Vector2D,
    ) -> Result<Client, Error> {
        if self.clients.contains_key(&window) {
            Err(Error::ClientAlreadyExists)
        } else {
            let client = Client { window, pos, size };
            self.clients.insert(window, client);

            Ok(client)
        }
    }

    /// Remove a client from the state.
    ///
    /// Return an error if the client is not found.
    pub fn remove_client(&mut self, window: x::Window) -> Result<(), Error> {
        if self.clients.shift_remove(&window).is_none() {
            Err(Error::ClientNotFound)
        } else {
            Ok(())
        }
    }

    /// Drag a client.
    ///
    /// Return an error if the client is not found.
    pub fn drag_client(&mut self, window: x::Window, mouse_pos: Vector2D) -> Result<Client, Error> {
        if let Some(client) = self.clients.get_mut(&window) {
            client.pos = self.drag_start_frame_pos + mouse_pos - self.drag_start_pos;

            Ok(client.to_owned())
        } else {
            Err(Error::ClientNotFound)
        }
    }

    /// Resize a client by dragging it.
    ///
    /// Return an error if the client is not found.
    pub fn drag_resize_client(
        &mut self,
        window: x::Window,
        mouse_pos: Vector2D,
    ) -> Result<Client, Error> {
        if let Some(client) = self.clients.get_mut(&window) {
            client.size = (mouse_pos - client.pos).max(MIN_CLIENT_SIZE);

            Ok(client.to_owned())
        } else {
            Err(Error::ClientNotFound)
        }
    }

    /// Teleport a client to a new position.
    ///
    /// Return an error if the client is not found.
    pub fn teleport_client(&mut self, window: x::Window, pos: Vector2D) -> Result<Client, Error> {
        if let Some(client) = self.clients.get_mut(&window) {
            client.pos = pos;

            Ok(client.to_owned())
        } else {
            Err(Error::ClientNotFound)
        }
    }

    /// Focus a client, saving the last focused client.
    ///
    /// Return an error if the client is not found.
    pub fn focus_client(&mut self, window: x::Window) -> Result<(), Error> {
        if self.root == window {
            self.set_focused(None);
            return Ok(());
        }

        if self.clients.contains_key(&window) {
            self.set_focused(Some(window));

            Ok(())
        } else {
            Err(Error::ClientNotFound)
        }
    }

    /// Focus the closest client in the given direction or none if there is no client.
    ///
    /// Accepts a Selector.
    /// Return an error if no client can be selected.
    pub fn focus_closest_client(
        &mut self,
        selector: Selector,
        direction: Direction,
    ) -> Result<Option<Client>, Error> {
        let client = if let Some(client) = self.select_client(selector) {
            client
        } else {
            return Err(Error::ClientNotFound);
        };

        let mut distance: i32;
        let mut min_distance = std::i32::MAX;
        let mut closest_client = None;

        for (_, c) in self.clients.clone() {
            if c.window == client.window {
                continue; // Skip the focused window
            }
            let dx = c.pos.x - client.pos.x;
            let dy = c.pos.y - client.pos.y;
            // Euclidean distance approximation
            // We do not need to calculate the square root to compare distances
            distance = dx.pow(2) + dy.pow(2);

            match direction {
                Direction::East => {
                    if c.pos.x > client.pos.x && distance < min_distance {
                        min_distance = distance;
                        closest_client = Some(c);
                    }
                }
                Direction::West => {
                    if c.pos.x < client.pos.x && distance < min_distance {
                        min_distance = distance;
                        closest_client = Some(c);
                    }
                }
                Direction::North => {
                    if c.pos.y < client.pos.y && distance < min_distance {
                        min_distance = distance;
                        closest_client = Some(c);
                    }
                }
                Direction::South => {
                    if c.pos.y > client.pos.y && distance < min_distance {
                        min_distance = distance;
                        closest_client = Some(c);
                    }
                }
            }
        }

        match closest_client {
            None => Ok(None),
            Some(closest_client) => {
                self.set_focused(Some(closest_client.window));
                Ok(Some(closest_client.to_owned()))
            }
        }
    }

    fn select_client(&self, selector: Selector) -> Option<&Client> {
        let window = match selector {
            Selector::Focused => self.focused?,
            Selector::Window(window) => unsafe { x::Window::new(window) },
        };

        self.clients.get(&window)
    }

    fn set_focused(&mut self, window: Option<x::Window>) {
        self.last_focused = self.focused;
        self.focused = window;
    }

    pub fn last_focused(&self) -> Option<x::Window> {
        self.last_focused
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::assert_matches::assert_matches;
    use xcb::XidNew;

    #[test]
    fn test_add_client() {
        let mut state = State::default();
        let window = unsafe { x::Window::new(123) };
        let pos = Vector2D::new(0, 0);
        let size = Vector2D::new(100, 100);

        let client = state.add_client(window, pos, size).unwrap();

        assert_eq!(client.window, window);
        assert_eq!(client.pos, pos);
        assert_eq!(client.size, size);
    }

    #[test]
    fn test_add_client_already_exists() {
        let mut state = State::default();
        let window = unsafe { x::Window::new(123) };
        let pos = Vector2D::new(0, 0);
        let size = Vector2D::new(100, 100);

        state.add_client(window, pos, size).unwrap();

        let result = state.add_client(window, pos, size);

        assert_matches!(result, Err(Error::ClientAlreadyExists));
    }

    #[test]
    fn test_remove_client() {
        let mut state = State::default();
        let window = unsafe { x::Window::new(123) };
        let pos = Vector2D::new(0, 0);
        let size = Vector2D::new(100, 100);

        state.add_client(window, pos, size).unwrap();

        let result = state.remove_client(window);

        assert_matches!(result, Ok(()));
        assert_eq!(state.clients.len(), 0);
    }

    #[test]
    fn test_remove_client_not_found() {
        let mut state = State::default();
        let window = unsafe { x::Window::new(123) };

        let result = state.remove_client(window);

        assert_matches!(result, Err(Error::ClientNotFound));
    }
}
