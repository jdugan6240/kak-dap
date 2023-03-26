import sys

from utils.installer_utils import check_call


def install():
    try:
        check_call(['gem', 'install', 'readapt'])
    except:
        print("Install failed. Perhaps Ruby or RubyGems isn't installed?")


def uninstall():
    try:
        check_call(['gem', 'uninstall', 'readapt'])
    except:
        print("Uninstall failed. Perhaps readapt was installed with a different method?")


# If installing, there will be no command line arguments.
# If uninstalling, there will be command line arguments, and we don't care what.
if len(sys.argv) > 1:
    uninstall()
else:
    install()

