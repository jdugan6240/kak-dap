# This option indicates the kak-dap source directory.
decl -hidden str dap_dir %sh{echo $(dirname $kak_source)/../}
# This option indicates the arguments passed to the kak-dap server.
decl -hidden str dap_args ""
# This option dictates the command run to run the kak-dap binary.
#decl -hidden str dap_cmd "poetry run python %opt{dap_dir}/src/main.py -s %val{session}"
# This option indicates whether the kak-dap server for this session is running.
decl -hidden bool dap_running false
# This option contains the path of the kak-dap socket for this session.
decl -hidden str dap_socket ""

# This option indicates the client in which the "stacktrace" buffer will be shown
decl str stacktraceclient

# This option indicates the client in which the "variables" buffer will be shown
decl str variablesclient

set-face global DapBreakpoint red,default
set-face global DapLocation blue,default

decl str dap_breakpoint_active_symbol "●"
decl str dap_location_symbol "➡"

# Contains all line breakpoints in this format
# line|file line|file line|file ...
decl -hidden str-list dap_breakpoints_info
# If execution is currently stopped, shows the current location in this format
# line|file
decl -hidden str dap_location_info

decl -hidden line-specs dap_breakpoints_flags
decl -hidden line-specs dap_location_flags
decl -hidden int dap_variables_cursor_line
# Initial setting to ensure cursor is set to top
set-option global dap_variables_cursor_line 1

addhl shared/dap group -passes move
addhl shared/dap/ flag-lines DapLocation dap_location_flags
addhl shared/dap/ flag-lines DapBreakpoint dap_breakpoints_flags

hook global WinDisplay .* %{
    try %{
        addhl window/dap-ref ref -passes move dap
    }
    dap-refresh-breakpoints-flags %val{buffile}
    dap-refresh-location-flag %val{buffile}
}

hook global BufOpenFile .* %{
    dap-refresh-breakpoints-flags %val{buffile}
    dap-refresh-location-flag %val{buffile}
}

define-command -hidden dap-setup-ui-tmux %{
    # Setup the jump client
    rename-client main
    set-option global jumpclient main

    # Setup the stacktrace client
    tmux-terminal-vertical kak -c %val{session} -e "rename-client stacktrace"
    set-option global stacktraceclient stacktrace

    # Setup the variables client
    tmux-terminal-horizontal kak -c %val{session} -e "rename-client variables"
    set-option global variablesclient variables
}

define-command -hidden dap-setup-ui-wezterm %{
    # Setup the jump client
    rename-client main
    set-option global jumpclient main

    # Setup the stacktrace client
    wezterm-terminal-vertical kak -c %val{session} -e "rename-client stacktrace"
    set-option global stacktraceclient stacktrace

    # Setup the variables client
    wezterm-terminal-horizontal kak -c %val{session} -e "rename-client variables"
    set-option global variablesclient variables
}

define-command -hidden dap-setup-ui-default %{
    # Setup the jump client
    rename-client main
    set global jumpclient main
    
    # Setup the stacktrace client
    new rename-client stacktrace
    set global stacktraceclient stacktrace

    # Setup the variables client
    new rename-client variables
    set global variablesclient variables
}

define-command dap-setup-ui %{
    evaluate-commands %sh{
        # Determine which windowing system is in use,
        # and choose the correct one to setup our layout with.
        if [ -n "$TMUX" ]; then
            printf "%s\n" "dap-setup-ui-tmux"
        elif [ -n "$WEZTERM_PANE" ]; then
            printf "%s\n" "dap-setup-ui-wezterm"
        else
            printf "%s\n" "dap-setup-ui-default"
        fi
    }
}

define-command dap-takedown-ui %{
    # Kill the stacktrace client
    evaluate-commands -try-client %opt{stacktraceclient} %{
        quit!
    }
    # Kill the variables client
    evaluate-commands -try-client %opt{variablesclient} %{
        quit!
    }
}

