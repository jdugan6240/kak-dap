import argparse
import debug_session
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

args = parser.parse_args()

session = args.session

# Create directory where logfiles go, if it doesn't exist already
logfile_path = xdg.xdg_data_home() / 'kak-dap'
if not logfile_path.exists():
    logfile_path.mkdir()

logfile = logfile_path.as_posix() + '/kak-dap.log'

# Delete previous logfile, if it exists.
# No need to have lots of sessions worth of logs.
logging.basicConfig(filename=logfile, level=logging.DEBUG, filemode='w')

# Setup stderr to redirect to log
stderr_log = logging.getLogger('stderr')
sys.stderr = StreamToLogger(stderr_log, logging.ERROR)

logging.info(f'Starting kak-dap server for session {session}')
logging.info(f'CWD: {os.getcwd()}')

debug_session.start(session)
