if cargo build; then
    XEPHYR=$(whereis -b Xephyr | cut -f2 -d' ')
    xinit ./hack/xinitrc -- \
        "$XEPHYR" \
            :1 \
            -ac \
            -screen 1920x1080 \
            -host-cursor
fi
