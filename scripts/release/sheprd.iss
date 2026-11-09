; Per-user Windows installer for Sheprd, compiled by
; `windows-installer.py build`, which passes the defines below. It installs the
; release zip's tree unchanged: spaceport.exe, a console program because it is
; also the command line terminals and agents run, spaceport-launcher.exe,
; which starts it without a console window, sheprd-acp.exe, which the app starts
; chats with, and the licenses.
; Closing applications before replacing files also ends running chats, whose
; agents' sessions can be resumed. It never installs, starts, or stops the Herdr daemon, and uninstalling
; keeps settings and Herdr sessions.
; The directory and shortcut match site/install.ps1, so either replaces the other.

#ifndef Version
  #error Pass /DVersion=YYYYMMDD.COUNTER
#endif
#ifndef Target
  #error Pass /DTarget=TARGET_TRIPLE
#endif
#ifndef Architecture
  #error Pass /DArchitecture=x64compatible or arm64
#endif
#ifndef SourceDir
  #error Pass /DSourceDir=the extracted zip root
#endif
#ifndef OutputDir
  #error Pass /DOutputDir=DIRECTORY
#endif
; Optional: the release workflow passes /DSignTool=NAME with a matching
; /SNAME=COMMAND, and ISCC then Authenticode-signs the uninstaller and Setup.exe.
; Local and pull request builds pass neither and stay unsigned.

#define AppName "Sheprd"
#define StartupName "Herdr UI"
#define AppExe "spaceport.exe"
#define Launcher "spaceport-launcher.exe"
; The app's AppUserModelID (sheprd_updater::APP_ID), which each window carries with
; relaunch properties naming the launcher (src/taskbar.rs). Shortcuts carry it
; too, so a pinned shortcut and the window share one taskbar button.
#define AppUserModelID "so.pen.herdr-gpui"
#define Icons "..\..\assets\icons"

