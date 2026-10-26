param(
    [ValidateSet('help','doctor','setup','dev','build','local','run','start','cli')]
    [string]$Action = 'help',
    [string]$PythonExe = 'python'
)
$ErrorActionPreference = 'Stop'
& $PythonExe (Join-Path $PSScriptRoot 'scripts/project.py') $Action
exit $LASTEXITCODE
