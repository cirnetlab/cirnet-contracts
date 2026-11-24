[CmdletBinding(DefaultParameterSetName = 'Source')]
param(
    [Parameter(ParameterSetName = 'Source')]
    [string]$LegacyCheckout = (Join-Path $PSScriptRoot '../../cirnet-contracts'),
    [Parameter(Mandatory, ParameterSetName = 'Wheel')]
    [string]$SealedWheel,
    [string]$Wheelhouse
)

$ErrorActionPreference = 'Stop'
$PSNativeCommandUseErrorActionPreference = $true
$releaseRevision = '524cc7a5e2e65f39077feefe70c7fb2ae3a773d6'
$releaseTree = 'bb21f5f02c100d475ae90dfbbc96e95228066120'
$releaseWheelSha = '7b092af5817b0e15f02f9f5f292a8e1b20fe1d512600f9c0260d05a58ddaaf42'
$repository = (Resolve-Path (Join-Path $PSScriptRoot '..')).Path
if (!$env:CIRNET_ARTIFACTS_SESSION) {
    throw 'Allocate and export a fresh CIRNET_ARTIFACTS_SESSION before running parity.'
}

Push-Location $repository
try {
    $output = node tools/artifact-paths/artifact-paths.mjs legacy-parity
    $sealed = $PSCmdlet.ParameterSetName -eq 'Wheel'
    $archiveSha = $null
    if ($sealed) {
        $wheel = (Resolve-Path -LiteralPath $SealedWheel).Path
        if ((Get-FileHash -Algorithm SHA256 -LiteralPath $wheel).Hash -ne $releaseWheelSha) {
            throw 'The supplied wheel SHA-256 differs from the sealed Contracts 0.1.2 wheel.'
        }
    } else {
        $legacy = (Resolve-Path -LiteralPath $LegacyCheckout).Path
        $context = node tools/artifact-paths/artifact-paths.mjs --context-info | ConvertFrom-Json
        if ($legacy -notin $context.roots) {
            throw 'The artifact allocation context must list the legacy Contracts root.'
        }
        $tree = git -C $legacy rev-parse "${releaseRevision}^{tree}"
        if ($tree -ne $releaseTree) { throw 'Unexpected legacy Contracts release tree.' }
        $source = New-Item -ItemType Directory (Join-Path $output 'source')
        $archive = Join-Path $output 'reference.tar'
        git -C $legacy archive --format=tar --output=$archive $releaseRevision
        $archiveSha = (Get-FileHash -Algorithm SHA256 -LiteralPath $archive).Hash.ToLowerInvariant()
        tar -xf $archive -C $source.FullName
    }

    $scratch = New-Item -ItemType Directory (Join-Path $output 'tmp')
    $env:TMP = $env:TEMP = $env:TMPDIR = $scratch.FullName
    $env:PYTHONDONTWRITEBYTECODE = '1'
    python -m venv (Join-Path $output 'venv')
    $python = Join-Path $output 'venv/Scripts/python.exe'
    $dependencySource = @()
    if ($Wheelhouse) { $dependencySource = @('--no-index', '--find-links', (Resolve-Path -LiteralPath $Wheelhouse).Path) }
    & $python -I -B -m pip install --no-cache-dir --disable-pip-version-check @dependencySource -r requirements-verification.lock
    if (!$sealed) {
        $env:CIRNET_BUILD_ROOT = Join-Path $output 'reference-build'
        Push-Location $source.FullName
        try {
            & $python -I -B -m pip wheel --no-cache-dir --no-deps --no-build-isolation --wheel-dir $output .
        } finally {
            Pop-Location
        }
        $wheel = Join-Path $output 'cirnet_contracts-0.1.2-py3-none-any.whl'
    }

    $wheelSha = (Get-FileHash -Algorithm SHA256 -LiteralPath $wheel).Hash.ToLowerInvariant()
    if ($sealed -and $wheelSha -ne $releaseWheelSha) { throw 'The sealed wheel changed before installation.' }
    $wheelRequirement = Join-Path $output 'reference-wheel.txt'
    "cirnet-contracts @ $([uri]::new($wheel).AbsoluteUri) --hash=sha256:$wheelSha" |
        Set-Content -LiteralPath $wheelRequirement -Encoding utf8
    & $python -I -B -m pip install --no-index --no-deps --require-hashes -r $wheelRequirement
    & $python -I -B -m pip check

    $env:PYO3_PYTHON = $python
    $env:CARGO_TARGET_DIR = Join-Path $output 'cargo-target'
    cargo build --locked --release --lib --features python-tests
    $binding = Join-Path $output 'cirnet_contracts_native.pyd'
    Copy-Item -LiteralPath (Join-Path $env:CARGO_TARGET_DIR 'release/cirnet_contracts_native.dll') -Destination $binding
    $env:CIRNET_CONTRACTS_NATIVE_BINDING = $binding
    $env:CIRNET_CONTRACTS_PYTHON_SHA256 = $wheelSha
    [ordered]@{
        legacyRevision = $releaseRevision
        legacyTree = $releaseTree
        sealedWheel = $sealed
        sourceArchiveSha256 = $archiveSha
        referenceWheelSha256 = $wheelSha
        bindingSha256 = (Get-FileHash -Algorithm SHA256 -LiteralPath $binding).Hash.ToLowerInvariant()
    } | ConvertTo-Json | Set-Content -LiteralPath (Join-Path $output 'inputs.json') -Encoding utf8
    Push-Location $output
    try {
        & $python -I -B -m pytest -q -p no:cacheprovider --basetemp (Join-Path $output 'pytest') `
            (Join-Path $repository 'tests/test_installed_native_parity.py')
    } finally {
        Pop-Location
    }
} finally {
    Pop-Location
}
