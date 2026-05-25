#!/usr/bin/env pwsh
#
# Run `cargo cross build` for all Windows targets from tools/cross-builds.sh.
# Any script arguments are forwarded to `cargo cross build`.
# If DIROUT is set, copy built s4.exe files there using target-specific names.
#
# Usage:
#   pwsh ./tools/cross-builds.ps1
#   pwsh ./tools/cross-builds.ps1 --release
#   $env:DIROUT = 'C:\temp\s4-cross-builds'; pwsh ./tools/cross-builds.ps1 --release

[CmdletBinding()]
param(
    [Parameter(ValueFromRemainingArguments = $true)]
    [string[]] $CrossArgs
)

$ErrorActionPreference = 'Stop'

$WindowsTargets = @(
    'aarch64-pc-windows-gnullvm',
    'aarch64-pc-windows-msvc',
    'aarch64-uwp-windows-msvc',
    'arm64ec-pc-windows-msvc',
    'i686-pc-windows-gnu',
    'i686-pc-windows-gnullvm',
    'i686-pc-windows-msvc',
    'i686-uwp-windows-gnu',
    'i686-uwp-windows-msvc',
    'i686-win7-windows-gnu',
    'i686-win7-windows-msvc',
    'thumbv7a-pc-windows-msvc',
    'thumbv7a-uwp-windows-msvc',
    'x86_64-pc-windows-gnu',
    'x86_64-pc-windows-gnullvm',
    'x86_64-pc-windows-msvc',
    'x86_64-uwp-windows-gnu',
    'x86_64-uwp-windows-msvc',
    'x86_64-win7-windows-gnu',
    'x86_64-win7-windows-msvc'
)

function Get-BuildProfile {
    param([string[]] $ArgsList)

    # In strict mode, $null.Count throws. Normalize to an array first.
    $argsNormalized = @($ArgsList)

    for ($i = 0; $i -lt $argsNormalized.Count; $i++) {
        $arg = $argsNormalized[$i]
        if ($arg -eq '--release') {
            return 'release'
        }

        if ($arg -eq '--profile' -and ($i + 1) -lt $argsNormalized.Count) {
            return $argsNormalized[$i + 1]
        }

        if ($arg -like '--profile=*') {
            return $arg.Substring('--profile='.Length)
        }
    }

    return 'debug'
}

function Write-Sha256ChecksumFile {
    param(
        [Parameter(Mandatory = $true)]
        [string] $FilePath
    )

    $fileItem = Get-Item -LiteralPath $FilePath -ErrorAction Stop
    $hash = (Get-FileHash -LiteralPath $FilePath -Algorithm SHA256).Hash.ToLowerInvariant()
    $shaPath = Join-Path -Path $fileItem.DirectoryName -ChildPath ($fileItem.Name + '.sha256')
    Remove-Item -LiteralPath $shaPath -ErrorAction Ignore -Force
    # Match common sha256sum text format: "<hash><two spaces><filename>"
    Set-Content -LiteralPath $shaPath -Value ("{0}  {1}" -f $hash, $fileItem.Name)
    Set-FileNoWrite -Path $shaPath
}

function Set-FileNoWrite {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Path
    )

    $item = Get-Item -LiteralPath $Path
    if (-not $item.PSIsContainer) {
        # XXX: not sure which is more correct, just try both
        $item.Attributes = $item.Attributes -BOR [System.IO.FileAttributes]::ReadOnly
        Set-ItemProperty -Path $Path -Name IsReadOnly -Value $true
    }
}

$PROJECT_ROOT = Join-Path $PSScriptRoot '..'
$PROJECT_MANIFEST_ITEM = Get-Item -LiteralPath (Join-Path $PROJECT_ROOT 'Cargo.toml') -ErrorAction Stop

function Get-ProgramVersion {
    param()
    $manifest = Get-Content -LiteralPath $PROJECT_MANIFEST_ITEM.FullName -Raw
    $versionMatch = [regex]::Match($manifest, '^\s*version\s*=\s*"(.*?)"\s*$' , [System.Text.RegularExpressions.RegexOptions]::Multiline)
    if ($versionMatch.Success) {
        return $versionMatch.Groups[1].Value
    }
    return $null
}

function Write-Line {
    $line = -join ('─' * ($Host.UI.RawUI.WindowSize.Width - 1))
    Write-Host $line
}

$BIN = 's4'

