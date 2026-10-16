import os


def window_env(real_window, **extra):
    env = dict(os.environ, **extra)

    if real_window:
        env["DWELLER_BACKGROUND"] = "1"
    else:
        env["SDL_VIDEODRIVER"] = "dummy"

    return env
