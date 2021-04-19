#This option dictates where the kak-dap binary is located.
decl -hidden str dap_bin %sh{ printf "%s/../target/debug/%s" "${kak_source%/*}" "kak-dap" }
#This option indicates whether the kak-dap binary for this session is running.
decl -hidden bool dap_running false
#The directory indicating where the input FIFO is located
decl -hidden str dap_dir "/tmp/kak-dap/%val{session}"
#This option indicates whether autojump is enabled.
decl -hidden bool dap_autojump false

set-face global DapBreakpoint red,default
set-face global DapLocation blue,default

decl str dap_breakpoint_active_symbol "●"
decl str dap_location_symbol "➡"

#Contains all breakpoints in this format
#line|file line|file line|file ...
decl -hidden str-list dap_breakpoints_info
#If execution is currently stopped, shows the current location in this format
#line|file
decl -hidden str dap_location_info

decl -hidden line-specs dap_breakpoints_flags
decl -hidden line-specs dap_location_flags

hook global WinDisplay .* %{
    try %{
        addhl window/ flag-lines DapLocation dap_location_flags
        addhl window/ flag-lines DapBreakpoint dap_breakpoints_flags
    }
    dap-refresh-breakpoints-flags %val{buffile}
    dap-refresh-location-flag %val{buffile}
}

define-command dap-start %{ eval %sh{
    #Create the input FIFO
    mkdir -p $kak_opt_dap_dir
    mkfifo "$kak_opt_dap_dir"/input_pipe
    #Start the kak-dap binary
    ( tail -f "$kak_opt_dap_dir"/input_pipe | "${kak_opt_dap_bin}" -s "${kak_session}" 2>&1 & ) > /dev/null 2>&1 < /dev/null
}}

define-command dap-cmd -params 1..2 %{ eval %sh{
    echo "$1 $2" > "$kak_opt_dap_dir"/input_pipe
}}

define-command dap-stop %{
    #Stop the kak-dap binary
    dap-cmd "stop"
    #Reset the location flag
    dap-reset-location
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
    #Go through every existing breakpoint
    for current in $kak_opt_dap_breakpoints_info; do
        buffer=${current#*|*}
        line=${current%%|*}

        #If the current file and cursor line match this currently existing breakpoint
        if [ "$buffer" = "$kak_buffile" ] && [ "$line" = "$kak_cursor_line" ]; then
            printf "set-option -remove global dap_breakpoints_info '%s|%s'\n" "$line" "$buffer"
            printf "dap-refresh-breakpoints-flags %s\n" "$buffer"
            exit
        fi
    done
    #If we're here, we don't have this breakpoint yet
    printf "set-option -add global dap_breakpoints_info '%s|%s'\n" "$kak_cursor_line" "$kak_buffile"
    printf "dap-refresh-breakpoints-flags %s\n" "$kak_buffile"
}}

define-command dap-set-location -params 2 %{
    set-option global dap_location_info "%arg{1}|%arg{2}"
    dap-refresh-location-flag %arg{2}
}

define-command dap-reset-location %{
    set-option global dap_location_info ""
    dap-refresh-location-flag %val{buffile}
}

define-command dap-jump-to-location %{
    try %{ eval %sh{
        #Get the current location info
        eval set -- "$kak_quoted_opt_dap_location_info"
        [ $# -eq 0 ] && exit
        #Extract the line and buffer
        line="${1%%|*}"
        buffer="${1#*|*}"
        #Edit the file at the given line, failing if it doesn't exist (it should be open already, fingers crossed)
        printf "edit -existing '%s' %s; exec gi" "$buffer" "$line"
    }}
}

define-command -hidden -params 1 dap-refresh-breakpoints-flags %{
    try %{
        set-option "buffer=%arg{1}" dap_breakpoints_flags %val{timestamp}
        eval %sh{
            #Loop through all the current breakpoints
            for current in $kak_opt_dap_breakpoints_info; do
                buffer=${current#*|*}
                #If the current buffer is correct
                if [ "$buffer" = "$1" ]; then
                    line=${current%%|*}
            	    #Set the breakpoint flag
                    printf "set-option -add \"buffer=%s\" dap_breakpoints_flags %s|$kak_opt_dap_breakpoint_active_symbol\n" "$buffer" "$line"
                fi
            done
        }
    }
}

define-command -hidden -params 1 dap-refresh-location-flag %{
    try %{
        set-option "buffer=%arg{1}" dap_location_flags %val{timestamp}
        eval %sh{
            current=$kak_opt_dap_location_info
            buffer=${current#*|*}
            #If the current buffer is correct
            if [ "$buffer" = "$1" ]; then
                line=${current%%|*}
                #Set the location flag
                printf "set-option -add \"buffer=%s\" dap_location_flags %s|$kak_opt_dap_location_symbol\n" "$buffer" "$line"
            fi
        }
    }
}

#
#Responses to events
#

define-command dap-stopped -params 2 %{
    dap-set-location %arg{1} %arg{2}
    dap-jump-to-location
}
