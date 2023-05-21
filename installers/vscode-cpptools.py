import os
import shutil
import sys

from utils.installer_utils import (
    adapter_install_dir,
    download_file,
    extract_zip_to,
    get_platform,
    make_executable,
)


def get_correct_url():
    version = "1.11.5"

    file_map = {
        ("Linux", "X86_64"): "cpptools-linux.vsix",
        ("Linux", "ARM_8_64bit"): "cpptools-linux-aarch64.vsix",
        ("Linux", "ARM_7"): "cpptools-linux-armhf.vsix",
        ("Darwin", "X86_64"): "cpptools-osx.vsix",
        ("Darwin", "ARM_8_64bit"): "cpptools-osx-arm64.vsix",
    }

    os, arch = get_platform()

    filename = file_map[(os, arch)]

    return f"https://github.com/Microsoft/vscode-cpptools/releases/download/v{version}/{filename}"


def install():
    url = get_correct_url()
    install_dir = adapter_install_dir()

    path = install_dir + "/vscode-cpptools.vsix"
    dest = install_dir + "/vscode-cpptools"

    print("Downloading vsix...")
    download_file(url, path)

    print("Extracting vsix...")
    extract_zip_to(path, dest, "zip")

    print("Removing vsix...")
    os.remove(path)

    # For some reason, the execute bits for the adapter itself aren't set. So they
    # actually have Typescript code that does it in the extension. Let's mimic that.
    for binary in [
        "OpenDebugAD7",
        "OpenDebugAD7.exe",
        "debugserver",
        "lldb-mi",
        "lldb-argdumper",
        "lldb-launcher",
    ]:
        path = f"{dest}/extension/debugAdapters/bin/{binary}"
        if os.path.exists(path):
            make_executable(path)

    mac_exclusive_path = f"{dest}/extension/debugAdapters/lldb-mi/bin/lldb-mi"
    if os.path.exists(mac_exclusive_path):
        make_executable(mac_exclusive_path)


def uninstall():
    print("Removing vscode-cpptools...")

    install_dir = adapter_install_dir()
    path = install_dir + "/vscode-cpptools"

    shutil.rmtree(path, ignore_errors=True)


# If installing, there will be no command line arguments.
# If uninstalling, there will be command line arguments, and we don't care what.
if len(sys.argv) > 1:
    uninstall()
else:
    install()
