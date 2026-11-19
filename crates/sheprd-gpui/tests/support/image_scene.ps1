# ABOUTME: Emits four bounded inline Kitty images in the fixture's existing PowerShell pane.
# ABOUTME: Keeps authored scrollback and a hidden cursor stable until the owned case finishes.
param(
  [Parameter(Mandatory=$true)][ValidateRange(22,100)][int]$Rows,
  [Parameter(Mandatory=$true)][guid]$Token
)
$ErrorActionPreference = 'Stop'
$esc = [string][char]27
$out = [Console]::OpenStandardOutput()
$scene = [Text.StringBuilder]::new()
for ($index = 0; $index -lt 80; $index++) {
  [void]$scene.AppendFormat("SP_IMAGE_HISTORY_{0:D3}`r`n", $index)
}
[void]$scene.Append($esc + '[2J' + $esc + '[H' + $esc + '[0m')
function Add-Image([int]$row, [int]$column, [int]$id, [int]$height, [int]$cols, [int]$rows, [int]$z, [string]$data) {
  [void]$scene.Append($esc + "[$row;${column}H")
  [void]$scene.Append($esc + "_Ga=T,f=32,t=d,i=$id,p=$id,s=1,v=$height,c=$cols,r=$rows,C=1,z=$z,q=2;$data" + $esc + '\')
}
[void]$scene.Append($esc + '[6;6HBELOW' + $esc + '[6;26HABOVE')
Add-Image 5 5 7 2 12 8 -1 ([Convert]::ToBase64String([byte[]](255,0,0,255,0,0,255,255)))
Add-Image 7 8 8 1 6 4 2 ([Convert]::ToBase64String([byte[]](0,255,0,128)))
Add-Image 5 25 9 1 8 4 3 ([Convert]::ToBase64String([byte[]](255,255,0,255)))
Add-Image ($Rows - 1) 35 10 1 6 6 4 ([Convert]::ToBase64String([byte[]](0,255,255,255)))
[void]$scene.Append($esc + "[18;1HSP_IMAGE_NATIVE_$Token" + $esc + '[?25l')
$bytes = [Text.Encoding]::ASCII.GetBytes($scene.ToString())
if ($bytes.Length -gt 8192) { throw 'Authored native image producer exceeds 8 KiB' }
$out.Write($bytes, 0, $bytes.Length)
$out.Flush()
# Stay in this existing pane process; do not return to a prompt that changes its cursor.
$deadline = [DateTime]::UtcNow.AddSeconds(60)
while (-not [IO.File]::Exists("$PSScriptRoot\image-done.request")) {
  if ([DateTime]::UtcNow -ge $deadline) { throw 'Owned native image case did not finish before producer deadline' }
  Start-Sleep -Milliseconds 20
}
