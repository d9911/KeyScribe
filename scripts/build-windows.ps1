param([string]$PythonExe = 'python')
$ErrorActionPreference = 'Stop'
& $PythonExe (Join-Path $PSScriptRoot 'project.py') build --platform windows
exit $LASTEXITCODE
