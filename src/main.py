import argparse
import debug_session
from kakoune import KakConnection
import logging
import os
import sys
import xdg


class StreamToLogger:
    """
    Fake file-like stream object that redirects writes to a logger instance.
    """

    def __init__(self, logger, log_level=logging.INFO):
        self.logger = logger
        self.log_level = log_level
        self.linebuf = ''

    def write(self, buf):
        for line in buf.rstrip().splitlines():
            self.logger.log(self.log_level, line.rstrip())

    def flush(self):
        pass


parser = argparse.ArgumentParser(description='DAP Client for Kakoune')
parser.add_argument(
    '-s',
    '--session',
    required=True,
    help='Kakoune session to communicate with',
)
parser.add_argument(
    '-l', '--log', help='Write log to file', action='store_true'
)
parser.add_argument(
    '-v', '--verbosity', help='increase logging verbosity', action='count'
)
parser.add_argument(
    '-r',
    '--request',
    help='Send stdin as request to kak-dap server',
    action='store_true',
)

args = parser.parse_args()

session = args.session

# If the --request argument is present, grab stdin
# and send it to the kak-dap FIFO
if args.request:
    fifo_path = KakConnection._get_in_fifo_path(session)
    fifo_write = open(fifo_path, 'w')
    input_str = sys.stdin.read()
    fifo_write.write(input_str)
    fifo_write.flush()
    sys.exit(0)

# Determine log level
verbosity = 2
if args.verbosity > 0:
    verbosity = args.verbosity
log_level = logging.CRITICAL
if verbosity == 2:
    log_level = logging.ERROR
elif verbosity == 3:
    log_level = logging.WARNING
elif verbosity == 4:
    log_level = logging.INFO
elif verbosity == 5:
    log_level = logging.DEBUG

# If the log flag is set, write log to a file
if args.log:
    # Create directory where logfiles go, if it doesn't exist already
    logfile_path = xdg.xdg_data_home() / 'kak-dap'
    if not logfile_path.exists():
        logfile_path.mkdir()

    logfile = logfile_path.as_posix() + '/kak-dap.log'

    # If the file exists, delete it.
    # We don't want many sessions worth of logs.
    if os.path.exists(logfile):
        os.remove(logfile)

    logging.basicConfig(
        format='%(levelname)s@%(filename)s:%(lineno)d - %(message)s',
        filename=logfile,
        level=log_level,
        filemode='w',
    )


# Otherwise, log to the terminal
else:
    logging.basicConfig(
        format='%(levelname)s@%(filename)s:%(lineno)d - %(message)s',
        level=log_level,
    )


# Setup stderr to redirect to log
stderr_log = logging.getLogger('stderr')
sys.stderr = StreamToLogger(stderr_log, logging.ERROR)

logging.info(f'Starting kak-dap server for session {session}')
logging.info(f'CWD: {os.getcwd()}')

debug_session.start(session)
