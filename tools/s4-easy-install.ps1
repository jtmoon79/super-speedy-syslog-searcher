#!powershell
#
# s4-easy-install.ps1

<#
.SYNOPSIS
    Download and install the Super Speedy Syslog Searcher (s4) binary program for Windows.
.DESCRIPTION
    This script will attempt to download the latest release of s4 for Windows from GitHub, verify the SHA-256 checksums of the downloaded files, and install the s4.exe binary to a writable directory in the user's PATH.

    To run:
    PS> Invoke-WebRequest -Uri https://raw.githubusercontent.com/jtmoon79/super-speedy-syslog-searcher/refs/heads/main/tools/s4-easy-install.ps1 -OutFile s4-easy-install.ps1
    PS> .\s4-easy-install.ps1 -?
.PARAMETER Version
    Version of Super Speedy Syslog Searcher to install.
.PARAMETER Triple
    Optional override of target triple; e.g. `x86_64-pc-windows-gnu` vs `x86_64-pc-windows-msvc`.
.PARAMETER Abi
    Optional override of target triple last ABI field; e.g. `gnu` vs `msvc` for Windows targets.
.PARAMETER InstallDir
    Optional override of install directory; must be a writable directory in the user's PATH.
.PARAMETER Test
    Test the download and run, do not install.
.PARAMETER trace
    Turn on debug tracing.
.PARAMETER Help
    Show this help message and exit.
.LINK
    https://github.com/jtmoon79/super-speedy-syslog-searcher
.NOTES
    Author: James Thomas Moon
#>


[CmdletBinding()]
param(
    [Parameter()]
    [string] $Version = '0.9.82',
    [Parameter()]
    [string] $Triple,
    [Parameter()]
    [string] $Abi,
    [Parameter()]
    [string] $InstallDir,
    [Parameter()]
    [switch] $Test,
    [Parameter()]
    [switch] $trace,
    [Parameter()]
    [Alias('?')]
    [switch] $Help
)

if ($Help) {
    $helpTarget = $PSCommandPath
    if ([string]::IsNullOrWhiteSpace($helpTarget)) {
        $helpTarget = $MyInvocation.MyCommand.Path
    }
    $helpTargetName = $helpTarget.Split([IO.Path]::DirectorySeparatorChar)[-1]
    Get-Help -Name $helpTargetName -Path $helpTarget -Detailed -ErrorAction SilentlyContinue
    return
}

if ($trace) {
    Set-PSDebug -Trace 1
}

$ErrorActionPreference = 'Stop'

$SecurityProtocolType = [Net.SecurityProtocolType]::Tls12

# see target platform triples at
# https://doc.rust-lang.org/nightly/rustc/platform-support.html

$TargetTriplesAMD64 = @(
    'x86_64-pc-windows-msvc',
    'x86_64-pc-windows-gnu',
    'x86_64-pc-windows-gnullvm',
    'x86_64-win7-windows-gnu',
    'x86_64-win7-windows-msvc',
    'x86_64-uwp-windows-msvc',
    'x86_64-uwp-windows-gnu'
)
$TargetTriplesI686 = @(
    'i686-pc-windows-msvc',
    'i686-pc-windows-gnu',
    'i686-pc-windows-gnullvm',
    'i686-win7-windows-gnu',
    'i686-win7-windows-msvc'
)
$TargetTriplesAarch64 = @(
    'aarch64-pc-windows-msvc',
    'aarch64-pc-windows-gnullvm',
    'arm64ec-pc-windows-msvc',
    'aarch64-uwp-windows-msvc'
)
# try to download and run these target triples until one succeeds
$TargetTriples = @()

if (-not [string]::IsNullOrWhiteSpace($Triple)) {
    $TargetTriples = @($Triple)
    if (-not [string]::IsNullOrWhiteSpace($Abi)) {
        $Abi = $Abi.Trim().ToLower()
        Write-Warning "ABI override '${Abi}' is ignored because target triple override '${Triple}' was provided."
    }
} else {
    switch ("${env:PROCESSOR_ARCHITECTURE}".toLower()) {
        'amd64' { $TargetTriples = $TargetTriplesAMD64 }
        'x86' { $TargetTriples = $TargetTriplesI686 }
        'arm64' { $TargetTriples = $TargetTriplesAarch64 }
        default {
            Write-Warning "unrecognized processor architecture '$(${env:PROCESSOR_ARCHITECTURE})'; will attempt all target triples"
            $TargetTriples = $TargetTriplesAMD64 + $TargetTriplesI686 + $TargetTriplesAarch64
        }
    }
}

