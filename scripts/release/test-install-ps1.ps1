# Tests for scripts/install/install.ps1 against crafted local (file:///) releases.
#   pwsh -NoProfile -File scripts/release/test-install-ps1.ps1 [-Archive <windows zip>]
# Rejection paths run on any host. The install path runs only on Windows with
# the real release zip, since only there can the verified binary execute.
param([string] $Archive = '')
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.IO.Compression, System.IO.Compression.FileSystem
$root = Resolve-Path (Join-Path $PSScriptRoot '../..')
$installer = Join-Path $root 'scripts/install/install.ps1'
$work = Join-Path ([System.IO.Path]::GetTempPath()) ("eplyx-ps-test-" + [guid]::NewGuid().ToString('N'))
New-Item -ItemType Directory -Path $work | Out-Null
$version = 'v0.1.0'
$name = "eplyx-$version-windows-x86_64.zip"
$failures = 0

function New-Zip([string] $Path, [hashtable] $Entries) {
  $zip = [System.IO.Compression.ZipFile]::Open($Path, 'Create')
  try {
    foreach ($entry in $Entries.GetEnumerator()) {
      $item = $zip.CreateEntry($entry.Key)
      $stream = $item.Open(); $bytes = [Text.Encoding]::UTF8.GetBytes($entry.Value); $stream.Write($bytes, 0, $bytes.Length); $stream.Dispose()
    }
  } finally { $zip.Dispose() }
}

function New-Release([string] $Id, [string] $Zip, [string] $Hash = '') {
  $dir = Join-Path $work "release-$Id"
  New-Item -ItemType Directory -Path $dir | Out-Null
  Copy-Item -LiteralPath $Zip -Destination (Join-Path $dir $name)
  if (-not $Hash) { $Hash = (Get-FileHash -LiteralPath (Join-Path $dir $name) -Algorithm SHA256).Hash.ToLowerInvariant() }
  Set-Content -LiteralPath (Join-Path $dir 'SHA256SUMS') -Value "$Hash  $name`n$('1' * 64)  eplyx-$version-darwin-arm64.tar.gz" -NoNewline
  return 'file:///' + ($dir -replace '\\', '/').TrimStart('/')
}

function Test-Case([string] $Id, [string] $Expect, [string] $Needle, [hashtable] $Environment) {
  $caseHome = Join-Path $work "home-$Id"; $temp = Join-Path $work "tmp-$Id"
  New-Item -ItemType Directory -Path $caseHome, $temp | Out-Null
  $saved = @{}
  $names = @('EPLYX_DOWNLOAD_BASE', 'EPLYX_PLATFORM', 'EPLYX_VERSION', 'EPLYX_INSTALL_DIR', 'TMPDIR', 'TEMP', 'TMP')
  foreach ($n in $names) { $saved[$n] = [Environment]::GetEnvironmentVariable($n); [Environment]::SetEnvironmentVariable($n, $null) }
  $env:EPLYX_INSTALL_DIR = Join-Path $caseHome 'bin'
  $env:TMPDIR = $temp; $env:TEMP = $temp; $env:TMP = $temp
  foreach ($pair in $Environment.GetEnumerator()) { [Environment]::SetEnvironmentVariable($pair.Key, $pair.Value) }
  $output = & pwsh -NoProfile -NonInteractive -File $installer 2>&1 | Out-String
  $code = $LASTEXITCODE
  foreach ($n in $names) { [Environment]::SetEnvironmentVariable($n, $saved[$n]) }
  # PowerShell wraps long error records across "|" lines; compare flattened text.
  $flat = ($output -replace '(?m)^\s*\|\s?', '') -replace '\s+', ' '
  $ok = (($Expect -eq 'ok') -eq ($code -eq 0)) -and $flat.Contains($Needle)
  if (@(Get-ChildItem -LiteralPath $temp -Force).Count -ne 0) { $ok = $false; $output += "`n(temporary files left behind)" }
  if ($Expect -eq 'fail' -and (Test-Path (Join-Path $caseHome 'bin/eplyx.exe'))) { $ok = $false; $output += "`n(binary installed despite failure)" }
  if ($ok) { Write-Host "ok   $Id" } else { Write-Host "FAIL $Id (exit $code)"; Write-Host ($output -replace '(?m)^', '     '); $script:failures++ }
}

