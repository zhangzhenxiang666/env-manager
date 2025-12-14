#env-manage
function em
    set -l output
    env CLICOLOR_FORCE=1 EM_SHELL={{SHELL_TYPE}} {{BINARY_PATH}} $argv | read -z output
    
    # Capture the exit code of the binary (the first command in the pipe)
    set -l cmd_status $pipestatus[1]

    if string match -q "{{SHELL_CMD_MARKER}}*" -- $output
        string sub -s {{MARKER_LENGTH_PLUS_ONE}} -- $output | source
    else if test -n "$output"
        echo -n "$output"
    end

    return $cmd_status
end

em global init