$TargetsAttempted = @()

if ([string]::IsNullOrWhiteSpace($Version)) {
    if (-not [string]::IsNullOrWhiteSpace($env:VER)) {
        $Version = $env:VER
    }
    else {
        $Version = '0.9.82' # PROJECT VERSION LAST PUBLISHED
    }
}

function Write-Info {
    [OutputType([System.Void])]
    param([Parameter(Mandatory = $true)][string] $Message)
    Write-Host "info: $Message" -ForegroundColor Green
}

function Test-UriExists {
    [OutputType([bool])]
    param([Parameter(Mandatory = $true)][Uri] $Uri)
    Write-Verbose "Test-UriExists('${Uri}')"

    try {
        if ($PSVersionTable.PSVersion.Major -ge 7) {
            $response = Invoke-WebRequest -UseBasicParsing -Uri $Uri -Method Head -SkipHttpErrorCheck
            return ($response.StatusCode -eq 200)
        }
        $null = Invoke-WebRequest -UseBasicParsing -Uri $Uri -Method Head
        return $true
    }
    catch {
        return $false
    }
}

function Download-File {
    [OutputType([System.Void])]
    param(
        [Parameter(Mandatory = $true)][Uri] $Uri,
        [Parameter(Mandatory = $true)][string] $OutFile
    )
    Write-Verbose "Download('${Uri}', '${OutFile}')"

    $start_time = Get-Date
    [Net.ServicePointManager]::SecurityProtocol = $SecurityProtocolType
    $ProgressPreference = 'SilentlyContinue'
    $wr1 = Invoke-WebRequest -UseBasicParsing -Uri $Uri -OutFile $OutFile
    # BUG: why does Invoke-WebRequest sometimes return no object!?
    $sc = "URI "
    if ($wr1) {
        $sc = "URI " + $wr1.StatusCode.ToString() + " "
    }
    Write-Host ($sc + $Uri.ToString() + " downloaded to temporary directory " + $OutFile)

    Write-Verbose "Downloaded time: $((Get-Date).Subtract($start_time).Seconds) second(s)"
}

function Test-WritableDirectory {
    [OutputType([bool])]
    param([Parameter(Mandatory = $true)][string] $Path)
    Write-Verbose "Test-WritableDirectory('${Path}')"

    if (-not (Test-Path -LiteralPath $Path -PathType Container)) {
        return $false
    }

    $probe = Join-Path -Path $Path -ChildPath ('.s4_write_probe_' + [Guid]::NewGuid().ToString('N') + '.tmp')
    try {
        Set-Content -LiteralPath $probe -Value 'probe' -Encoding Ascii
        Remove-Item -LiteralPath $probe -Force -ErrorAction SilentlyContinue
        return $true
    }
    catch {
        return $false
    }
}

function Choose-InstallDirectory {
    [OutputType([string])]
    param()

    if (-not [string]::IsNullOrWhiteSpace($InstallDir)) {
        if (Test-WritableDirectory -Path $InstallDir) {
            Write-Verbose "Choose-InstallDirectory: selected override install directory '$InstallDir'"
            return $InstallDir
        }
        else {
            Write-Error "-InstallDir '$InstallDir' is not writable or not a directory"
            return $null
        }
    }

    $homePath = [Environment]::GetFolderPath('UserProfile')
    $localAppData = [Environment]::GetFolderPath('LocalApplicationData')
    $priority = @(
        (Join-Path -Path $homePath -ChildPath '.cargo/bin'),
        (Join-Path -Path $homePath -ChildPath 'scoop/shims'),
        (Join-Path -Path $homePath -ChildPath 'bin'),
        (Join-Path -Path $localAppData -ChildPath 'Microsoft/WindowsApps')
    )

    foreach ($dir in $priority) {
        if (Test-WritableDirectory -Path $dir) {
            return $dir
        }
    }

    $seen = [System.Collections.Generic.HashSet[string]]::new([System.StringComparer]::OrdinalIgnoreCase)
    $pathEntries = @($env:PATH -split [IO.Path]::PathSeparator)
    foreach ($entry in $pathEntries) {
        if ([string]::IsNullOrWhiteSpace($entry)) {
            continue
        }

        $trimmed = $entry.Trim()
        if ($seen.Contains($trimmed)) {
            continue
        }
        [void]$seen.Add($trimmed)

        if (Test-WritableDirectory -Path $trimmed) {
            Write-Verbose "Choose-InstallDirectory: selected '$trimmed'"
            return $trimmed
        }
    }

    return $null
}

