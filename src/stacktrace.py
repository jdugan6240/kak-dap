import debug_session
import logging

cur_thread = -1
threads = []


def handle_stopped_event(msg):
    global cur_thread
    # Annoyingly, the stopped thread id is an optional value
    # in the stopped event.
    # So, as a substitute, once we request threads and retrieve
    # the results, the first thread's id will be used as the
    # "current" thread id.
    if "threadId" in msg["body"]:
        cur_thread = msg["body"]["threadId"]

    # Request threads
    debug_session.debug_adapter.write_request(
        "threads", {}, handle_threads_response
    )


def handle_threads_response(msg):
    global threads, cur_thread
    threads = msg["body"]["threads"]

    logging.debug(f"Threads: {threads}")

    # If the server didn't provide a thread id on the stopped event,
    # use the first thread as a backup
    if cur_thread == -1:
        cur_thread = threads[0]["id"]

    # Request stack trace for current thread
    stack_trace_args = {"threadId": cur_thread}
    debug_session.debug_adapter.write_request(
        "stackTrace", stack_trace_args, handle_stack_trace_response
    )


def handle_stack_trace_response(msg):
    frames = msg["body"]["stackFrames"]

    # Get first stack frame to obtain current execution location
    frame = frames[0]
    line = frame["line"]
    # Each stack frame isn't guaranteed to have a source, so show an
    # unknown path in that case. However, if it does, figure out the
    # file path.
    # Rant: technically, all sources from the debug adapter are required
    # to have a name. However, some debug adapters (debugpy cough cough)
    # decide it's OK to ignore the protocol and omit this value. In that
    # case, our best bet is to use the source's path, if we even can, since
    # it's a value that the DAP spec makes optional. A bad situation all round.
    jump_file = "<unknown>"
    if "source" in frame:
        if "name" in frame["source"]:
            jump_file = frame["source"]["name"]
        elif "path" in frame["source"]:
            jump_file = frame["source"]["path"]

    # Construct Kakoune command to jump to location
    cmd = f"dap-stack-trace {line} {jump_file} '"
    # Add contents to push to stacktrace buffer
    for thread in threads:
        cmd += f"Thread: {thread['name']}\n"
        if thread["id"] == cur_thread:
            for stack_frame in frames:
                frame_id = stack_frame["id"]
                frame_name = stack_frame["name"]
                frame_line = stack_frame["line"]
                # Each stack frame is not guaranteed to have a source
                # If it does - well, see rant above.
                source = {"name": "<unknown>"}
                if "source" in stack_frame:
                    source = stack_frame["source"]
                source_name = ""
                if "name" in source:
                    source_name = source["name"]
                elif "path" in source:
                    source_name = source["path"]
                else:
                    source_name = "<unknown>"
                cmd += f"{frame_id}: {frame_name}@{source_name}:{frame_line}\n"
    cmd += "'"
    debug_session.kak_connection.send_cmd(cmd)

    # TODO: Send a "Scopes" command to begin filling out the variable heirarchy
    # of the current stack frame.
