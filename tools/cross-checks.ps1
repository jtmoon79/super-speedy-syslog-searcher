#!/usr/bin/env pwsh
#
# Run `cargo cross build` for all Windows targets from tools/cross-checks.sh.
# Any script arguments are forwarded to `cargo cross build`.
# If DIROUT is set, copy built s4.exe files there using target-specific names.
#
# Usage:
#   pwsh ./tools/cross-checks.ps1
#   pwsh ./tools/cross-checks.ps1 --release
#   $env:DIROUT = 'C:\temp\s4-cross-builds'; pwsh ./tools/cross-checks.ps1 --release

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
        [string] $FilePath,

        [Parameter(Mandatory = $true)]
        [string] $OutputPath
    )

    $hash = (Get-FileHash -LiteralPath $FilePath -Algorithm SHA256).Hash.ToLowerInvariant()
    $fileName = Split-Path -Path $FilePath -Leaf
    # Match common sha256sum text format: "<hash><two spaces><filename>"
    Set-Content -LiteralPath $OutputPath -Value ("{0}  {1}" -f $hash, $fileName)
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
    if ($env:DIROUT) {
        New-Item -ItemType Directory -Path $env:DIROUT -Force | Out-Null
        $outputDir = (Resolve-Path -LiteralPath $env:DIROUT).Path
    }

    $builtTargets = New-Object System.Collections.Generic.List[string]
    $failedTargets = New-Object System.Collections.Generic.List[string]

    foreach ($target in $WindowsTargets) {
        Write-Host ''
        Write-Host "Building target $target ..."

        $env:S4_BUILD_REGEX_PRINT = '1'
        & cargo cross build --target $target @CrossArgs

        if ($LASTEXITCODE -ne 0) {
            Write-Warning "Build failed for $target"
            $failedTargets.Add($target)
            continue
        }

        $builtTargets.Add($target)

        if ($outputDir) {
            $exePath = Join-Path -Path (Join-Path -Path 'target' -ChildPath $target) -ChildPath "$buildProfile\\${BIN}.exe"
            if (Test-Path -LiteralPath $exePath) {
                $destName = "${BIN}__${target}__${buildProfile}.exe"
                $destPath = Join-Path -Path $outputDir -ChildPath $destName
                Copy-Item -Verbose -LiteralPath $exePath -Destination $destPath -Force

                $destNameBin = "${BIN}.exe"
                $destPathBin = Join-Path -Path $outputDir -ChildPath $destNameBin
                Remove-Item -Path $destPathBin -ErrorAction Ignore -Force
                Copy-Item -Verbose -LiteralPath $exePath -Destination $destPathBin -Force

                $shaName = "${destName}.sha256"
                $shaPath = Join-Path -Path $outputDir -ChildPath $shaName
                Write-Sha256ChecksumFile -FilePath $destPath -OutputPath $shaPath

                $shaNameBin = "${BIN}.sha256"
                $shaPathBin = Join-Path -Path $outputDir -ChildPath $shaNameBin
                Write-Sha256ChecksumFile -FilePath $destPathBin -OutputPath $shaPathBin

                $zipName = "${BIN}__${target}__${buildProfile}.zip"
                $zipPath = Join-Path -Path $outputDir -ChildPath $zipName
                if (Test-Path -LiteralPath $zipPath) {
                    Remove-Item -LiteralPath $zipPath -Force
                }

                # Archive the copied executable and its checksum file.
                Push-Location $outputDir
                Compress-Archive -Verbose -LiteralPath @($destNameBin, $shaNameBin) -DestinationPath $zipPath -CompressionLevel Optimal
                Pop-Location

                $zipShaPath = "$zipPath.sha256"
                Write-Sha256ChecksumFile -FilePath $zipPath -OutputPath $zipShaPath

                Remove-Item -Path $destPathBin, $shaPathBin -ErrorAction Ignore -Force
            }
            else {
                Write-Warning "Built executable not found at $exePath"
            }
        }
    }

    Write-Host ''
    Write-Host "Built:  $($builtTargets.Count)"
    Write-Host "Failed: $($failedTargets.Count)"

    if ($failedTargets.Count -gt 0) {
        Write-Host 'Failed targets:'
        foreach ($failed in $failedTargets) {
            Write-Host "  $failed"
        }
        exit 1
    }

    exit 0
}
catch {
    Write-Host -ForegroundColor Red 'cross-checks.ps1 failed with an exception:'
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