function Get-ExpectedSha256 {
    [OutputType([string])]
    param(
        [Parameter(Mandatory = $true)][string] $ChecksumFile,
        [Parameter(Mandatory = $true)][string] $ExpectedFileName
    )

    $line = (Get-Content -LiteralPath $ChecksumFile -ErrorAction Stop | Select-Object -First 1)
    if ([string]::IsNullOrWhiteSpace($line)) {
        Write-Error "checksum file '$ChecksumFile' is empty"
    }

    $m = [regex]::Match($line, '^\s*([A-Fa-f0-9]{64})\s+\*?(\S+)\s*$')
    if (-not $m.Success) {
        Write-Error "checksum file '$ChecksumFile' has unexpected format"
    }

    $name = $m.Groups[2].Value
    if ($name -ne $ExpectedFileName) {
        Write-Warning "checksum file references '$name', expected '$ExpectedFileName'"
    }
    Write-Verbose "Get-ExpectedSha256: extracted SHA-256 '$($m.Groups[1].Value)' for file '$name' from checksum file '$ChecksumFile'"

    return $m.Groups[1].Value.ToLowerInvariant()
}

function Confirm-FileSha256 {
    [OutputType([System.Void])]
    param(
        [Parameter(Mandatory = $true)][string] $FilePath,
        [Parameter(Mandatory = $true)][string] $ExpectedSha256
    )

    $actual = (Get-FileHash -LiteralPath $FilePath -Algorithm SHA256).Hash.ToLowerInvariant()
    Write-Verbose "Confirm-FileSha256('${FilePath}', '${ExpectedSha256}'): actual SHA-256 is '$actual'"
    if ($actual -ne $ExpectedSha256.ToLowerInvariant()) {
        Write-Error "checksum mismatch for '$FilePath'. expected=$ExpectedSha256 actual=$actual"
    }
}

