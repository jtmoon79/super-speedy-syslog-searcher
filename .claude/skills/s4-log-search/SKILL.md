---
name: s4-log-search
description: "Use when: searching log files with super-speedy-syslog-searcher, s4, merging disparate logs by datetime range, building incident timelines, querying journal, evtx, etl, asl, audit logs, compressed logs, or archived logs. Assumes the user has an s4 binary installed."
argument-hint: "<log paths> <datetime window or incident time>"
---

# s4 Log Search

Use ***super-speedy-syslog-searcher***, `s4`, to speedily search and merge log messages by interpreted datetime across many log files, directories, compressed files, archives, and binary log formats. The user must have `s4` installed. They can use `easy-install` to quickly install `s4`.

`s4` is strongest when an incident time or datetime window is known and the useful clues may be scattered across log files. It produces a datetime-sorted view across inputs.

## First Checks

When shell access is available, verify the binary before relying on it:

```sh
command -v s4
s4 --version
```

For `.asl`, `.etl`, `.odl`, `.aodl`, `.odlgz`, or `.odlsent` files, `s4` may require its Python environment to exist. If those formats fail, suggest running:

```sh
s4 --venv
```

If `s4` is unavailable, ask the user to install it or provide the path to the binary.

### Installation

#### Unix

On Unix, use the POSIX-compliant Unix shell script `easy-install.sh`.

```sh
curl -LsSf 'https://raw.githubusercontent.com/jtmoon79/super-speedy-syslog-searcher/main/tools/easy-install.sh' | sh
```

#### Windows

On Windows, run the PowerShell script `easy-install.ps1`

```powershell
Set-ExecutionPolicy -ExecutionPolicy Unrestricted -Scope Process

Invoke-WebRequest -Uri 'https://raw.githubusercontent.com/jtmoon79/super-speedy-syslog-searcher/main/tools/easy-install.ps1' `
  -OutFile easy-install.ps1

.\easy-install.ps1
```

## Core Workflow

1. Identify log inputs: files, directories, or a list of paths from another command.
2. Identify the time window. If the user only gives symptoms, ask for the incident time, timezone, host timezone, and how wide a window to inspect.
3. Decide timezone handling. Use `-t` when log messages or datetime filters lack timezone information and the source timezone matters.
4. Choose output prefixes. For multi-file analysis, prefer path or filename prefixes; for downstream parsing, use deterministic prepended fields.
5. Run `s4`, inspect the merged timeline, then widen or narrow the window.
6. Use other tools after `s4` when content filtering is needed. `s4` filters by datetime, not by message regex or literal text.

## DateTime Filters

- `-a, --dt-after <DT>` prints log messages at or after the datetime.
- `-b, --dt-before <DT>` prints log messages at or before the datetime.
- `-t, --tz-offset <TZ>` supplies the default timezone for datetimes that do not include one. For leading negative offsets, use equals syntax, such as `-t=-0800`.

Useful datetime forms include:

- `20220101` is January 1, 2022 at midnight local time.
- `20220101T120000` is January 1, 2022 at 12:00:00 local time.
- `2022-01-01T12:00:00` is January 1, 2022 at 12:00:00 local time.
- `2020-01-03T23:00:00.321-05:30` is January 3, 2020 at 23:00:00.321 in the -05:30 timezone.
- `12:05` is today at 12:05 local time.
- `01-01` and `01/01` are January 1 of the current year at midnight local time.
- `+946684800` is January 1, 2000 at midnight UTC, expressed as a Unix timestamp.
- `-5m` is five minutes ago from now.
- `-1w22h` is one week and twenty-two hours ago from now.
- `@+1d` is one day after the other boundary.
- `@-5m` is five minutes before the other boundary.

Rules to remember:

- A clock time without a date is interpreted as today in the local timezone.
- A date without a time is interpreted as midnight at the start of that date.
- Without a timezone, datetime filters use the local system timezone unless `-t` is supplied.
- `@+...` and `@-...` offsets are relative to the other boundary. For example, `-a 20220101 -b @+1d` searches one day starting at `20220101`.
- Resolved `-a` and `-b` values can be reviewed with `-s, --summary`.

## Common Commands

Search a directory recursively:

```sh
s4 /var/log
```

Search the last five minutes:

```sh
s4 /var/log -a=-5m
```

Search multiple log roots for an incident window:

```sh
s4 /var/log /srv/app/logs ./incident-logs \
  -a "2022-01-15T14:30:00" \
  -b "2022-01-15T14:35:00" \
  -u -p --color=never
