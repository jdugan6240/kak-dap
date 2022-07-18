# This option dictates the command run to run the kak-dap binary.
decl -hidden str dap_cmd "kak-dap -s %val{session}"
# This option indicates whether the kak-dap binary for this session is running.
decl -hidden bool dap_running false

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
decl -hidden int dap_previous_cursor_line

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

define-command dap-setup-ui %{
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
        if [ "$kak_opt_dap_running" = false ]; then
            # Setup the UI
            printf "%s\n" "dap-setup-ui"
            # Start the kak-dap binary
            mkdir /tmp/kak-dap
            printf '{
            "breakpoints": "%s"
            }' "$kak_opt_dap_breakpoints_info" > /tmp/kak-dap/${kak_session}_breakpoints
            export CUR_FILE=$kak_buffile
            (eval "${kak_opt_dap_cmd}") > /dev/null 2>&1 < /dev/null &
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
        }' | eval "${kak_opt_dap_cmd} --request"
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

#
# Commands sent directly to debug adapter
#

define-command dap-continue %{ eval %sh{
    if [ "$kak_opt_dap_running" = false ]; then
        printf "%s\n" "dap-start"
    else
        printf '{
        "cmd": "continue" 
        }' | eval "${kak_opt_dap_cmd} --request" > /dev/null 2>&1
    fi
}}

define-command dap-next %{ nop %sh{
    printf '{
    "cmd": "next" 
    }' | eval "${kak_opt_dap_cmd} --request"
}}

define-command dap-step-in %{ nop %sh{
    printf '{
    "cmd": "stepIn" 
    }' | eval "${kak_opt_dap_cmd} --request"
}}

define-command dap-step-out %{ nop %sh{
    printf '{
    "cmd": "stepOut" 
    }' | eval "${kak_opt_dap_cmd} --request"
}}

define-command dap-evaluate -params 1 %{ nop %sh{
    printf '{
    "cmd": "evaluate",
    "args": "%s"
    }' "$1" | eval "${kak_opt_dap_cmd} --request"
}}

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
        execute-keys "P%opt{dap_previous_cursor_line}g"
        map buffer normal '<ret>' ':<space>dap-expand-variable<ret>'
    }
}

define-command -hidden dap-expand-variable %{
    evaluate-commands -try-client %opt{variablesclient} %{
        # Get variable we're expanding
        execute-keys -save-regs '' "ghwwwW"
        set-register t %val{selection}
        set-option global dap_previous_cursor_line %val{cursor_line}
        nop %sh{
            value="${kak_reg_t}"
            printf '{
            "cmd": "expand",
            "args": "%s"
            }' $value | eval "${kak_opt_dap_cmd} --request"
        }
    }
}

#
# Responses to reverseRequests
#

define-command -hidden dap-run-in-terminal -params 1.. %{
    terminal %arg{@}
    nop %sh{
        printf '{
        "cmd": "pid"
        }' | eval "${kak_opt_dap_cmd} --request"
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
