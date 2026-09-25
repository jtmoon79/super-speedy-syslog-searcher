#!/usr/bin/env pwsh
#
# etl-copy.ps1
#
# Run tracerpt.exe against each .etl file in its original directory (so nearby
# schema/key files can be found), then copy every file from each directory that
# directly contains a .etl file — including hidden files — into the destination.

<#
.SYNOPSIS
    Convert .etl traces with tracerpt.exe and copy the containing directories.
.DESCRIPTION
    Finds every .etl file under Source, including files in subdirectories and
    hidden files. tracerpt.exe is run against each original .etl path so it can
    see schema or key files beside the trace. Reports are written under
    Destination, preserving the source directory name and any relative
    subdirectories.

    After every trace is processed, each directory that has a .etl file
    directly in it is copied to the matching destination directory. The copy
    includes hidden and system files.

    Example:
      .\tools\etl-copy.ps1 C:\Windows\Logs\WindowsUpdate\ C:\Temp\

    processes:
      C:\Windows\Logs\WindowsUpdate\WindowsUpdate.20260923.192435.263.1.etl

    and writes:
      C:\Temp\WindowsUpdate\WindowsUpdate.20260923.192435.263.1.etl.xml
      C:\Temp\WindowsUpdate\WindowsUpdate.20260923.192435.263.1.etl.int
      C:\Temp\WindowsUpdate\WindowsUpdate.20260923.192435.263.1.etl.tmf

    then copies the original directory, including the .etl file, to:
      C:\Temp\WindowsUpdate\
.PARAMETER Source
    Directory to search for .etl files.
.PARAMETER Destination
    Directory that receives tracerpt reports and copied files.
.PARAMETER TracerptPath
    Optional path to tracerpt.exe. Defaults to the system tracerpt.exe.
.NOTES
    Author: James Thomas Moon
