use anyhow::{anyhow, Result};
use crossbeam::channel;
use std::{sync::Arc, thread};
use xcb::{x, Xid};

use crate::commands::Command;
use crate::state::State;
use crate::vector::Vector2D;

pub struct WindowManager {
    state: State,
    conn: Arc<xcb::Connection>,
    client_receiver: channel::Receiver<Command>,
    screen_num: usize,
}

impl WindowManager {
    pub fn new(
        conn: xcb::Connection,
        client_receiver: channel::Receiver<Command>,
        screen_num: usize,
    ) -> WindowManager {
        let conn = Arc::new(conn);

        WindowManager {
            state: State::default(),
            conn,
            client_receiver,
            screen_num,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        let conn = Arc::clone(&self.conn);
        let setup = conn.get_setup();

        // TODO handle no screen?
        let screen = setup.roots().nth(self.screen_num).unwrap();
        self.state.root_window = screen.root();

        if self.become_window_manager().is_err() {
            return Err(anyhow!("Another window manager is running."));
        }

        // Spawn XCB event thread
        let (sender, receiver) = crossbeam::channel::unbounded();
        let conn = Arc::clone(&self.conn);
        thread::spawn(move || loop {
            let event = conn.wait_for_event().unwrap();
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
                    }
                   _ev => {
                        // TODO: logging
                    }
                },
                recv(self.client_receiver) -> message => match message.unwrap() {
                    Command::Quit => {
                        println!("Quitting");
                        break;
                    }
                    Command::FocusClosest{ direction } => {
                        // TODO add selector
                        // it could be focused window(default), or window by id
                        let window  = match self.state.focused_window() {
                            Some(window) => window,
                            None => continue,
                        };

                        if let Some(closest_client) = self.state.closest_client(window, direction) {
                            self.focus_window(closest_client.window())?;
                            println!("Focus closest left, window: {:?}", closest_client.window());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn become_window_manager(&self) -> Result<()> {
        let cookie = self.conn.send_request_checked(&x::ChangeWindowAttributes {
            window: self.state.root_window,
            value_list: &[
                x::Cw::EventMask(
                    x::EventMask::SUBSTRUCTURE_NOTIFY
                        | x::EventMask::SUBSTRUCTURE_REDIRECT
                        | x::EventMask::BUTTON_PRESS
                        | x::EventMask::BUTTON_RELEASE,
                ),
                x::Cw::Cursor(Xid::none()),
            ],
        });

        self.conn.check_request(cookie)?;

        Ok(())
    }

    fn handle_map_request_event(&mut self, ev: x::MapRequestEvent) -> Result<()> {
        // Ask the X server for the window's geometry
        let cookie = self.conn.send_request(&x::GetGeometry {
            drawable: x::Drawable::Window(ev.window()),
        });
        let resp = self.conn.wait_for_reply(cookie)?;

        // Add the window to the state
        let pos = Vector2D::new(resp.x().into(), resp.y().into());
        let size = Vector2D::new(resp.width().into(), resp.height().into());
        self.state.add_client(ev.window(), pos, size)?;

        // Set border width
        let border_cookie = self.conn.send_request_checked(&x::ConfigureWindow {
            window: ev.window(),
            value_list: &[x::ConfigWindow::BorderWidth(10)],
        });
        self.conn.check_request(border_cookie)?;

        // Set border color
        let attr_cookie = self.conn.send_request_checked(&x::ChangeWindowAttributes {
            window: ev.window(),
            value_list: &[
                x::Cw::BorderPixel(123),
                x::Cw::EventMask(
                    x::EventMask::SUBSTRUCTURE_NOTIFY | x::EventMask::SUBSTRUCTURE_REDIRECT,
                ),
            ],
        });
        self.conn.check_request(attr_cookie)?;

        let save_set_cookie = self.conn.send_request_checked(&x::ChangeSaveSet {
            mode: x::SetMode::Insert,
            window: ev.window(),
        });
        self.conn.check_request(save_set_cookie)?;

        // Reparent the window
        let reparent_cookie = self.conn.send_request_checked(&x::ReparentWindow {
            window: ev.window(),
            parent: self.state.root_window,
            x: 0,
            y: 0,
        });
        self.conn.check_request(reparent_cookie)?;

        // Manage the window
        let map_cookie = self.conn.send_request_checked(&x::MapWindow {
            window: ev.window(),
        });
        self.conn.check_request(map_cookie)?;

        // Focus the window
        let focus_cookie = self.conn.send_request_checked(&x::SetInputFocus {
            revert_to: x::InputFocus::PointerRoot,
            focus: ev.window(),
            time: x::CURRENT_TIME,
        });
        self.conn.check_request(focus_cookie)?;

        // Add button grab settings
        let button_cookie = self.conn.send_request_checked(&x::GrabButton {
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
        self.conn.check_request(button_cookie)?;

        // Allow events
        let allow_events_cookie = self.conn.send_request_checked(&x::AllowEvents {
            mode: x::Allow::AsyncPointer,
            time: x::CURRENT_TIME,
        });
        self.conn.check_request(allow_events_cookie)?;

        // Drag settings
        let drag_cookie = self.conn.send_request_checked(&x::GrabButton {
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
        self.conn.check_request(drag_cookie)?;

        // Resize settings
        let resize_cookie = self.conn.send_request_checked(&x::GrabButton {
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
        self.conn.check_request(resize_cookie)?;

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
            let client = self.state.drag_client(ev.event(), mouse_pos)?;

            let cookie = self.conn.send_request_checked(&x::ConfigureWindow {
                window: ev.event(),
                value_list: &[
                    x::ConfigWindow::X(client.pos().x),
                    x::ConfigWindow::Y(client.pos().y),
                ],
            });
            self.conn.check_request(cookie)?;
        } else if ev.state().contains(crate::config::RESIZE_BUTTON_MASK) {
            let client = self.state.drag_resize_client(ev.event(), mouse_pos)?;
            let cookie = self.conn.send_request_checked(&x::ConfigureWindow {
                window: ev.event(),
                value_list: &[
                    x::ConfigWindow::Width(client.size().x as u32),
                    x::ConfigWindow::Height(client.size().y as u32),
                ],
            });
            self.conn.check_request(cookie)?;
        }

        Ok(())
    }

    fn handle_configure_request_event(&self, ev: x::ConfigureRequestEvent) -> Result<()> {
        let cookie = self.conn.send_request_checked(&x::ConfigureWindow {
            window: ev.window(),
            value_list: &[
                x::ConfigWindow::X(ev.x() as i32),
                x::ConfigWindow::Y(ev.y() as i32),
                x::ConfigWindow::Width(ev.width() as u32),
                x::ConfigWindow::Height(ev.height() as u32),
                x::ConfigWindow::BorderWidth(crate::config::BORDER_WIDTH as u32),
                x::ConfigWindow::StackMode(ev.stack_mode()),
            ],
        });
        self.conn.check_request(cookie)?;

        Ok(())
    }

    fn focus_window(&mut self, window: x::Window) -> Result<()> {
        if let Some(window) = self.state.focused_window() {
            let unselected_window_cookie =
                self.conn.send_request_checked(&x::ChangeWindowAttributes {
                    window,
                    value_list: &[x::Cw::BorderPixel(crate::config::BORDER_COLOR)],
                });
            self.conn.check_request(unselected_window_cookie)?;
        }
        self.state.focus_client(window)?;

        let selected_window_cookie = self.conn.send_request_checked(&x::ChangeWindowAttributes {
            window,
            value_list: &[x::Cw::BorderPixel(crate::config::BORDER_COLOR_FOCUS)],
        });
        self.conn.check_request(selected_window_cookie)?;

        let focus_cookie = self.conn.send_request_checked(&x::SetInputFocus {
            revert_to: x::InputFocus::PointerRoot,
            focus: window,
            time: x::CURRENT_TIME,
        });
        self.conn.check_request(focus_cookie)?;

        let above_cookie = self.conn.send_request_checked(&x::ConfigureWindow {
            window,
            value_list: &[x::ConfigWindow::StackMode(x::StackMode::Above)],
        });
        self.conn.check_request(above_cookie)?;

        Ok(())
    }
}
