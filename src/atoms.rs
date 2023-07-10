use xcb::atoms_struct;

atoms_struct! {
    pub struct Atoms {
        // For some reason xcb::x::ATOM_STRING works for some requests but not others.
        // For instance, it works for _NET_WM_NAME but not for _NET_DESKTOP_NAMES.
        // Using UTF8_STRING as a type works for both.
        pub utf8_string => b"UTF8_STRING" only_if_exists = false,
        // ICCCM hints
        pub wm_protocols  => b"WM_PROTOCOLS" only_if_exists = false,
        pub wm_delete_window  => b"WM_DELETE_WINDOW" only_if_exists = false,
        // Supported EWMH hints
        pub net_supported  => b"_NET_SUPPORTED" only_if_exists = false,
        pub net_active_window  => b"_NET_ACTIVE_WINDOW" only_if_exists = false,
        pub net_supporting_wm_check  => b"_NET_SUPPORTING_WM_CHECK" only_if_exists = false,
        pub net_wm_name  => b"_NET_WM_NAME" only_if_exists = false,
        pub net_number_of_desktops  => b"_NET_NUMBER_OF_DESKTOPS" only_if_exists = false,
        pub net_desktop_names => b"_NET_DESKTOP_NAMES" only_if_exists = false,
        pub net_current_desktop => b"_NET_CURRENT_DESKTOP" only_if_exists = false,
        // EWMH window types
        pub net_wm_window_type => b"_NET_WM_WINDOW_TYPE" only_if_exists = false,
        pub net_wm_window_type_dock => b"_NET_WM_WINDOW_TYPE_DOCK" only_if_exists = false,
    }
}
