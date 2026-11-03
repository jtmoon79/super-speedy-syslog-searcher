#!/usr/bin/env pwsh
#
# Compare Windows EVTX, ODL, and ETL readers using hyperfine 2.
#
# Usage:
#   .\.venv\Scripts\Activate.ps1
#   .\tools\compare-log-mergers\compare.ps1
#   .\tools\compare-log-mergers\compare.ps1 -Sections evtx,etl -EvtxFiles C:\logs\one.evtx,C:\logs\two.evtx
#
# Optional environment variables:
#   PROGRAMS_S4_LISTING   Newline-separated s4 paths, optionally followed by
#                         a tab and a label. Relative paths start at the repo.
#   PROGRAM_EVTX_DUMP     Path to the Rust evtx_dump.exe.
#   PROGRAM_EVTXECMD      Path to EvtxECmd.exe.
#   PROGRAM_ODL_PY        Path to ydkhatri's odl.py.
#   PROGRAM_ODE           Path to Beercow's OneDriveExplorer.py.
#   PROGRAM_ODE_CSTRUCTS  Directory of OneDriveExplorer cstruct files.
#   HRUNS                 Number of measured runs, default 10.
#   DIROUT                Optional additional directory for the Markdown files.
#
# Python comparisons require an activated venv. OneDriveExplorer is timed
# through its parser API with Python optimization level 1; its CLI can exit
# without parsing --LOGS, and Cerberus cannot run at optimization level 2.
# Each compare-<section>.md is written in the caller's current directory.
# Reports use project-relative input paths, including ../ for files outside
# the project. Inputs must be on the project's volume. Multiple EVTX inputs
# are listed with numbers matching the result-row suffixes.

