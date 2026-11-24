import atexit
import os
import shutil
import tempfile


def window_env(real_window, **extra):
    env = dict(os.environ, **extra)

    if real_window:
        env["DWELLER_BACKGROUND"] = "1"
    else:
        env["SDL_VIDEODRIVER"] = "dummy"

    # A measured window never opens the person's ~/.dweller: a window without
    # --frames does (ui.run), restores its saved tabs and writes history,
    # cookies and storage there (W7's idle runs did, 2026-11-24).
    if "DWELLER_PROFILE" not in env:
        env["DWELLER_PROFILE"] = scratch_profile()

    return env


def scratch_profile():
    path = tempfile.mkdtemp(prefix="dweller-profile-")
    atexit.register(shutil.rmtree, path, True)

    return path
