#!/usr/bin/env bash
# `env bash` rather than `/bin/bash`, matching every other script here — see the
# note at the top of check-i18n-escapes.sh for what that mismatch cost.
# Load rustconn/assets/style.css through the installed GTK and fail on any
# complaint the parser makes.
#
# WHY THIS EXISTS
# ---------------
# `install_glib_css_warning_filter` in rustconn/src/main.rs drops every GLib
# message containing "Theme parser" or "gtk.css". It was added for a real problem:
# a libadwaita stylesheet newer than the GTK parser reading it produced a flood of
# harmless warnings. But the predicate matches on wording, not on origin, so it also
# silenced complaints about *our own* stylesheet — and four real errors lived there
# unnoticed:
#
#     line 506  No property named "margin-start"
#     line 507  No property named "margin-end"
#     line 838  No property named "margin-start"
#     line 839  No property named "margin-end"
#
# GTK's CSS has no logical margin properties. Those four declarations parsed as
# unknown and were discarded, so the monitoring bar's horizontal margins had never
# applied in either the normal or the compact layout. Nothing failed; the app just
# quietly ignored them.
#
# A runtime filter cannot be narrowed reliably — the message text is all there is to
# match on — so the durable fix is to check the stylesheet where no filter is in the
# way and no display is needed. `RUSTCONN_CSS_WARNINGS=1` disables the filter for
# interactive debugging; this script is the gate.
#
# WHAT IT DOES NOT CATCH
# ----------------------
# Only what the parser rejects: unknown properties, bad syntax, malformed values. A
# selector that parses but matches nothing, or a property that is valid on some other
# node, still passes. Deprecated-but-accepted syntax also passes, deliberately —
# GTK 4.22 takes `@named_color` and `alpha()` without complaint, so this script has
# no opinion on them either.
#
# Usage: ./scripts/check-css.sh        (exit 0 = clean, 1 = parser complaints)

set -euo pipefail

cd "$(dirname "$0")/.."

STYLESHEET="rustconn/assets/style.css"

if [ ! -f "$STYLESHEET" ]; then
    echo "error: $STYLESHEET not found" >&2
    exit 1
fi

if ! python3 -c 'import gi' 2>/dev/null; then
    echo "SKIP: python3 gi bindings not available; cannot validate $STYLESHEET" >&2
    echo "      install python3-gi (Debian/Ubuntu) or python3-gobject (Fedora)" >&2
    exit 0
fi

STYLESHEET="$STYLESHEET" python3 - <<'PY'
import os
import sys

import gi

gi.require_version("Gtk", "4.0")
from gi.repository import Gtk  # noqa: E402

path = os.environ["STYLESHEET"]
complaints: list[str] = []


def on_parsing_error(_provider, section, error):
    start = section.get_start_location()
    end = section.get_end_location()
    complaints.append(
        f"  {path}:{start.lines + 1}:{start.line_chars}"
        f"-{end.lines + 1}:{end.line_chars}: {error.message}"
    )


# No Gtk.init(): loading a provider and parsing it needs no display, which is what
# makes this usable in CI.
provider = Gtk.CssProvider()
provider.connect("parsing-error", on_parsing_error)
provider.load_from_path(path)

gtk_version = f"{Gtk.get_major_version()}.{Gtk.get_minor_version()}.{Gtk.get_micro_version()}"

if complaints:
    print(f"FAIL: GTK {gtk_version} rejected {len(complaints)} declaration(s):", file=sys.stderr)
    for line in complaints:
        print(line, file=sys.stderr)
    print(file=sys.stderr)
    print(
        "These are discarded at runtime and the app's log filter hides them.\n"
        "Run with RUSTCONN_CSS_WARNINGS=1 to see them in a live session.",
        file=sys.stderr,
    )
    sys.exit(1)

print(f"OK: {path} parses cleanly under GTK {gtk_version}")
PY
