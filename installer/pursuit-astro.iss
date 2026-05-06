; Pursuit Astro — Inno Setup installer
; Builds .exe that installs dashboard + scraper, creates Start Menu
; shortcut with AUMID property bound (this is what makes Windows toast
; notifications resolve — registry alone is not enough), and registers
; the AUMID under HKCU.
;
; Build:  cargo build --release --bin dashboard --bin scraper
;         "C:\Program Files (x86)\Inno Setup 6\ISCC.exe" installer\pursuit-astro.iss
; Output: installer\Output\PursuitAstro-Setup.exe

#define MyAppName "Pursuit Astro"
#define MyAppVersion "11.0.0"
#define MyAppPublisher "Aisling Leiva"
#define MyAppExeName "dashboard.exe"
#define MyAppId "PursuitAstro.Dashboard"

[Setup]
AppId={#MyAppId}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
DefaultDirName={userappdata}\{#MyAppName}
DefaultGroupName={#MyAppName}
DisableProgramGroupPage=yes
PrivilegesRequired=lowest
OutputBaseFilename=PursuitAstro-Setup
Compression=lzma
SolidCompression=yes
WizardStyle=modern
UninstallDisplayIcon={app}\{#MyAppExeName}

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
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; AppUserModelID: "{#MyAppId}"; Comment: "Pursuit NYC Week 4 — astro-finance dashboard"
Name: "{userdesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; AppUserModelID: "{#MyAppId}"; Tasks: desktopicon

[Registry]
Root: HKCU; Subkey: "Software\Classes\AppUserModelId\{#MyAppId}"; ValueType: string; ValueName: "DisplayName"; ValueData: "{#MyAppName}"; Flags: uninsdeletekey
Root: HKCU; Subkey: "Software\Classes\AppUserModelId\{#MyAppId}"; ValueType: expandsz; ValueName: "IconUri"; ValueData: "%windir%\System32\imageres.dll,-5302"

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "Launch {#MyAppName}"; Flags: nowait postinstall skipifsilent
