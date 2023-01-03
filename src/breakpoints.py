import logging
import os

breakpoint_data = {}  # filepath: line_nos


def process_breakpoints():
    global breakpoint_data
    # Retrieve breakpoints
    breakpoints_info = os.environ('kak_opt_dap_breakpoints_info')
    logging.debug(f'Breakpoint string: {breakpoints_info}')
    split_breaks = breakpoints_info.split()
    # Parse breakpoints
    for val in split_breaks:
        break_vals = val.split('|')
        line_no = break_vals[0]
        filepath = break_vals[1]
        logging.debug(f'Breakpoint filepath: {filepath}, line no: {line_no}')
        # If we already have breakpoints under this filepath, just add another
        if filepath in breakpoint_data.keys():
            breakpoint_data[filepath].append(line_no)
        # Otherwise, add a new entry for this filepath
        else:
            breakpoint_data[filepath] = [line_no]
