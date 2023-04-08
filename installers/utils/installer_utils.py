import contextlib
import pathlib
import platform
import os
import re
import subprocess
import tarfile
from urllib import request
import zipfile


# Python's ZipFile module ignores execute bits, so
# we need to create a subclass that doesn't.
class FixedZipFile(zipfile.ZipFile):
    def extract(self, member, path=None, pwd=None):
        # Ensure we have the zipinfo object we need
        # This contains the execute bits we must preserve
        if not isinstance(member, zipfile.ZipInfo):
            member = self.getinfo(member)

        if path is None:
            path = os.getcwd()

        # Extract the file
        ret_val = self._extract_member(member, path, pwd)

        # Now put the execute bits back
        attr = member.external_attr >> 16
        os.chmod(ret_val, attr)

        return attr


def adapter_install_dir():
    # Adapter install dir is ~/.kak-dap/adapters/
    dir = os.path.expanduser("~/.kak-dap/adapters")
    pathlib.Path(dir).mkdir(parents=True, exist_ok=True)
    return dir


def check_call(cmd, *args, **kwargs):
    subprocess.check_call(cmd, *args, **kwargs)


def download_file(url, dest):
    # First ensure destination directory exists
    dest_dir = os.path.dirname(dest)
    pathlib.Path(dest_dir).mkdir(parents=True, exist_ok=True)

    # Remove any existing file
    if os.path.exists(dest):
        print(f"Removing existing file at {dest}")
        os.remove(dest)

    # Download file
    r = request.Request(url, headers={"User-Agent": "kak-dap"})
    with contextlib.closing(request.urlopen(r)) as u:
        with open(dest, "wb") as f:
            f.write(u.read())


def extract_zip_to(filepath, dest, format):
    if os.path.exists(dest):
        print(f"Removing existing file at {dest}")
        os.remove(dest)
    print(f"Extracting {filepath} to {dest}...")

    if format == "tar":
        try:
            with tarfile.open(filepath) as f:
                f.extractall(path=dest)
        except Exception:
            # This can fail on Windows, but we don't care about Windows.
            # So ignore this exception that should never happen.
            pass
    elif format == "zip":
        with FixedZipFile(filepath) as f:
            f.extractall(path=dest)


def get_platform():
    # Since Kakoune only runs on linux and osx,
    # we only support linux and osx.
    os = platform.system()
    machine = platform.machine()

    # Determine the architecture being run on
    # We only support X86 and ARM
    arch = None
    if re.match(r"^i\d86$|^x86$|^x86_32$|^i86pc$|^ia32$|^ia-32$|^bepc$", machine):
        arch = "X86_32"
    elif re.match(
        r"^x64$|^x86_64$|^x86_64t$|^i686-64$|^amd64$|^ia64$|^ia-64$", machine
    ):
        arch = "X86_64"
    elif re.match(r"^armv8-a|aarch64|arm64$", machine):
        arch = "ARM_8_64bit"
    elif re.match(r"^armv7$|^armv7[a-z]$|^armv7-[a-z]$|^armv6[a-z]$", machine):
        arch = "ARM_7"
    elif re.match(r"^armv8$|^armv8[a-z]$|^armv8-[a-z]$", machine):
        arch = "ARM_8_32bit"

    return (os, arch)


def make_executable(path):
    print(f"Making {path} executable")
    os.chmod(path, 0o755)
