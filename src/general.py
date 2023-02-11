import debug_session
import logging

capabilities = None
last_adapter_seq = 0


def handle_initialize_response(msg):
    global capabilities
    capabilities = msg['body']
    if capabilities is not None:
        logging.debug(f'Adapter capabilities: {capabilities}')

    # We need to send the launch request after the initialize response.
    # For background: https://github.com/microsoft/vscode/issues/4902
    launch_args = debug_session.selected_config['launch_args']
    debug_session.debug_adapter.write_request(
        'launch', launch_args, lambda *args: None
    )


def initialize_adapter():
    # Send the intialize request
    init_args = {
        'adapterID': debug_session.selected_config['adapter'],
        'linesStartAt1': True,
        'columnsStartAt1': True,
        'pathFormat': 'path',
        'supportsRunInTerminalRequest': True,
    }
    debug_session.debug_adapter.write_request(
        'initialize', init_args, handle_initialize_response
    )


def handle_run_in_terminal_request(msg):
    global last_adapter_seq
    last_adapter_seq = int(msg['seq'])

    # Construct command to send to Kakoune
    args = msg['arguments']['args']
    cmd = 'dap-run-in-terminal '
    for arg in args:
        cmd += arg
        cmd += ' '

    debug_session.kak_connection.send_cmd(cmd)