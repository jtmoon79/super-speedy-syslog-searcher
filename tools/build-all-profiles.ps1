#!/usr/bin/env pwsh
#
# Build all important profiles, up to three at a time in parallel.
# Usage: pwsh .\tools\build-all-profiles.ps1
# Wait for every build in a batch; stop on any build or version-check failure.

$ErrorActionPreference = 'Stop'
Set-StrictMode -Version 3.0

function Invoke-BuildBatch {
    param(
        [Parameter(Mandatory = $true)]
        [string[]] $Profiles,
        [switch] $Check
    )

    $builds = [System.Collections.Generic.List[object]]::new()
    try {
        foreach ($profile_ in $Profiles) {
            $cargoArgs = @('build', '--locked', '--profile', $profile_)
            if ($profile_ -in @('jemalloc', 'rpmalloc', 'mimalloc', 'tcmalloc', 'alloc_tracker')) {
                $cargoArgs += @('--features', $profile_)
            }

            $rustFlags = $env:RUSTFLAGS
            try {
                if ($profile_ -in @('flamegraph', 'valgrind')) {
                    $env:RUSTFLAGS = '-g'
                }
                Write-Host "cargo $($cargoArgs -join ' ')"
                $process = Start-Process -FilePath $CargoPath -ArgumentList $cargoArgs `
                    -WorkingDirectory $ProjectRoot -NoNewWindow -PassThru
                $builds.Add([pscustomobject]@{
                    Process = $process
                    Command = "cargo $($cargoArgs -join ' ')"
                })
            }
            finally {
                $env:RUSTFLAGS = $rustFlags
            }
        }

        if ($Check) {
            Write-Host 'cargo check --locked'
            $process = Start-Process -FilePath $CargoPath -ArgumentList @('check', '--locked') `
                -WorkingDirectory $ProjectRoot -NoNewWindow -PassThru
            $builds.Add([pscustomobject]@{
                Process = $process
                Command = 'cargo check --locked'
            })
        }

        $failures = @(
            foreach ($build in $builds) {
                $build.Process.WaitForExit()
                if ($build.Process.ExitCode -ne 0) {
                    "$($build.Command) exited with code $($build.Process.ExitCode)"
                }
            }
        )
        if ($failures.Count -gt 0) {
            throw ($failures -join '; ')
        }
    }
    finally {
        # Also drain started builds if launching a later process fails.
        foreach ($build in $builds) {
            try {
                $build.Process.WaitForExit()
            }
            finally {
                $build.Process.Dispose()
            }
        }
    }

    foreach ($profile_ in $Profiles) {
        $outputProfile = if ($profile_ -eq 'dev') { 'debug' } else { $profile_ }
        $binary = Join-Path (Join-Path (Join-Path $ProjectRoot 'target') $outputProfile) $BinaryName
        Write-Host "PS> $binary --version" -ForegroundColor Green
        & $binary --version
        if ($LASTEXITCODE -ne 0) {
            throw "$binary --version exited with code $LASTEXITCODE"
        }
    }
}

$ProjectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..')).Path
$CargoPath = (Get-Command cargo -CommandType Application -ErrorAction Stop | Select-Object -First 1).Source
$BinaryName = if ([Environment]::OSVersion.Platform -eq [PlatformID]::Win32NT) { 's4.exe' } else { 's4' }
$buildPrint = ${env:S4_BUILD_PRINT}
$buildRegexPrint = ${env:S4_BUILD_REGEX_PRINT}

Push-Location -LiteralPath $ProjectRoot
try {
    ${env:S4_BUILD_PRINT} = '1'
    ${env:S4_BUILD_REGEX_PRINT} = '1'

    Invoke-BuildBatch -Profiles @('dev', 'release')
    Invoke-BuildBatch -Profiles @('rpmalloc', 'mimalloc')
    Invoke-BuildBatch -Profiles @('alloc_tracker', 'flamegraph')
    Invoke-BuildBatch -Profiles @('valgrind') -Check
    Invoke-BuildBatch -Profiles @('release_Opt0')
    Invoke-BuildBatch -Profiles @('release_O0_cgu1', 'release_O0_cgu512_pa')
    Invoke-BuildBatch -Profiles @('release_O1_cgu1', 'release_O1_cgu256_pa', 'release_O1_cgu512_pa')
    Invoke-BuildBatch -Profiles @('release_O2_cgu1', 'release_O2_cgu256_pa', 'release_O2_cgu512_pa')
    Invoke-BuildBatch -Profiles @('release_O3_cgu1_pa', 'release_O3_cgu256_pa', 'release_O3_cgu512_pa')
}
finally {
    ${env:S4_BUILD_PRINT} = $buildPrint
    ${env:S4_BUILD_REGEX_PRINT} = $buildRegexPrint
    Pop-Location
}
