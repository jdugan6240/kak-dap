import errno
import json
import logging
import os
from pathlib import Path
from schema import Optional, Schema, SchemaError
import socket
import sys
import xdg

kak_schema = Schema({'cmd': str, Optional('args'): {str: object}})


class KakConnection:
    """
    Helper to communicate with Kakoune's remote API using Unix sockets,
    as well as receive input from the Kakoune session.
    """

    def __init__(self, session: str) -> None:
        self.session = session
        self.out_socket_path = self._get_out_socket_path(self.session)
        self.in_fifo_path = self._get_in_fifo_path(self.session)
        try:
            os.mkfifo(self.in_fifo_path)
        except OSError as e:
            if e.errno != errno.EEXIST:
                raise e
        self.is_open = True

    def get_msg(self) -> object:
        """
        Retrieves a message on the input socket.
        """
        # Retrieve the message from the FIFO
        result_str = ''
        with open(self.in_fifo_path, 'r') as fifo:
            while True:
                data = fifo.read()
                if len(data) == 0:
                    break
                result_str += data
        # Ensure message is kosher
        result_msg = json.loads(result_str)
        try:
            kak_schema.validate(result_msg)
        except SchemaError as e:
            logging.error(f'Error validating command: {e}')
            return None

        return result_msg

    def cleanup(self) -> None:
        """
        Close the input fifo and cleanup the /tmp/kak-dap dir.
        """
        os.remove(self.in_fifo_path)
        self.is_open = False

    def send_cmd(self, cmd: str) -> bool:
        """
        Send a command string to the Kakoune session. Sent data is a
        concatenation of:
           - Header
             - Magic byte indicating type is "command" (\x02)
             - Length of whole message in uint32
           - Content
             - Length of command string in uint32
             - Command string
        Return whether the communication was successful.
        """
        b_cmd = cmd.encode('utf-8')
        sock = socket.socket(socket.AF_UNIX)
        sock.connect(self.out_socket_path)
        b_content = self._encode_length(len(b_cmd)) + b_cmd
        b_header = b'\x02' + self._encode_length(len(b_content) + 5)
        b_message = b_header + b_content
        return sock.send(b_message) == len(b_message)

    @staticmethod
    def escape_str(val: str) -> str:
        """
        Replaces "'" characters with "''".
        """
        result = val.replace("'", "''")
        return result

    @staticmethod
    def _encode_length(str_length: int) -> bytes:
        """
        Convert the given string length value to bytes.
        """
        return str_length.to_bytes(4, byteorder=sys.byteorder)

    @staticmethod
    def _get_out_socket_path(session: str) -> str:
        """
        Retrieves the path for the socket to send Kakoune commands to.
        """
        # Kakoune has a socket for IPC communication in the following
        # locations, in order:
        # - if XDG_RUNTIME_DIR is defined, $XDG_RUNTIME_DIR/kakoune/<session>
        # - if TMPDIR is defined, $TMPDIR/kakoune-$USER/<session>
        # - otherwise, /tmp/kakoune-$USER/<session>.
        xdg_runtime_dir = xdg.xdg_runtime_dir()
        if xdg_runtime_dir is None:
            tmpdir = os.environ.get('TMPDIR', '/tmp')
            session_path = (
                Path(tmpdir) / f'kakoune-{os.environ["USER"]}/{session}'
            )
        else:
            session_path = xdg_runtime_dir / f'kakoune/{session}'
        return session_path.as_posix()

    @staticmethod
    def _get_in_fifo_path(session: str) -> str:
        """
        Retrieves the path for the fifo through which the Kakoune session
        gives us commands.
        """
        # According to the XDG Base Directory specification, XDG_RUNTIME_DIR
        # can be undefined. Therefore, as a backup, we use the ~/.kak-dap/
        # directory.
        fifo_path = Path.home() / '/.kak-dap'
        if xdg.xdg_runtime_dir() is not None:
            fifo_path = xdg.xdg_runtime_dir() / 'kak-dap'
        if not fifo_path.exists():
            fifo_path.mkdir()
        return fifo_path.as_posix() + f'/{session}.fifo'