function Main {
    [OutputType([System.Void])]
    param()

    $startLocation = Get-Location
    $workdir = $null

    try {
        Set-StrictMode -Version 3.0

        $workdir = New-Item -ItemType Directory -Path (Join-Path -Path ([IO.Path]::GetTempPath()) -ChildPath ('easy-install.ps1.tmpd.' + [Guid]::NewGuid().ToString('N'))) -Force
        Write-Info "temporary directory is $($workdir.FullName)"

        $installDir = Choose-InstallDirectory
        if ([string]::IsNullOrWhiteSpace($installDir)) {
            Write-Error 'could not find a writable install directory.'
        }

        $selectedBinaryPath = $null
        $selectedTarget = $null

        foreach ($targetTriple in $TargetTriples) {
            Write-Host ''
            Write-Info "trying target $targetTriple ..."

            if (-not [string]::IsNullOrWhiteSpace($Abi)) {
                if ($targetTriple -notmatch '^(.*-)([^-]+)$') {
                    Write-Warning "cannot override target triple with ABI '${Abi}' because target triple '$targetTriple' does not match expected format"
                    continue
                }
                $targetTriple = "$($matches[1])${Abi}"
                Write-Info "overriding target triple with ABI '${Abi}'; new target triple is '${targetTriple}'"
                Write-Host ''
            }

            if ($TargetsAttempted.Contains($targetTriple)) {
                Write-Verbose "skipping target triple '$targetTriple' because it has already been attempted"
                continue
            }
            $TargetsAttempted += $targetTriple

            $candidateDir = New-Item -ItemType Directory -Path (Join-Path -Path $workdir.FullName -ChildPath $targetTriple) -Force
            $zipName = "s4_${targetTriple}_v${Version}.zip"
            $zipPath = Join-Path -Path $candidateDir.FullName -ChildPath $zipName
            $urlZip = [Uri]("https://github.com/jtmoon79/super-speedy-syslog-searcher/releases/download/{0}/{1}" -f $Version, $zipName)

            $checksumName = "$zipName.sha256"
            $checksumPath = Join-Path -Path $candidateDir.FullName -ChildPath $checksumName
            $urlChecksum = [Uri]($urlZip.ToString() + '.sha256')

            try {
                Write-Info "download release $Version for target $targetTriple ..."
                Download-File -Uri $urlZip -OutFile $zipPath

                if (-not (Test-UriExists -Uri $urlChecksum)) {
                    Write-Warning "checksum file not found at $urlChecksum"
                    continue
                }

                Write-Info 'download checksum file ...'
                Download-File -Uri $urlChecksum -OutFile $checksumPath
                Write-Info 'verify SHA-256 checksum of zip file ...'
                $expectedZipSha = Get-ExpectedSha256 -ChecksumFile $checksumPath -ExpectedFileName $zipName
                Confirm-FileSha256 -FilePath $zipPath -ExpectedSha256 $expectedZipSha

                Write-Info 'extract archive ...'
                Write-Verbose "Expand-Archive -LiteralPath '${zipPath}' -DestinationPath '$($candidateDir.FullName)' -Force"
                Expand-Archive -LiteralPath $zipPath -DestinationPath $candidateDir.FullName -Force

                $binaryPath = Join-Path -Path $candidateDir.FullName -ChildPath 's4.exe'
                if (-not (Test-Path -LiteralPath $binaryPath -PathType Leaf)) {
                    Write-Warning "downloaded archive for $targetTriple did not contain s4.exe"
                    continue
                }

                $binaryChecksumPath = Join-Path -Path $candidateDir.FullName -ChildPath 's4.exe.sha256'
                if (-not (Test-Path -LiteralPath $binaryChecksumPath -PathType Leaf)) {
                    Write-Warning "checksum file 's4.exe.sha256' was not found in archive for $targetTriple"
                    continue
                }

                Write-Info 'verify SHA-256 checksum of binary file ...'
                $expectedBinarySha = Get-ExpectedSha256 -ChecksumFile $binaryChecksumPath -ExpectedFileName 's4.exe'
                Confirm-FileSha256 -FilePath $binaryPath -ExpectedSha256 $expectedBinarySha

                Write-Info 'check downloaded binary can run ...'
                Push-Location -Path $candidateDir.FullName
                try {
                    # confirm this binary can be executed, if it does then it's
                    # the correct platform target
                    & .\s4.exe --version
                    if ($LASTEXITCODE -ne 0) {
                        Write-Warning "candidate $targetTriple failed with exit code $LASTEXITCODE"
                        continue
                    }
                }
                finally {
                    Pop-Location
                }

                $selectedBinaryPath = $binaryPath
                $selectedTarget = $targetTriple
                break
            }
            catch {
                Write-Warning "candidate $targetTriple failed: $($_.Exception.Message)"
                continue
            }
        }

        if ([string]::IsNullOrWhiteSpace($selectedBinaryPath)) {
            Write-Error 'none of the candidate Windows binaries could be executed successfully.'
        }

        $installPath = Join-Path -Path $installDir -ChildPath 's4.exe'
        Write-Info "install binary to $installPath"
        if ($Test) {
            throw "-Test: skipping installation"
        }

        Copy-Item -LiteralPath $selectedBinaryPath -Destination $installPath -Force

        Write-Info 'verify installed binary path ...'
        $resolved = Get-Command -Name 's4.exe' -ErrorAction SilentlyContinue
        if ($null -ne $resolved) {
            Write-Host $resolved.Source -ForegroundColor Green
        }
        else {
            Write-Warning "'s4.exe' was not found in current PATH."
        }

        Write-Host ''
        Write-Info 'check installed binary can run ...'
        & $installPath --version

        Write-Info "installed s4.exe for platform $selectedTarget version $Version to $installPath"
    }
    catch {
        Write-Host -ForegroundColor Red $_.Exception.Message
        throw
    }
    finally {
        Set-StrictMode -Off
        if ($null -ne $workdir -and (Test-Path -LiteralPath $workdir.FullName)) {
            Remove-Item -LiteralPath $workdir.FullName -Recurse -Force -ErrorAction SilentlyContinue
        }
        Set-Location $startLocation
    }
}

Main
