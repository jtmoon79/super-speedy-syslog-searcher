#!/usr/bin/env pwsh

<#
.SYNOPSIS
    Quickly setup a portable python environment for Windows using an embed.zip.
.PARAMETER trace
    Turn on debug tracing.
.NOTES
    Author: James Thomas Moon
#>
[Cmdletbinding(DefaultParameterSetName = 'Update')]
Param (
    [switch] $trace
)

if ($trace) {
    Set-PSDebug -Trace 1
}

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$stopWatch = New-Object -TypeName System.Diagnostics.Stopwatch
$stopWatch.Start()


function Normalize-ComparePath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Path
    )

    $normalized = $Path.Trim()
    $normalized = $normalized -replace '\\', '/'

    if ($normalized.StartsWith('./')) {
        $normalized = $normalized.Substring(2)
    }

    return $normalized
}

function Parse-ListingLine {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Line
    )

    $sep = $Line.IndexOf('|')
    if ($sep -lt 0) {
        return [pscustomobject]@{
            File = $Line
            Date = ''
        }
    }

    return [pscustomobject]@{
        File = $Line.Substring(0, $sep)
        Date = $Line.Substring($sep + 1)
    }
}

function Format-LastWriteTime {
    param(
        [Parameter(Mandatory = $true)]
        [System.IO.FileInfo]$FileInfo
    )

    return $FileInfo.LastWriteTime.ToString('yyyy-MM-dd HH:mm:ss.fffffff zzz')
}

function GetRelativePath {
    param(
        [Parameter(Mandatory = $true)]
        [string]$BasePath,

        [Parameter(Mandatory = $true)]
        [string]$TargetPath
    )
    try {
        return [System.IO.Path]::GetRelativePath($BasePath, $TargetPath)
    }
    catch {
        # GetRelativePath is probably not available
    }

    # try to create a relative path manually
    $baseUri = New-Object System.Uri($BasePath + [System.IO.Path]::DirectorySeparatorChar)
    $targetUri = New-Object System.Uri($TargetPath)
    $relUri = $baseUri.MakeRelativeUri($targetUri).ToString()
    return [System.Uri]::UnescapeDataString($relUri) -replace '/', [System.IO.Path]::DirectorySeparatorChar
}

function Try-ParseListingDate {
    param(
        [Parameter(Mandatory = $true)]
        [string]$InputDate,

        [Parameter(Mandatory = $true)]
        [ref]$Result
    )

    $value = $InputDate.Trim()
    if ([string]::IsNullOrEmpty($value)) {
        return $false
    }

    # Reduce very long sub-second precision to 7 digits for .NET DateTime/DateTimeOffset.
    $value = [System.Text.RegularExpressions.Regex]::Replace(
        $value,
        '(?<prefix>\d{2}:\d{2}:\d{2})\.(?<frac>\d{8,})(?<suffix>(?:\s*(?:Z|[+-]\d{2}:?\d{2}))?$)',
        {
            param($m)
            "{0}.{1}{2}" -f $m.Groups['prefix'].Value, $m.Groups['frac'].Value.Substring(0, 7), $m.Groups['suffix'].Value
        }
    )

    $styles = [System.Globalization.DateTimeStyles]::AllowWhiteSpaces

    [DateTimeOffset]$dtoParsed = [DateTimeOffset]::MinValue
    if ([DateTimeOffset]::TryParse($value, [System.Globalization.CultureInfo]::InvariantCulture, $styles, [ref]$dtoParsed)) {
        $Result.Value = $dtoParsed
        return $true
    }

    [DateTime]$dtParsed = [DateTime]::MinValue
    if ([DateTime]::TryParse($value, [System.Globalization.CultureInfo]::InvariantCulture, $styles, [ref]$dtParsed)) {
        if ($dtParsed.Kind -eq [System.DateTimeKind]::Utc) {
            $Result.Value = [DateTimeOffset]::new($dtParsed, [TimeSpan]::Zero)
        }
        else {
            # Keep existing behavior for timezone-less dates: treat as UTC instant.
            $Result.Value = [DateTimeOffset]::new([DateTime]::SpecifyKind($dtParsed, [System.DateTimeKind]::Utc), [TimeSpan]::Zero)
        }
        return $true
    }

    $knownFormats = [string[]]@(
        'MMM d HH:mm:ss yyyy',
        'MMM  d HH:mm:ss yyyy',
        'MMM d HH:mm:ss.FFFFFFF yyyy',
        'MMM  d HH:mm:ss.FFFFFFF yyyy',
        'dd-MMM-yyyy HH:mm:ss',
        'yyyy/MM/dd HH:mm:ss',
        'yyyy/MM/dd HH:mm:ss.FFFFFFF',
        'yyyy-M-d HH:mm:ss',
        'yyyy-MM-dd HH:mm:ss',
        'yyyy-MM-dd HH:mm:ss.FFFFFFF',
        'yyyyMMdd HH:mm:ss',
        'yyyyMMdd HH:mm:ss.FFFFFFF',
        'yyyy-MM-ddTHH:mm:ss',
        'yyyy-MM-ddTHH:mm:ss.FFFFFFF',
        'yyyy-MM-ddTHH:mm:ssK',
        'yyyy-MM-ddTHH:mm:ss.FFFFFFFK'
    )

    [DateTime]$dtExact = [DateTime]::MinValue
    if ([DateTime]::TryParseExact($value, $knownFormats, [System.Globalization.CultureInfo]::InvariantCulture, $styles, [ref]$dtExact)) {
        $Result.Value = [DateTimeOffset]::new([DateTime]::SpecifyKind($dtExact, [System.DateTimeKind]::Utc), [TimeSpan]::Zero)
        return $true
    }

    return $false
}