try {
    Set-StrictMode -Version 3.0
    # Set-PSDebug -Trace 1

    $erroractionpreference_ = $ErrorActionPreference
    $ErrorActionPreference = 'Stop'
    $startLocation = Get-Location

    Set-Location (Join-Path $PSScriptRoot '..')

    $buildProfile = Get-BuildProfile -ArgsList $CrossArgs

    $outputDir = $null
    $releaseDir = $null
    if ($env:DIROUT) {
        New-Item -ItemType Directory -Path $env:DIROUT -Force | Out-Null
        $outputDir = (Resolve-Path -LiteralPath $env:DIROUT).Path
        $releaseDir = Join-Path -Path $outputDir -ChildPath 'release'
        New-Item -ItemType Directory -Path $releaseDir -Force | Out-Null
    }
    else {
        Write-Host "DIROUT not set, built executables will not be copied." -ForegroundColor Yellow
    }

    $builtTargets = New-Object System.Collections.Generic.List[string]
    $failedTargets = New-Object System.Collections.Generic.List[string]

    $s4_version = Get-ProgramVersion
    if (-not ($s4_version)) {
        Write-Error "Could not determine s4 version from " + $PROJECT_MANIFEST_ITEM.FullName
        exit 1
    }

    foreach ($target in $WindowsTargets) {
        Write-Host ''
        Write-Line
        Write-Host "+ cargo cross build --target $target" @CrossArgs -ForegroundColor Green

        $env:S4_BUILD_REGEX_PRINT = '1'
        & cargo cross build --target $target @CrossArgs

        if ($LASTEXITCODE -ne 0) {
            Write-Warning "Build failed for $target"
            $failedTargets.Add($target)
            continue
        }
        $builtTargets.Add($target)

        if (-not ($outputDir)) {
            Write-Host "Skipping copying built executable for $target since DIROUT is not set."
            continue
        }

        $exePath = Join-Path -Path (Join-Path -Path 'target' -ChildPath $target) -ChildPath "$buildProfile\\${BIN}.exe"
        if (-not (Test-Path -LiteralPath $exePath)) {
            Write-Warning "Built executable not found at $exePath"
            continue
        }

        $destPath = Join-Path -Path $outputDir -ChildPath "${BIN}_${target}_v${s4_version}.exe"
        Remove-Item -Path $destPath -ErrorAction Ignore -Force
        Copy-Item -Verbose -LiteralPath $exePath -Destination $destPath -Force
        Set-FileNoWrite -Path $destPath
        Write-Sha256ChecksumFile -FilePath $destPath

        $destPathBin = Join-Path -Path $outputDir -ChildPath "${BIN}.exe"
        $shaPathBin = Join-Path -Path $outputDir -ChildPath "${BIN}.exe.sha256"
        Remove-Item -Path $destPathBin -ErrorAction Ignore -Force
        Copy-Item -Verbose -LiteralPath $exePath -Destination $destPathBin -Force
        Set-FileNoWrite -Path $destPathBin
        Write-Sha256ChecksumFile -FilePath $destPathBin

        # Archive the copied executable and its checksum file.
        $zipPath = Join-Path -Path $releaseDir -ChildPath "${BIN}_${target}_v${s4_version}.zip"
        if (Test-Path -LiteralPath $zipPath) {
            Remove-Item -LiteralPath $zipPath -Force
        }
        Push-Location $outputDir
        Compress-Archive -LiteralPath @($destPathBin, $shaPathBin) -DestinationPath $zipPath -CompressionLevel Optimal
        Pop-Location
        Write-Sha256ChecksumFile -FilePath $zipPath
        Set-FileNoWrite -Path $zipPath

        Remove-Item -Verbose -Path $destPathBin -Force
        Remove-Item -Verbose -Path $shaPathBin -Force
    }

    Write-Line
    Write-Host ''
    Write-Host "Built:  $($builtTargets.Count)"
    Write-Host "Failed: $($failedTargets.Count)"
    Write-Host ''

    if ($builtTargets.Count -gt 0) {
        Write-Host 'Built targets:'
        foreach ($built in $builtTargets) {
            Write-Host "  $built" -ForegroundColor Green
        }
    }

    if ($failedTargets.Count -gt 0) {
        Write-Host 'Failed targets:'
        foreach ($failed in $failedTargets) {
            Write-Host "  $failed" -ForegroundColor Red
        }
        exit 1
    }

    exit 0
}
catch {
    Write-Host -ForegroundColor Red 'cross-builds.ps1 failed with an exception:'
    Write-Host -ForegroundColor Red $_
    exit 1
}
finally {
    Set-PSDebug -Trace 0
    Set-StrictMode -Off
    if ($null -ne $startLocation) {
        Set-Location $startLocation
    }
    if ($null -ne $erroractionpreference_) {
        $ErrorActionPreference = $erroractionpreference_
    }
}
