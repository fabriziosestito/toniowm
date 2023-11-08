use indexmap::{map::MutableKeys, IndexMap};
use thiserror::Error;
use xcb::{x, Xid, XidNew};

use crate::{
    commands::{CardinalDirection, CycleDirection, WindowSelector, WorkspaceSelector},
    vector::Vector2D,
};

const MIN_CLIENT_SIZE: Vector2D = Vector2D { x: 32, y: 32 };

#[derive(Error, Debug)]
pub enum Error {
    #[error("Client not found.")]
    ClientNotFound,
    #[error("Client already exists.")]
    ClientAlreadyExists,
    #[error("Workspace already exists.")]
    WorkspaceAlreadyExists,
    #[error("Workspace not found.")]
    WorkspaceNotFound,
}

#[derive(Debug, PartialEq, Default)]
pub struct Workspace {
    /// The list of clients managed by the workspace
    clients: IndexMap<x::Window, Client>,
}

#[derive(Clone, Debug, PartialEq)]
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
}

pub struct State {
    /// The root window,.
    pub root: x::Window,
    /// The window manager window.
    pub child: x::Window,
    /// The list of workspaces managed by the window manager
    workspaces: IndexMap<String, Workspace>,
    /// The currently active workspace.
    active_workspace: usize,
    /// The currently focused window.
    focused: Option<x::Window>,
    /// The last focused window.
    last_focused: Option<x::Window>,
    /// The start position of the cursor when dragging a window.
    /// This is used to calculate the new position of the window.
    pub drag_start_pos: Vector2D,
    /// The start position of the frame when dragging a window
    /// This is used to calculate the new position of the window.
    pub drag_start_frame_pos: Vector2D,
    /// The size of the monitor.
    pub monitor_size: Vector2D,
}

impl Default for State {
    fn default() -> Self {
        let mut state = Self {
            root: x::Window::none(),
            child: x::Window::none(),
            workspaces: Default::default(),
            active_workspace: 0,
            focused: Default::default(),
            last_focused: Default::default(),
            drag_start_pos: Default::default(),
            drag_start_frame_pos: Default::default(),
            monitor_size: Default::default(),
        };

        state.add_workspace(None).unwrap();

        state
    }
}

impl State {
    /// Add a workspace to the state.
    ///
    /// If no name is provided, the workspace will be named after the index + 1.
    /// The name of the workspace must be unique.
    pub fn add_workspace(&mut self, name: Option<String>) -> Result<(), Error> {
        let name = if let Some(name) = name {
            name
        } else {
            (self.workspaces.len() + 1).to_string()
        };

        if self.workspaces.contains_key(&name) {
            Err(Error::WorkspaceAlreadyExists)
        } else {
            let workspace = Workspace {
                clients: IndexMap::new(),
            };

            self.workspaces.insert(name, workspace);
            Ok(())
        }
    }
    /// Rename a workspace.
    ///
    /// Accepts a selector.
    /// Return an error if no matching workspace is not found.
    /// Return an error if the new name is already taken.
    pub fn rename_workspace(
        &mut self,
        selector: WorkspaceSelector,
        name: String,
    ) -> Result<(), Error> {
        let (old_name, _) = match selector {
            WorkspaceSelector::Index(index) => {
                if let Some((old_name, workspace)) = self.workspaces.get_index_mut2(index) {
                    (old_name, workspace)
                } else {
                    return Err(Error::WorkspaceNotFound);
                }
            }
            WorkspaceSelector::Name(name) => {
                if let Some((_, old_name, workspace)) = self.workspaces.get_full_mut2(&name) {
                    (old_name, workspace)
                } else {
                    return Err(Error::WorkspaceNotFound);
                }
            }
        };

        *old_name = name;

        Ok(())
    }

