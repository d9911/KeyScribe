$ErrorActionPreference = 'Stop'
Push-Location (Split-Path $PSScriptRoot -Parent)
try {
    rustup target add x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { throw 'rustup target add failed' }
    cargo build --release --locked --target x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { throw 'cargo build failed' }
    New-Item -ItemType Directory -Force -Path dist/windows-x64 | Out-Null
    Copy-Item target/x86_64-pc-windows-msvc/release/keyscribe.exe dist/windows-x64/Keyscribe.exe -Force
    Copy-Item README.md, README.ru.md, README.es.md, LICENSE -Destination dist/windows-x64 -Force
    foreach ($directory in @('assets', 'samples')) {
        $destination = Join-Path dist/windows-x64 $directory
        if (Test-Path -LiteralPath $destination) {
            Remove-Item -LiteralPath $destination -Recurse -Force
        }
        New-Item -ItemType Directory -Path $destination | Out-Null
        Copy-Item -Path (Join-Path $directory '*') -Destination $destination -Recurse -Force
        Get-ChildItem -LiteralPath $destination -File -Recurse -Force |
            Where-Object { $_.Name -in @('.DS_Store', 'Icon', "Icon$([char]13)") } |
            Remove-Item -Force
    }
    Compress-Archive -Path dist/windows-x64/* -DestinationPath dist/Keyscribe-windows-x64.zip -Force
    Write-Host "Ready: $((Get-Location).Path)/dist/windows-x64/Keyscribe.exe"
} finally {
    Pop-Location
}