#>
[CmdletBinding()]
param(
    [Parameter(Mandatory = $true, Position = 0)]
    [ValidateNotNullOrEmpty()]
    [string] $Source,

    [Parameter(Mandatory = $true, Position = 1)]
    [ValidateNotNullOrEmpty()]
    [string] $Destination,

    [Parameter()]
    [string] $TracerptPath
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

function Get-NormalizedFullPath {
    param(
        [Parameter(Mandatory = $true)]
        [string] $Path,

        [switch] $MustExist
    )

    if ($MustExist) {
        if (-not (Test-Path -LiteralPath $Path)) {
            throw "Path does not exist: $Path"
        }
        $resolved = (Resolve-Path -LiteralPath $Path).ProviderPath
    }
    else {
        $resolved = [System.IO.Path]::GetFullPath($Path)
    }

    # Keep the trailing separator only for a drive root such as C:\.
    if ($resolved.Length -gt 3) {
        $resolved = $resolved.TrimEnd('\', '/')
    }
    return $resolved
}

function Get-RelativePath {
    param(
        [Parameter(Mandatory = $true)]
        [string] $BasePath,

        [Parameter(Mandatory = $true)]
        [string] $TargetPath
    )

    $baseFull = Get-NormalizedFullPath -Path $BasePath
    $targetFull = Get-NormalizedFullPath -Path $TargetPath
    if ($baseFull -eq $targetFull) {
        return '.'
    }

    try {
        $relative = [System.IO.Path]::GetRelativePath($baseFull, $targetFull)
        if (-not [string]::IsNullOrEmpty($relative)) {
            return $relative
        }
    }
    catch {
        # Windows PowerShell 5.1 does not provide Path.GetRelativePath.
    }

    $baseUri = New-Object System.Uri ($baseFull + [System.IO.Path]::DirectorySeparatorChar)
    $targetUri = New-Object System.Uri $targetFull
    $relativeUri = $baseUri.MakeRelativeUri($targetUri).ToString()
    return ([System.Uri]::UnescapeDataString($relativeUri) -replace '/', [System.IO.Path]::DirectorySeparatorChar)
}

function Get-TracerptPath {
    $name = 'tracerpt.exe'
    if ([Environment]::Is64BitOperatingSystem -and -not [Environment]::Is64BitProcess) {
        $sysnative = Join-Path $env:SystemRoot "Sysnative\$name"
        if (Test-Path -LiteralPath $sysnative) {
            return $sysnative
        }
    }

    $system32 = Join-Path $env:SystemRoot "System32\$name"
    if (Test-Path -LiteralPath $system32) {
        return $system32
    }

    $command = Get-Command -Name $name -CommandType Application -ErrorAction SilentlyContinue
    if ($command) {
        return $command.Source
    }

    throw "tracerpt.exe was not found. Expected $system32."
}

function Copy-DirectoryContents {
    param(
        [Parameter(Mandatory = $true)]
        [string] $SourceDir,

        [Parameter(Mandatory = $true)]
        [string] $DestinationDir
    )

    if (-not (Test-Path -LiteralPath $DestinationDir)) {
        New-Item -ItemType Directory -Path $DestinationDir | Out-Null
    }

    # Wildcard copies skip hidden files. Enumerate with -Force, then Copy-Item
    # each literal path. -Recurse is safe only when the destination child does
    # not already exist; otherwise Copy-Item nests the directory inside itself.
    $children = @(Get-ChildItem -LiteralPath $SourceDir -Force)
    foreach ($child in $children) {
        $target = Join-Path $DestinationDir $child.Name
        if ($child.PSIsContainer -and (Test-Path -LiteralPath $target)) {
            Copy-DirectoryContents -SourceDir $child.FullName -DestinationDir $target
            continue
        }

        Copy-Item -LiteralPath $child.FullName -Destination $DestinationDir -Recurse -Verbose -Force
    }
}

$sourceFull = Get-NormalizedFullPath -Path $Source -MustExist
if (-not (Test-Path -LiteralPath $sourceFull -PathType Container)) {
    throw "Source is not a directory: $sourceFull"
}

$destinationFull = Get-NormalizedFullPath -Path $Destination
if (Test-Path -LiteralPath $destinationFull -PathType Leaf) {
    throw "Destination exists and is not a directory: $destinationFull"
}
if (-not (Test-Path -LiteralPath $destinationFull)) {
    New-Item -ItemType Directory -Path $destinationFull | Out-Null
}

$sourceLeaf = Split-Path -Path $sourceFull -Leaf
if ([string]::IsNullOrEmpty($sourceLeaf)) {
    $destinationBase = $destinationFull
}
else {
    $destinationBase = Join-Path $destinationFull $sourceLeaf
}

$etlFiles = @(Get-ChildItem -LiteralPath $sourceFull -Filter '*.etl' -File -Recurse -Force)
if ($etlFiles.Count -eq 0) {
    Write-Warning "No .etl files found under $sourceFull"
    exit 0
}

if ([string]::IsNullOrWhiteSpace($TracerptPath)) {
    $tracerpt = Get-TracerptPath
}
else {
    $tracerpt = Get-NormalizedFullPath -Path $TracerptPath -MustExist
}

$failures = New-Object System.Collections.Generic.List[string]
$directories = @{}

foreach ($etl in $etlFiles) {
    $sourceDir = $etl.DirectoryName
    $relativeDir = Get-RelativePath -BasePath $sourceFull -TargetPath $sourceDir
    if ($relativeDir -eq '.') {
        $destinationDir = $destinationBase
    }
    else {
        $destinationDir = Join-Path $destinationBase $relativeDir
    }

    if (-not $directories.ContainsKey($sourceDir)) {
        $directories[$sourceDir] = $destinationDir
    }

    if (-not (Test-Path -LiteralPath $destinationDir)) {
        New-Item -ItemType Directory -Path $destinationDir | Out-Null
    }

    $etlDestination = Join-Path $destinationDir $etl.Name
    $xmlPath = $etlDestination + '.xml'
    $intPath = $etlDestination + '.int'
    $tmfPath = $etlDestination + '.tmf'
    $tracerptArgs = @(
        '-y',
        '-lr',
        '-l', $etl.FullName,
        '-of', 'XML',
        '-o', $xmlPath,
        '-int', $intPath,
        '-rts',
        '-tmf', $tmfPath
    )

    Write-Host "tracerpt.exe -l $($etl.FullName)"
    Write-Verbose ("{0} {1}" -f $tracerpt, ($tracerptArgs -join ' '))
    & $tracerpt @tracerptArgs
    if ($LASTEXITCODE -ne 0) {
        $message = "tracerpt.exe exited with code $LASTEXITCODE for $($etl.FullName)"
        $failures.Add($message)
        Write-Warning $message
    }
}

foreach ($sourceDir in $directories.Keys) {
    $destinationDir = $directories[$sourceDir]
    Write-Host "Copying $sourceDir -> $destinationDir"
    Copy-DirectoryContents -SourceDir $sourceDir -DestinationDir $destinationDir
}

if ($failures.Count -gt 0) {
    Write-Warning "$($failures.Count) tracerpt.exe invocation(s) failed. Directory copies were still attempted."
    exit 1
}
