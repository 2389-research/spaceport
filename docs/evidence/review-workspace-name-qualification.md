# Native workspace name acknowledgement

Source `a8f7bd371` passes the image case and all three Core native cases.
Its [x64 native job](https://github.com/2389-research/spaceport/actions/runs/44661227704/job/136039952924) fails only workspace-close setup.
The expected workspace name is `spaceport-close-fixture`.
The actual native tree shows the created workspace as `spaceport-clo`.
The failure occurs before close consent; it does not prove a close-confirmation regression.

The fixture sent the entire name and Enter in one SendWait batch.
It never observed the field's complete value before allowing submission.
[SendWait](https://learn.microsoft.com/en-us/dotnet/api/system.windows.forms.sendkeys.sendwait?view=netframework-4.8) does not guarantee cross-process message processing with its SendInput implementation.
That timing limitation is a plausible cause; this run does not identify the precise lost-character mechanism.

The corrected fixture requires one private foreground window, its owned focus, Create button, and visible unnamed Edit.
That field matches the production dialog's TextInput role and value metadata.
Native Ctrl-A and Backspace clear the suggestion.
Each bounded literal character is sent once.
The actual ValuePattern must acknowledge its exact prefix before the next character runs.
Missing acknowledgement fails without replay or submission.
The caller sends Enter once, after the complete-name receipt.
All original target, consent, cancel-first, confirmation, checkout, and daemon assertions remain unchanged.

Both Windows feature configurations pass the targeted accessibility-harness Clippy check with warnings denied.
Logs: `/tmp/spaceport-9s44-workspace-name-windows-{all,minimal}.log`.
Independent read-only review finds no material targeting, acknowledgement, helper syntax, or consent regression.
PowerShell execution and native confirmation qualification remain pending the next Windows run.
