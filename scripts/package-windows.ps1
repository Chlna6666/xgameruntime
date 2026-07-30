param(
    [Parameter(Mandatory = $true)]
    [string]$Version,

    [Parameter(Mandatory = $true)]
    [string]$SourceCommit,

    [Parameter(Mandatory = $true)]
    [string]$OutputDirectory
)

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$repositoryRoot = Split-Path -Parent $PSScriptRoot
$dllPath = Join-Path $repositoryRoot "target/x86_64-pc-windows-msvc/release/xgameruntime.dll"
if (-not (Test-Path -LiteralPath $dllPath -PathType Leaf)) {
    throw "Windows DLL was not found: $dllPath"
}

$outputRoot = [System.IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Force -Path $outputRoot | Out-Null

$packageName = "xgameruntime-$Version-windows-x64"
$stageDirectory = Join-Path ([System.IO.Path]::GetTempPath()) $packageName
$archivePath = Join-Path $outputRoot "$packageName.zip"
$validationRoot = Join-Path ([System.IO.Path]::GetTempPath()) "$packageName-validation"

Remove-Item -LiteralPath $stageDirectory -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $validationRoot -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $archivePath -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $stageDirectory | Out-Null

Copy-Item -LiteralPath $dllPath -Destination (Join-Path $stageDirectory "xgameruntime.dll")
Copy-Item -LiteralPath (Join-Path $repositoryRoot "packaging/WINDOWS.md") -Destination (Join-Path $stageDirectory "README.md")
Copy-Item -LiteralPath (Join-Path $repositoryRoot "packaging/WINDOWS.zh-CN.md") -Destination (Join-Path $stageDirectory "README.zh-CN.md")
Copy-Item -LiteralPath (Join-Path $repositoryRoot "docs/BMCBL_PROTOCOL.md") -Destination (Join-Path $stageDirectory "BMCBL_PROTOCOL.md")
Copy-Item -LiteralPath (Join-Path $repositoryRoot "docs/BMCBL_PROTOCOL.zh-CN.md") -Destination (Join-Path $stageDirectory "BMCBL_PROTOCOL.zh-CN.md")
Copy-Item -LiteralPath (Join-Path $repositoryRoot "docs/preauth-v2.schema.json") -Destination (Join-Path $stageDirectory "preauth-v2.schema.json")
Copy-Item -LiteralPath (Join-Path $repositoryRoot "CHANGELOG.md") -Destination (Join-Path $stageDirectory "CHANGELOG.md")
Copy-Item -LiteralPath (Join-Path $repositoryRoot "LICENSE") -Destination (Join-Path $stageDirectory "LICENSE")
Copy-Item -LiteralPath (Join-Path $repositoryRoot "NOTICE.md") -Destination (Join-Path $stageDirectory "NOTICE.md")

$rustcVersion = (& rustc --version).Trim()
$manifest = [ordered]@{
    schema_version = 2
    package = "xgameruntime"
    version = $Version
    variant = "windows-native-proxy"
    architecture = "x86_64"
    rust_target = "x86_64-pc-windows-msvc"
    source_repository = "https://github.com/Chlna6666/xgameruntime"
    source_commit = $SourceCommit
    toolchain = $rustcVersion
    documentation_languages = @("en", "zh-CN")
    native_runtime = [ordered]@{
        override_environment = "BMCBL_NATIVE_XGAMERUNTIME"
        sibling_proxy_name = "xgameruntime_o.dll"
        system_fallback = "C:\Windows\System32\xgameruntime.dll"
        load_order = @(
            "environment_override",
            "sibling_xgameruntime_o",
            "system32_xgameruntime"
        )
        diagnostics = @(
            "standard_error",
            "OutputDebugStringW"
        )
        microsoft_runtime_included = $false
    }
    experimental = $true
}
$manifest | ConvertTo-Json -Depth 8 | Set-Content -LiteralPath (Join-Path $stageDirectory "manifest.json") -Encoding utf8NoBOM

$checksumLines = Get-ChildItem -LiteralPath $stageDirectory -File |
    Sort-Object Name |
    ForEach-Object {
        $hash = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        "$hash  $($_.Name)"
    }
$checksumLines | Set-Content -LiteralPath (Join-Path $stageDirectory "SHA256SUMS") -Encoding ascii

Compress-Archive -Path $stageDirectory -DestinationPath $archivePath -CompressionLevel Optimal

Expand-Archive -LiteralPath $archivePath -DestinationPath $validationRoot
$validatedPackage = Join-Path $validationRoot $packageName
$requiredFiles = @(
    "xgameruntime.dll",
    "README.md",
    "README.zh-CN.md",
    "BMCBL_PROTOCOL.md",
    "BMCBL_PROTOCOL.zh-CN.md",
    "preauth-v2.schema.json",
    "CHANGELOG.md",
    "LICENSE",
    "NOTICE.md",
    "manifest.json",
    "SHA256SUMS"
)
foreach ($requiredFile in $requiredFiles) {
    $requiredPath = Join-Path $validatedPackage $requiredFile
    if (-not (Test-Path -LiteralPath $requiredPath -PathType Leaf)) {
        throw "Windows package validation failed, missing: $requiredFile"
    }
}

$manifestPath = Join-Path $validatedPackage "manifest.json"
$validatedManifest = Get-Content -LiteralPath $manifestPath -Raw | ConvertFrom-Json
if ($validatedManifest.version -ne $Version) {
    throw "Windows package validation failed, manifest version mismatch"
}
if ($validatedManifest.native_runtime.sibling_proxy_name -ne "xgameruntime_o.dll") {
    throw "Windows package validation failed, unexpected sibling proxy name"
}
if ($validatedManifest.native_runtime.system_fallback -ne "C:\Windows\System32\xgameruntime.dll") {
    throw "Windows package validation failed, unexpected System32 fallback"
}

$checksumPath = Join-Path $validatedPackage "SHA256SUMS"
foreach ($line in Get-Content -LiteralPath $checksumPath) {
    if ($line -notmatch '^([0-9a-f]{64})  (.+)$') {
        throw "Invalid SHA256SUMS line: $line"
    }
    $expectedHash = $Matches[1]
    $fileName = $Matches[2]
    $filePath = Join-Path $validatedPackage $fileName
    $actualHash = (Get-FileHash -LiteralPath $filePath -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actualHash -ne $expectedHash) {
        throw "Checksum mismatch for $fileName"
    }
}

Remove-Item -LiteralPath $validationRoot -Recurse -Force
$archiveHash = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
Write-Host "Created $archivePath"
Write-Host "SHA256 $archiveHash"
