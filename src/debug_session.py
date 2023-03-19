from adapter import Adapter
import breakpoints
import config
import general
from kakoune import KakConnection
import logging
import stacktrace
import threading
import time
import variables

adapter_config = None  # The global adapter configuration
debug_adapter = None  # The connection to the debug adapter
kak_connection = None  # The connection to the Kakoune session
project_config = None  # The current project configuration
selected_config = None  # The selected adapter configuration
current_session = None


def quit(session):
    logging.debug("Quitting the session...")
    kak_connection.send_cmd(
        "try %{ eval -client %opt{jumpclient} %{ dap-reset-location  }}"
    )
    kak_connection.send_cmd(
        "try %{ eval -client %opt{jumpclient} %{ dap-takedown-ui }}"
    )
    time.sleep(0.1)  # Leave time for things to settle
    kak_connection.send_cmd("set-option global dap_running false")
    time.sleep(0.1)  # Again, leave time before killing the socket
    kak_connection.cleanup()
    exit(0)


def start_adapter():
    global debug_adapter, selected_config
    # For the time being, we're taking the first project configuration
    names = []
    for name in project_config["configurations"]:
        names.append(name)

    # TODO: send menu request to Kakoune with configuration names,
    # and choose the name the user chooses, instead of just going
    # with the first one.
    selected_config = project_config["configurations"][names[0]]

    selected_adapter = selected_config["adapter"]

    # Now that we have the selected adapter,
    # find the matching adapter config and start the adapter
    selected_adapter_cfg = adapter_config["adapters"][selected_adapter]

    if selected_adapter_cfg is None:
        logging.error(f"Invalid adapter in project config: {selected_adapter}")
        quit(kak_connection.session)

    logging.debug(f"Selected adapter config: {selected_adapter_cfg}")

    adapter_exec = selected_adapter_cfg["executable"]
    adapter_args = selected_adapter_cfg["args"]

    debug_adapter = Adapter(adapter_exec, adapter_args)


def handle_response(msg):
    # Grab the request seq and call the correct callback
    # We do this because different messages may require
    # different handling, depending on context (for example,
    # differentiating watch expressions from other evaluate
    # commands)
    request_seq = msg["request_seq"]
    callback = debug_adapter.get_callback(request_seq)
    callback(msg)


def handle_event(msg):
    logging.debug("Received event")
    event = msg["event"]
    if event == "output":
        output_category = msg["body"]["category"]
        # Toss out telemetry events - no one likes telemetry.
        if output_category is None or output_category != "telemetry":
            # We don't have a telemetry event - send it to Kakoune.
            category_str = output_category if output_category is not None else "output"
            kak_cmd = f"dap-output {category_str} '{kak_connection.escape_str(msg['output'])}'"
            kak_connection.send_cmd(kak_cmd)
    elif event == "initialized":
        breakpoints.handle_initialized_event(msg)
    elif event == "stopped":
        stacktrace.handle_stopped_event(msg)


def handle_reverse_request(msg):
    # There's only one reverse request at the moment, runInTerminal.
    # Therefore, we simply handle that request.
    general.handle_run_in_terminal_request(msg)


def adapter_msg_thread():
    msg = debug_adapter.get_msg()
    while msg is not None:
        # Handle message properly depending on message type
        msg_type = msg["type"]
        logging.debug(f"Message type: {msg_type}")
        if msg_type == "response":
            handle_response(msg)
        elif msg_type == "event":
            handle_event(msg)
        elif msg_type == "request":
            handle_reverse_request(msg)
        # Get next message
        msg = debug_adapter.get_msg()


def handle_kak_command(cmd):
    logging.debug(f"Received command: {cmd}")
    if cmd["cmd"] == "stop":
        # We currently rely on the adapter terminating the debuggee
        # once stdio streams are closed
        quit(current_session)
    elif cmd["cmd"] == "continue":
        continue_args = {
            "threadId": stacktrace.cur_thread
        }
        debug_adapter.write_request("continue", continue_args, lambda *args: None)
    elif cmd["cmd"] == "next":
        next_args = {
            "threadId": stacktrace.cur_thread
        }
        debug_adapter.write_request("next", next_args, lambda *args: None)
    elif cmd["cmd"] == "pid":
        debug_adapter.write_response(general.last_adapter_seq)
    elif cmd["cmd"] == "stepIn":
        step_in_args = {
            "threadId": stacktrace.cur_thread
        }
        debug_adapter.write_request("stepIn", step_in_args, lambda *args: None)
    elif cmd["cmd"] == "stepOut":
        step_out_args = {
            "threadId": stacktrace.cur_thread
        }
        debug_adapter.write_request("stepOut", step_out_args, lambda *args: None)
    elif cmd["cmd"] == "evaluate":
        # TODO
        pass
    elif cmd["cmd"] == "expand":
        variables.expand_variable(int(cmd["args"]["line"]))


def start(session):
    global debug_adapter, kak_connection
    global adapter_config, project_config
    global current_session

    current_session = session

    # We need to give some time for the variables and stacktrace clients
    # to be created
    time.sleep(0.5)

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
    msg_thread = threading.Thread(target=adapter_msg_thread)
    msg_thread.start()

    # Set dap_running flag in Kakoune; process breakpoints; initialize adapter
    kak_connection.send_cmd("set-option global dap_running true")
    breakpoints.process_breakpoints()
    general.initialize_adapter()

    # Begin listening for Kakoune messages
    while kak_connection.is_open:
        msg = kak_connection.get_msg()
        handle_kak_command(msg)
