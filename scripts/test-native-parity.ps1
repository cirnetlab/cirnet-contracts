[CmdletBinding()]
param(
    [string]$Wheelhouse
)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
$repository = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
if (!$env:CIRNET_ARTIFACTS_SESSION) {
    throw 'Allocate and export a fresh CIRNET_ARTIFACTS_SESSION before running parity.'
}

Push-Location $repository
try {
    $output = node tools/artifact-paths/artifact-paths.mjs native-parity
    $scratch = New-Item -ItemType Directory (Join-Path $output 'tmp')
    $env:TMP = $env:TEMP = $env:TMPDIR = $scratch.FullName
    python -m venv (Join-Path $output 'venv')
    $python = Join-Path $output 'venv/Scripts/python.exe'
    $dependencySource = @()
    if ($Wheelhouse) {
        $dependencySource = @('--no-index', '--find-links', (Resolve-Path -LiteralPath $Wheelhouse).Path)
    }
    & $python -I -B -m pip install --no-cache-dir --disable-pip-version-check @dependencySource -r requirements-verification.lock
    & $python -I -B -m pip check

    $env:PYO3_PYTHON = $python
    if (!$env:CARGO_TARGET_DIR) {
        $env:CARGO_TARGET_DIR = Join-Path $output 'cargo-target'
    }
    cargo build --locked --release --lib --features python-tests
    $binding = Join-Path $output 'cirnet_contracts_native.pyd'
    Copy-Item -LiteralPath (Join-Path $env:CARGO_TARGET_DIR 'release/cirnet_contracts_native.dll') -Destination $binding
    $env:CIRNET_CONTRACTS_NATIVE_BINDING = $binding

    [ordered]@{
        fixtureSha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath 'tests/fixtures/contracts-0.1.2-parity.json').Hash.ToLowerInvariant()
        descriptorSha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath 'tests/fixtures/contracts-0.1.2-rpc-v2.textproto').Hash.ToLowerInvariant()
        bindingSha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $binding).Hash.ToLowerInvariant()
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $output 'inputs.json') -Encoding utf8

    Push-Location $output
    try {
        & $python -I -B -m pytest -q -p no:cacheprovider --basetemp (Join-Path $output 'pytest') `
            (Join-Path $repository 'tests/test_native_parity.py')
    } finally {
        Pop-Location
    }
} finally {
    Pop-Location
}
