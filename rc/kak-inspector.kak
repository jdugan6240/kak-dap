decl -hidden str inspect_jar %sh{ printf "%s/../build/libs/%s" "${kak_source%/*}" "kak-inspector.jar" }
decl -hidden bool debug_running false

set-face global Breakpoint red,default

decl str breakpoint_active_symbol "●"
decl str location_symbol "➡"

#Contains all breakpoints in this format
#line|file line|file line|file ...
decl -hidden str-list breakpoints_info

decl -hidden line-specs breakpoints_flags

hook global WinDisplay .* %{ eval %sh{ printf "refresh-breakpoints-flags %s" "$kak_buffile" } }

add-highlighter global/ flag-lines Breakpoint breakpoints_flags

define-command kak-debug %{
    nop %sh{ ( java -jar "${kak_opt_inspect_jar}" -s "${kak_session}" 2>&1 & ) > /dev/null 2>&1 < /dev/null }
}

define-command kak-debug-cmd -params 1.. %{
    nop %sh{ ( java -jar "${kak_opt_inspect_jar}" -c "$@" 2>&1 & ) > /dev/null 2>&1 < /dev/null }
}

define-command dap-set-breakpoint -params 2 %{
    set-option -add global breakpoints_info "%arg{1}|%arg{2}"
    refresh-breakpoints-flags %arg{2}
}

define-command dap-clear-breakpoint -params 2 %{
    set-option -remove global breakpoints_info "%arg{1}|%arg{2}"
    refresh-breakpoints-flags %arg{2}
}

define-command dap-toggle-breakpoint %{ eval %sh{
    #Go through every existing breakpoint
	for current in $kak_opt_breakpoints_info; do
		buffer=${current#*|*}
		line=${current%%|*}

		#If the current file and cursor line match this currently existing breakpoint
		if [ "$buffer" = "$kak_buffile" ] && [ "$line" = "$kak_cursor_line" ]; then
			printf "set-option -remove global breakpoints_info '%s|%s'\n" "$line" "$buffer"
			printf "refresh-breakpoints-flags %s\n" "$buffer"
			exit
		fi
	done
	#We don't have this breakpoint yet
	printf "set-option -add global breakpoints_info '%s|%s'\n" "$kak_cursor_line" "$kak_buffile"
	printf "refresh-breakpoints-flags %s\n" "$kak_buffile"
}}


define-command -hidden -params 1 refresh-breakpoints-flags %{
    try %{
        unset-option "buffer=%arg{1}" breakpoints_flags
        eval %sh{
            #Loop through all the current breakpoints
            for current in $kak_opt_breakpoints_info; do
            	buffer=${current#*|*}
            	#If the current buffer is correct
            	if [ "$buffer" = "$1" ]; then
            		line=${current%%|*}
            		#Set the breakpoint flag
            		printf "set-option -add \"buffer=%s\" breakpoints_flags %s|$kak_opt_breakpoint_active_symbol\n" "$buffer" "$line"
            	fi
            done
        }
    }
}

