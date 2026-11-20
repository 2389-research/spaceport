# ABOUTME: Loads the native fixture's two required system command modules explicitly.
# ABOUTME: Avoids automatic command discovery across unrelated installed modules.
[Console]::WriteLine('POWERSHELL stage: importing Utility')
Microsoft.PowerShell.Core\Import-Module -Name Microsoft.PowerShell.Utility -ErrorAction Stop
[Console]::WriteLine('POWERSHELL stage: importing Management')
Microsoft.PowerShell.Core\Import-Module -Name Microsoft.PowerShell.Management -ErrorAction Stop
[Console]::WriteLine('POWERSHELL stage: system modules ready')
