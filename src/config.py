import json
import logging
import os
from pathlib import Path
import pwd
import xdg_utils as xdg

import vendor.json_minify as json_minify


def validate_project_config(config):
    # We need the top-level key to be "configurations"
    if "configurations" not in config.keys():
        logging.error(".kak-dap.json must have 'configurations' table")
        return False

    for project_key in config["configurations"]:
        project = config["configurations"][project_key]
        # Every configuration must have an adapter and a set of launch arguments.
        if "adapter" not in project.keys():
            logging.error(
                f"{project} config in .kak-dap.json must have an adapter")
            return False
        if "launch_args" not in project.keys():
            logging.error(
                f"{project} config in .kak-dap.json must have launch arguments")
            return False

        # Adapter must be a string
        if not isinstance(project["adapter"], str):
            logging.error(f"adapter in {project_key} config must be a string")
            return False

    return True


def validate_adapter_config(config):
    # We need the top-level key to be "adapters"
    if "adapters" not in config.keys():
        logging.error("adapters.json must have 'adapters' table")
        return False

    for adapter_key in config["adapters"]:
        adapter = config["adapters"][adapter_key]
        # Every adapter must have an executable and a list of arguments.
        if "executable" not in adapter.keys():
            logging.error(
                f"{adapter_key} adapter in adapters.json must have an executable")
            return False
        if "args" not in adapter.keys():
            logging.error(
                f"{adapter_key} adapter in adapters.json must have a list of args")
            return False

        # Executable must be a string
        if not isinstance(adapter["executable"], str):
            logging.error(f"executable in {adapter_key} adapter must be a string")
            return False

        # Args must be a list of strings
        if not isinstance(adapter["args"], list):
            logging.error(f"args in {adapter_key} adapter must be a list of strings")
            return False

    return True


def get_adapter_config():
    # Get the config path, if it exists (otherwise use the file we ship with)
    config_home = xdg.xdg_config_home()
    config_path = Path(config_home + "/kak-dap/adapters.json")
    if not config_path.exists():
        current_dir = Path(__file__).parent.resolve()
        config_path = current_dir / Path("../adapters.json")

    logging.debug(f"Found adapter config at {config_path}")
    config_data = config_path.read_text()
    config_data = json_minify.minify(config_data)

    # Perform any substitutions
    config_data = config_data.replace("${HOME}", os.getenv("HOME"))
    config_data = config_data.replace(
        "${USER}", pwd.getpwuid(os.getuid()).pw_name)
    config_data = config_data.replace("${CUR_DIR}", os.getcwd())
    config_data = config_data.replace(
        "${ADAPTER_DIR}", os.path.expanduser("~/.kak-dap/adapters")
    )
    config_data = config_data.replace("$$", "$")

    # Parse json and attempt to validate adapter config
    try:
        config = json.loads(config_data)
    except json.JSONDecodeError as e:
        logging.error(f"Error validating adapter config: {e}")
        return None

    if not validate_adapter_config(config):
        return None

    logging.debug(f"Adapter config: {config}")
    return config


def get_project_config():
    cur_path = Path(os.getcwd())
    # Ensure we find a .kak-dap.json file somewhere
    cur_file = cur_path / ".kak-dap.json"
    logging.debug(f"Checking for {cur_file}")
    while not cur_file.exists() and not cur_path.parent == cur_path:
        cur_path = cur_path.parent
        cur_file = cur_path / ".kak-dap.json"
        logging.debug(f"Checking for {cur_file}")

    # If we've reached the filesystem root, the file is nowhere to be seen.
    if cur_path.parent == cur_path:
        logging.error("Couldn't find .kak-dap.json file")
        return None

    logging.debug(f"Found project config at {cur_file}")
    config_data = cur_file.read_text()
    config_data = json_minify.minify(config_data)
    logging.debug(f"config data: {config_data}")

    # Perform any substitutions
    config_data = config_data.replace("${HOME}", os.getenv("HOME"))
    config_data = config_data.replace(
        "${USER}", pwd.getpwuid(os.getuid()).pw_name)
    config_data = config_data.replace("${CUR_DIR}", os.getcwd())
    config_data = config_data.replace(
        "${ADAPTER_DIR}", os.path.expanduser("~/.kak-dap/adapters")
    )
    config_data = config_data.replace("$$", "$")

    # Parse json and attempt to validate against project schema
    try:
        config = json.loads(config_data)
    except json.JSONDecodeError as e:
        logging.error(f"Error validating project config: {e}")
        return None

    logging.debug(f"Config: {config}")
    if not validate_project_config(config):
        return None

    logging.debug(f"Project config: {config}")
    return config
