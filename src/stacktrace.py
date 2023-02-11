import debug_session
import logging

cur_thread = 0
threads = []

def handle_stopped_event(msg):
    global cur_thread
    cur_thread = msg['body']['threadId']

    # request threads
    debug_session.debug_adapter.write_request('threads', {}, handle_threads_response)


def handle_threads_response(msg):
    global threads
    threads = msg['body']['threads']

    logging.debug(f'Threads: {threads}')

    # TODO: Retrieve the scack trace for the current thread.
