use anyhow::{anyhow, Context, Result};
use crossbeam::channel;
use std::path::PathBuf;
use std::process;
use std::{sync::Arc, thread};
use xcb::{x, Xid};

use crate::atoms::Atoms;
use crate::commands::{Command, WorkspaceSelector};
use crate::config::Config;
use crate::state::State;
use crate::vector::Vector2D;
use crate::{ewmh, icccm};

pub struct WindowManager {
    state: State,
    conn: Arc<xcb::Connection>,
    atoms: Atoms,
    client_receiver: channel::Receiver<Command>,
    screen_num: i32,
    config: Config,
}

impl WindowManager {
    pub fn new(
        conn: xcb::Connection,
        screen_num: i32,
        client_receiver: channel::Receiver<Command>,
        config: Config,
    ) -> WindowManager {
        let conn = Arc::new(conn);
        let atoms = Atoms::intern_all(&conn).unwrap();

        WindowManager {
            state: State::default(),
            conn,
            atoms,
            client_receiver,
            screen_num,
            config,
        }
    }

    pub fn run(&mut self, autostart_file_path: PathBuf) -> Result<()> {
        let conn = Arc::clone(&self.conn);
        let setup = conn.get_setup();
        // TODO handle no screen?
        let screen = setup.roots().nth(self.screen_num as usize).unwrap();
        self.state.root = screen.root();

        if self.become_window_manager().is_err() {
            return Err(anyhow!("Another window manager is running."));
        }

        ewmh::set_supported(&conn, &self.atoms, screen.root());

        // Create a child window for EWMH compliance
        // See: https://specifications.freedesktop.org/wm-spec/wm-spec-1.3.html
        self.state.child = conn.generate_id();
        self.conn.send_request(&x::CreateWindow {
            depth: 0,
            wid: self.state.child,
            parent: self.state.root,
            x: 0,
            y: 0,
            width: 1,
            height: 1,
            border_width: 0,
            class: x::WindowClass::InputOnly,
            visual: 0,
            value_list: &[],
        });

        ewmh::set_wm_name(&conn, &self.atoms, self.state.child, "toniowm");
        ewmh::set_supporting_wm_check(&conn, &self.atoms, self.state.root, self.state.child);
        ewmh::set_active_window(&conn, &self.atoms, self.state.root, self.state.child);
        ewmh::set_current_desktop(&conn, &self.atoms, self.state.root, 0);

        process::Command::new(&autostart_file_path)
            .spawn()
            .with_context(|| "Failed to load toniorc")?;

        self.refresh_workspaces();

        conn.flush()?;

        // Spawn XCB event thread
        let (sender, receiver) = crossbeam::channel::unbounded();
        let conn = Arc::clone(&self.conn);
        thread::spawn(move || loop {
            // TODO: handle error, maybe just log?
            let event = conn.wait_for_event().unwrap();
            println!("Received event: {:?}", event);
            match event {
                xcb::Event::X(event) => sender.send(event).unwrap(),
                xcb::Event::Unknown(_) => {}
            };
        });

        loop {
            channel::select! {
                recv(receiver) -> event => match event.unwrap() {
                    x::Event::ButtonPress(ev) => {
                        self.handle_button_press_event(ev)?;
                    }
                    x::Event::MotionNotify(ev) => {
                        self.handle_motion_notify_event(ev)?;
                    }
                    x::Event::ConfigureRequest(ev) => {
                        self.handle_configure_request_event(ev)?;
                    }
                    x::Event::MapRequest(ev) => {
                        self.handle_map_request_event(ev)?;
                    },
                    x::Event::DestroyNotify(ev) => {
                        self.handle_destroy_notify_event(ev);
                    }
                    x::Event::ClientMessage(ev) => {
                        // This event is sent if a pager wants to switch ti antoher workspace.
                        if ev.r#type().resource_id() == self.atoms.net_current_desktop.resource_id() {
                            if let x::ClientMessageData::Data32([index, ..]) = ev.data() {
                                self.activate_workspace(WorkspaceSelector::Index(index as usize))?;
                            }
                        }
                    }
                    ev => {
                        println!("Unhandled event: {:?}", ev);
                    }
                },
                recv(self.client_receiver) -> message => match message.unwrap() {
                    Command::Quit => {
                        println!("Quitting");
                        break;
                    }
                    Command::FocusClosest{ selector, direction} => {
                        match self.state.focus_closest_client(selector, direction) {
                            Ok(window) => {
                                if let Some(window) = window {
                                    self.focus_window(window)?;
                                };
                            }
                            Err(e) => {
                                println!("Error: {:?}", e);
                            }
                        }
                    }
                    Command::Close{ selector } => {
                        match self.state.select_client(selector) {
                            Ok(client) => {
                                self.delete_window(client.window())?;
                            }
                            // TODO: return error in result channel
                            _ => {
                                println!("Client not found");
                            }
                        }
                    }
                    Command::AddWorkspace{ name } => {
                        self.state.add_workspace(name)?;
                        self.refresh_workspaces();
                    }
                    Command::RenameWorkspace{ selector, name } => {
                        self.state.rename_workspace(selector, name)?;
                        self.refresh_workspaces();
                    }
                    Command::SelectWorkspace{ selector } => {
                        self.activate_workspace(selector)?;
                    }
                    Command::SetBorderWidth{ width } => {
                        self.config.border_width = width;
                        for (window, _) in self.state.active_workspace_clients().iter() {
                            self.conn.send_request(&x::ConfigureWindow {
                                window: *window,
                                value_list: &[x::ConfigWindow::BorderWidth(self.config.border_width)],
                            });
                        }
                    }
                    Command::SetBorderColor{ color } => {
                        self.config.border_color = color;
                        for (window, _) in self.state.active_workspace_clients().iter() {
                            if Some(*window) == self.state.focused() {
                                continue;
                            }

                            self.conn.send_request(&x::ChangeWindowAttributes {
                                window: *window,
                                value_list: &[
                                    x::Cw::BorderPixel(self.config.border_color),
                                ],
                            });
                        }
                    }
                    Command::SetFocusedBorderColor{ color } => {
                        self.config.focused_border_color = color;
                        if let Some(window) = self.state.focused() {
                            self.conn.send_request(&x::ChangeWindowAttributes {
                                window,
                                value_list: &[x::Cw::BorderPixel(self.config.focused_border_color)],
                            });
                        }
                    }
                }
            }

            self.conn.flush()?;
        }
        Ok(())
    }

    /// Become the window manager.
    /// This is done by changing the root window's event mask.
    ///
    /// If another window manager is already running, this will fail.
    fn become_window_manager(&self) -> Result<()> {
        self.conn
            .send_and_check_request(&x::ChangeWindowAttributes {
                window: self.state.root,
                value_list: &[
                    x::Cw::EventMask(
                        x::EventMask::SUBSTRUCTURE_NOTIFY
                            | x::EventMask::SUBSTRUCTURE_REDIRECT
                            | x::EventMask::BUTTON_PRESS
                            | x::EventMask::BUTTON_RELEASE,
                    ),
                    x::Cw::Cursor(Xid::none()),
                ],
            })?;

        Ok(())
    }

    /// This is called when a new window is created.
    fn handle_map_request_event(&mut self, ev: x::MapRequestEvent) -> Result<()> {
        // Map the window
        self.conn.send_request(&x::MapWindow {
            window: ev.window(),
        });

        if ewmh::get_wm_window_type(&self.conn, &self.atoms, ev.window())?
            .contains(&self.atoms.net_wm_window_type_dock)
        {
            // Do not manage dock windows
            return Ok(());
        }

        // Ask the X server for the window's geometry
        let cookie = self.conn.send_request(&x::GetGeometry {
            drawable: x::Drawable::Window(ev.window()),
        });
        let reply = self.conn.wait_for_reply(cookie)?;

        // Add the window to the state
        let pos = Vector2D::new(reply.x().into(), reply.y().into());
        let size = Vector2D::new(reply.width().into(), reply.height().into());
        self.state.add_client(ev.window(), pos, size)?;

        // Set border width
        self.conn.send_request(&x::ConfigureWindow {
            window: ev.window(),
            value_list: &[x::ConfigWindow::BorderWidth(self.config.border_width)],
        });

        // Set border color and event mask
        self.conn.send_request(&x::ChangeWindowAttributes {
            window: ev.window(),
            value_list: &[
                x::Cw::BorderPixel(self.config.border_color),
                x::Cw::EventMask(
                    x::EventMask::SUBSTRUCTURE_NOTIFY | x::EventMask::SUBSTRUCTURE_REDIRECT,
                ),
            ],
        });

        self.conn.send_request(&x::ChangeSaveSet {
            mode: x::SetMode::Insert,
            window: ev.window(),
        });

        // Reparent the window
        self.conn.send_request(&x::ReparentWindow {
            window: ev.window(),
            parent: self.state.root,
            x: 0,
            y: 0,
        });

        // Focus the window
        self.conn.send_request(&x::SetInputFocus {
            revert_to: x::InputFocus::PointerRoot,
            focus: ev.window(),
            time: x::CURRENT_TIME,
        });

        // Add button grab settings
        self.conn.send_request(&x::GrabButton {
            owner_events: true,
            grab_window: ev.window(),
            event_mask: x::EventMask::BUTTON_PRESS | x::EventMask::BUTTON_RELEASE,
            pointer_mode: x::GrabMode::Async,
            keyboard_mode: x::GrabMode::Async,
            confine_to: xcb::Xid::none(),
            cursor: xcb::Xid::none(),
            button: crate::config::SELECT_BUTTON,
            modifiers: crate::config::MOD_KEY,
        });

        // Allow events
        self.conn.send_request(&x::AllowEvents {
            mode: x::Allow::AsyncPointer,
            time: x::CURRENT_TIME,
        });

        // Drag settings
        self.conn.send_request(&x::GrabButton {
            owner_events: false,
            grab_window: ev.window(),
            event_mask: x::EventMask::BUTTON_PRESS
                | x::EventMask::BUTTON_RELEASE
                | x::EventMask::BUTTON_MOTION,
            pointer_mode: x::GrabMode::Async,
            keyboard_mode: x::GrabMode::Async,
            confine_to: xcb::Xid::none(),
            cursor: xcb::Xid::none(),
            button: crate::config::DRAG_BUTTON,
            modifiers: crate::config::MOD_KEY,
        });

        // Resize settings
        self.conn.send_request(&x::GrabButton {
            owner_events: false,
            grab_window: ev.window(),
            event_mask: x::EventMask::BUTTON_PRESS
                | x::EventMask::BUTTON_RELEASE
                | x::EventMask::BUTTON_MOTION,
            pointer_mode: x::GrabMode::Async,
            keyboard_mode: x::GrabMode::Async,
            confine_to: xcb::Xid::none(),
            cursor: xcb::Xid::none(),
            button: crate::config::RESIZE_BUTTON,
            modifiers: crate::config::MOD_KEY,
        });

        Ok(())
    }

    fn handle_button_press_event(&mut self, ev: x::ButtonPressEvent) -> Result<()> {
        let cookie = self.conn.send_request(&x::GetGeometry {
            drawable: x::Drawable::Window(ev.event()),
        });

        let resp = self.conn.wait_for_reply(cookie)?;

        self.state.drag_start_pos = Vector2D::new(ev.root_x().into(), ev.root_y().into());
        self.state.drag_start_frame_pos = Vector2D::new(resp.x().into(), resp.y().into());

        if ev.detail() == x::ButtonIndex::N1 as u8 {
            self.state.focus_client(ev.event())?;
            self.focus_window(ev.event())?;
        }

        Ok(())
    }

    fn handle_motion_notify_event(&mut self, ev: x::MotionNotifyEvent) -> Result<()> {
        let mouse_pos = Vector2D::new(ev.root_x().into(), ev.root_y().into());
        if !ev.state().contains(crate::config::MOD_KEY_BUT) {
            return Ok(());
        }

        if ev.state().contains(crate::config::DRAG_BUTTON_MASK) {
            let new_pos = self.state.drag_client(ev.event(), mouse_pos)?;

            self.conn.send_request(&x::ConfigureWindow {
                window: ev.event(),
                value_list: &[x::ConfigWindow::X(new_pos.x), x::ConfigWindow::Y(new_pos.y)],
            });
        } else if ev.state().contains(crate::config::RESIZE_BUTTON_MASK) {
            let new_size = self.state.drag_resize_client(ev.event(), mouse_pos)?;
            self.conn.send_request(&x::ConfigureWindow {
                window: ev.event(),
                value_list: &[
                    x::ConfigWindow::Width(new_size.x as u32),
                    x::ConfigWindow::Height(new_size.y as u32),
                ],
            });
        }

        Ok(())
    }

    fn handle_configure_request_event(&self, ev: x::ConfigureRequestEvent) -> Result<()> {
        // Do not manage dock windows
        if !ewmh::get_wm_window_type(&self.conn, &self.atoms, ev.window())?
            .contains(&self.atoms.net_wm_window_type_dock)
        {
            self.conn.send_request(&x::ConfigureWindow {
                window: ev.window(),
                value_list: &[
                    x::ConfigWindow::X(ev.x() as i32),
                    x::ConfigWindow::Y(ev.y() as i32),
                    x::ConfigWindow::Width(ev.width() as u32),
                    x::ConfigWindow::Height(ev.height() as u32),
                    x::ConfigWindow::BorderWidth(self.config.border_width),
                    x::ConfigWindow::StackMode(ev.stack_mode()),
                ],
            });
        }

        Ok(())
    }

    fn handle_destroy_notify_event(&mut self, ev: x::DestroyNotifyEvent) {
        if let Err(err) = self.state.remove_client(ev.window()) {
            println!("Failed to remove client: {}", err);
        }
    }

    fn focus_window(&mut self, window: x::Window) -> Result<()> {
        // Unfocus last focused window
        if let Some(last_focused) = self.state.last_focused() {
            self.conn.send_request(&x::ChangeWindowAttributes {
                window: last_focused,
                value_list: &[x::Cw::BorderPixel(self.config.border_color)],
            });
        }

        // Set the input focus
        self.conn.send_request(&x::SetInputFocus {
            revert_to: x::InputFocus::PointerRoot,
            focus: window,
            time: x::CURRENT_TIME,
        });

        // Select and focus
        self.conn.send_request(&x::ChangeWindowAttributes {
            window,
            value_list: &[x::Cw::BorderPixel(self.config.focused_border_color)],
        });

        self.conn.send_request(&x::SetInputFocus {
            revert_to: x::InputFocus::PointerRoot,
            focus: window,
            time: x::CURRENT_TIME,
        });

        // Raise the window above the others
        self.conn.send_request(&x::ConfigureWindow {
            window,
            value_list: &[x::ConfigWindow::StackMode(x::StackMode::Above)],
        });

        // Set the EWMH hint
        ewmh::set_active_window(&self.conn, &self.atoms, self.state.root, window);
        Ok(())
    }

    fn delete_window(&self, window: x::Window) -> Result<()> {
        // Check if the window supports the delete protocol
        // If it doesnt, just kill it
        let wm_protocols = icccm::get_wm_protocols(&self.conn, &self.atoms, window)?;
        if wm_protocols.contains(&self.atoms.wm_delete_window) {
            icccm::send_wm_delete_window(&self.conn, &self.atoms, window)?;
        } else {
            self.conn.send_request(&x::KillClient {
                resource: window.resource_id(),
            });
        }

        Ok(())
    }

    fn activate_workspace(&mut self, selector: WorkspaceSelector) -> Result<()> {
        // Unmap all windows on the current workspace
        for (window, _) in self.state.active_workspace_clients().iter() {
            self.conn.send_request(&x::UnmapWindow { window: *window });
        }

        let workspace_index = self.state.activate_workspace(selector)?;
        ewmh::set_current_desktop(
            &self.conn,
            &self.atoms,
            self.state.root,
            workspace_index as u32,
        );

        // Map all windows on the new workspace
        for (window, _) in self.state.active_workspace_clients().iter() {
            self.conn.send_request(&x::MapWindow { window: *window });
        }

        Ok(())
    }

    fn refresh_workspaces(&self) {
        ewmh::set_number_of_desktops(
            &self.conn,
            &self.atoms,
            self.state.root,
            self.state.workspaces_names().len() as u32,
        );

        ewmh::set_desktop_names(
            &self.conn,
            &self.atoms,
            self.state.root,
            self.state.workspaces_names(),
        );
    }
}