[CmdletBinding()]
param(
    [string[]] $EvtxFiles,
    [string] $OdlFile,
    [string] $EtlFile,
    [ValidateNotNullOrEmpty()]
    [ValidateSet('evtx', 'odl', 'etl')]
    [string[]] $Sections = @('evtx', 'odl', 'etl')
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Quote-CommandArgument {
    [OutputType([string])]
    param([AllowEmptyString()][string] $Value)

    $quote = "'"
    $escapedQuote = "'`"'`"'"
    return $quote + $Value.Replace($quote, $escapedQuote) + $quote
}

function Write-Cmd {
    [OutputType([System.Void])]
    param([string] $Program, [string[]] $CommandArgs)

    $parts = @($Program) + $CommandArgs
    $display = ($parts | ForEach-Object { "'" + ([string] $_).Replace("'", "''") + "'" }) -join ' '
    [Console]::Error.WriteLine("PS> & $display")
}

function Write-Notice {
    [OutputType([System.Void])]
    param([string] $Message)

    [Console]::Error.WriteLine("WARNING: $Message")
}

function Invoke-Checked {
    [OutputType([pscustomobject])]
    param(
        [string] $Program,
        [string[]] $CommandArgs = @(),
        [switch] $AllowFailure
    )

    # Inspect exit codes ourselves, including expected version-lookup failures.
    $PSNativeCommandUseErrorActionPreference = $false
    Write-Cmd $Program $CommandArgs
    $lines = @(& $Program @CommandArgs 2>&1)
    $exitCode = $LASTEXITCODE
    $stdout = [System.Collections.Generic.List[string]]::new()
    foreach ($line in $lines) {
        [Console]::Error.WriteLine([string] $line)
        if ($line -isnot [System.Management.Automation.ErrorRecord]) {
            $stdout.Add([string] $line)
        }
    }

    if ($exitCode -ne 0 -and -not $AllowFailure) {
        throw "$Program exited with code $exitCode."
    }
    return [pscustomobject]@{
        Success = ($exitCode -eq 0)
        Output = ($stdout -join "`n")
    }
}

function Require-File {
    [OutputType([string])]
    param([string] $Path, [string] $Hint)

    if ([string]::IsNullOrWhiteSpace($Path) -or
        -not (Test-Path -LiteralPath $Path -PathType Leaf)) {
        throw "$Hint Path: $Path"
    }
    return (Resolve-Path -LiteralPath $Path).ProviderPath
}

function ConvertTo-ReportInputPath {
    [OutputType([string])]
    param([string] $ProjectRoot, [string] $Path)

    $relative = [IO.Path]::GetRelativePath($ProjectRoot, $Path)
    if ([IO.Path]::IsPathRooted($relative)) {
        throw "Input cannot be reported relative to the project directory (different volume): $Path"
    }
    return $relative.Replace('\', '/')
}

function Find-Application {
    [OutputType([string])]
    param([string] $Name, [string] $Hint)

    $command = Get-Command -Name $Name -CommandType Application -ErrorAction SilentlyContinue |
        Select-Object -First 1
    if (-not $command) {
        throw "$Name was not found. $Hint"
    }
    return $command.Source
}

function Get-TracerptPath {
    [OutputType([string])]
    param()

    if ([Environment]::Is64BitOperatingSystem -and -not [Environment]::Is64BitProcess) {
        $sysnative = Join-Path $env:SystemRoot 'Sysnative\tracerpt.exe'
        if (Test-Path -LiteralPath $sysnative -PathType Leaf) {
            return $sysnative
        }
    }

    $system32 = Join-Path $env:SystemRoot 'System32\tracerpt.exe'
    if (Test-Path -LiteralPath $system32 -PathType Leaf) {
        return $system32
    }
    return Find-Application 'tracerpt.exe' 'Install the Windows tracing tools.'
}

function Get-MatchedValue {
    [OutputType([string])]
    param([string] $Text, [string] $Pattern)

    $match = [regex]::Match($Text, $Pattern)
    if ($match.Success) {
        return $match.Groups[1].Value.Trim()
    }
    return ''
}

function Get-VersionField {
    [OutputType([string])]
    param([string] $Text, [string] $Name)

    return Get-MatchedValue $Text ("(?m)^" + [regex]::Escape($Name) + ':[ \t]*([^\r\n]*)')
}

function Get-FirstVersion {
    [OutputType([string])]
    param([string] $Text)

    return Get-MatchedValue $Text '(?<!\d)(\d+(?:\.\d+)+)'
}

function Get-RustPlatform {
    [OutputType([string])]
    param([string] $Text)

    $platform = Get-MatchedValue $Text '(?i)\b([a-z0-9_]+-[a-z0-9_]+-(?:windows|linux|darwin|freebsd)(?:-[a-z0-9_]+)?)\b'
    if ($platform) {
        return $platform
    }
    return 'Windows'
}

function Get-FileVersion {
    [OutputType([string])]
    param([string] $Path, [switch] $ProductFirst)

    $info = (Get-Item -LiteralPath $Path).VersionInfo
    $values = if ($ProductFirst) {
        @($info.ProductVersion, $info.FileVersion)
    }
    else {
        @($info.FileVersion, $info.ProductVersion)
    }
    foreach ($value in $values) {
        $version = Get-FirstVersion ([string] $value)
        if ($version) {
            return $version
        }
    }
    return ''
}

function ConvertFrom-S4Version {
    [OutputType([pscustomobject])]
    param([string] $Text, [string] $Path)

    $version = Get-VersionField $Text 'Version'
    $allocator = Get-VersionField $Text 'Allocator'
    $platform = Get-VersionField $Text 'Platform'
    if (-not $version) {
        $version = Get-MatchedValue $Text '(?m),[ \t]*Version[ \t]+([^,\r\n]+)'
    }
    if (-not $allocator) {
        $allocator = Get-MatchedValue $Text '(?m),[ \t]*Allocator[ \t]+([^,\r\n]+)'
    }
    if (-not $platform) {
        $platform = Get-MatchedValue ([IO.Path]::GetFileName($Path)) '^s4_(.+)_v\d'
    }

    return [pscustomobject]@{
        Version = $version
        Allocator = $allocator
        Platform = $platform
        CpuFeatures = (Get-VersionField $Text 'CPU Features')
        OptLevel = (Get-VersionField $Text 'Optimization Level')
    }
}

function Get-OdlVersion {
    [OutputType([string])]
    param([string] $Source, [string] $ImportedVersion)

    # A wildcard import can expose a dependency's __version__ as odl.__version__.
    if ($Source -match '(?m)^__version__[ \t]*=') {
        if ($ImportedVersion) {
            return $ImportedVersion
        }
        return Get-MatchedValue $Source '(?m)^__version__[ \t]*=[ \t]*.([0-9][0-9A-Za-z._-]*)'
    }
    return Get-MatchedValue $Source '(?m)^Version[ \t]*:[ \t]*([0-9][0-9A-Za-z._-]*)'
}

function Test-FiniteNumber {
    [OutputType([bool])]
    param($Value)

    return (($Value -is [int] -or $Value -is [long] -or
        $Value -is [double] -or $Value -is [decimal]) -and
        [double]::IsFinite([double] $Value))
}

function ConvertFrom-HyperfineDocument {
    [OutputType([string])]
    param($Document, [int] $Runs)

    try {
        if ($Document.schema_version -ne 2 -or @($Document.results).Count -ne 1) {
            throw 'expected schema_version 2 and exactly one result'
        }
        $result = $Document.results[0]
        $measurements = @($result.measurements)
        $summary = $result.summary.time_wall_clock
        if ($measurements.Count -ne $Runs -or $summary.count -ne $Runs -or
            $summary.unit -ne 'second') {
            throw 'missing measurements or an invalid wall-clock summary'
        }
        foreach ($measurement in $measurements) {
            if ($measurement.exit_code -ne 0 -or
                $measurement.time_wall_clock.unit -ne 'second' -or
                -not (Test-FiniteNumber $measurement.time_wall_clock.value) -or
                $measurement.time_wall_clock.value -lt 0) {
                throw 'invalid wall-clock measurement or failed benchmark'
            }
        }
        foreach ($value in @($summary.mean, $summary.min, $summary.max)) {
            if (-not (Test-FiniteNumber $value) -or $value -lt 0) {
                throw 'invalid wall-clock summary value'
            }
        }
        if ($summary.min -gt $summary.mean -or $summary.mean -gt $summary.max) {
            throw 'inconsistent wall-clock summary'
        }
        if (($Runs -eq 1 -and $null -ne $summary.stddev) -or
            ($Runs -gt 1 -and (-not (Test-FiniteNumber $summary.stddev) -or $summary.stddev -lt 0))) {
            throw 'invalid wall-clock standard deviation'
        }

        $culture = [Globalization.CultureInfo]::InvariantCulture
        $mean = [string]::Format($culture, '{0:F2}', 1000 * $summary.mean)
        $stddev = if ($null -eq $summary.stddev) {
            'N/A'
        }
        else {
            [string]::Format($culture, '{0:F2}', 1000 * $summary.stddev)
        }
        return "$mean ± $stddev"
    }
    catch {
        throw "Invalid hyperfine 2 JSON: $($_.Exception.Message)"
    }
}

function Invoke-Hyperfine {
    [OutputType([string])]
    param(
        [string] $Hyperfine,
        [string] $JsonPath,
        [int] $Runs,
        [string[]] $Command,
        [string] $Environment
    )

    $commandLine = ($Command | ForEach-Object { Quote-CommandArgument $_ }) -join ' '
    $hyperfineArgs = @(
        '-N',
        '--warmup', '2',
        '--runs', [string] $Runs,
        '--metrics', 'time_wall_clock:ms',
        '--style', 'basic',
        '--output', 'null',
        '--export-json', $JsonPath
    )
    if ($Environment) {
        $hyperfineArgs += @('--env', $Environment)
    }
    $hyperfineArgs += @('--', $commandLine)
    $null = Invoke-Checked $Hyperfine $hyperfineArgs
    $document = Get-Content -LiteralPath $JsonPath -Raw | ConvertFrom-Json
    return ConvertFrom-HyperfineDocument $document $Runs
}

function New-Row {
    [OutputType([pscustomobject])]
    param(
        [string] $Program,
        [string] $Version,
        [string] $Allocator,
        [string] $Platform,
        [string] $CpuFeatures,
        [string] $OptLevel,
        [string] $Timing
    )

    return [pscustomobject][ordered]@{
        'Program' = $Program
        'Version' = $Version
        'Allocator' = $Allocator
        'Platform' = $Platform
        'CPU Features' = $CpuFeatures
        'Opt Level' = $OptLevel
        'Wall Clock (ms)' = $Timing
    }
}

function Format-MarkdownTable {
    [OutputType([string])]
    param([object[]] $Rows)

    $columns = @('Program', 'Version', 'Allocator', 'Platform', 'CPU Features', 'Opt Level', 'Wall Clock (ms)')
    $cells = [System.Collections.Generic.List[object]]::new()
    foreach ($row in $Rows) {
        $values = @(
            foreach ($column in $columns) {
                ([string] $row.$column).Replace('\', '\\').Replace('|', '\|').Replace("`r", ' ').Replace("`n", ' ')
            }
        )
        $cells.Add($values)
    }
    $widths = @(
        for ($i = 0; $i -lt $columns.Count; $i++) {
            $width = [Math]::Max(3, $columns[$i].Length)
            foreach ($values in $cells) {
                $width = [Math]::Max($width, $values[$i].Length)
            }
            $width
        }
    )
    $header = for ($i = 0; $i -lt $columns.Count; $i++) {
        $columns[$i].PadRight($widths[$i])
    }
    $separator = for ($i = 0; $i -lt $columns.Count; $i++) {
        if ($i -eq $columns.Count - 1) {
            ('-' * ($widths[$i] - 1)) + ':'
        }
        else {
            '-' * $widths[$i]
        }
    }
    $lines = @()
    $lines += '| ' + ($header -join ' | ') + ' |'
    $lines += '| ' + ($separator -join ' | ') + ' |'
    foreach ($values in $cells) {
        $formatted = for ($i = 0; $i -lt $columns.Count; $i++) {
            if ($i -eq $columns.Count - 1) {
                $values[$i].PadLeft($widths[$i])
            }
            else {
                $values[$i].PadRight($widths[$i])
            }
        }
        $lines += '| ' + ($formatted -join ' | ') + ' |'
    }
    return $lines -join "`n"
}

function Write-Section {
    [OutputType([string])]
    param(
        [string] $Name,
        [string[]] $Inputs,
        [object[]] $Rows,
        [string] $Note,
        [string] $OutputDirectory,
        [string] $CopyDirectory
    )

    $inputText = if ($Inputs.Count -eq 1) {
        "Input: ``$($Inputs[0])``"
    }
    else {
        $entries = for ($i = 0; $i -lt $Inputs.Count; $i++) {
            "- [$($i + 1)] ``$($Inputs[$i])``"
        }
        "Inputs:`n" + ($entries -join "`n")
    }
    $markdown = "## $($Name.ToUpperInvariant())`n`n$inputText`n`n$(Format-MarkdownTable $Rows)"
    if ($Note) {
        $markdown += "`n`n$Note"
    }
    $markdown += "`n"

    [Console]::Error.WriteLine("`n$($Name.ToUpperInvariant()): $($Inputs -join ', ')")
    [Console]::Error.WriteLine(($Rows | Format-Table -AutoSize | Out-String -Width 4096).TrimEnd())
    $file = Join-Path $OutputDirectory "compare-$Name.md"
    Set-Content -LiteralPath $file -Value $markdown -Encoding utf8 -NoNewline
    if ($CopyDirectory) {
        if (-not (Test-Path -LiteralPath $CopyDirectory -PathType Container)) {
            New-Item -ItemType Directory -Path $CopyDirectory -Force | Out-Null
        }
        $copy = Join-Path $CopyDirectory "compare-$Name.md"
        if (-not [string]::Equals($file, $copy, [StringComparison]::OrdinalIgnoreCase)) {
            Copy-Item -LiteralPath $file -Destination $copy -Force
        }
    }
    Write-Output $markdown
}

function Invoke-Comparison {
    [OutputType([string])]
    param([string[]] $EvtxInputs, [string] $OdlInput, [string] $EtlInput, [string[]] $SelectedSections)

    if ($PSVersionTable.PSVersion.Major -lt 7 -or -not $IsWindows) {
        throw 'This comparison requires Windows and PowerShell 7.'
    }
    $projectRoot = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot '..\..')).ProviderPath
    $outputDirectory = (Get-Location).ProviderPath
    $copyDirectory = if ($env:DIROUT) {
        if ([IO.Path]::IsPathRooted($env:DIROUT)) {
            [IO.Path]::GetFullPath($env:DIROUT)
        }
        else {
            [IO.Path]::GetFullPath((Join-Path $outputDirectory $env:DIROUT))
        }
    }
    else {
        ''
    }
    $runs = 10
    if ($env:HRUNS -and (-not [int]::TryParse($env:HRUNS, [ref] $runs) -or $runs -lt 1)) {
        throw 'HRUNS must be a positive integer.'
    }

    $hyperfine = Find-Application 'hyperfine.exe' 'Install hyperfine 2.x and add it to PATH.'
    $hyperfineVersion = Invoke-Checked $hyperfine @('--version')
    if ($hyperfineVersion.Output -notmatch '(?m)^hyperfine 2\.') {
        throw 'hyperfine 2.x is required.'
    }

    $entries = [System.Collections.Generic.List[object]]::new()
    if ($env:PROGRAMS_S4_LISTING) {
        foreach ($line in ($env:PROGRAMS_S4_LISTING -split '\r?\n')) {
            if (-not $line.Trim() -or $line.TrimStart().StartsWith('#')) {
                continue
            }
            $parts = $line -split "`t", 2
            $path = $parts[0].Trim()
            if (-not [IO.Path]::IsPathRooted($path)) {
                $path = Join-Path $projectRoot $path
            }
            $extra = if ($parts.Count -gt 1) { $parts[1].Trim() } else { '' }
            $entries.Add([pscustomobject]@{ Path = $path; Extra = $extra })
        }
    }
    else {
        $target = Join-Path $projectRoot 'target'
        if (Test-Path -LiteralPath $target -PathType Container) {
            foreach ($directory in (Get-ChildItem -LiteralPath $target -Directory | Sort-Object Name)) {
                if ($directory.Name -notmatch '^(release(?:_.+)?|jemalloc|mimalloc|rpmalloc|tcmalloc)$') {
                    continue
                }
                $path = Join-Path $directory.FullName 's4.exe'
                if (Test-Path -LiteralPath $path -PathType Leaf) {
                    $entries.Add([pscustomobject]@{ Path = $path; Extra = "[$($directory.Name)]" })
                }
            }
        }
    }
    if ($entries.Count -eq 0) {
        throw 'No s4 builds found. Set PROGRAMS_S4_LISTING or build s4.'
    }
    foreach ($entry in $entries) {
        $entry.Path = Require-File $entry.Path 'An s4 build from PROGRAMS_S4_LISTING is missing.'
    }

    $evtxPaths = @()
    if ('evtx' -in $SelectedSections) {
        if (-not $EvtxInputs) {
            $EvtxInputs = @(Join-Path $projectRoot 'logs\programs\evtx\Microsoft-Windows-Kernel-PnP__Configuration.evtx')
        }
        $evtxPaths = @(foreach ($path in $EvtxInputs) { Require-File $path 'EVTX input is missing.' })
        $evtxReportInputs = @(foreach ($path in $evtxPaths) { ConvertTo-ReportInputPath $projectRoot $path })
        $rustEvtx = $env:PROGRAM_EVTX_DUMP
        if (-not $rustEvtx) {
            $cargoEvtx = Join-Path $HOME '.cargo\bin\evtx_dump.exe'
            $rustEvtx = if (Test-Path -LiteralPath $cargoEvtx -PathType Leaf) {
                $cargoEvtx
            }
            else {
                Find-Application 'evtx_dump.exe' 'Install it or set PROGRAM_EVTX_DUMP.'
            }
        }
        $rustEvtx = Require-File $rustEvtx 'Install evtx_dump.exe or set PROGRAM_EVTX_DUMP.'
        $evtxECmd = $env:PROGRAM_EVTXECMD
        if (-not $evtxECmd) {
            $evtxECmd = Find-Application 'EvtxECmd.exe' 'Add it to PATH or set PROGRAM_EVTXECMD.'
        }
        $evtxECmd = Require-File $evtxECmd 'Add EvtxECmd.exe to PATH or set PROGRAM_EVTXECMD.'
    }
    if ('odl' -in $SelectedSections) {
        if (-not $OdlInput) {
            $OdlInput = Join-Path $projectRoot 'logs\programs\OneDrive\Local\Microsoft\OneDrive\logs\Personal\SyncEngine-2025-12-21.1217.11144.2.odl'
        }
        $odlPath = Require-File $OdlInput 'ODL input is missing.'
        $odlReportInput = ConvertTo-ReportInputPath $projectRoot $odlPath
        $keyPath = Require-File (Join-Path (Split-Path -Parent $odlPath) 'general.keystore') 'The ODL input needs its adjacent general.keystore.'
    }
    if ('etl' -in $SelectedSections) {
        if (-not $EtlInput) {
            $EtlInput = Join-Path $projectRoot 'logs\programs\Event_Trace_Log\PITR\PointInTimeRestore.etl'
        }
        $etlPath = Require-File $EtlInput 'ETL input is missing.'
        $etlReportInput = ConvertTo-ReportInputPath $projectRoot $etlPath
        $tracerpt = Get-TracerptPath
        $pwsh = (Get-Process -Id $PID).Path
        if ([IO.Path]::GetFileName($pwsh) -ine 'pwsh.exe') {
            $pwsh = Find-Application 'pwsh.exe' 'Install PowerShell 7 and add it to PATH.'
        }
    }

    $needsPython = 'evtx' -in $SelectedSections -or 'odl' -in $SelectedSections
    if ($needsPython) {
        if (-not $env:VIRTUAL_ENV) {
            throw 'Activate a Python virtual environment first, for example: .\.venv\Scripts\Activate.ps1'
        }
        $venvPython = Require-File (Join-Path $env:VIRTUAL_ENV 'Scripts\python.exe') 'The activated venv has no Python.'
        if ('evtx' -in $SelectedSections) {
            $evtxPy = Require-File (Join-Path $env:VIRTUAL_ENV 'Scripts\evtx_dump.py') 'Install python-evtx 0.7.4 in the activated venv.'
        }
        if ('odl' -in $SelectedSections) {
            $odlPy = if ($env:PROGRAM_ODL_PY) { $env:PROGRAM_ODL_PY } else { Join-Path $env:VIRTUAL_ENV 'Scripts\odl.py' }
            $odlPy = Require-File $odlPy 'Install odl.py in the venv or set PROGRAM_ODL_PY.'
            $ode = if ($env:PROGRAM_ODE) { $env:PROGRAM_ODE } else { Join-Path $env:VIRTUAL_ENV 'Scripts\OneDriveExplorer.py' }
            $ode = Require-File $ode 'Place OneDriveExplorer.py in the venv Scripts directory or set PROGRAM_ODE.'
            if ([IO.Path]::GetFileName($ode) -ine 'OneDriveExplorer.py') {
                throw 'PROGRAM_ODE must name OneDriveExplorer.py.'
            }
            $odeDirectory = Split-Path -Parent $ode
            $cstructs = if ($env:PROGRAM_ODE_CSTRUCTS) {
                if (-not (Test-Path -LiteralPath $env:PROGRAM_ODE_CSTRUCTS -PathType Container)) {
                    throw "PROGRAM_ODE_CSTRUCTS is not a directory: $env:PROGRAM_ODE_CSTRUCTS"
                }
                (Resolve-Path -LiteralPath $env:PROGRAM_ODE_CSTRUCTS).ProviderPath
            }
            elseif (Test-Path -LiteralPath (Join-Path $odeDirectory 'cstructs') -PathType Container) {
                (Resolve-Path -LiteralPath (Join-Path $odeDirectory 'cstructs')).ProviderPath
            }
            else {
                Write-Notice 'OneDriveExplorer has no cstructs directory; fewer messages may be decoded.'
                ''
            }
        }
    }

    $temp = Join-Path ([IO.Path]::GetTempPath()) ("s4-compare-" + [guid]::NewGuid().ToString('N'))
    $oldPythonPath = $env:PYTHONPATH
    $oldPythonOptimize = $env:PYTHONOPTIMIZE
    $created = $false
    $pushed = $false
    try {
        New-Item -ItemType Directory -Path $temp | Out-Null
        $created = $true
        Push-Location -LiteralPath $temp
        $pushed = $true
        $jsonPath = Join-Path $temp 'hyperfine.json'
        $s4Programs = [System.Collections.Generic.List[object]]::new()
        foreach ($entry in $entries) {
            $check = Invoke-Checked $entry.Path @('--version')
            $info = ConvertFrom-S4Version $check.Output $entry.Path
            if (-not $info.Version) {
                Write-Notice "Could not determine the s4 version: $($entry.Path)"
            }
            if (-not $info.CpuFeatures) {
                Write-Notice "No CPU Features in s4 --version: $($entry.Path)"
            }
            $label = if ($entry.Extra) { "s4 $($entry.Extra)" } else { 's4' }
            $s4Programs.Add([pscustomobject]@{ Path = $entry.Path; Label = $label; Info = $info })
        }

        if ($needsPython) {
            $pythonInfoCode = 'import json,sys,sysconfig; print(json.dumps({"base":sys._base_executable,"site":sysconfig.get_path("purelib"),"version":sys.version.split()[0]}))'
            $pythonInfo = (Invoke-Checked $venvPython @('-c', $pythonInfoCode)).Output | ConvertFrom-Json
            $python = Require-File $pythonInfo.base 'The venv base Python interpreter is missing.'
            if (-not (Test-Path -LiteralPath $pythonInfo.site -PathType Container)) {
                throw "The venv site-packages directory is missing: $($pythonInfo.site)"
            }
            $site = (Resolve-Path -LiteralPath $pythonInfo.site).ProviderPath
            $pythonPlatform = "Python $($pythonInfo.version)"
            $env:PYTHONPATH = $site
            $env:PYTHONOPTIMIZE = '2'
            $compileArgs = @('-m', 'compileall', '-q')
            if ('odl' -in $SelectedSections) {
                $compileArgs += @('-o', '1')
            }
            $compileArgs += @('-o', '2', $site)
            $null = Invoke-Checked $python $compileArgs
            $pythonEnv2 = "PYTHONPATH=$(Quote-CommandArgument $site) PYTHONOPTIMIZE=2"
            $pythonEnv1 = "PYTHONPATH=$(Quote-CommandArgument $site) PYTHONOPTIMIZE=1"
        }

        if ('evtx' -in $SelectedSections) {
            $null = Invoke-Checked $python @($evtxPy, '--help')
            $evtxVersionCheck = Invoke-Checked $python @('-c', 'import Evtx; print(Evtx.__version__)') -AllowFailure
            $evtxVersion = if ($evtxVersionCheck.Success) { $evtxVersionCheck.Output.Trim() } else { '' }
            if (-not $evtxVersion) {
                Write-Notice 'Evtx.__version__ is unavailable; checking pip metadata.'
                $pipCheck = Invoke-Checked $python @('-m', 'pip', 'show', 'python-evtx')
                $evtxVersion = Get-VersionField $pipCheck.Output 'Version'
            }
            if ($evtxVersion -ne '0.7.4') {
                throw "python-evtx 0.7.4 is required; found '$evtxVersion'."
            }

            $rustCheck = Invoke-Checked $rustEvtx @('--version')
            $rustVersion = Get-FirstVersion $rustCheck.Output
            if (-not $rustVersion) { $rustVersion = Get-FileVersion $rustEvtx }
            if (-not $rustVersion) { Write-Notice 'Could not determine the Rust evtx_dump version.' }
            $rustPlatform = Get-RustPlatform $rustCheck.Output

            $evtxECmdCheck = Invoke-Checked $evtxECmd @('--version') -AllowFailure
            if (-not $evtxECmdCheck.Success) {
                Write-Notice 'EvtxECmd --version failed; checking --help instead.'
                $null = Invoke-Checked $evtxECmd @('--help')
            }
            $evtxECmdVersion = Get-FirstVersion $evtxECmdCheck.Output
            if (-not $evtxECmdVersion) { $evtxECmdVersion = Get-FileVersion $evtxECmd -ProductFirst }
            if (-not $evtxECmdVersion) { Write-Notice 'Could not determine the EvtxECmd version.' }
            $evtxOutput = Join-Path $temp 'evtxecmd'
            New-Item -ItemType Directory -Path $evtxOutput | Out-Null
            $maps = Join-Path (Split-Path -Parent $evtxECmd) 'Maps'
        }

        if ('odl' -in $SelectedSections) {
            $null = Invoke-Checked $python @($odlPy, '--help')
            $odlCode = 'import sys; sys.path.insert(0,sys.argv[1]); import odl; print(odl.__version__)'
            $odlCheck = Invoke-Checked $python @('-c', $odlCode, (Split-Path -Parent $odlPy)) -AllowFailure
            if (-not $odlCheck.Success) { Write-Notice 'Could not import odl.__version__; checking the source.' }
            $odlImportedVersion = if ($odlCheck.Success) {
                [string] ($odlCheck.Output -split '\r?\n' | Where-Object { $_.Trim() } | Select-Object -Last 1)
            }
            else {
                ''
            }
            $odlSource = Get-Content -LiteralPath $odlPy -Raw
            if ($odlImportedVersion -and $odlSource -notmatch '(?m)^__version__[ \t]*=') {
                Write-Notice 'Ignoring odl.__version__ because the script does not declare it; checking its documented version.'
            }
            $odlVersion = Get-OdlVersion $odlSource $odlImportedVersion
            if (-not $odlVersion) { Write-Notice 'Could not determine the odl.py version.' }

            $env:PYTHONOPTIMIZE = '1'
            $odeHelp = Invoke-Checked $python @($ode, '--help')
            $odeVersionCode = 'import sys; sys.path.insert(0,sys.argv[1]); import OneDriveExplorer; print(OneDriveExplorer.__version__)'
            $odeCheck = Invoke-Checked $python @('-c', $odeVersionCode, $odeDirectory) -AllowFailure
            $odeVersion = if ($odeCheck.Success) {
                [string] ($odeCheck.Output -split '\r?\n' | Where-Object { $_.Trim() } | Select-Object -Last 1)
            }
            else {
                Write-Notice 'Could not import OneDriveExplorer.__version__; checking the help banner.'
                ''
            }
            if (-not $odeVersion) {
                $odeVersion = Get-MatchedValue $odeHelp.Output '\bv([0-9]+(?:\.[0-9]+)+)'
            }
            if (-not $odeVersion) {
                $odeVersion = Get-MatchedValue (Get-Content -LiteralPath $ode -Raw) '(?m)^__version__[ \t]*=[ \t]*.([0-9][0-9A-Za-z._-]*)'
            }
            if (-not $odeVersion) { Write-Notice 'Could not determine the OneDriveExplorer version.' }

            $odlDirectory = Join-Path $temp 'odl-input'
            $odeOutput = Join-Path $temp 'ode.csv'
            New-Item -ItemType Directory -Path $odlDirectory | Out-Null
            Copy-Item -LiteralPath $odlPath, $keyPath -Destination $odlDirectory
            $loadCstructs = if ($cstructs) { 'load_cparser(sys.argv[4])' } else { 'load_cparser()' }
            $odeParserCode = 'import sys; sys.path.insert(0,sys.argv[1]); import OneDriveExplorer; from ode.parsers.odl import load_cparser, parse_odl; ' +
                $loadCstructs + '; df=parse_odl(sys.argv[2]); df.to_csv(sys.argv[3],index=False)'
            $odeArgs = @('-c', $odeParserCode, $odeDirectory, $odlDirectory, $odeOutput)
            if ($cstructs) { $odeArgs += $cstructs }
            $null = Invoke-Checked $python $odeArgs
            if (-not (Test-Path -LiteralPath $odeOutput -PathType Leaf) -or (Get-Item -LiteralPath $odeOutput).Length -eq 0) {
                throw 'OneDriveExplorer parser did not produce a CSV file.'
            }
            $env:PYTHONOPTIMIZE = '2'
        }

        if ('etl' -in $SelectedSections) {
            $traceCheckOutput = Join-Path $temp 'tracerpt-check.csv'
            # tracerpt /? prints help but returns exit code 1, so check a real conversion.
            $null = Invoke-Checked $tracerpt @($etlPath, '-y', '-of', 'CSV', '-o', $traceCheckOutput)
            if (-not (Test-Path -LiteralPath $traceCheckOutput -PathType Leaf)) {
                throw 'tracerpt did not produce its run-check CSV file.'
            }
            $tracerptVersion = Get-FileVersion $tracerpt
            if (-not $tracerptVersion) { Write-Notice 'Could not determine the tracerpt file version.' }

            $etlLiteral = "'" + $etlPath.Replace("'", "''") + "'"
            $winCheckCode = 'Get-WinEvent -Path ' + $etlLiteral +
                ' -Oldest -MaxEvents 1 -ErrorAction Stop | Out-Null; $PSVersionTable.PSVersion.ToString()'
            $winCheck = Invoke-Checked $pwsh @('-NoLogo', '-NoProfile', '-NonInteractive', '-Command', $winCheckCode)
            $winVersion = $winCheck.Output.Trim()
            if (-not $winVersion) { Write-Notice 'Could not determine the Get-WinEvent PowerShell version.' }
            $winCode = 'Get-WinEvent -Path ' + $etlLiteral +
                ' -Oldest -ErrorAction Stop | Format-List TimeCreated,Id,ProviderName,Message | Out-String -Width 4096'
        }

        if ('evtx' -in $SelectedSections) {
            $rows = [System.Collections.Generic.List[object]]::new()
            for ($i = 0; $i -lt $evtxPaths.Count; $i++) {
                $evtxPath = $evtxPaths[$i]
                $suffix = if ($evtxPaths.Count -gt 1) { " [$($i + 1)]" } else { '' }
                foreach ($program in $s4Programs) {
                    $time = Invoke-Hyperfine $hyperfine $jsonPath $runs @($program.Path, $evtxPath) ''
                    $rows.Add((New-Row ($program.Label + $suffix) $program.Info.Version $program.Info.Allocator $program.Info.Platform $program.Info.CpuFeatures $program.Info.OptLevel $time))
                }
                $time = Invoke-Hyperfine $hyperfine $jsonPath $runs @($python, $evtxPy, $evtxPath) $pythonEnv2
                $rows.Add((New-Row ("evtx_dump.py" + $suffix) $evtxVersion '' $pythonPlatform '' '2' $time))
                $time = Invoke-Hyperfine $hyperfine $jsonPath $runs @($rustEvtx, $evtxPath) ''
                $rows.Add((New-Row ("evtx_dump (Rust)" + $suffix) $rustVersion '' $rustPlatform '' '' $time))
                $evtxECmdArgs = @($evtxECmd, '-f', $evtxPath, '--csv', $evtxOutput, '--csvf', 'evtxecmd.csv')
                if (Test-Path -LiteralPath $maps -PathType Container) { $evtxECmdArgs += @('--maps', $maps) }
                $time = Invoke-Hyperfine $hyperfine $jsonPath $runs $evtxECmdArgs ''
                $rows.Add((New-Row ("EvtxECmd" + $suffix) $evtxECmdVersion '' '.NET' '' '' $time))
            }
            Write-Section 'evtx' $evtxReportInputs $rows.ToArray() '' $outputDirectory $copyDirectory
        }

        if ('odl' -in $SelectedSections) {
            $rows = [System.Collections.Generic.List[object]]::new()
            foreach ($program in $s4Programs) {
                $time = Invoke-Hyperfine $hyperfine $jsonPath $runs @($program.Path, $odlPath) ''
                $rows.Add((New-Row $program.Label $program.Info.Version $program.Info.Allocator $program.Info.Platform $program.Info.CpuFeatures $program.Info.OptLevel $time))
            }
            $odlOutput = Join-Path $temp 'odl.csv'
            $time = Invoke-Hyperfine $hyperfine $jsonPath $runs @($python, $odlPy, $odlDirectory, '-o', $odlOutput) $pythonEnv2
            $rows.Add((New-Row 'odl.py' $odlVersion '' $pythonPlatform '' '2' $time))
            $time = Invoke-Hyperfine $hyperfine $jsonPath $runs (@($python) + $odeArgs) $pythonEnv1
            $rows.Add((New-Row 'OneDriveExplorer (parser API)' $odeVersion '' $pythonPlatform '' '1' $time))
            $note = 'OneDriveExplorer uses its parser API at Python optimization level 1; the readers may decode different numbers of messages.'
            if (-not $cstructs) { $note += ' OneDriveExplorer ran without cstructs.' }
            Write-Section 'odl' @($odlReportInput) $rows.ToArray() $note $outputDirectory $copyDirectory
        }

        if ('etl' -in $SelectedSections) {
            $rows = [System.Collections.Generic.List[object]]::new()
            foreach ($program in $s4Programs) {
                $time = Invoke-Hyperfine $hyperfine $jsonPath $runs @($program.Path, $etlPath) ''
                $rows.Add((New-Row $program.Label $program.Info.Version $program.Info.Allocator $program.Info.Platform $program.Info.CpuFeatures $program.Info.OptLevel $time))
            }
            $traceOutput = Join-Path $temp 'tracerpt.csv'
            $time = Invoke-Hyperfine $hyperfine $jsonPath $runs @($tracerpt, $etlPath, '-y', '-of', 'CSV', '-o', $traceOutput) ''
            $rows.Add((New-Row 'tracerpt.exe' $tracerptVersion '' 'Windows' '' '' $time))
            $time = Invoke-Hyperfine $hyperfine $jsonPath $runs @($pwsh, '-NoLogo', '-NoProfile', '-NonInteractive', '-Command', $winCode) ''
            $rows.Add((New-Row 'Get-WinEvent' $winVersion '' 'pwsh' '' '' $time))
            Write-Section 'etl' @($etlReportInput) $rows.ToArray() 'Get-WinEvent timings include PowerShell startup.' $outputDirectory $copyDirectory
        }
    }
    finally {
        $env:PYTHONPATH = $oldPythonPath
        $env:PYTHONOPTIMIZE = $oldPythonOptimize
        if ($pushed) { Pop-Location }
        if ($created) { Remove-Item -LiteralPath $temp -Recurse -Force }
    }
}

if ($MyInvocation.InvocationName -ne '.') {
    Invoke-Comparison $EvtxFiles $OdlFile $EtlFile $Sections
}