define-command dap-start %{
    eval %sh{
        # kak_opt_dap_breakpoints_info
        # kak_buffile
        if [ "$kak_opt_dap_running" = false ]; then
            # Setup the UI
            printf "%s\n" "dap-setup-ui"

            #printf "echo -debug %s\n" "%opt{dap_cmd}"
            # Start the kak-dap binary
            (eval "poetry run python $kak_opt_dap_dir/src/main.py -s $kak_session $kak_opt_dap_args") > /dev/null 2>&1 < /dev/null &
        else
            printf "echo %s\n" "kak-dap already running"
        fi
    }
}

define-command dap-stop %{
    # Stop the kak-dap binary
    nop %sh{
        printf '{
        "cmd": "stop"
        }' > $kak_opt_dap_socket
    }
}

define-command dap-set-breakpoint -params 2 %{
    set-option -add global dap_breakpoints_info "%arg{1}|%arg{2}"
    dap-refresh-breakpoints-flags %arg{2}
}

define-command dap-clear-breakpoint -params 2 %{
    set-option -remove global dap_breakpoints_info "%arg{1}|%arg{2}"
    dap-refresh-breakpoints-flags %arg{2}
}

define-command dap-toggle-breakpoint %{ eval %sh{
    if [ "$kak_opt_dap_running" = false ]; then
        # Go through every existing breakpoint
        for current in $kak_opt_dap_breakpoints_info; do
            buffer=${current#*|*}
            line=${current%%|*}

            # If the current file and cursor line match this currently existing breakpoint
            if [ "$buffer" = "$kak_buffile" ] && [ "$line" = "$kak_cursor_line" ]; then
                printf "set-option -remove global dap_breakpoints_info '%s|%s'\n" "$line" "$buffer"
                printf "dap-refresh-breakpoints-flags %s\n" "$buffer"
                exit
            fi
        done
        # If we're here, we don't have this breakpoint yet
        printf "set-option -add global dap_breakpoints_info '%s|%s'\n" "$kak_cursor_line" "$kak_buffile"
        printf "dap-refresh-breakpoints-flags %s\n" "$kak_buffile"
    else
        printf "echo %s\n" "Can't toggle breakpoints while running"
    fi
}}

define-command dap-install -params 1 -shell-script-candidates %{
    files=$(ls $kak_opt_dap_dir/installers)
    for file in $files; do
        if [ -f "${kak_opt_dap_dir}/installers/${file}" ]; then
            printf "%s\n" "${file%.*}"
        fi
    done
} %{
    evaluate-commands %sh{
        output=$(mktemp -d "${TMPDIR:-/tmp}"/kak-dap-install-XXXXXXX)/fifo
        mkfifo ${output}

        ( {
            poetry run python -u "${kak_opt_dap_dir}/installers/${1}.py"
            printf "Done. Press <esc> to exit.\n"
        } > "$output" 2>&1 & ) > /dev/null 2>&1 < /dev/null

        printf "%s\n" \
            "edit! -fifo ${output} -scroll *dap-install*" \
            'map buffer normal <esc> %{: delete-buffer *dap-install*<ret>}' \
            "hook -always -once buffer BufCloseFifo .* %{ nop %sh{ rm -r $(dirname ${output}) } }"

    }
}


define-command dap-uninstall -params 1 -shell-script-candidates %{
    files=$(ls $kak_opt_dap_dir/installers)
    for file in $files; do
        if [ -f "${kak_opt_dap_dir}/installers/${file}" ]; then
            printf "%s\n" "${file%.*}"
        fi
    done
} %{
    evaluate-commands %sh{
        output=$(mktemp -d "${TMPDIR:-/tmp}"/kak-dap-uninstall-XXXXXXX)/fifo
        mkfifo "$output"

        ( {
            poetry run python -u "${kak_opt_dap_dir}/installers/${1}.py" "uninstall"
            printf "Done. Press <esc> to exit.\n"
        } > "$output" 2>&1 & ) > /dev/null 2>&1 < /dev/null

        printf "%s\n" \
            "edit! -fifo ${output} -scroll *dap-uninstall*" \
            'map buffer normal <esc> %{: delete-buffer *dap-uninstall*<ret>}' \
            "hook -always -once buffer BufCloseFifo .* %{ nop %sh{ rm -r $(dirname ${output}) } }"
    }
}

#
# Commands sent directly to debug adapter
#

define-command dap-continue %{ eval %sh{
    if [ "$kak_opt_dap_running" = false ]; then
        printf "%s\n" "dap-start"
    else
        printf '{
        "cmd": "continue" 
        }' > $kak_opt_dap_socket
    fi
}}

