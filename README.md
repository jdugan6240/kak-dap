# Kakoune Debug Adapter Protocol Client

**NOTE: Using this plugin is not recommended. It barely functions.**

kak-dap is a [Debug Adapter Protocol](https://microsoft.github.io/debug-adapter-protocol/) client for [Kakoune](http://kakoune.org) implemented in [Rust](https://www.rust-lang.org).
This allows Kakoune to support debugging in a variety of different languages, provided the language has a debug adapter implementation.

## Features

- Starting debug adapter (currently hardcoded)
- Showing program output in external terminal
- Continue/step/next

## FAQ

Q: Does it work? 

A: Not yet. At least not in any capacity that could be considered useful.

Q: What's the point of this? kakoune-gdb and kakoune-dbgp exist. 

A: kakoune-gdb is limited to languages supported by gdb - that is, C languages and rust. 
kakoune-dbgp also only supports languages currently supported by the dbgp protocol, which
is mainly PHP at the moment as far as I know. The debug adapter protocol is much more widely 
supported, which allows for more languages to be debugged.

## License

kak-dap is "licensed" under the Unlicense.