    ///  Active a workspace as active and return its index.
    ///
    /// Accepts a selector.
    /// Return an error if no matching workspace is not found.
    pub fn activate_workspace(&mut self, selector: WorkspaceSelector) -> Result<usize, Error> {
        let index = match selector {
            WorkspaceSelector::Index(index) => Some(index),
            WorkspaceSelector::Name(name) => self.workspaces.get_index_of(&name),
        };
        if let Some(index) = index {
            self.active_workspace = index;

            Ok(index)
        } else {
            Err(Error::WorkspaceNotFound)
        }
    }

    /// Return a list of the workspaces names.
    pub fn workspaces_names(&self) -> Vec<String> {
        self.workspaces.keys().cloned().collect()
    }

    /// Add a client to the state.
    ///
    /// Return an error if the client already exists.
    pub fn add_client(
        &mut self,
        window: x::Window,
        pos: Vector2D,
        size: Vector2D,
    ) -> Result<(), Error> {
        if self.active_workspace_clients().contains_key(&window) {
            Err(Error::ClientAlreadyExists)
        } else {
            let client = Client { window, pos, size };
            self.active_workspace_clients_mut().insert(window, client);

            Ok(())
        }
    }

    /// Remove a client from the state.
    ///
    /// Return an error if the client is not found.
    pub fn remove_client(&mut self, window: x::Window) -> Result<(), Error> {
        if self
            .active_workspace_clients_mut()
            .shift_remove(&window)
            .is_none()
        {
            Err(Error::ClientNotFound)
        } else {
            if self.focused == Some(window) {
                self.focused = None;
            }
            Ok(())
        }
    }

    /// Drag a client and return its new position.
    ///
    /// Return an error if the client is not found.
    pub fn drag_client(
        &mut self,
        window: x::Window,
        mouse_pos: Vector2D,
    ) -> Result<Vector2D, Error> {
        let new_pos = self.drag_start_frame_pos + mouse_pos - self.drag_start_pos;
        if let Some(client) = self.active_workspace_clients_mut().get_mut(&window) {
            client.pos = new_pos;

            Ok(new_pos)
        } else {
            Err(Error::ClientNotFound)
        }
    }

    /// Resize a client by dragging it and return its new size.
    ///
    /// Return an error if the client is not found.
    pub fn drag_resize_client(
        &mut self,
        window: x::Window,
        mouse_pos: Vector2D,
    ) -> Result<Vector2D, Error> {
        if let Some(client) = self.active_workspace_clients_mut().get_mut(&window) {
            let new_size = (mouse_pos - client.pos).max(MIN_CLIENT_SIZE);
            client.size = new_size;

            Ok(new_size)
        } else {
            Err(Error::ClientNotFound)
        }
    }

    /// Teleport a client to a new position.
    ///
    /// Return an error if the client is not found.
    pub fn teleport_client(&mut self, window: x::Window, pos: Vector2D) -> Result<(), Error> {
        if let Some(client) = self.active_workspace_clients_mut().get_mut(&window) {
            client.pos = pos;

            Ok(())
        } else {
            Err(Error::ClientNotFound)
        }
    }

    /// Focus a client, saving the last focused client.
    ///
    /// Return an error if the client is not found.
    pub fn focus_client(&mut self, selector: WindowSelector) -> Result<Option<x::Window>, Error> {
        // Root window focus is used to unfocus the current window.
        if let WindowSelector::Window(window) = selector {
            if self.root.resource_id() == window {
                self.set_focused(None);
                return Ok(None);
            }
        }

        let client = self.select_client(selector)?.clone();

        self.set_focused(Some(client.window));
        Ok(Some(client.window))
    }

    /// Get the active workspace clients.
    pub fn active_workspace_clients(&self) -> &IndexMap<x::Window, Client> {
        // We can unwrap here because we know the workspace exists.
        let (_, workspace) = self.workspaces.get_index(self.active_workspace).unwrap();

        &workspace.clients
    }

    /// Get the active workspace clients.
    fn active_workspace_clients_mut(&mut self) -> &mut IndexMap<x::Window, Client> {
        // We can unwrap here because we know the workspace exists.
        let (_, workspace) = self
            .workspaces
            .get_index_mut(self.active_workspace)
            .unwrap();

        &mut workspace.clients
    }

