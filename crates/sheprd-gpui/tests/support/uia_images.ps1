# ABOUTME: Delivers one native wheel event to the exact private foreground terminal.
# ABOUTME: Waits for in-app image captures and restores its pointer without replacing later user movement.
param([int]$ProcessId, [long]$DeadlineEpochMs, [switch]$PointerFailure)
. "$PSScriptRoot\uia_tree.ps1" -ProcessId $ProcessId
Add-Type -Path "$PSScriptRoot\native_image_input.cs"
if ($windows.Count -ne 1) { throw 'Native images require exactly one owned window' }
$root = $windows[0]
$handle = [IntPtr]$root.Current.NativeWindowHandle
if ($handle -eq [IntPtr]::Zero) { throw 'Native image fixture has no owned HWND' }
$condition = [System.Windows.Automation.AndCondition]::new(
  [System.Windows.Automation.PropertyCondition]::new($automation::ControlTypeProperty, [System.Windows.Automation.ControlType]::Document),
  [System.Windows.Automation.PropertyCondition]::new($automation::NameProperty, 'Terminal')
)
$documents = $root.FindAll([System.Windows.Automation.TreeScope]::Descendants, $condition)
if ($documents.Count -ne 1) { throw 'Native images have no unique private Terminal document' }
$document = $documents[0]
function Require-Terminal {
  if ([HerdrUia]::GetForegroundWindow() -ne $handle) { throw 'Native image fixture lost exact foreground HWND' }
  $element = $automation::FocusedElement
  if (-not [System.Windows.Automation.Automation]::Compare($element, $document)) { throw 'Private image terminal lost native focus' }
  while ($null -ne $element) {
    if ($element.Current.ProcessId -ne $ProcessId) { throw 'Native image focus left its owned process' }
    if ([System.Windows.Automation.Automation]::Compare($element, $root)) { return }
    $element = $walker.GetParent($element)
  }
  throw 'Native image focus left its owned window'
}
function Require-Released {
  foreach ($key in @(0x01,0x02,0x04,0x10,0x11,0x12,0x5b,0x5c)) {
    if (([HerdrUia]::GetAsyncKeyState($key) -band 0x8000) -ne 0) { throw 'A mouse button or modifier is already held' }
  }
}
function Wait-Image([string]$marker, [bool]$pointer) {
  while ($true) {
    Require-Terminal
    Require-Released
    if ($pointer -and -not [ImageWheel]::AtTarget()) { throw 'Native image pointer moved after its fixture claim' }
    if ([IO.File]::Exists("$PSScriptRoot\image-release.request")) { throw 'Native image parent requested cleanup' }
    if ([DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds() -ge $DeadlineEpochMs - 5000) { throw "Native image marker deadline: $marker" }
    $path = "$PSScriptRoot\gui.log"
    if ((Get-Item -LiteralPath $path).Length -gt 1048576) { throw 'Native image app log exceeds 1 MiB' }
    $stream = [IO.File]::Open($path, [IO.FileMode]::Open, [IO.FileAccess]::Read, [IO.FileShare]::ReadWrite)
    try {
      $bytes = [byte[]]::new(1048577)
      $count = 0
      while ($count -lt $bytes.Length) {
        $read = $stream.Read($bytes, $count, $bytes.Length - $count)
        if ($read -eq 0) { break }
        $count += $read
      }
      if ($count -gt 1048576) { throw 'Native image app log exceeds 1 MiB' }
      $log = [Text.Encoding]::UTF8.GetString($bytes, 0, $count)
    } finally { $stream.Dispose() }
    if ($log.Contains('IMAGE_NATIVE_FAIL ')) { throw 'Owned app reported a native image failure' }
    if ($log.Contains($marker)) { return }
    Start-Sleep -Milliseconds 10
  }
}
Require-Released
try { [HerdrUia]::keybd_event(0x12,0,0,[UIntPtr]::Zero) }
finally { [HerdrUia]::keybd_event(0x12,0,2,[UIntPtr]::Zero) }
[HerdrUia]::SetForegroundWindow($handle) | Out-Null
$document.SetFocus()
$focusDeadline = [DateTime]::UtcNow.AddSeconds(3)
while ([HerdrUia]::GetForegroundWindow() -ne $handle -or
    -not [System.Windows.Automation.Automation]::Compare($automation::FocusedElement, $document)) {
  if ([DateTime]::UtcNow -ge $focusDeadline) { throw 'Private image terminal did not acknowledge native focus' }
  Start-Sleep -Milliseconds 10
}
Require-Terminal
[ImageWheel]::Begin($handle, $DeadlineEpochMs)
$restored = $false
try {
  if (-not $PointerFailure) { Wait-Image 'IMAGE_NATIVE_WHEEL_READY ' $false }
  $bounds = $document.Current.BoundingRectangle
  if ($bounds.Width -le 0 -or $bounds.Height -le 0) { throw 'Native image terminal has empty bounds' }
  $x = [int][Math]::Floor($bounds.Left + $bounds.Width / 2)
  $y = [int][Math]::Floor($bounds.Top + $bounds.Height / 2)
  $original = [ImageWheel]::Position()
  if ($original.x -eq $x -and $original.y -eq $y) { $x += 1 }
  [ImageWheel]::Move($x, $y)
  if ($PointerFailure) { throw 'Intentional native image pointer failure' }
  $hit = $automation::FromPoint([System.Windows.Point]::new($x,$y))
  $inside = $false
  while ($null -ne $hit) {
    if ($hit.Current.ProcessId -ne $ProcessId) { throw 'Native image pointer hit another process' }
    if ([System.Windows.Automation.Automation]::Compare($hit, $document)) { $inside = $true; break }
    $hit = $walker.GetParent($hit)
  }
  if (-not $inside) { throw 'Native image pointer does not hit its exact terminal document' }
  Require-Terminal
  Require-Released
  [ImageWheel]::Wheel()
  Wait-Image 'IMAGE_NATIVE_CAPTURED ' $true
} finally {
  $restored = [ImageWheel]::Finish()
  [Console]::WriteLine("IMAGE_INPUT_POINTER_RELEASE restored=$restored")
}
if (-not $restored) { throw 'Native image pointer claim was not restored' }
Wait-Image 'IMAGE_NATIVE_PASS ' $false
Require-Terminal
Require-Released
[Console]::WriteLine('IMAGE_INPUT native=true wheel=1 pointer_restored=true released=true')
