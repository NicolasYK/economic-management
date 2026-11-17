# CI-only OpenGL renderer. Never install globally or include in release archives.
$ErrorActionPreference = 'Stop'
$root = Split-Path -Parent $PSScriptRoot
$toolsDirectory = Join-Path $root '.tools'
$archive = Join-Path $toolsDirectory 'mesa3d-25.0.7-release-msvc.7z'
$destination = Join-Path $toolsDirectory 'mesa-25.0.7'
New-Item -ItemType Directory -Force -Path $destination | Out-Null
if (-not (Test-Path -LiteralPath $archive)) {
    Invoke-WebRequest 'https://github.com/pal1000/mesa-dist-win/releases/download/25.0.7/mesa3d-25.0.7-release-msvc.7z' -OutFile $archive
}
$expected = 'e81be9e5990fabc5e528f544b1a2e2f37f70fe098822eb9c70555501f3621287'
if ((Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant() -ne $expected) {
    throw 'Mesa archive checksum mismatch; refusing to extract'
}
tar -xf $archive -C $destination
if ($LASTEXITCODE -ne 0) { throw 'Mesa extraction failed' }
$drivers = Join-Path $destination 'x64'
if (-not (Test-Path -LiteralPath (Join-Path $drivers 'opengl32.dll'))) { throw 'Missing Mesa OpenGL driver' }
if (-not $env:GITHUB_ENV) { throw 'Run this setup in GitHub Actions' }
"MESA_DIRECTORY=$drivers" | Out-File -FilePath $env:GITHUB_ENV -Encoding utf8 -Append
