use indexmap::{map::MutableKeys, IndexMap};
use thiserror::Error;
use xcb::{x, Xid, XidNew};

use crate::{
    commands::{Direction, WindowSelector, WorkspaceSelector},
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

#[derive(Clone, Debug, PartialEq, Default)]
pub struct Workspace {
    /// The name of the workspace
    name: String,
    /// The list of clients managed by the workspace
    clients: IndexMap<x::Window, Client>,
}

impl Workspace {
    pub fn name(&self) -> String {
        self.name.to_owned()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
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
        };

        state.add_workspace(None).unwrap();

        state
    }
}

impl State {
    /// Add a workspace to the state.
    ///
    /// If no name is provided, the workspace will be named after the index.
    /// The name of the workspace must be unique.
    pub fn add_workspace(&mut self, name: Option<String>) -> Result<Workspace, Error> {
        let name = if let Some(name) = name {
            name
        } else {
            self.workspaces.len().to_string()
        };

        if self.workspaces.contains_key(&name) {
            Err(Error::WorkspaceAlreadyExists)
        } else {
            let workspace = Workspace {
                name: name.clone(),
                clients: IndexMap::new(),
            };

            self.workspaces.insert(name, workspace.clone());
            Ok(workspace)
        }
    }

    pub fn rename_workspace(
        &mut self,
        selector: WorkspaceSelector,
        name: String,
    ) -> Result<(), Error> {
        let (old_name, workspace) = match selector {
            WorkspaceSelector::Index(index) => {
                if let Some((old_name, workspace)) = self.workspaces.get_index_mut(index) {
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

        *old_name = name.clone();
        workspace.name = name;

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

    /// Return a list of the workspaces
    pub fn workspaces(&self) -> Vec<Workspace> {
        self.workspaces.values().cloned().collect()
    }

    /// Add a client to the state.
    ///
    /// Return an error if the client already exists.
    pub fn add_client(
        &mut self,
        window: x::Window,
        pos: Vector2D,
        size: Vector2D,
    ) -> Result<Client, Error> {
        if self.active_workspace_clients().contains_key(&window) {
            Err(Error::ClientAlreadyExists)
        } else {
            let client = Client { window, pos, size };
            self.active_workspace_clients_mut().insert(window, client);

            Ok(client)
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

    /// Drag a client.
    ///
    /// Return an error if the client is not found.
    pub fn drag_client(&mut self, window: x::Window, mouse_pos: Vector2D) -> Result<Client, Error> {
        let new_pos = self.drag_start_frame_pos + mouse_pos - self.drag_start_pos;
        if let Some(client) = self.active_workspace_clients_mut().get_mut(&window) {
            client.pos = new_pos;
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
        if let Some(client) = self.active_workspace_clients_mut().get_mut(&window) {
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
        if let Some(client) = self.active_workspace_clients_mut().get_mut(&window) {
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

        if self.active_workspace_clients().contains_key(&window) {
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
        selector: WindowSelector,
        direction: Direction,
    ) -> Result<Option<Client>, Error> {
        let client = self.select_client(selector)?;

        let mut distance: i32;
        let mut min_distance = std::i32::MAX;
        let mut closest_client = None;

        for (_, c) in self.active_workspace_clients().clone() {
            if c.window == client.window {
                continue; // Skip the focused window
            }
            let dx = c.pos.x - client.pos.x;
            let dy = c.pos.y - client.pos.y;
            // Euclidean distance approximation
            // We do not need to calculate the square root to compare distances.
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
    pub fn select_client(&self, selector: WindowSelector) -> Result<Client, Error> {
        let window = match selector {
            WindowSelector::Focused => {
                if let Some(window) = self.focused {
                    window
                } else {
                    return Err(Error::ClientNotFound);
                }
            }
            WindowSelector::Window(window) => unsafe { x::Window::new(window) },
        };

        match self.active_workspace_clients().get(&window).cloned() {
            Some(client) => Ok(client),
            None => Err(Error::ClientNotFound),
        }
    }

    /// Set the focused window.
    /// Save the last focused window.
    fn set_focused(&mut self, window: Option<x::Window>) {
        self.last_focused = self.focused;
        self.focused = window;
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
    fn workspaces() {
        let mut state = State::default();
        let workspace_2 = state.add_workspace(Some("2".to_owned())).unwrap();
        let workspace_3 = state.add_workspace(Some("3".to_owned())).unwrap();

        let workspaces = state.workspaces();

        assert_eq!(workspaces.len(), 3);
        assert_eq!(workspaces[1], workspace_2);
        assert_eq!(workspaces[2], workspace_3);
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

        let client = state.add_client(window, pos, size).unwrap();

        assert_eq!(client.window, window);
        assert_eq!(client.pos, pos);
        assert_eq!(client.size, size);

        assert_eq!(
            &indexmap::indexmap! {window => client},
            state.active_workspace_clients()
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
        state.focus_client(window).unwrap();

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
        let client = state.drag_client(window, new_pos).unwrap();

        assert_eq!(
            state
                .active_workspace_clients()
                .get(&window)
                .unwrap()
                .clone(),
            client
        );
        assert_eq!(client.pos, new_pos);
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
        let client = state.drag_resize_client(window, new_size).unwrap();

        assert_eq!(
            state
                .active_workspace_clients()
                .get(&window)
                .unwrap()
                .clone(),
            client
        );
        assert_eq!(client.size, new_size);
    }

    #[test]
    fn test_drag_resize_client_min_value() {
        let mut state = State::default();
        let window = unsafe { x::Window::new(123) };
        let pos = Vector2D::new(0, 0);
        let size = Vector2D::new(100, 100);

        state.add_client(window, pos, size).unwrap();

        let client = state
            .drag_resize_client(window, Vector2D::new(0, 0))
            .unwrap();

        assert_eq!(client.size(), MIN_CLIENT_SIZE);
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
        let client = state.teleport_client(window, new_pos).unwrap();

        assert_eq!(
            state
                .active_workspace_clients()
                .get(&window)
                .unwrap()
                .clone(),
            client
        );
        assert_eq!(client.pos, new_pos);
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

        let result = state.focus_client(window);

        assert!(matches!(result, Ok(())));
        assert_eq!(state.focused, Some(window));

        state.focus_client(state.root).unwrap();
        assert_eq!(state.focused, None);
        assert_eq!(state.last_focused, Some(window));
    }

    #[test]
    fn test_focus_client_not_found() {
        let mut state = State::default();
        let window = unsafe { x::Window::new(123) };

        let result = state.focus_client(window);

        assert!(matches!(result, Err(Error::ClientNotFound)));
    }

    #[test]
    fn test_focus_closest_client() {
        let mut state = State::default();
        let window_ne = unsafe { x::Window::new(1) };
        let window_nw = unsafe { x::Window::new(2) };
        let window_se = unsafe { x::Window::new(3) };
        let window_sw = unsafe { x::Window::new(4) };

        let client_ne = state
            .add_client(window_ne, Vector2D::new(0, 0), Vector2D::new(100, 100))
            .unwrap();

        let client_nw = state
            .add_client(window_nw, Vector2D::new(150, 0), Vector2D::new(100, 100))
            .unwrap();

        let client_se = state
            .add_client(window_se, Vector2D::new(0, 150), Vector2D::new(100, 100))
            .unwrap();

        let client_sw = state
            .add_client(window_sw, Vector2D::new(150, 150), Vector2D::new(100, 100))
            .unwrap();

        let client = state
            .focus_closest_client(
                WindowSelector::Window(window_ne.resource_id()),
                Direction::East,
            )
            .unwrap();

        assert_eq!(Some(client_nw), client);
        assert_eq!(state.focused, Some(client_nw.window));

        let client = state
            .focus_closest_client(WindowSelector::Focused, Direction::South)
            .unwrap();

        assert_eq!(Some(client_sw), client);
        assert_eq!(state.focused, Some(client_sw.window));

        let client = state
            .focus_closest_client(
                WindowSelector::Window(window_sw.resource_id()),
                Direction::West,
            )
            .unwrap();

        assert_eq!(Some(client_se), client);
        assert_eq!(state.focused, Some(client_se.window));

        let client = state
            .focus_closest_client(WindowSelector::Focused, Direction::North)
            .unwrap();

        assert_eq!(Some(client_ne), client);
        assert_eq!(state.focused, Some(client_ne.window));
    }

    #[test]
    fn test_focus_closest_client_not_found() {
        let window = unsafe { x::Window::new(123) };
        let mut state = State::default();

        let result = state.focus_closest_client(
            WindowSelector::Window(window.resource_id()),
            Direction::East,
        );

        assert!(matches!(result, Err(Error::ClientNotFound)));
    }

    #[test]
    fn test_set_focused() {
        let old_focused = unsafe { x::Window::new(123) };
        let mut state = State {
            focused: Some(old_focused),
            ..Default::default()
        };

        let new_focused = unsafe { x::Window::new(456) };
        state.set_focused(Some(new_focused));

        assert_eq!(state.focused, Some(new_focused));
        assert_eq!(state.last_focused, Some(old_focused));

        state.set_focused(None);
        assert_eq!(state.focused, None);
        assert_eq!(state.last_focused, Some(new_focused));
    }

    #[test]
    fn test_select_client() {
        let mut state = State::default();
        let window = unsafe { x::Window::new(123) };
        let pos = Vector2D::new(0, 0);
        let size = Vector2D::new(100, 100);

        let expected_client = state.add_client(window, pos, size).unwrap();

        let client = state
            .select_client(WindowSelector::Window(window.resource_id()))
            .unwrap();
        assert_eq!(client, expected_client);

        state.focus_client(window).unwrap();

        let client = state.select_client(WindowSelector::Focused).unwrap();
        assert_eq!(client, expected_client);
    }

    #[test]
    fn test_select_client_not_found() {
        let state = State::default();
        let window = unsafe { x::Window::new(123) };

        let result = state.select_client(WindowSelector::Window(window.resource_id()));
        assert!(matches!(result, Err(Error::ClientNotFound)));

        let result = state.select_client(WindowSelector::Focused);
        assert!(matches!(result, Err(Error::ClientNotFound)));
    }
}