[Setup]
; Never change: Windows finds an existing installation by it.
AppId={{7FA70894-4FC0-4923-BDCB-B05F7475610A}
AppName={#AppName}
AppVersion={#Version}
AppVerName={#AppName} {#Version}
; winget matches the deployed publisher as well as the permanent AppId.
AppPublisher=Herdr UI
AppPublisherURL=https://sheprd.org
AppSupportURL=https://github.com/sheprd-labs/sheprd/issues
; The deployed bridge only updates this directory.
DefaultDirName={localappdata}\Programs\Herdr UI
DisableDirPage=yes
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
ArchitecturesAllowed={#Architecture}
ArchitecturesInstallIn64BitMode={#Architecture}
; Broadcasts the PATH change so new terminals see it without signing out.
ChangesEnvironment=yes
; A running app is closed before its files are replaced; Herdr sessions live
; in the daemon and keep running.
CloseApplications=yes
RestartApplications=no
OutputDir={#OutputDir}
OutputBaseFilename=Sheprd-{#Version}-{#Target}-setup
Compression=lzma2/max
SolidCompression=yes
WizardStyle=modern
; Sheprd's icon in place of Inno's stock box: the setup file's own icon, the
; corner image on every page, and, centered rather than stretched, the welcome
; and finish pages' panel. Each list runs from 100% to 250% DPI.
SetupIconFile={#Icons}\Sheprd.ico
WizardSmallImageFile={#Icons}\windows\wizard-small-55.png,{#Icons}\windows\wizard-small-69.png,{#Icons}\windows\wizard-small-83.png,{#Icons}\windows\wizard-small-110.png,{#Icons}\windows\wizard-small-138.png
WizardImageFile={#Icons}\windows\wizard-128.png,{#Icons}\windows\wizard-160.png,{#Icons}\windows\wizard-192.png,{#Icons}\windows\wizard-256.png
WizardImageStretch=no
UninstallDisplayName={#AppName}
UninstallDisplayIcon={app}\{#AppExe}
SetupLogging=yes
#ifdef SignTool
SignTool={#SignTool}
SignedUninstaller=yes
#endif
; Asks only when no language below matches Windows' display language.
ShowLanguageDialog=auto

; Each language takes Inno Setup's own translation of the wizard, then this
; installer's text from installer\<tag>.isl, named for the app's language tag
; (docs/i18n.md). Text the installer writes into the registry is in the
; language Setup ran in.
[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl,installer\en-US.isl"

[Tasks]
Name: "addtopath"; Description: "{cm:AddToPath}"
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked

[Files]
Source: "{#SourceDir}\{#AppExe}"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\{#Launcher}"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\sheprd-acp.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#SourceDir}\licenses\*"; DestDir: "{app}\licenses"; Flags: ignoreversion

; Uninstall removes only directories Setup created. Over an install.ps1 copy
; these already existed, and would otherwise be left behind empty.
[UninstallDelete]
Type: dirifempty; Name: "{app}\licenses"
Type: dirifempty; Name: "{app}"
; A pre-Setup install may have supplied the desktop shortcut we renamed.
Type: files; Name: "{userdesktop}\{#AppName}.lnk"

; The app registers its AppUserModelID for notifications on first use; that
; key goes on uninstall. Protocol links need a separately owned installer ID.
[Registry]
Root: HKCU; Subkey: "Software\Classes\AppUserModelId\{#AppUserModelID}"; Flags: uninsdeletekey dontcreatekey
; Settings > General > Launch at login writes a Run value naming the launcher,
; and Task Manager's Startup apps may mark it off. Setup creates neither; both
; go on uninstall, so Windows is not left starting a missing program.
; Never rename these on upgrade: StartupApproved holds the user's disabled state.
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Run"; ValueName: "{#StartupName}"; Flags: uninsdeletevalue dontcreatekey
Root: HKCU; Subkey: "Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run"; ValueName: "{#StartupName}"; Flags: uninsdeletevalue dontcreatekey

; Shortcuts open the launcher, so the app starts without a console window.
[Icons]
Name: "{userprograms}\{#AppName}"; Filename: "{app}\{#Launcher}"; WorkingDir: "{app}"; IconFilename: "{app}\{#AppExe}"; AppUserModelID: "{#AppUserModelID}"
Name: "{userdesktop}\{#AppName}"; Filename: "{app}\{#Launcher}"; WorkingDir: "{app}"; IconFilename: "{app}\{#AppExe}"; AppUserModelID: "{#AppUserModelID}"; Tasks: desktopicon

[Run]
Filename: "{app}\{#Launcher}"; Description: "{cm:LaunchProgram,{#AppName}}"; Flags: nowait postinstall skipifsilent
; The app's in-app update runs Setup silently with /RELAUNCH=1 and quits; this
; starts the updated app, through the launcher, once its files are in place.
Filename: "{app}\{#Launcher}"; WorkingDir: "{app}"; Flags: nowait; Check: RelaunchRequested

[Code]
const
  EnvironmentKey = 'Environment';

{ Set by the app's own updater, which has quit for the install. }
function RelaunchRequested(): Boolean;
begin
  Result := ExpandConstant('{param:RELAUNCH|0}') = '1';
end;

function UserPath(): String;
begin
  if not RegQueryStringValue(HKCU, EnvironmentKey, 'Path', Result) then
    Result := '';
end;

{ Path without Entry. Entries compare without case or a trailing backslash. }
function WithoutEntry(Path, Entry: String): String;
var
  Rest, Part: String;
  Separator: Integer;
begin
  Result := '';
  Rest := Path;
  while Rest <> '' do
  begin
    Separator := Pos(';', Rest);
    if Separator = 0 then
    begin
      Part := Rest;
      Rest := '';
    end
    else
    begin
      Part := Copy(Rest, 1, Separator - 1);
      Delete(Rest, 1, Separator);
    end;
    if (Part <> '') and (CompareText(RemoveBackslashUnlessRoot(Part), RemoveBackslashUnlessRoot(Entry)) <> 0) then
    begin
      if Result <> '' then
        Result := Result + ';';
      Result := Result + Part;
    end;
  end;
end;

{ The user PATH without this installation's entries. Pre-release builds of this
  installer put the bin subdirectory there instead, so that goes too. }
function PathWithoutApp(): String;
begin
  Result := WithoutEntry(WithoutEntry(UserPath(), ExpandConstant('{app}')), ExpandConstant('{app}\bin'));
end;

procedure SetUserPath(Path: String);
begin
  if Path <> UserPath() then
    RegWriteExpandStringValue(HKCU, EnvironmentKey, 'Path', Path);
end;

{ Only rename shortcuts owned by this installation. A legacy desktop shortcut
  survives even when a fresh Setup run did not select the desktop task. }
procedure RenameLegacyShortcut(Folder: String);
var
  OldPath, NewPath, Target, Name: String;
  Shell, Link: Variant;
begin
  OldPath := Folder + '\Herdr UI.lnk';
  NewPath := Folder + '\{#AppName}.lnk';
  if not FileExists(OldPath) then
    exit;
  try
    Shell := CreateOleObject('WScript.Shell');
    Link := Shell.CreateShortcut(OldPath);
    Target := Link.TargetPath;
    Name := ExtractFileName(Target);
    if (CompareText(RemoveBackslashUnlessRoot(ExtractFilePath(Target)), ExpandConstant('{app}')) <> 0) or
       ((CompareText(Name, 'herdr-gpui.exe') <> 0) and
        (CompareText(Name, 'herdr-gpui-launcher.exe') <> 0) and
        (CompareText(Name, '{#AppExe}') <> 0) and (CompareText(Name, '{#Launcher}') <> 0)) then
      exit;
    if FileExists(NewPath) or FileCopy(OldPath, NewPath, True) then
      DeleteFile(OldPath);
  except
    Log('Could not rename the legacy shortcut: ' + GetExceptionMessage);
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
var
  Path: String;
begin
  if CurStep <> ssPostInstall then
    exit;
  Path := PathWithoutApp();
  if WizardIsTaskSelected('addtopath') then
  begin
    if Path <> '' then
      Path := Path + ';';
    Path := Path + ExpandConstant('{app}');
  end;
  SetUserPath(Path);
  RenameLegacyShortcut(ExpandConstant('{userprograms}'));
  RenameLegacyShortcut(ExpandConstant('{userdesktop}'));
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
    SetUserPath(PathWithoutApp());
end;
