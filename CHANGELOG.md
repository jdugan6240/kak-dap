## Unreleased

Additions:
- Added repeatable -v flag to `kak-dap` binary to customize verbosity of logging
- `dap-continue` now starts debugging session if one isn't running
- Added dap-output command to show output events to the user
- Basic syntax highlighting for variables buffer
- Added `dap` user mode

Bug Fixes:
- Logging to file would occasionally produce binary (aka unreadable) output
- Expanding/collapsing variables no longer causes cursor to jump to line 1

## 1.1.0 - 2022-03-22

Additions:
- Added --request flag to `kak-dap` binary to send request to currently running `kak-dap` session
- Added --kakoune flag to `kak-dap` binary to allow for getting the `kak-lsp.kak` contents from the binary
- Vastly improved documentation (added `kak-dap` demo asciicast, troubleshooting information, etc)

Bug Fixes:
- Fixed default `dap-spawn-in-terminal` implementation sending invalid message to `kak-dap` binary

## 1.0.0 - 2022-03-05

First public release of kak-dap
