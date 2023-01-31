import logging
import os
from pathlib import Path
from schema import Optional, Schema, SchemaError
import xdg
import yaml

project_schema = Schema(
    {'configurations': {str: {'adapter': str, 'launch_args': {str: object}}}}
)

adapter_schema = Schema(
    {'adapters': {str: {'executable': str, 'args': [str]}}}
)


def get_adapter_config():
    # Get the config path, if it exists (otherwise use the file we ship with)
    config_home = xdg.xdg_config_home().as_posix()
    config_path = Path(config_home + '/kak-dap/adapters.yaml')
    if not config_path.exists():
        current_dir = Path(__file__).parent.resolve()
        config_path = current_dir / Path('../adapters.yaml')

    logging.debug(f'Found adapter config at {config_path}')
    config_data = config_path.read_text()

    # Perform any substitutions
    config_data = config_data.replace('${HOME}', os.getenv('HOME'))
    config_data = config_data.replace('${USER}', os.getlogin())
    config_data = config_data.replace('${CUR_DIR}', os.getcwd())
    config_data = config_data.replace('$$', '$')

    # Parse yaml and attempt to validate against adapter schema
    try:
        config = yaml.safe_load(config_data)
    except yaml.YAMLError as e:
        logging.error(f'Error validating adapter config: {e}')
        return None

    try:
        adapter_schema.validate(config)
    except SchemaError as e:
        logging.error(f'Error validating adapter config: {e}')
        return None

    logging.debug(f'Adapter config: {config}')
    return config


def get_project_config():
    cur_path = Path(os.getcwd())
    # Ensure we find a .kak-dap.yaml file somewhere
    cur_file = cur_path / '.kak-dap.yaml'
    logging.debug(f'Checking for {cur_file}')
    while not cur_file.exists() and not cur_path.parent == cur_path:
        cur_path = cur_path.parent
        cur_file = cur_path / '.kak-dap.yaml'
        logging.debug(f'Checking for {cur_file}')

    # If we've reached the filesystem root, the file is nowhere to be seen.
    if cur_path.parent == cur_path:
        logging.error("Couldn't find .kak-dap.yaml file")
        return None

    logging.debug(f'Found project config at {cur_file}')
    config_data = cur_file.read_text()
    logging.debug(f'config data: {config_data}')

    # Perform any substitutions
    config_data = config_data.replace('${HOME}', os.getenv('HOME'))
    config_data = config_data.replace('${USER}', os.getlogin())
    config_data = config_data.replace('${CUR_DIR}', os.getcwd())
    config_data = config_data.replace('$$', '$')

    # Parse yaml and attempt to validate against project schema
    try:
        config = yaml.safe_load(config_data)
    except yaml.YamlError as e:
        logging.error(f'Error validating project config: {e}')
        return None

    logging.debug(f'Config: {config}')
    try:
        project_schema.validate(config)
    except SchemaError as e:
        logging.error(f'Error validating project config: {e}')
        return None

    logging.debug(f'Project config: {config}')
    return config