define-command dap-next %{ nop %sh{
    printf '{
    "cmd": "next" 
    }' > $kak_opt_dap_socket
}}

define-command dap-step-in %{ nop %sh{
    printf '{
    "cmd": "stepIn" 
    }' > $kak_opt_dap_socket
}}

define-command dap-step-out %{ nop %sh{
    printf '{
    "cmd": "stepOut" 
    }' > $kak_opt_dap_socket
}}

define-command dap-evaluate -params 1 %{ nop %sh{
    printf '{
    "cmd": "evaluate",
    "args": {
    "expression": "%s"
    }
    }' "$1" > $kak_opt_dap_socket
}}

#
# Misc commands called by kak-dap server
#

define-command dap-select-config -params 2.. %{
    evaluate-commands %sh{
        command="menu "
        for config in "$@"; do
            command=$command"$config "
            config_cmd=$(printf '{"cmd": "select-config", "args": {"config": "%s"}}' "$config")
            printf "%s\n" "echo -debug $config_cmd"
            command=$command"%{ nop %sh{ printf '%s' '$config_cmd' > $kak_opt_dap_socket } } "
        done
        printf "%s\n" "$command"
    }
}

define-command dap-set-location -params 2 %{
    set-option global dap_location_info "%arg{1}|%arg{2}"
    try %{ eval -client %opt{jumpclient} dap-refresh-location-flag %arg{2} }
}

define-command dap-reset-location %{
    set-option global dap_location_info ""
    try %{ eval -client %opt{jumpclient} dap-refresh-location-flag %val{buffile} }
}

