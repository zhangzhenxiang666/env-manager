    $origColorForce = $env:CLICOLOR_FORCE
    $env:CLICOLOR_FORCE = '1'

    try {
        $output = & "{{BINARY_PATH}}" $args
        $exitCode = $LASTEXITCODE
    } finally {
        if ($null -eq $origColorForce) {
            Remove-Item Env:\CLICOLOR_FORCE -ErrorAction SilentlyContinue
        } else {
            $env:CLICOLOR_FORCE = $origColorForce
        }
    }

    if ($null -ne $output) {
        $outputStr = $output -join "`n"
        if ($outputStr -match "^{{SHELL_CMD_MARKER}}") {
            $cmd = $outputStr.Substring({{MARKER_LENGTH}})
            Invoke-Expression $cmd
        } else {
            $output | Out-Host
        }
    }

    $global:LASTEXITCODE = $exitCode

em global init
