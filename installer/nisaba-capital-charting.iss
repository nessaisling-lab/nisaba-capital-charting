; Nisaba Capital Charting — Inno Setup installer
; Builds .exe that installs Nisaba Terminal (desktop) + scraper, creates
; per-machine Start Menu shortcut with AUMID property bound (load-bearing
; piece for Windows toast notifications), and registers the AUMID under
; HKLM so all user accounts on the machine inherit it.
;
; v12.2 — per-machine install scope. Installer runs WITH admin elevation
; once; the installed app runs UNELEVATED in each user's desktop session
; (required for toast notifications to be delivered by SystemEventsBroker).
;
; Build:  cargo build --release --bin dashboard --bin scraper
;         "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer\nisaba-capital-charting.iss
; Output: installer\Output\NisabaCapitalCharting-Setup.exe

#define MyAppName "Nisaba Terminal"
#define MyAppBrand "Nisaba Capital Charting"
#define MyAppVersion "12.2.0"
#define MyAppPublisher "Aisling Leiva"
#define MyAppExeName "dashboard.exe"
#define MyAppId "NisabaCapitalCharting.Terminal"

[Setup]
AppId={#MyAppId}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
; Per-machine install: Program Files, requires admin during install.
; App itself runs UNELEVATED per user session (required for toasts).
DefaultDirName={commonpf}\{#MyAppBrand}\{#MyAppName}
DefaultGroupName={#MyAppBrand}
DisableProgramGroupPage=yes
PrivilegesRequired=admin
PrivilegesRequiredOverridesAllowed=dialog
OutputBaseFilename=NisabaCapitalCharting-Setup
Compression=lzma
SolidCompression=yes
WizardStyle=modern
UninstallDisplayIcon={app}\{#MyAppExeName}
ArchitecturesInstallIn64BitMode=x64compatible

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "Create a desktop shortcut"; GroupDescription: "Additional shortcuts:"; Flags: unchecked

[Files]
; Custom target-dir per .cargo/config.toml: C:\rustbuild\release\
Source: "C:\rustbuild\release\dashboard.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "C:\rustbuild\release\scraper.exe"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\migrations\*.sql"; DestDir: "{app}\migrations"; Flags: ignoreversion recursesubdirs

[Icons]
; AppUserModelID parameter binds PKEY_AppUserModel_ID on the .lnk —
; the load-bearing piece for toast notifications.
; {commonprograms} = All-Users Start Menu (per-machine install).
; {commondesktop} = All-Users Desktop.
Name: "{commonprograms}\{#MyAppBrand}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; AppUserModelID: "{#MyAppId}"; Comment: "Nisaba Capital Charting — financial astrology terminal"
Name: "{commondesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; AppUserModelID: "{#MyAppId}"; Tasks: desktopicon

[Registry]
; HKLM (per-machine) — all user accounts inherit the AUMID registration.
; Each user still creates their own first-run shortcut at %APPDATA% as a
; safety net; the installer-created shortcut at {commonprograms} is the
; primary registration path.
Root: HKLM; Subkey: "Software\Classes\AppUserModelId\{#MyAppId}"; ValueType: string; ValueName: "DisplayName"; ValueData: "{#MyAppName}"; Flags: uninsdeletekey
Root: HKLM; Subkey: "Software\Classes\AppUserModelId\{#MyAppId}"; ValueType: expandsz; ValueName: "IconUri"; ValueData: "%windir%\System32\imageres.dll,-5302"

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Launch {#MyAppName}"; Flags: nowait postinstall skipifsilent runascurrentuser
