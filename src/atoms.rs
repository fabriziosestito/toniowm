use xcb::atoms_struct;

atoms_struct! {
    pub struct Atoms {
        pub wm_protocols  => b"WM_PROTOCOLS" only_if_exists = false,
        pub wm_delete_window  => b"WM_DELETE_WINDOW" only_if_exists = false,
        /// Supported EWMH hints
        pub net_supported  => b"_NET_SUPPORTED" only_if_exists = false,
        pub net_active_window  => b"_NET_ACTIVE_WINDOW" only_if_exists = false,
        pub net_supporting_wm_check  => b"_NET_SUPPORTING_WM_CHECK" only_if_exists = false,
        pub net_wm_name  => b"_NET_WM_NAME" only_if_exists = false,
        pub net_wm_state  => b"_NET_WM_STATE" only_if_exists = false,
        pub net_wm_state_fullscreen  => b"_NET_WM_STATE_FULLSCREEN" only_if_exists = false,
    }
}
