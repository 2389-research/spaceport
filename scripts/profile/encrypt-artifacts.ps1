# ABOUTME: Encrypts a private fixture archive with the reviewed public-only recipient.
# ABOUTME: Keeps plaintext and GPG state outside the ciphertext upload target.
<#
.SYNOPSIS
Encrypt a Windows fixture directory before uploading its artifacts.
.EXAMPLE
./encrypt-artifacts.ps1 -SourceDirectory C:/private/fixture -RecipientFile C:/repo/recipient.asc -OutputFile C:/private/fixture.zip.gpg
#>
param(
  [Parameter(Mandatory = $true)][string]$SourceDirectory,
  [Parameter(Mandatory = $true)][string]$RecipientFile,
  [Parameter(Mandatory = $true)][string]$OutputFile
)
$ErrorActionPreference = 'Stop'
$source = [IO.Path]::GetFullPath($SourceDirectory)
$recipient = [IO.Path]::GetFullPath($RecipientFile)
$sealed = [IO.Path]::GetFullPath($OutputFile)
if (-not (Test-Path $source -PathType Container)) { throw 'The private fixture directory is missing.' }
if (-not (Test-Path $recipient -PathType Leaf)) { throw 'The public recipient file is missing.' }
if (Test-Path $sealed) { throw 'Refusing an existing encrypted output.' }
$parent = [IO.Path]::GetDirectoryName($sealed)
if (-not (Test-Path $parent -PathType Container)) { throw 'The encrypted output parent is missing.' }
$sourcePrefix = $source.TrimEnd([char[]]'\/') + [IO.Path]::DirectorySeparatorChar
if ($sealed.StartsWith($sourcePrefix, [StringComparison]::OrdinalIgnoreCase)) {
  throw 'The encrypted output must stay outside the private fixture directory.'
}
$temporary = Join-Path $parent ('spaceport-seal-' + [Guid]::NewGuid().ToString('N'))
[IO.Directory]::CreateDirectory($temporary) | Out-Null
try {
  $keyring = Join-Path $temporary 'gnupg'
  [IO.Directory]::CreateDirectory($keyring) | Out-Null
  Add-Type -AssemblyName System.IO.Compression.FileSystem
  $archive = Join-Path $temporary 'private.zip'
  [IO.Compression.ZipFile]::CreateFromDirectory($source, $archive)
  $gpg = Get-Command gpg.exe -CommandType Application -ErrorAction Stop | Select-Object -First 1
  $cygpath = Join-Path ([IO.Path]::GetDirectoryName($gpg.Source)) 'cygpath.exe'
  if (-not (Test-Path $cygpath -PathType Leaf)) { throw 'The installed GPG path converter is missing.' }
  $gpgPaths = @(foreach ($value in @($keyring, $recipient, $sealed, $archive)) {
    $converted = & $cygpath -u $value
    if ($LASTEXITCODE -ne 0 -or [string]::IsNullOrWhiteSpace($converted)) { throw 'GPG path conversion failed.' }
    $converted
  })
  & $gpg.Source --no-options --homedir $gpgPaths[0] --version
  if ($LASTEXITCODE -ne 0) { throw 'GPG version inspection failed.' }
  & $gpg.Source --no-options --homedir $gpgPaths[0] --batch --no-tty --recipient-file $gpgPaths[1] --output $gpgPaths[2] --encrypt $gpgPaths[3]
  if ($LASTEXITCODE -ne 0) { throw 'Artifact encryption failed; nothing may upload.' }
  if ((Get-Item $sealed).Length -le 0) { throw 'Encrypted output is empty.' }
  Get-FileHash -Algorithm SHA256 $sealed
} finally {
  Remove-Item -LiteralPath $temporary -Recurse -Force
}
