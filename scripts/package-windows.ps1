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

Remove-Item -LiteralPath $stageDirectory -Recurse -Force -ErrorAction SilentlyContinue
Remove-Item -LiteralPath $archivePath -Force -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force -Path $stageDirectory | Out-Null

Copy-Item -LiteralPath $dllPath -Destination (Join-Path $stageDirectory "xgameruntime.dll")
Copy-Item -LiteralPath (Join-Path $repositoryRoot "packaging/WINDOWS.md") -Destination (Join-Path $stageDirectory "README.md")
Copy-Item -LiteralPath (Join-Path $repositoryRoot "packaging/WINDOWS.zh-CN.md") -Destination (Join-Path $stageDirectory "README.zh-CN.md")
Copy-Item -LiteralPath (Join-Path $repositoryRoot "docs/BMCBL_PROTOCOL.md") -Destination (Join-Path $stageDirectory "BMCBL_PROTOCOL.md")
Copy-Item -LiteralPath (Join-Path $repositoryRoot "docs/BMCBL_PROTOCOL.zh-CN.md") -Destination (Join-Path $stageDirectory "BMCBL_PROTOCOL.zh-CN.md")
Copy-Item -LiteralPath (Join-Path $repositoryRoot "docs/preauth-v2.schema.json") -Destination (Join-Path $stageDirectory "preauth-v2.schema.json")
Copy-Item -LiteralPath (Join-Path $repositoryRoot "LICENSE") -Destination (Join-Path $stageDirectory "LICENSE")
Copy-Item -LiteralPath (Join-Path $repositoryRoot "NOTICE.md") -Destination (Join-Path $stageDirectory "NOTICE.md")

$rustcVersion = (& rustc --version).Trim()
$manifest = [ordered]@{
    schema_version = 1
    package = "xgameruntime"
    version = $Version
    variant = "windows-native"
    architecture = "x86_64"
    rust_target = "x86_64-pc-windows-msvc"
    source_repository = "https://github.com/Chlna6666/xgameruntime"
    source_commit = $SourceCommit
    toolchain = $rustcVersion
    documentation_languages = @("en", "zh-CN")
    experimental = $true
}
$manifest | ConvertTo-Json -Depth 4 | Set-Content -LiteralPath (Join-Path $stageDirectory "manifest.json") -Encoding utf8NoBOM

$checksumLines = Get-ChildItem -LiteralPath $stageDirectory -File |
    Sort-Object Name |
    ForEach-Object {
        $hash = (Get-FileHash -LiteralPath $_.FullName -Algorithm SHA256).Hash.ToLowerInvariant()
        "$hash  $($_.Name)"
    }
$checksumLines | Set-Content -LiteralPath (Join-Path $stageDirectory "SHA256SUMS") -Encoding ascii

Compress-Archive -Path (Join-Path $stageDirectory "*") -DestinationPath $archivePath -CompressionLevel Optimal
$archiveHash = (Get-FileHash -LiteralPath $archivePath -Algorithm SHA256).Hash.ToLowerInvariant()
Write-Host "Created $archivePath"
Write-Host "SHA256 $archiveHash"
