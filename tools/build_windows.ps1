# Run in a Windows x64 Visual Studio Developer PowerShell with Rust installed.
$ErrorActionPreference = 'Stop'
$taskRoot = Split-Path $PSScriptRoot -Parent
Push-Location $taskRoot
try {
    cargo test --manifest-path physics/Cargo.toml
    if ($LASTEXITCODE -ne 0) { throw 'Rust tests failed' }
    cargo build --release --manifest-path physics/Cargo.toml --target x86_64-pc-windows-msvc
    if ($LASTEXITCODE -ne 0) { throw 'Windows build failed' }
    $nativeDir = Join-Path $taskRoot 'physics/target/x86_64-pc-windows-msvc/release'
    $testDir = Join-Path $taskRoot 'dist/windows-native-check'
    New-Item -ItemType Directory -Force $testDir | Out-Null
    cl /nologo /W4 physics/examples/c_host.c /Iphysics/include /link "/LIBPATH:$nativeDir" avatar_physics.dll.lib "/OUT:$testDir/avatar-c-host.exe"
    if ($LASTEXITCODE -ne 0) { throw 'C host compilation failed' }
    Copy-Item "$nativeDir/avatar_physics.dll" $testDir -Force
    & "$testDir/avatar-c-host.exe"
    if ($LASTEXITCODE -ne 0) { throw 'C host checks failed' }
    Write-Host 'Native Windows C host passed. BeamNG loading remains a separate check.'
} finally { Pop-Location }