try {
    Set-StrictMode -Version 3.0
    # save current values
    # TODO: is there a way to push and pop context like this?
    $erroractionpreference_ = $ErrorActionPreference
    $ErrorActionPreference = "Stop"
    $startLocation = Get-Location

    Set-Location (Join-Path $PSScriptRoot '..')
    $timesListing = (Resolve-Path -LiteralPath (Join-Path $PSScriptRoot 'log-files-time-update.txt')).Path

    $filesListed = New-Object 'System.Collections.Generic.List[string]'
    $filesListedNormalized = New-Object 'System.Collections.Generic.HashSet[string]'
    $filesNoDate = New-Object 'System.Collections.Generic.List[string]'
    $filesNoExist = New-Object 'System.Collections.Generic.List[string]'
    $filesTouchFailed = @{}

    # For each file in listing, set LastWriteTime.
    foreach ($line in (Get-Content -LiteralPath $timesListing)) {
        if ([string]::IsNullOrEmpty($line)) {
            continue
        }

        if ($line[0] -eq '#') {
            continue
        }

        $parsed = Parse-ListingLine -Line $line
        $file = $parsed.File
        $date = $parsed.Date

        $filesListed.Add($file)
        [void]$filesListedNormalized.Add((Normalize-ComparePath -Path $file))

        if (-not (Test-Path -LiteralPath $file -PathType Leaf)) {
            $filesNoExist.Add($file)
            continue
        }

        if ([string]::IsNullOrEmpty($date)) {
            $filesNoDate.Add($file)
            continue
        }

        try {
            $dto = $null
            if (-not (Try-ParseListingDate -InputDate $date -Result ([ref]$dto))) {
                throw "unsupported datetime format in line '{0}'" -f $line
            }
            $item = Get-Item -LiteralPath $file
            # Set UTC directly to avoid locale/offset ambiguity while preserving absolute instant.
            $item.LastWriteTimeUtc = $dto.UtcDateTime
            $updated = Get-Item -LiteralPath $file
            Write-Host -ForegroundColor Green ("{0}|{1}" -f $file, (Format-LastWriteTime -FileInfo $updated))
        }
        catch {
            $filesTouchFailed[$file] = $date
            continue
        }
    }

    # Let developer know about potential problems.
    if ($filesTouchFailed.Count -gt 0) {
        Write-Host -ForegroundColor Red ("Files touch failed listed in '{0}'" -f $timesListing)
    }
    foreach ($file in $filesTouchFailed.Keys) {
        $date = $filesTouchFailed[$file]
        Write-Host -ForegroundColor Red ("date: '{0}', file: '{1}'" -f $date, $file)
    }

    if ($filesNoExist.Count -gt 0) {
        Write-Host -ForegroundColor Yellow ("Files do not exist listed in '{0}'" -f $timesListing)
    }
    foreach ($file in $filesNoExist) {
        Write-Host -ForegroundColor Yellow $file
    }

    if ($filesNoDate.Count -gt 0) {
        Write-Host -ForegroundColor Magenta ("Files without a datetime listed in '{0}'" -f $timesListing)
    }
    foreach ($file in $filesNoDate) {
        Write-Host -ForegroundColor Magenta ("'{0}'" -f $file)
    }

    $bannerShown = $false
    $actualFiles = Get-ChildItem -LiteralPath './logs' -Recurse -File | Sort-Object -Property FullName

    foreach ($actual in $actualFiles) {
        $relativePath = GetRelativePath -BasePath (Get-Location).Path -TargetPath $actual.FullName
        $relativePathNormalized = Normalize-ComparePath -Path $relativePath

        if (-not $filesListedNormalized.Contains($relativePathNormalized)) {
            if (-not $bannerShown) {
                Write-Host -ForegroundColor Cyan ("Files found on filesystem but not found in listing '{0}'" -f $timesListing)
                $bannerShown = $true
            }

            $fileTime = Format-LastWriteTime -FileInfo $actual
            Write-Host -ForegroundColor Cyan ("{0}|{1}" -f ("./{0}" -f $relativePathNormalized), $fileTime)

            try {
                Get-Content -LiteralPath $actual.FullName -Tail 2 | ForEach-Object {
                    Write-Host -ForegroundColor Yellow ("`t{0}" -f $_)
                }
                Write-Host ""
            }
            catch {
                Write-Host -ForegroundColor Red "	<unable to read tail>"
            }
        }
    }

    Write-Host "`nCompleted in" $stopWatch.Elapsed.ToString()
} catch {
    $ErrorActionPreference = "Continue"
    Write-Error $_.ScriptStackTrace
    Write-Error -Message $_.Exception.Message
} finally {
    # XXX: would be ideal to restore PS-Debug value instead of overwriting
    Set-PSDebug -Trace 0
    Set-StrictMode -Off
    if ($null -ne $path_tmp1) {
        Remove-Item -Recurse $path_tmp1 -ErrorAction Continue
    }
    if ($null -ne $erroractionpreference_) {
        $ErrorActionPreference = $erroractionpreference_
    }
    if ($null -ne $startLocation) {
        Set-Location -Path  $startLocation
    }
}