define-command dap-jump-to-location %{
    try %{ eval %sh{
        # Get the current location info
        eval set -- "$kak_quoted_opt_dap_location_info"
        [ $# -eq 0 ] && exit
        # Extract the line and buffer
        line="${1%%|*}"
        buffer="${1#*|*}"
        # Edit the file at the given line, failing if it doesn't exist (it should be open already, fingers crossed)
        printf "edit -existing '%s' %s; exec gi" "$buffer" "$line"
    }}
}

define-command -hidden -params 1 dap-refresh-breakpoints-flags %{
    try %{
        set-option "buffer=%arg{1}" dap_breakpoints_flags %val{timestamp}
        eval %sh{
            # Loop through all the current breakpoints
            for current in $kak_opt_dap_breakpoints_info; do
                buffer=${current#*|*}
                # If the current buffer is correct
                if [ "$buffer" = "$1" ]; then
                    line=${current%%|*}
            	    # Set the breakpoint flag
                    printf "set-option -add \"buffer=%s\" dap_breakpoints_flags %s|$kak_opt_dap_breakpoint_active_symbol\n" "$buffer" "$line"
                fi
            done
        }
    }
}

define-command -hidden -params 1 dap-refresh-location-flag %{
    try %{
        set-option global dap_location_flags %val{timestamp}
        set-option "buffer=%arg{1}" dap_location_flags %val{timestamp}
        eval %sh{
            current=$kak_opt_dap_location_info
            buffer=${current#*|*}
            # If the current buffer is correct
            if [ "$buffer" = "$1" ]; then
                line=${current%%|*}
                # Set the location flag
                printf "set-option -add \"buffer=%s\" dap_location_flags %s|$kak_opt_dap_location_symbol\n" "$buffer" "$line"
            fi
        }
    }
}

#
# Handle the variable/stacktrace buffers
#

define-command -hidden dap-show-stacktrace -params 1 %{
    # Show the stack trace in the stack trace buffer
    evaluate-commands -save-regs '"' -try-client %opt[stacktraceclient] %{
        edit! -scratch *stacktrace*
        set-register '"' %arg{1}
        execute-keys Pgg
    }
}

define-command -hidden dap-show-variables -params 1 %{
    evaluate-commands -save-regs '"' -try-client %opt[variablesclient] %{
        edit! -scratch *variables*
        set-register '"' %arg{1}
        execute-keys "P%opt{dap_variables_cursor_line}g"
        map buffer normal '<ret>' ':<space>dap-expand-variable<ret>'
        # Reset to ensure default value, will be set by expand-variable
        set-option global dap_variables_cursor_line 1

        # strings, keep first
        add-highlighter buffer/vals regions
        add-highlighter buffer/vals/double_string region '"'  (?<!\\)(\\\\)*" fill string
        add-highlighter buffer/vals/single_string region "'"  (?<!\\)(\\\\)*' fill string
        # Scope and varialbe lines
        add-highlighter buffer/scope regex "^(Scope):\s([\w\s]+)" 2:attribute
        add-highlighter buffer/variable_line regex "^\s+([+|-]\s)?(<\d+>)\s([^\s]+)\s\(([A-Za-z]+)\)" 2:comment 3:variable 4:type
        # values
        add-highlighter buffer/type_num regex "(-?\d+)$" 1:value
        add-highlighter buffer/type_bool regex "((?i)true|false)$" 0:value
        add-highlighter buffer/type_null regex "((?i)null|nil|undefined)$" 0:keyword
        add-highlighter buffer/type_array regex "(array\(\d+\))$" 0:default+i
    }
}

define-command -hidden dap-expand-variable %{
    evaluate-commands -try-client %opt{variablesclient} %{
        # Send current line to kak-dap to expand
        set-option global dap_variables_cursor_line %val{cursor_line}
        nop %sh{
            value="${kak_opt_dap_variables_cursor_line}"
            printf '{
            "cmd": "expand",
            "args": {
            "line": "%s"
            }
            }' $value > $kak_opt_dap_socket
        }
    }
}

#
# Responses to reverseRequests
#

define-command -hidden dap-run-in-terminal-tmux -params 1.. %{
    evaluate-commands -try-client %opt{stacktraceclient} %{
        tmux-terminal-horizontal %arg{@}
    }
}

define-command -hidden dap-run-in-terminal-wezterm -params 1.. %{
    evaluate-commands -try-client %opt{stacktraceclient} %{
        wezterm-terminal-horizontal %arg{@}
    }
}

define-command -hidden dap-run-in-terminal-default -params 1.. %{
    terminal %arg{@}
}

define-command -hidden dap-run-in-terminal -params 1.. %{
    evaluate-commands %sh{
        # Determine which windowing system is in use,
        # and choose the correct one.
        if [ -n "$TMUX" ]; then
            printf "%s %s\n" "dap-run-in-terminal-tmux" "$*"
        elif [ -n "$WEZTERM_PANE" ]; then
            printf "%s %s\n" "dap-run-in-terminal-wezterm" "$*"
        else
            printf "%s %s\n" "dap-run-in-terminal-default" "$*"
        fi
    }
    nop %sh{
        printf '{
        "cmd": "pid"
        }' > $kak_opt_dap_socket
    }
}

#
# Responses to debug adapter responses
#

define-command -hidden dap-output -params 2 %{
    evaluate-commands -client %opt{jumpclient} %{
        echo -debug DAP ADAPTER %arg{1}: %arg{2}
    }
}

define-command -hidden dap-stack-trace -params 3 %{
    dap-set-location %arg{1} %arg{2}
    try %{ eval -client %opt{jumpclient} dap-jump-to-location }
    dap-show-stacktrace %arg{3}
}

define-command -hidden dap-evaluate-response -params 2.. %{
    try %{ eval -client %opt{jumpclient} %{ info -title "Result" " %arg{1}:%arg{2} "}}
}

declare-user-mode dap
map global dap s -docstring 'start' ': dap-start<ret>'
map global dap b -docstring 'toggle breakpoints' ': dap-toggle-breakpoint<ret>'
map global dap c -docstring 'continue' ': dap-continue<ret>'
map global dap n -docstring 'next' ': dap-next<ret>'
map global dap o -docstring 'step out' ': dap-step-out<ret>'
map global dap i -docstring 'step in' ': dap-step-in<ret>'
map global dap e -docstring 'eval' ':dap-evaluate '
map global dap q -docstring 'stop' ': dap-stop<ret>'
map global dap . -docstring 'lock' ': enter-user-mode -lock dap<ret>'