    /// Select a client using a selector.
    ///
    /// Return an error if no matching client has been found.
    pub fn select_client(&self, selector: WindowSelector) -> Result<&Client, Error> {
        match selector {
            WindowSelector::Focused => {
                if let Some(window) = self.focused {
                    self.active_workspace_clients()
                        .get(&window)
                        .ok_or(Error::ClientNotFound)
                } else {
                    Err(Error::ClientNotFound)
                }
            }
            WindowSelector::Window(window) => unsafe {
                self.active_workspace_clients()
                    .get(&x::Window::new(window))
                    .ok_or(Error::ClientNotFound)
            },
            WindowSelector::Closest(direction) => self.select_closest_client(direction),
            WindowSelector::Cycle(direction) => match direction {
                CycleDirection::Next => {
                    todo!()
                }
                CycleDirection::Prev => todo!(),
            },
        }
    }

    /// Return the closest client in the given cardinal direction
    fn select_closest_client(&self, direction: CardinalDirection) -> Result<&Client, Error> {
        let client = if let Some(focused) = self.focused {
            self.active_workspace_clients()
                .get(&focused)
                .expect("Focused client not found")
        } else {
            return Err(Error::ClientNotFound);
        };

        let mut distance: i32;
        let mut min_distance = std::i32::MAX;
        let mut closest_client = None;

        for (_, c) in self.active_workspace_clients() {
            if c.window == client.window {
                continue; // Skip the focused window
            }
            let dx = c.pos.x - client.pos.x;
            let dy = c.pos.y - client.pos.y;
            // Euclidean distance approximation
            // We do not need to calculate the square root to compare distances.
            distance = dx.pow(2) + dy.pow(2);

            match direction {
                CardinalDirection::East => {
                    if c.pos.x > client.pos.x && distance < min_distance {
                        min_distance = distance;
                        closest_client = Some(c);
                    }
                }
                CardinalDirection::West => {
                    if c.pos.x < client.pos.x && distance < min_distance {
                        min_distance = distance;
                        closest_client = Some(c);
                    }
                }
                CardinalDirection::North => {
                    if c.pos.y < client.pos.y && distance < min_distance {
                        min_distance = distance;
                        closest_client = Some(c);
                    }
                }
                CardinalDirection::South => {
                    if c.pos.y > client.pos.y && distance < min_distance {
                        min_distance = distance;
                        closest_client = Some(c);
                    }
                }
            }
        }

        match closest_client {
            None => Err(Error::ClientNotFound),
            Some(closest_client) => Ok(closest_client),
        }
    }

    /// Set the focused window.
    /// Save the last focused window.
    fn set_focused(&mut self, window: Option<x::Window>) {
        self.last_focused = self.focused;
        self.focused = window;
    }

    /// Get the focused window.
    pub fn focused(&self) -> Option<x::Window> {
        self.focused
    }

