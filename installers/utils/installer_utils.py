import os
import subprocess


def check_call(cmd, *args, **kwargs):
    subprocess.check_call(cmd, *args, **kwargs)


def adapter_install_dir():
    # Adapter install dir is ~/.kak-dap/adapters/
    return os.path.expanduser("~/.kak-dap/adapters")
