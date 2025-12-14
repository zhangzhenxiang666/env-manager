#env-manage
function em() {
    local output exit_code
    
    output=$(CLICOLOR_FORCE=1 EM_SHELL={{SHELL_TYPE}} {{BINARY_PATH}} "$@" 2>&2)
    exit_code=$?
    
    if [[ "$output" == {{SHELL_CMD_MARKER}}* ]]; then
        eval -- "${output:{{MARKER_LENGTH}}}"
    elif [[ -n "$output" ]]; then
        echo "$output"
    fi
    
    return $exit_code
}

em global init