# Kakoune Debug Adapter Protocol Client

**NOTE: Using this plugin is not recommended. It barely functions.**

kak-dap is a [Debug Adapter Protocol](https://microsoft.github.io/debug-adapter-protocol/) client for [Kakoune](http://kakoune.org) implemented in [Rust](https://www.rust-lang.org).
This allows Kakoune to support debugging in a variety of different languages, provided the language has a debug adapter implementation.

## Features

- Launch debug adapter (currently hardcoded)
- Launch debuggee in external terminal (Kakoune doesn't have an integrated terminal)
- Stop at breakpoints (currently hardcoded)
- Continue/step/next
- Call stack display (current thread only)
- Heirarchical variable display
- Arbitrary expression evaluation

## FAQ

Q: Does it work? 

A: Technically yes, but everything is hardcoded at the moment.

Q: What's the point of this? kakoune-gdb and kakoune-dbgp exist. 

A: kakoune-gdb is limited to languages supported by gdb - that is, C languages and rust. 
kakoune-dbgp also only supports languages currently supported by the dbgp protocol, which
is mainly PHP at the moment as far as I know. The debug adapter protocol is much more widely 
supported, which allows for more languages to be debugged.

Q: Why is development taking so long?

A: Mostly because I'm a master procrastinator, but also because this is my first Rust project,
and Rust is a language with paradigms that I'm not used to. This is very much a learning experience
for me, so this is going to take some time.

## License

kak-dap is "licensed" under the Unlicense.
