#This option dictates where the kak-dap binary is located.
decl -hidden str dap_jar %sh{ printf "%s/../target/debug/%s" "${kak_source%/*}" "kak-dap" }
#This option indicates whether a dap session is running.
decl -hidden bool dap_running false

set-face global DapBreakpoint red,default

decl str dap_breakpoint_active_symbol "●"
decl str dap_location_symbol "➡"

#Contains all breakpoints in this format
#line|file line|file line|file ...
decl -hidden str-list dap_breakpoints_info

decl -hidden line-specs dap_breakpoints_flags

hook global WinDisplay .* %{
    try %{
        addhl window/ flag-lines DapBreakpoint dap_breakpoints_flags
    }
    eval %sh{ printf "dap-refresh-breakpoints-flags %s" "$kak_buffile" }
}

define-command dap-start %{ eval %sh{
    #Start the kak-dap binary
    ( "${kak_opt_inspect_jar}" -s "${kak_session}" 2>&1 & ) > /dev/null 2>&1 < /dev/null
}}

define-command dap-cmd -params 1.. %{
    nop %sh{ ( java -jar "${kak_opt_inspect_jar}" -c "$@" 2>&1 & ) > /dev/null 2>&1 < /dev/null }
}

define-command dap-stop %{
    #Stop the kak-dap binary
    dap-cmd "quit"
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
