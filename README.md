# Kakoune Debug Adapter Protocol Client

kak-dap is a [Debug Adapter Protocol](https://microsoft.github.io/debug-adapter-protocol/) client for [Kakoune](http://kakoune.org) implemented in [Rust](https://www.rust-lang.org).
This allows Kakoune to support debugging in a variety of different languages, provided the language has a debug adapter implementation.

## Features

- Launch debug adapter
- Launch debuggee in external terminal (Kakoune doesn't have an integrated terminal)
- Stop at breakpoints
- Continue/step/next
- Call stack display (current thread only)
- Heirarchical variable display
- Arbitrary expression evaluation

## Install

### Requirements

- Rust/Cargo
- Ensure cargo packages are in your path. (eg: `PATH=$HOME/.cargo/bin:$PATH`)

### Pre-built Binary

If using a binary distribution of `kak-dap`, place the following in your kakrc:

```
eval %sh{kak-dap --kakoune -s $kak_session}
```

### Plug.kak

If using `plug.kak` as your plugin manager, add the following to your kakrc:

```
plug "https://codeberg.org/jdugan6240/kak-dap" do %{
  cargo install --locked --force --path .
}
```

### kak-bundle

If using `kak-bundle` as your plugin manager, add the following to your kakrc:

```
bundle "https://codeberg.org/jdugan6240/kak-dap" %{
  cd ${kak_opt_bundle_path}/kak-dap
  cargo install --locked --force --path .
}
```

### Manual

If not using a plugin manager, clone the repository anywhere on your system:

```
git clone https://codeberg.org/jdugan6240/kak-dap
cd <repository_dir>
cargo install --locked --force --path .
```

where <repository_dir> is the directory you cloned the repository to.

Then, add the following to your kakrc:

```
source <repository_dir>/rc/kak-dap.kak
```

where, once again, <repository_dir> is the directory you cloned the repository to.

## Debug Adapters

`kak-dap` doesn't manage installation of debug adapters, so you'll need to install
them yourself. Examples of installing adapters that `kak-dap` has been tested with
are in the [Debug Adapter Installation](https://codeberg.org/jdugan6240/kak-dap/wiki/Debug-Adapter-Installation) wiki page.

## Usage

An (old) demo of `kak-dap` can be found here: [![asciicast](https://asciinema.org/a/fjU1GBrXSxplfP6lEo7cqYcj9.svg)](https://asciinema.org/a/fjU1GBrXSxplfP6lEo7cqYcj9)

### .kak-dap.json File

`kak-dap` requires a file to be present in your project's root directory, named
`.kak-dap.json`. This is a standard JSON file, with support for comments for
convenience. In general, it will look like the following:

```
{
  // The binary run to start the debug adapter
  "adapter": "adapter",

  // The arguments to the debug adapter
  "adapter_args": ["args"],

  // The adapter ID. Needed by some debug adapters.
  "adapterID": "mydbg",

  // The arguments sent to the "launch" request
  "launch_args": {
    // This will depend on the debug adapter used.
  }
}
```

The "adapter", "adapter_args", "adapterID", and "launch_args" values must be present.

Since many debug adapters are Visual Studio Code extensions, it's not always obvious
what to put in this file. Let's take this VSCode `launch.json` file as an example:

```
{
    "version": "0.2.0",
    "configurations": [
        {
            "name": "Python: Current File",
            "type": "python",
            "request": "launch",
            "program": "${file}",
            "console": "integratedTerminal"
        }
    ]
}
```

All we need to look at here is the first {} block under "configurations". Taking out
the "name", "type", and "request" values, the remainder of this block can be placed
under "launch_args" in our `.kak-dap.json` file. In other words, the equivalent
`.kak-dap.json` file for the above `launch.json` file is:

```
{
  "adapter": "python",
  "adapter_args": ["path_to_debugpy_adapter"],
  "adapterID": "python",
  "launch_args": {
    "program": "path_to_file.py",
    "console": "integratedTerminal"
  }
}
```

You may notice that the "${file}" expansion couldn't be translated over. This is
because `kak-dap` only supports the following expansions:

```
${CUR_DIR} - The directory containing the `.kak-dap.json` file.
${USER} - The current username.
${HOME} - The current user's home directory.
$$ - A literal dollar.
```

If this is still confusing, examples of `.kak-dap.json` file configurations for
various debug adapters can be found in the [Debug Adapter Installation](https://codeberg.org/jdugan6240/kak-dap/wiki/Debug-Adapter-Installation) wiki page.

### Setting breakpoints

In the source file, run the `dap-toggle-breakpoint` command to toggle a breakpoint on
the given line.

### Starting and interacting with the debug session

Once you're ready to begin debugging, run the following command:

```
dap-start
```

If configured correctly, the debug adapter will launch and the debug session will begin.
At every stopping point (usually breakpoints), the "code window" will show the current
line. At this point, the following commands are available:

```
dap-continue - Continue running from the current point, or start debug session if one isn't already running
dap-next - Execute and stop at the next line
dap-step-in - Step into a function/method
dap-step-out - Step out (return from) a function/method
dap-evaluate <expression> - Evaluate an arbitrary expression and show the result
```

In addition, the stacktrace and variables buffers will be populated with the current
stack trace and a variable heirarchy, respectively. In the variables buffer, some
variables are expandable, and can be expanded by pressing Enter.

When you're finished debugging, run the following command to stop debugging:

```
dap-stop
```

### Custom user mode

A `kak-dap` user mode is provided with mappings to several commands. You may map this to a key of your liking, below we're using `x`:

```
map global user x -docstring 'dap' ': enter-user-mode dap<ret>'
```

## Troubleshooting

`kak-dap` isn't perfect, and may fail from time to time. In case this happens, `kak-dap`'s
logging can be enabled by inserting the following command in your kakrc:

```
set global dap_cmd "kak-dap -s %val{session} --log /tmp/kak-dap.log -vvvv"
```

This will cause `kak-dap` to create a debug log in the `/tmp/kak-dap.log` file. If this isn't
enough to diagnose the problem, please don't hesitate to raise an issue.

## Not-so-FAQ

Q: Does it work?

A: Yes, but it's rather unpolished and limited at the moment.

Q: What's the point of this? kakoune-gdb and kakoune-dbgp exist.

A: kakoune-gdb is limited to languages supported by gdb - that is, C languages and rust.
kakoune-dbgp also only supports languages currently supported by the dbgp protocol, which
is mainly PHP at the moment as far as I know. The debug adapter protocol is much more widely
supported, which allows for more languages to be debugged.

## License

kak-dap is licensed under the BSD 0-Clause License.

## Contributors

James Dugan (https://codeberg.org/jdugan6240)
in0ni (https://github.com/in0ni)
