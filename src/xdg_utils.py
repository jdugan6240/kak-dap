import os


def xdg_config_home():
    if (value := os.getenv("XDG_CONFIG_HOME")) and os.path.isabs(value):
        return os.path.expanduser(value)
    return os.path.expanduser("~/.config")


def xdg_runtime_dir():
    if (value := os.getenv("XDG_RUNTIME_DIR")) and os.path.isabs(value):
        return value
    return None
