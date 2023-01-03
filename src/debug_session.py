from adapter import Adapter
import breakpoints
import config
from kakoune import KakConnection
import logging
import threading
import time

adapter_config = None  # The global adapter configuration
debug_adapter = None  # The connection to the debug adapter
kak_connection = None  # The connection to the Kakoune session
project_config = None  # The current project configuration
selected_config = None  # The selected adapter configuration


def quit(session):
    logging.debug('Quitting the session...')
    kak_connection.send_cmd(
        'try %{ eval -client %opt{jumpclient} %{ dap-reset-location  }}'
    )
    kak_connection.send_cmd(
        'try %{ eval -client %opt{jumpclient} %{ dap-takedown-ui }}'
    )
    time.sleep(0.1)  # Leave time for things to settle
    kak_connection.send_cmd('set-option global dap_running false')
    time.sleep(0.1)  # Again, leave time before killing the socket
    kak_connection.cleanup()
    exit(0)


def start_adapter():
    global debug_adapter, selected_config
    # For the time being, we're taking the first project configuration
    names = []
    for name in project_config['configurations']:
        names.append(name)

    # TODO: send menu request to Kakoune with configuration names,
    # and choose the name the user chooses, instead of just going
    # with the first one.
    selected_config = project_config['configurations'][names[0]]

    selected_adapter = selected_config['adapter']

    # Now that we have the selected adapter,
    # find the matching adapter config and start the adapter
    selected_adapter_cfg = adapter_config['adapters'][selected_adapter]

    if selected_adapter_cfg is None:
        logging.error(f'Invalid adapter in project config: {selected_adapter}')
        quit(kak_connection.session)

    logging.debug(f'Selected adapter config: {selected_adapter_cfg}')

    adapter_exec = selected_adapter_cfg['executable']
    adapter_args = selected_adapter_cfg['args']
    # debug_adapter = Adapter(adapter_exec, adapter_args)


def adapter_msg_thread():
    msg = debug_adapter.get_msg()
    while msg is not None:
        # Handle message
        # Get next message
        msg = debug_adapter.get_msg()


def start(session):
    global debug_adapter, kak_connection
    global adapter_config, project_config

    # We need to give some time for the variables and stacktrace clients
    # to be created
    time.sleep(0.5)

    logging.debug('Hello')

    kak_connection = KakConnection(session)

    # Get configurations
    adapter_config = config.get_adapter_config()
    if adapter_config is None:
        quit(session)
    project_config = config.get_project_config()
    if project_config is None:
        quit(session)

    start_adapter()

    # Begin receiving adapter messages
    # msg_thread = threading.Thread(target=adapter_msg_thread())
    # msg_thread.start()

    # Set dap_running flag in Kakoune; process breakpoints; initialize adapter
    kak_connection.send_cmd('set-option global dap_running true')
    breakpoints.process_breakpoints()
