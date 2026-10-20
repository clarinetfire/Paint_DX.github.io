; Inno Setup Script for Paint DX
; Modernized Windows XP Paint for Windows 11

#define MyAppName "Paint DX"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "Paint DX Contributors"
#define MyAppURL "https://github.com/PaintDX/Paint_DX"
#define MyAppExeName "paint-app.exe"
#define MyProjectExt ".paintproj"
#define MyProjectAssocName "Paint DX Project"

[Setup]
AppId={{D37E6B82-9B7E-4C8D-9E9C-1F67B5492318}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\{#MyAppName}
DefaultGroupName={#MyAppName}
AllowNoIcons=yes
OutputDir=..\dist
OutputBaseFilename=PaintDX-Setup-{#MyAppVersion}
SetupIconFile=..\crates\paint-app\assets\app.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ArchitecturesInstallIn64BitMode=x64compatible
ChangesAssociations=yes

[Languages]
Name: "english"; MessagesFile: "compiler:Default.isl"

[Tasks]
Name: "desktopicon"; Description: "{cm:CreateDesktopIcon}"; GroupDescription: "{cm:AdditionalIcons}"; Flags: unchecked
Name: "associate_project"; Description: "Associate .paintproj files with Paint DX"; GroupDescription: "File Associations:"

[Files]
Source: "..\target\release\{#MyAppExeName}"; DestDir: "{app}"; Flags: ignoreversion
Source: "..\crates\paint-app\assets\app.ico"; DestDir: "{app}\assets"; Flags: ignoreversion
Source: "..\crates\paint-app\assets\app.png"; DestDir: "{app}\assets"; Flags: ignoreversion
Source: "..\README.md"; DestDir: "{app}"; Flags: ignoreversion isreadme; Tasks: ; Languages: 

[Icons]
Name: "{group}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\app.ico"
Name: "{group}\{cm:UninstallProgram,{#MyAppName}}"; Filename: "{uninstallexe}"
Name: "{autodesktop}\{#MyAppName}"; Filename: "{app}\{#MyAppExeName}"; IconFilename: "{app}\assets\app.ico"; Tasks: desktopicon

[Registry]
; .paintproj Association
Root: HKA; Subkey: "Software\Classes\{#MyProjectExt}"; ValueType: string; ValueName: ""; ValueData: "PaintDX.Project"; Flags: uninsdeletevalue; Tasks: associate_project
Root: HKA; Subkey: "Software\Classes\PaintDX.Project"; ValueType: string; ValueName: ""; ValueData: "{#MyProjectAssocName}"; Flags: uninsdeletekey; Tasks: associate_project
Root: HKA; Subkey: "Software\Classes\PaintDX.Project\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\assets\app.ico,0"; Tasks: associate_project
Root: HKA; Subkey: "Software\Classes\PaintDX.Project\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""; Tasks: associate_project

; Image Context Menu "Edit with Paint DX"
Root: HKA; Subkey: "Software\Classes\SystemFileAssociations\image\shell\EditWithPaintDX"; ValueType: string; ValueName: ""; ValueData: "Edit with Paint DX"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\SystemFileAssociations\image\shell\EditWithPaintDX\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\assets\app.ico,0"
Root: HKA; Subkey: "Software\Classes\SystemFileAssociations\image\shell\EditWithPaintDX\command"; ValueType: string; ValueName: ""; ValueData: """{app}\{#MyAppExeName}"" ""%1"""

[Run]
Filename: "{app}\{#MyAppExeName}"; Description: "{cm:LaunchProgram,{#StringChange(MyAppName, '&', '&&')}}"; Flags: nowait postinstall skipifsilent
