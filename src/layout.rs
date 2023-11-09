use indexmap::IndexMap;
use xcb::x;

use crate::{state::Client, vector::Vector2D};

pub struct VerticalSplitLayout {
    pub monitor_size: Vector2D,
    pub top_gap: i32,
    pub bottom_gap: i32,
    pub left_gap: i32,
    pub right_gap: i32,
    pub window_gap: i32,
}

impl VerticalSplitLayout {
    pub fn apply_layout(self, clients: &mut IndexMap<x::Window, Client>) {
        let clients_number = clients.len() as i32;

        let x = self.monitor_size.x
            - self.right_gap
            - self.left_gap
            - self.window_gap * (clients_number - 1);
        let y = self.monitor_size.y - self.top_gap - self.bottom_gap;

        for (index, (_, client)) in clients.iter_mut().enumerate() {
            let size = Vector2D::new(x / clients_number, y);

            let pos = Vector2D::new(
                self.left_gap + size.x * index as i32 + index as i32 * self.window_gap,
                self.top_gap,
            );

            client.size = size;
            client.pos = pos;
        }
    }
}
