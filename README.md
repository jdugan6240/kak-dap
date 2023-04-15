# Kakoune Debug Adapter Protocol Client

kak-dap is a [Debug Adapter Protocol](https://microsoft.github.io/debug-adapter-protocol/) client for [Kakoune](http://kakoune.org).
This allows Kakoune to support debugging in a variety of different languages, provided the language has a debug adapter implementation.

## WARNING

This plugin is currently undergoing a complete rewrite and is not *quite* ready for use.
When finished, it will work drastically differently from kak-dap 1.1.

## Features

- Multiple debug configs per project
- Launch debug adapter
- Launch debuggee in external terminal (Kakoune doesn't have an integrated terminal)
- Stop at breakpoints
- Continue/step/next
- Multithreaded support (show names of threads; show call stack of current thread)
- Heirarchical variable display
- Arbitrary expression evaluation

## Install

### Requirements

kak-dap requires Python 3.7 or later. Additionally, the following Python packages must be installed:

- pyyaml
- schema
- xdg

If you haven't already, run `python -m pip install pyyaml schema xdg` to install the necessary packages.

### Optional requirements

For WezTerm support, install the following plugin:

https://github.com/rzeigler/wezterm.kak

### Plug.kak

If using `plug.kak` as your plugin manager, add the following to your kakrc:

```
plug "https://git.sr.ht/~jdugan6240/kak-dap"
```

### kak-bundle

If using `kak-bundle` as your plugin manager, add the following to your kakrc
(requires the `big-rewrite` branch of `kak-bundle`:

```
bundle kak-dap "https://git.sr.ht/~jdugan6240/kak-dap"
```

### cork.kak

If using `cork.kak` as your plugin manager, add the following to your kakrc:

```
cork kak-dap "https://git.sr.ht/~jdugan6240/kak-dap"
```

### Manual

If not using a plugion manager, clone the repository anywhere on your system:

```
git clone https://git.sr.ht/~jdugan6240/kak-dap
```

Then, add the following to your kakrc:

```
source <repository_dir>/rc/kak-dap.kak
```

where <repository_dir> is the directory you cloned the repository to.

## Usage

An (old) demo of `kak-dap` can be found here: [![asciicast](https://asciinema.org/a/fjU1GBrXSxplfP6lEo7cqYcj9.svg)](https://asciinema.org/a/fjU1GBrXSxplfP6lEo7cqYcj9)

### .kak-dap.yaml File

kak-dap requires a file to be present in your project's root directory, named .kak-dap.yaml. This is a standard YAML file. In general,
it will look like the following:

```yaml
configurations:
    <my_config>:
        adapter: <adapter>
        launch_args:
            # This will depend on the debug adapter used.
    # You can optionally define multiple configs in the same project.
    <second_config>:
        adapter: <second_adapter>
        launch_args:
            # This will depend on the debug adapter used.
```

where <my_config> is an arbitrary name for this debug configuration, and <adapter> is the name of an adapter defined in adapter.yaml.

For examples of .kak-dap.yaml file configurations, check out the demo/ directory in the repo.

### Setting breakpoints

In the source file, run the `dap-toggle-breakpoint` command to toggle a breakpoint on the given line.

### Starting and interacting with the debug session

Once you're ready to begin debugging, run the following command:

```
dap-start
```

If there is more than one configuration defined in this project's .kak-dap.yaml file, kak-dap will display a menu allowing the user
to choose which configuration to run. After that, the debug adapter will launch and the debug session will begin. At every stopping point
(usually breakpoints), the "code window" will show the current line. At this point, the following commands are available:

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

### Keybindings

To avoid conflicting with other plugins, kak-dap doesn't come with any keybindings pre-registered, so you'll have to define
them yourself. The author recommends the following:

```
map global normal <F3> ':dap-stop<ret>'
map global normal <F5> ':dap-continue<ret>'
map global normal <F9> ':dap-toggle-breakpoint<ret>'
map global normal <F10> ':dap-next<ret>'
map global normal <F11> ':dap-step-in<ret>'
map global normal <F12> ':dap-step-out<ret>'
```

### Custom user mode

Additionally, a `kak-dap` user mode is provided with mappings to several commands. You may map this to a key of your liking, below we're using `x`:

```
map global user x -docstring 'dap' ': enter-user-mode dap<ret>'
```

## Troubleshooting

`kak-dap` isn't perfect, and may fail from time to time. In case this happens, `kak-dap`'s
logging can be enabled by inserting the following command in your kakrc:

```
set global dap_cmd "python %opt{dap_dir}/main.py -s %val{session} -l /tmp/kak-dap.log -vvvvv"
```

This will cause `kak-dap` to create a debug log located at /tmp/kak-dap.log.
If this isn't enough to diagnose the problem, please don't hesitate to raise an issue.

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
