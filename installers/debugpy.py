import os
import shutil
import sys

from utils.installer_utils import (
    adapter_install_dir,
    check_call,
    download_file,
    extract_zip_to,
)


def get_correct_url():
    version = "1.6.7"
    return f"https://github.com/microsoft/debugpy/archive/v{version}.zip"


def install():
    url = get_correct_url()
    install_dir = adapter_install_dir()

    path = install_dir + "/debugpy.zip"
    dest = install_dir + "/debugpy"

    print("Downloading zip...")
    download_file(url, path)

    print("Extracting zip...")
    extract_zip_to(path, dest, "zip")

    print("Removing zip...")
    os.remove(path)

    wd = os.getcwd()
    os.chdir(dest + "/debugpy-1.6.7")
    try:
        check_call([sys.executable, "setup.py", "build", "--build-platlib", "build/lib"])
    finally:
        os.chdir(wd)


def uninstall():
    print("Removing debugpy...")

    install_dir = adapter_install_dir()
    path = install_dir + "/debugpy"

    shutil.rmtree(path, ignore_errors=True)


# If installing, there will be no command line arguments.
# If uninstalling, there will be command line arguments, and we don't care what.
if len(sys.argv) > 1:
    uninstall()
else:
    install()
