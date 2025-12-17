function em {
    $env:EM_SHELL = "powershell"
    $output = & "{{BINARY_PATH}}" $args 2>&1
    $exitCode = $LASTEXITCODE

    if ($output -match "^{{SHELL_CMD_MARKER}}") {
        $cmd = $output.Substring({{MARKER_LENGTH}})
        Invoke-Expression $cmd
    } elseif ($null -ne $output) {
        Write-Host "$output"
    }

    $global:LASTEXITCODE = $exitCode
}

em global init
