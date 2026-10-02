#!/bin/bash
# Toggle AcerControl: if running, bring to front; if not, launch it.

if [ "$EUID" -eq 0 ]; then
    TARGET_USER=$(loginctl list-sessions --no-legend 2>/dev/null | awk '$3 != "root" && $3 != "gdm" { print $3; exit }')
    if [ -z "$TARGET_USER" ]; then
        TARGET_USER=$(awk -F: '$3 >= 1000 && $3 < 60000 && $1 != "nobody" { print $1; exit }' /etc/passwd)
    fi
    USER_ID=$(id -u "$TARGET_USER")
    
    DISPLAY=""
    XAUTHORITY=""
    WAYLAND_DISPLAY=""
    
    while IFS= read -r pid; do
        if [ -r "/proc/$pid/environ" ]; then
            ENV_DISPLAY=$(tr '\0' '\n' < "/proc/$pid/environ" 2>/dev/null | grep '^DISPLAY=' | cut -d= -f2- || true)
            ENV_WAYLAND=$(tr '\0' '\n' < "/proc/$pid/environ" 2>/dev/null | grep '^WAYLAND_DISPLAY=' | cut -d= -f2- || true)
            ENV_XAUTH=$(tr '\0' '\n' < "/proc/$pid/environ" 2>/dev/null | grep '^XAUTHORITY=' | cut -d= -f2- || true)
            
            [ -n "$ENV_DISPLAY" ] && DISPLAY="$ENV_DISPLAY"
            [ -n "$ENV_WAYLAND" ] && WAYLAND_DISPLAY="$ENV_WAYLAND"
            [ -n "$ENV_XAUTH" ] && [ -f "$ENV_XAUTH" ] && XAUTHORITY="$ENV_XAUTH"
            
            if [ -n "$DISPLAY" ] || [ -n "$WAYLAND_DISPLAY" ]; then
                break
            fi
        fi
    done < <(pgrep -u "$TARGET_USER" || true)

    if [ -z "$XAUTHORITY" ]; then
        for candidate in "/run/user/$USER_ID/gdm/Xauthority" "/home/$TARGET_USER/.Xauthority" /run/user/$USER_ID/.mutter-Xwaylandauth.*; do
            if [ -f "$candidate" ]; then
                XAUTHORITY="$candidate"
                break
            fi
        done
    fi

    : "${DISPLAY:=:0}"
    : "${WAYLAND_DISPLAY:=wayland-0}"

    exec sudo -u "$TARGET_USER" env \
        HOME="/home/$TARGET_USER" \
        USER="$TARGET_USER" \
        LOGNAME="$TARGET_USER" \
        DISPLAY="$DISPLAY" \
        WAYLAND_DISPLAY="$WAYLAND_DISPLAY" \
        XAUTHORITY="$XAUTHORITY" \
        XDG_RUNTIME_DIR="/run/user/$USER_ID" \
        DBUS_SESSION_BUS_ADDRESS="unix:path=/run/user/$USER_ID/bus" \
        "$0" "$@"
fi

BINARY="/usr/local/bin/acercontrol-gui"

PID=$(pgrep -x "acercontrol-gui" | head -1)

if [ -n "$PID" ]; then
    WINDOW_ID=$(kdotool search --name "AcerControl" 2>/dev/null | head -1)
    if [ -n "$WINDOW_ID" ]; then
        kdotool windowactivate "$WINDOW_ID" 2>/dev/null
    else
        wmctrl -a "AcerControl" 2>/dev/null
    fi
else
    # Need to run it completely detached from the daemon's process group
    nohup "$BINARY" >/dev/null 2>&1 &
fi