    /// Get the last focused window.
    pub fn last_focused(&self) -> Option<x::Window> {
        self.last_focused
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use xcb::XidNew;

    #[test]
    fn test_add_workspace() {
        let mut state = State::default();
        state.add_workspace(Some("test".to_owned())).unwrap();

        assert_eq!(state.workspaces.len(), 2);
        assert!(state.workspaces.contains_key("test"));
    }

    #[test]
    fn test_add_workspace_no_name() {
        let mut state = State::default();
        state.add_workspace(None).unwrap();

        assert_eq!(state.workspaces.len(), 2);
        assert!(state.workspaces.contains_key("1"));
    }

    #[test]
    fn test_add_workspace_already_exists() {
        let mut state = State::default();
        state.add_workspace(Some("test".to_owned())).unwrap();

        assert!(matches!(
            state.add_workspace(Some("test".to_owned())),
            Err(Error::WorkspaceAlreadyExists)
        ));
    }

    #[test]
    fn workspaces_names() {
        let mut state = State::default();
        state.add_workspace(Some("2".to_owned())).unwrap();
        state.add_workspace(Some("3".to_owned())).unwrap();

        let workspaces_names = state.workspaces_names();

        assert_eq!(workspaces_names, vec!["1", "2", "3"]);
    }

    #[test]
    fn test_activate_workspace() {
        let mut state = State::default();
        state.add_workspace(Some("test".to_owned())).unwrap();

        let index = state
            .activate_workspace(WorkspaceSelector::Name("test".to_string()))
            .unwrap();

        assert_eq!(1, index);
        assert_eq!(1, state.active_workspace);
    }

    #[test]
    fn test_activate_workspace_not_found() {
        let mut state = State::default();
        let result = state.activate_workspace(WorkspaceSelector::Name("test".to_string()));

        assert!(matches!(result, Err(Error::WorkspaceNotFound)));
        assert_eq!(0, state.active_workspace);
    }

    #[test]
    fn test_add_client() {
        let mut state = State::default();
        let window = unsafe { x::Window::new(123) };
        let pos = Vector2D::new(0, 0);
        let size = Vector2D::new(100, 100);

        state.add_client(window, pos, size).unwrap();

        let expected_client = Client { window, pos, size };

        assert_eq!(
            &expected_client,
            state.active_workspace_clients().get(&window).unwrap(),
        );
    }

    #[test]
    fn test_add_client_already_exists() {
        let mut state = State::default();
        let window = unsafe { x::Window::new(123) };
        let pos = Vector2D::new(0, 0);
        let size = Vector2D::new(100, 100);

        state.add_client(window, pos, size).unwrap();

        let result = state.add_client(window, pos, size);

        assert!(matches!(result, Err(Error::ClientAlreadyExists)));
    }

    #[test]
    fn test_remove_client() {
        let mut state = State::default();
        let window = unsafe { x::Window::new(123) };
        let pos = Vector2D::new(0, 0);
        let size = Vector2D::new(100, 100);

        state.add_client(window, pos, size).unwrap();
        state.set_focused(Some(window));

        let result = state.remove_client(window);

        assert!(matches!(result, Ok(())));
        assert_eq!(state.active_workspace_clients().len(), 0);
        assert_eq!(state.focused, None);
    }

    #[test]
    fn test_remove_client_not_found() {
        let mut state = State::default();
        let window = unsafe { x::Window::new(123) };

        let result = state.remove_client(window);

        assert!(matches!(result, Err(Error::ClientNotFound)));
    }

    #[test]
    fn test_drag_client() {
        let mut state = State::default();
        let window = unsafe { x::Window::new(123) };
        let pos = Vector2D::new(0, 0);
        let size = Vector2D::new(100, 100);

        state.add_client(window, pos, size).unwrap();

        let new_pos = Vector2D::new(10, 10);
        let pos = state.drag_client(window, new_pos).unwrap();

        assert_eq!(
            new_pos,
            state.active_workspace_clients().get(&window).unwrap().pos
        );
        assert_eq!(new_pos, pos);
    }

    #[test]
    fn test_drag_client_not_found() {
        let mut state = State::default();
        let window = unsafe { x::Window::new(123) };

        let result = state.drag_client(window, Vector2D::new(10, 10));

        assert!(matches!(result, Err(Error::ClientNotFound)));
    }

    #[test]
    fn test_drag_resize_client() {
        let mut state = State::default();
        let window = unsafe { x::Window::new(123) };
        let pos = Vector2D::new(0, 0);
        let size = Vector2D::new(100, 100);

        state.add_client(window, pos, size).unwrap();

        let new_size = Vector2D::new(50, 50);
        let size = state.drag_resize_client(window, new_size).unwrap();

        assert_eq!(
            new_size,
            state.active_workspace_clients().get(&window).unwrap().size
        );
        assert_eq!(new_size, size);
    }

    #[test]
    fn test_drag_resize_client_min_value() {
        let mut state = State::default();
        let window = unsafe { x::Window::new(123) };
        let pos = Vector2D::new(0, 0);
        let size = Vector2D::new(100, 100);

        state.add_client(window, pos, size).unwrap();

        let size = state
            .drag_resize_client(window, Vector2D::new(0, 0))
            .unwrap();

        assert_eq!(size, MIN_CLIENT_SIZE);
    }

    #[test]
    fn test_drag_resize_client_not_found() {
        let mut state = State::default();
        let window = unsafe { x::Window::new(123) };

        let result = state.drag_resize_client(window, Vector2D::new(50, 50));

        assert!(matches!(result, Err(Error::ClientNotFound)));
    }

    #[test]
    fn test_teleport_client() {
        let mut state = State::default();
        let window = unsafe { x::Window::new(123) };
        let pos = Vector2D::new(0, 0);
        let size = Vector2D::new(100, 100);

        state.add_client(window, pos, size).unwrap();

        let new_pos = Vector2D::new(10, 10);
        state.teleport_client(window, new_pos).unwrap();

        assert_eq!(
            new_pos,
            state.active_workspace_clients().get(&window).unwrap().pos
        );
    }

    #[test]
    fn test_teleport_client_not_found() {
        let mut state = State::default();
        let window = unsafe { x::Window::new(123) };

        let result = state.teleport_client(window, Vector2D::new(10, 10));

        assert!(matches!(result, Err(Error::ClientNotFound)));
    }

    #[test]
    fn test_focus_client() {
        let mut state = State {
            root: unsafe { x::Window::new(0) },
            ..Default::default()
        };
        let window = unsafe { x::Window::new(123) };
        let pos = Vector2D::new(0, 0);
        let size = Vector2D::new(100, 100);

        state.add_client(window, pos, size).unwrap();

        state
            .focus_client(WindowSelector::Window(window.resource_id()))
            .unwrap();

        assert_eq!(state.focused, Some(window));

        state
            .focus_client(WindowSelector::Window(state.root.resource_id()))
            .unwrap();

        assert_eq!(state.focused, None);
        assert_eq!(state.last_focused, Some(window));
    }

    #[test]
    fn test_select_client_window_selector_focused() {
        let mut state = State::default();
        let window = unsafe { x::Window::new(123) };
        state
            .add_client(window, Vector2D::new(0, 0), Vector2D::new(100, 100))
            .unwrap();
        state.set_focused(Some(window));

        let client = state.select_client(WindowSelector::Focused).unwrap();

        assert_eq!(window, client.window);
    }

    #[test]
    fn test_select_client_window_selector_closest() {
        let mut state = State::default();
        let window_ne = unsafe { x::Window::new(1) };
        let window_nw = unsafe { x::Window::new(2) };
        let window_sw = unsafe { x::Window::new(3) };
        let window_se = unsafe { x::Window::new(4) };

        state
            .add_client(window_nw, Vector2D::new(0, 0), Vector2D::new(100, 100))
            .unwrap();

        state
            .add_client(window_ne, Vector2D::new(150, 0), Vector2D::new(100, 100))
            .unwrap();

        state
            .add_client(window_sw, Vector2D::new(0, 150), Vector2D::new(100, 100))
            .unwrap();

        state
            .add_client(window_se, Vector2D::new(150, 150), Vector2D::new(100, 100))
            .unwrap();

        state.set_focused(Some(window_ne));
        let client = state
            .select_client(WindowSelector::Closest(CardinalDirection::South))
            .unwrap();
        assert_eq!(window_se, client.window);

        state.set_focused(Some(window_se));
        let client = state
            .select_client(WindowSelector::Closest(CardinalDirection::West))
            .unwrap();
        assert_eq!(window_sw, client.window);

        state.set_focused(Some(window_sw));
        let client = state
            .select_client(WindowSelector::Closest(CardinalDirection::North))
            .unwrap();
        assert_eq!(window_nw, client.window);
    }
}
