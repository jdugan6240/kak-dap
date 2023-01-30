import logging
from multiprocessing import Process, Queue
from subprocess import Popen, PIPE

# Try to import ultrajson for performance
try:
    import ujson as json
except Exception:
    import json


def reader_thread(rfile, q):
    data = b''
    state = 'HEADER'
    content_length = 0
    body_length = 0

    # While the input stream is still alive
    while not rfile.closed:
        data += rfile.read(1)
        # If we are reading the header
        if state == 'HEADER':
            # Ensure the data string ends in \r\n\r\n
            if data.endswith(b'\r\n\r\n'):
                # Grab the content length
                state = 'BODY'
                headers = data.split(b'\r\n\r\n')[0]
                _, header = headers.split(b'Content-Length:')
                content_length = int(header.strip())
                data = b''
        else:
            # We are reading the message body
            body_length += 1
            # If we have read the whole message, parse it and reset
            if body_length == content_length:
                msg_str = data.decode('utf-8')
                logging.debug(f'From debug adapter: {msg_str}')
                msg = json.loads(msg_str)
                q.put(msg)
                state = 'HEADER'
                body_length = 0
                data = b''


def stderr_thread(rfile, q):
    while rfile.closed:
        line = rfile.readline()
        logging.error(f'From debug adapter: {line}')


class AdapterOutput:
    def __init__(self, rfile):
        self._rfile = rfile
        self._msg_queue = Queue()
        self._process = Process(
            target=reader_thread, args=(self._rfile, self._msg_queue)
        )

    def get_msg(self):
        return self._msg_queue.get()


class AdapterInput:
    def __init__(self, wfile):
        self._wfile = wfile

    def write_msg(self, msg):
        if self._wfile.closed:
            return
        try:
            # Construct message string with header
            body = json.dumps(msg)
            content_length = (
                len(body)
                if isinstance(body, bytes)
                else len(body.encode('utf-8'))
            )

            msg = 'Content-Length: {}\r\n\r\n' '{}'.format(
                content_length, body
            )
            logging.debug(f'Writing: {msg}')
            # Write to the process' stdout
            self._wfile.write(msg.encode('utf-8'))
            self._wfile.flush()
        except Exception as e:
            logging.error(f'Failed to write {msg} message to adapter')
            logging.error(f'Reason: {e}')


class Adapter(object):
    def __init__(self, binary, args):
        """
        Creates an Adapter object with the given executable and arguments.
        """

        # Spawn the adapter process
        proc_args = [binary]
        for i in range(len(args)):
            proc_args.append(args[i])
        self._adapter_process = Popen(
            proc_args, stdin=PIPE, stdout=PIPE, stderr=PIPE
        )

        self.next_req_id = 0

        self._handlers = {}

        self._adapter_output = AdapterOutput(self._adapter_process.stdout)
        self._adapter_output._process.start()
        self._adapter_input = AdapterInput(self._adapter_process.stdin)
        # self._stderr_process = Process(
        #    target=stderr_thread, args=(self._adapter_process.stderr)
        # )

    def write_request(self, cmd, args, callback):
        msg = {
            'type': 'request',
            'seq': self.next_req_id,
            'command': cmd,
            'arguments': args,
        }
        logging.debug(f'To debug adapter: {msg}')
        self._adapter_input.write_msg(msg)
        self._handlers[self.next_req_id] = callback
        self.next_req_id += 1

    def get_msg(self):
        return self._adapter_output.get_msg()

    def get_callback(self, req_id):
        return self._handlers.pop(req_id)
