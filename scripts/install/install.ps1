# Install the Eplyx CLI on Windows x86_64.
#
#   irm https://github.com/thomasdevving/eplyx-stock-transition-assurance/releases/latest/download/install.ps1 | iex
#
# Downloads one release zip and the release SHA256SUMS over HTTPS, verifies the
# zip's SHA-256, requires the zip to contain exactly `eplyx.exe`, and copies it
# to a user-owned directory. No Administrator rights, no registry or PATH edits,
# no telemetry and no stored credentials. Temporary files are removed.
#
# Environment (all optional):
#   EPLYX_VERSION        release tag to install, e.g. v0.1.0 (default: latest)
#   EPLYX_INSTALL_DIR    absolute install directory
#                        (default: %LOCALAPPDATA%\Programs\eplyx\bin)
#   EPLYX_DOWNLOAD_BASE  https:// (or file:/// for an offline mirror) directory
#                        holding the release assets
#   EPLYX_PLATFORM       override detection; only windows-x86_64 is supported
& {
  $ErrorActionPreference = 'Stop'
  $ProgressPreference = 'SilentlyContinue'
  $Repo = 'thomasdevving/eplyx-stock-transition-assurance'
  $Supported = @('windows-x86_64')

  function Fail([string] $Message) { throw "eplyx install: $Message" }

  function Get-Platform {
    if ($env:EPLYX_PLATFORM) { return $env:EPLYX_PLATFORM }
    $windows = ($env:OS -eq 'Windows_NT')
    if (-not $windows) { return 'unsupported-os' }
    $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString()
    switch ($arch) {
      'X64' { return 'windows-x86_64' }
      'Arm64' { return 'windows-arm64' }
      default { return "windows-$($arch.ToLowerInvariant())" }
    }
  }

  function Save-Asset([string] $Url, [string] $Destination) {
    if ($Url.StartsWith('https://')) {
      Invoke-WebRequest -Uri $Url -OutFile $Destination -UseBasicParsing
    } elseif ($Url.StartsWith('file:///')) {
      $local = ([System.Uri] $Url).LocalPath
      if (-not (Test-Path -LiteralPath $local -PathType Leaf)) { Fail "missing local release asset: $local" }
      Copy-Item -LiteralPath $local -Destination $Destination
    } else {
      Fail "refusing non-HTTPS download: $Url"
    }
  }

  $platform = Get-Platform
  if ($Supported -notcontains $platform) {
    Fail "unsupported platform '$platform'. Supported: $($Supported -join ', ') (macOS/Linux: use install.sh)."
  }
  if ($env:EPLYX_VERSION -and $env:EPLYX_VERSION -notmatch '^v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.]+)?$') {
    Fail 'EPLYX_VERSION must look like v0.1.0'
  }
  if ($env:EPLYX_DOWNLOAD_BASE) {
    $base = $env:EPLYX_DOWNLOAD_BASE.TrimEnd('/')
  } elseif ($env:EPLYX_VERSION) {
    $base = "https://github.com/$Repo/releases/download/$($env:EPLYX_VERSION)"
  } else {
    $base = "https://github.com/$Repo/releases/latest/download"
  }
  if (-not ($base.StartsWith('https://') -or $base.StartsWith('file:///'))) {
    Fail 'EPLYX_DOWNLOAD_BASE must be an https:// URL or an absolute file:/// path'
  }
  if ($env:EPLYX_INSTALL_DIR) {
    $installDir = $env:EPLYX_INSTALL_DIR
  } elseif ($env:LOCALAPPDATA) {
    $installDir = Join-Path $env:LOCALAPPDATA 'Programs\eplyx\bin'
  } else {
    Fail 'set EPLYX_INSTALL_DIR; LOCALAPPDATA is not available'
  }
  if (-not [System.IO.Path]::IsPathRooted($installDir)) { Fail 'EPLYX_INSTALL_DIR must be an absolute path' }

  $tmp = Join-Path ([System.IO.Path]::GetTempPath()) ("eplyx-install-" + [System.Guid]::NewGuid().ToString('N'))
  New-Item -ItemType Directory -Path $tmp | Out-Null
  try {
    $sums = Join-Path $tmp 'SHA256SUMS'
    Save-Asset "$base/SHA256SUMS" $sums

    $versionPattern = 'v[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.]+)?'
    if ($env:EPLYX_VERSION) { $versionPattern = [regex]::Escape($env:EPLYX_VERSION) }
    $namePattern = "^eplyx-$versionPattern-$([regex]::Escape($platform))\.zip$"
    $candidates = @(Get-Content -LiteralPath $sums | ForEach-Object {
      $fields = $_ -split '\s+', 2
      if ($fields.Count -eq 2 -and $fields[0] -cmatch '^[0-9a-f]{64}$' -and $fields[1] -cmatch $namePattern) {
        [pscustomobject] @{ Hash = $fields[0]; File = $fields[1] }
      }
    })
    if ($candidates.Count -ne 1) { Fail "SHA256SUMS must list exactly one archive for $platform (found $($candidates.Count))" }
    $archive = Join-Path $tmp $candidates[0].File

    Write-Host "Downloading $($candidates[0].File)"
    Save-Asset "$base/$($candidates[0].File)" $archive
    $actual = (Get-FileHash -LiteralPath $archive -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($actual -ne $candidates[0].Hash) {
      Fail "checksum mismatch for $($candidates[0].File) (expected $($candidates[0].Hash), got $actual); nothing was installed"
    }
    Write-Host "Verified SHA-256 $actual"

    # Exactly one entry, named eplyx.exe, at the zip root: no directories,
    # separators, traversal or extra files. Only that entry is ever written.
    Add-Type -AssemblyName System.IO.Compression.FileSystem
    $zip = [System.IO.Compression.ZipFile]::OpenRead($archive)
    try {
      $entries = @($zip.Entries)
      if ($entries.Count -ne 1 -or $entries[0].FullName -cne 'eplyx.exe') {
        Fail "unexpected archive layout; expected only 'eplyx.exe'"
      }
      $binary = Join-Path $tmp 'eplyx.exe'
      [System.IO.Compression.ZipFileExtensions]::ExtractToFile($entries[0], $binary)
    } finally {
      $zip.Dispose()
    }

    $installed = (& $binary --version | Select-Object -First 1)
    if ($LASTEXITCODE -ne 0) { Fail 'the verified binary does not run on this system' }

    New-Item -ItemType Directory -Force -Path $installDir | Out-Null
    $target = Join-Path $installDir 'eplyx.exe'
    Copy-Item -LiteralPath $binary -Destination $target -Force
    Write-Host "Installed $installed to $target"

    $userPath = [Environment]::GetEnvironmentVariable('Path', 'User')
    $onPath = @(($env:Path -split ';') + ($userPath -split ';')) | Where-Object { $_ -and ($_.TrimEnd('\') -ieq $installDir.TrimEnd('\')) }
    if (-not $onPath) {
      Write-Host ''
      Write-Host "$installDir is not on your PATH. For this session run:"
      Write-Host "  `$env:Path = `"$installDir;`$env:Path`""
      Write-Host 'To keep it for your user account (no Administrator rights needed), run:'
      Write-Host "  [Environment]::SetEnvironmentVariable('Path', `"$installDir;`" + [Environment]::GetEnvironmentVariable('Path', 'User'), 'User')"
      Write-Host 'This installer did not change PATH or the registry.'
    }
    Write-Host ''
    Write-Host 'Next, in your Solana project:'
    Write-Host '  eplyx init; eplyx doctor'
    Write-Host "Uninstall: Remove-Item `"$target`" (project data stays in each project's .eplyx\)."
  } finally {
    Remove-Item -LiteralPath $tmp -Recurse -Force -ErrorAction SilentlyContinue
  }
}
