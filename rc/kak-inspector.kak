decl -hidden str inspect_jar %sh{ printf "%s/../build/libs/%s" "${kak_source%/*}" "kak-inspector.jar" }
decl -hidden bool debug_running false

define-command kak-debug %{
    nop %sh{ ( java -jar "${kak_opt_inspect_jar}" -s "${kak_session}" 2>&1 & ) > /dev/null 2>&1 < /dev/null }
}

define-command kak-debug-cmd -params 1.. %{
    nop %sh{ ( java -jar "${kak_opt_inspect_jar}" -c "$@" 2>&1 & ) > /dev/null 2>&1 < /dev/null }
}

