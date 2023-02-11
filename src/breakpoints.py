import debug_session
import general
import logging
import os

breakpoint_data = {}  # filepath: line_nos


def process_breakpoints():
    global breakpoint_data
    # First ensure we have breakpoints to retrieve
    if "kak_opt_dap_breakpoints_info" not in os.environ:
        return
    # Retrieve breakpoints
    breakpoints_info = os.environ["kak_opt_dap_breakpoints_info"]
    logging.debug(f"Breakpoint string: {breakpoints_info}")
    split_breaks = breakpoints_info.split()
    # Parse breakpoints
    for val in split_breaks:
        break_vals = val.split("|")
        line_no = break_vals[0]
        filepath = break_vals[1]
        logging.debug(f"Breakpoint filepath: {filepath}, line no: {line_no}")
        # If we already have breakpoints under this filepath, just add another
        if filepath in breakpoint_data.keys():
            breakpoint_data[filepath].append(line_no)
        # Otherwise, add a new entry for this filepath
        else:
            breakpoint_data[filepath] = [line_no]


def handle_initialized_event(msg):
    # We need to send breakpoint requests after the initialized event
    # For background: https://github.com/microsoft/vscode/issues/4902
    requests = []
    for source in breakpoint_data:
        breakpoints = []
        lines = breakpoint_data[source]
        for line in lines:
            breakpoints.append({"line": int(line)})
        break_args = {"source": {"path": source}, "breakpoints": breakpoints}
        requests.append(break_args)
    for req in requests:
        debug_session.debug_adapter.write_request(
            "setBreakpoints", req, lambda *args: None
        )

    # Now send the configurationDone request.
    # We only do this if the adapter has advertised the "configurationDone"
    # capability.
    if (
        general.capabilities is not None
        and "supportsConfigurationDoneRequest" in general.capabilities
        and general.capabilities["supportsConfigurationDoneRequest"]
    ):
        debug_session.debug_adapter.write_request(
            "configurationDone", {}, lambda *args: None
        )
