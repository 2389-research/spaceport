; The installer's own English text, on top of Inno Setup's compiler:Default.isl.
; A translation is a copy of this file named for the app's language tag
; (installer/fr.isl), listed in sheprd.iss's [Languages] after the
; matching compiler:Languages\*.isl (docs/i18n.md). %1 is the argument the
; script passes; keep every name, and keep "herdr-gpui" and file paths as
; they are.

[CustomMessages]
AddToPath=Add sheprd to PATH, for terminals and agents
; The folder context menu's command; %1 is the app's name.
OpenIn=Open in %1