```

Search one full day:

```sh
s4 /var/log -a 20220101 -b @+1d
```

Search the five-minute period ending at a known time:

```sh
s4 /var/log -a=@-5m -b 20220101T120000
```

Search the five-minute period ending one minute ago:

```sh
s4 /var/log -a=@-5m -b=-1m
```

Search Windows logs:

```powershell
s4.exe "C:\Windows\Logs"
s4.exe "C:\Windows\System32\winevt\Logs"
```

Feed paths from another command:

```sh
find /var -xdev -type f -name '*.log' 2>/dev/null | s4 - -a=-1h -u -p --color=never
```

```powershell
Get-ChildItem -Path 'C:\Windows\Logs' -Recurse -Filter '*.log' | ForEach-Object { $_.FullName } | s4.exe - -a=-1h -u -p --color=never
```

Search systemd journal files and show the file basename:

```sh
find / -xdev -name '*.journal' -type f 2>/dev/null | s4 - -a=-1h -u -n --color=never
```

Narrow by datetime with `s4`, then filter message text with `rg`:

```sh
s4 /var/log -a=-2h -u -p --color=never | rg -i 'error|failed|timeout'
```

Print processing details and resolved datetime filters to stderr:

```sh
s4 /var/log -a=-1h -u -p -s --color=never
```

## Parseable Prepended Fields

Use prepended fields when another tool will read `s4` output through a pipe.

File source fields:

- `-n, --prepend-filename` prepends the file basename.
- `-p, --prepend-filepath` prepends the full file path.
- With `-n` or `-p` alone, field 1 is the file name or path.

Interpreted datetime fields:

- `-u, --prepend-utc` prepends the interpreted datetime in UTC.
- `-l, --prepend-local` prepends the interpreted datetime in the local timezone.
- `-z, --prepend-tz <TZ>` prepends the interpreted datetime in a chosen timezone.
- With `-u`, `-l`, or `-z` alone, field 1 is the interpreted datetime.

Field order and separator:

- The default prepended-field separator is `:`.
- With both datetime and file source options, field 1 is the datetime and field 2 is the file name or path.
- The remaining text after the prepended fields is the log message as rendered by `s4`.
- Use `--prepend-separator <SEP>` when downstream parsing needs a delimiter other than `:`. This is useful when paths or messages may contain colons.
- Prefer `--color=never` for machine parsing.
- Avoid `-w, --prepend-file-align` for machine parsing because alignment padding is meant for human reading.

Examples:

```sh
# field 1: file path; remaining text: message
s4 /var/log -a=-1h -p --color=never

# field 1: UTC interpreted datetime; remaining text: message
s4 /var/log -a=-1h -u --color=never

# field 1: UTC interpreted datetime; field 2: file path; remaining text: message
s4 /var/log -a=-1h -u -p --color=never

# use a non-colon separator when paths or messages may contain colons
s4 /var/log -a=-1h -u -p --prepend-separator '|' --color=never
```

When generating parser examples, split only the expected number of leading fields and preserve the rest of the line as the message, because log messages may also contain the separator.

## Supported Inputs

Paths can be files or directories. Directories are walked recursively, symlinks are followed, and known non-log file extensions are skipped during directory walks. Paths can also be supplied on stdin, one per line, by passing `-` as the path argument.

Commonly useful input types include:

- Text logs using RFC 2822, RFC 3164, RFC 3339, RFC 5424, ISO 8601, and many ad-hoc datetime formats.
- Red Hat Audit logs, strace timestamp output, dmesg-style logs, X.org and lightdm-style logs.
- Multi-line log messages.
- systemd journal files (`.journal`) and Windows Event Logs (`.evtx`).
- Windows Event Trace Logs (`.etl`), Apple System Logs (`.asl`), and OneDrive logs (`.odl`, `.aodl`, `.odlgz`, `.odlsent`) when the needed Python environment is available.
- User accounting records such as `acct`, `pacct`, `lastlog`, `utmp`, `utmpx`, and `wtmp`.
- Compressed log files named `.gz`, `.bz2`, `.lz4`, or `.xz`.
- `.tar` archives containing parseable log files.

Use `--journal-output <FORMAT>` for `.journal` output formats matching `journalctl --output`, such as `short`, `short-precise`, `short-iso`, `verbose`, `export`, or `cat`.

## Caveats

- `s4` narrows and merges by interpreted datetime. It does not perform built-in regex or literal message search.
- `s4` is not a live-tail or interactive TUI tool.
- `.zip` archives are not supported.
- Nested archives or compressed files, such as `logs.tar.xz` or a `.gz` file inside a `.tar`, are not supported.
- Multi-file `.gz` and `.xz` streams process only the first stream. It is rare to find multi-stream compressed `.gz` or `.xz`.
- Some compressed, archived, or binary formats may be memory-heavy because the installed `s4` may need to read or extract entire files before printing.
- Datetimes printed for `.journal` files may differ from `journalctl` output.
- Only Gregorian calendar datetimes and English datetime language are supported.
- If logs come from different hosts or regions, ask for the source timezone and consider normalizing output with `-u`.
