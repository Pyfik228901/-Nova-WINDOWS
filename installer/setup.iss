; ============================================================
; Inno Setup 6 Script для N&S (Nova Script)
; Сборка классического установщика Windows setup.exe
; ============================================================

#define MyAppName "N&S"
#define MyAppFullName "N&S (Nova Script)"
#define MyAppVersion "1.0.0"
#define MyAppPublisher "m1oky"
#define MyAppURL "https://github.com/m1oky/NovaScript"
#define MyAppExeName "ns.exe"

[Setup]
AppId={{5D9E3A71-8B42-4D2A-9377-B155B77AA1A1}
AppName={#MyAppName}
AppVersion={#MyAppVersion}
AppVerName={#MyAppFullName} {#MyAppVersion}
AppPublisher={#MyAppPublisher}
AppPublisherURL={#MyAppURL}
AppSupportURL={#MyAppURL}
AppUpdatesURL={#MyAppURL}
DefaultDirName={autopf}\N&S
DefaultGroupName={#MyAppFullName}
DisableProgramGroupPage=yes
LicenseFile=..\LICENSE
SetupIconFile=..\assets\icon.ico
UninstallDisplayIcon={app}\assets\icon.ico
Compression=lzma2/ultra64
SolidCompression=yes
WizardStyle=modern
ChangesEnvironment=yes
ChangesAssociations=yes
ArchitecturesInstallIn64BitMode=x64compatible
PrivilegesRequiredOverridesAllowed=dialog
OutputBaseFilename=setup
OutputDir=.

[Languages]
Name: "russian"; MessagesFile: "compiler:Languages\Russian.isl"
Name: "english"; MessagesFile: "compiler:Default.isl"

[Files]
; Основной исполняемый файл
Source: "..\target\release\ns.exe"; DestDir: "{app}"; Flags: ignoreversion; Check: FileExists(ExpandConstant('{src}\..\target\release\ns.exe'))
Source: "..\target\debug\ns.exe"; DestDir: "{app}"; Flags: ignoreversion; Check: not FileExists(ExpandConstant('{src}\..\target\release\ns.exe'))

; Иконка для ассоциации файлов и деинсталлятора
Source: "..\assets\icon.ico"; DestDir: "{app}\assets"; Flags: ignoreversion
Source: "..\assets\logo.svg"; DestDir: "{app}\assets"; Flags: ignoreversion

; Примеры скриптов
Source: "..\examples\*"; DestDir: "{app}\examples"; Flags: ignoreversion recursesubdirs createallsubdirs

[Registry]
; Ассоциация расширения .ns
Root: HKA; Subkey: "Software\Classes\.ns"; ValueType: string; ValueName: ""; ValueData: "NS.Script"; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.ns"; ValueType: string; ValueName: "Content Type"; ValueData: "text/plain"; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.ns\OpenWithProgids"; ValueType: none; ValueName: "NS.Script"; Flags: uninsdeletevalue

; Ассоциация расширения .nova (обратная совместимость)
Root: HKA; Subkey: "Software\Classes\.nova"; ValueType: string; ValueName: ""; ValueData: "NS.Script"; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\.nova\OpenWithProgids"; ValueType: none; ValueName: "NS.Script"; Flags: uninsdeletevalue

; Регистрация типа файла NS.Script и иконки
Root: HKA; Subkey: "Software\Classes\NS.Script"; ValueType: string; ValueName: ""; ValueData: "Исходный файл N&S (Nova Script)"; Flags: uninsdeletekey
Root: HKA; Subkey: "Software\Classes\NS.Script\DefaultIcon"; ValueType: string; ValueName: ""; ValueData: "{app}\assets\icon.ico,0"; Flags: uninsdeletevalue
Root: HKA; Subkey: "Software\Classes\NS.Script\shell\open\command"; ValueType: string; ValueName: ""; ValueData: """{app}\ns.exe"" ""%1"""; Flags: uninsdeletevalue

[Run]
; Предложение установить расширение VS Code на финальном экране мастера
Filename: "{app}\ns.exe"; Parameters: "setup-vscode"; Description: "Установить подсветку синтаксиса и автодополнение для VS Code"; Flags: postinstall skipifsilent nowait

[Code]
// Управление переменной окружения PATH без использования bat-файлов
const EnvironmentKey = 'Environment';

procedure AddToPath();
var
  OldPath, NewPath: string;
begin
  if RegQueryStringValue(HKEY_CURRENT_USER, EnvironmentKey, 'Path', OldPath) then
  begin
    if Pos(ExpandConstant('{app}'), OldPath) = 0 then
    begin
      if (Length(OldPath) > 0) and (OldPath[Length(OldPath)] <> ';') then
        OldPath := OldPath + ';';
      NewPath := OldPath + ExpandConstant('{app}');
      RegWriteStringValue(HKEY_CURRENT_USER, EnvironmentKey, 'Path', NewPath);
    end;
  end
  else
  begin
    RegWriteStringValue(HKEY_CURRENT_USER, EnvironmentKey, 'Path', ExpandConstant('{app}'));
  end;
end;

procedure RemoveFromPath();
var
  OldPath, AppDir, P: string;
  PosApp: Integer;
begin
  if RegQueryStringValue(HKEY_CURRENT_USER, EnvironmentKey, 'Path', OldPath) then
  begin
    AppDir := ExpandConstant('{app}');
    PosApp := Pos(AppDir, OldPath);
    if PosApp > 0 then
    begin
      // Удаляем директорию приложения и разделители
      Delete(OldPath, PosApp, Length(AppDir));
      StringChangeEx(OldPath, ';;', ';', True);
      if (Length(OldPath) > 0) and (OldPath[Length(OldPath)] = ';') then
        Delete(OldPath, Length(OldPath), 1);
      if (Length(OldPath) > 0) and (OldPath[1] = ';') then
        Delete(OldPath, 1, 1);
      RegWriteStringValue(HKEY_CURRENT_USER, EnvironmentKey, 'Path', OldPath);
    end;
  end;
end;

procedure CurStepChanged(CurStep: TSetupStep);
begin
  if CurStep = ssPostInstall then
  begin
    AddToPath();
  end;
end;

procedure CurUninstallStepChanged(CurUninstallStep: TUninstallStep);
begin
  if CurUninstallStep = usPostUninstall then
  begin
    RemoveFromPath();
  end;
end;
