import sys

from utils.installer_utils import check_call


def install():
    try:
        check_call([sys.executable, '-m', 'pip', 'install', 'debugpy'])
    except:
        print("Install failed. Perhaps pip isn't installed?")


def uninstall():
    try:
        check_call([sys.executable, '-m', 'pip', 'uninstall', 'debugpy', '-y'])
    except:
        print("Uninstall failed. Perhaps debugpy was installed with a different method?")


# If installing, there will be no command line arguments.
# If uninstalling, there will be command line arguments, and we don't care what.
if len(sys.argv) > 1:
    uninstall()
else:
    install()