try {
  $dummy = Join-Path $work 'dummy.zip'; New-Zip $dummy @{ 'eplyx.exe' = 'not a binary' }
  $good = New-Release 'good' $dummy
  Test-Case 'http-base' 'fail' 'must be an https:// URL' @{ EPLYX_DOWNLOAD_BASE = 'http://example.com/release'; EPLYX_PLATFORM = 'windows-x86_64' }
  Test-Case 'unsupported-arm64' 'fail' "unsupported platform 'windows-arm64'" @{ EPLYX_DOWNLOAD_BASE = $good; EPLYX_PLATFORM = 'windows-arm64' }
  if ($env:OS -ne 'Windows_NT') { Test-Case 'unsupported-os' 'fail' "unsupported platform 'unsupported-os'" @{ EPLYX_DOWNLOAD_BASE = $good } }
  Test-Case 'bad-version' 'fail' 'must look like v0.1.0' @{ EPLYX_DOWNLOAD_BASE = $good; EPLYX_PLATFORM = 'windows-x86_64'; EPLYX_VERSION = 'v1.*' }
  Test-Case 'relative-dir' 'fail' 'must be an absolute path' @{ EPLYX_DOWNLOAD_BASE = $good; EPLYX_PLATFORM = 'windows-x86_64'; EPLYX_INSTALL_DIR = 'relative\bin' }
  Test-Case 'sha-mismatch' 'fail' 'checksum mismatch' @{ EPLYX_DOWNLOAD_BASE = (New-Release 'mismatch' $dummy ('0' * 64)); EPLYX_PLATFORM = 'windows-x86_64' }
  foreach ($case in @(
    @{ Id = 'traversal'; Entries = @{ '../eplyx.exe' = 'x' } },
    @{ Id = 'nested'; Entries = @{ 'bin/eplyx.exe' = 'x' } },
    @{ Id = 'extra-member'; Entries = @{ 'eplyx.exe' = 'x'; 'README.txt' = 'x' } },
    @{ Id = 'missing-binary'; Entries = @{ 'README.txt' = 'x' } },
    @{ Id = 'case-variant'; Entries = @{ 'EPLYX.EXE' = 'x' } })) {
    $zip = Join-Path $work "$($case.Id).zip"; New-Zip $zip $case.Entries
    Test-Case $case.Id 'fail' 'unexpected archive layout' @{ EPLYX_DOWNLOAD_BASE = (New-Release $case.Id $zip); EPLYX_PLATFORM = 'windows-x86_64' }
  }
  $ambiguous = New-Release 'ambiguous' $dummy
  $sums = Join-Path ([Uri] $ambiguous).LocalPath 'SHA256SUMS'
  Add-Content -LiteralPath $sums -Value "`n$('2' * 64)  eplyx-v9.9.9-windows-x86_64.zip"
  Test-Case 'ambiguous' 'fail' 'exactly one archive for windows-x86_64 (found 2)' @{ EPLYX_DOWNLOAD_BASE = $ambiguous; EPLYX_PLATFORM = 'windows-x86_64' }

  if ($Archive) {
    if ($env:OS -ne 'Windows_NT') { throw 'the install path can only be tested on Windows' }
    $real = New-Release 'real' (Resolve-Path $Archive)
    Test-Case 'install' 'ok' 'Installed eplyx' @{ EPLYX_DOWNLOAD_BASE = $real }
    Test-Case 'path-guidance' 'ok' 'is not on your PATH' @{ EPLYX_DOWNLOAD_BASE = $real }
    Test-Case 'pinned-version' 'ok' 'Installed eplyx' @{ EPLYX_DOWNLOAD_BASE = $real; EPLYX_VERSION = $version }
    $installed = Join-Path $work 'home-install/bin/eplyx.exe'
    if (-not (Test-Path -LiteralPath $installed)) { Write-Host 'FAIL installed binary missing'; $failures++ }
    elseif (@(Get-ChildItem -LiteralPath (Split-Path $installed) -Force).Count -ne 1) { Write-Host 'FAIL install dir holds more than eplyx.exe'; $failures++ }
    else { Write-Host 'ok   install dir holds only eplyx.exe' }
  }
} finally {
  Remove-Item -LiteralPath $work -Recurse -Force -ErrorAction SilentlyContinue
}
if ($failures -ne 0) { Write-Host "$failures PowerShell installer tests failed"; exit 1 }
Write-Host 'PowerShell installer tests passed'
