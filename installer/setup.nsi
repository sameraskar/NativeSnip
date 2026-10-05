Unicode true
!include "MUI2.nsh"
!include "LogicLib.nsh"
!include "x64.nsh"
Name "NativeSnip Windows Screenshot Connection"
OutFile "${OUTPUT}"
InstallDir "$LOCALAPPDATA\Programs\AutoSnipsBrowser"
RequestExecutionLevel user
SetCompressor /SOLID lzma
!ifdef SIGN_PYTHON
 !ifndef SIGN_RECORD
  !error "Signed builds require SIGN_RECORD for verified uninstaller evidence."
 !endif
 !uninstfinalize '"${SIGN_PYTHON}" "${SOURCE}\scripts\sign.py" sign "%1" --record "${SIGN_RECORD}"' = 0
!endif
VIProductVersion "0.3.1.0"
VIAddVersionKey "ProductName" "NativeSnip Windows Screenshot Connection"
VIAddVersionKey "FileDescription" "Connect NativeSnip browser extension to Windows screenshots"
VIAddVersionKey "ProductVersion" "0.3.1"
VIAddVersionKey "FileVersion" "0.3.1"
VIAddVersionKey "LegalCopyright" "NativeSnip contributors · MIT license"
!define MUI_ABORTWARNING
!define MUI_WELCOMEPAGE_TITLE "Connect to Windows screenshots"
!define MUI_WELCOMEPAGE_TEXT "This small local helper lets the NativeSnip browser extension open Windows' built-in screenshot bar and save your captures.$\r$\n$\r$\nIt runs on demand. No startup task, permanent clipboard listener, or .NET installation is needed.$\r$\n$\r$\nSetup is for your Windows user only. An old AutoSnips startup shortcut, if present, will be removed. Your screenshots and settings are preserved."
!define MUI_FINISHPAGE_TITLE "Setup complete"
!define MUI_FINISHPAGE_TEXT "Return to your browser and click the NativeSnip extension button to take a screenshot using Windows' built-in capture bar.$\r$\n$\r$\nThe extension will check the connection and become ready to capture.$\r$\n$\r$\nYou can remove this helper from Windows Settings > Apps > Installed apps."
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "${SOURCE}\LICENSE"
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_UNPAGE_FINISH
!insertmacro MUI_LANGUAGE "English"
Function .onInit
 SetShellVarContext current
 SetRegView 64
 ${IfNot} ${RunningX64}
  MessageBox MB_OK|MB_ICONSTOP "NativeSnip needs 64-bit Windows."
  Abort
 ${EndIf}
FunctionEnd
Section "Install"
 SetShellVarContext current
 SetRegView 64
 IfFileExists "$INSTDIR\autosnips-host.exe" 0 +3
  ExecWait '"$INSTDIR\autosnips-host.exe" --cancel'
  Sleep 600
 SetOutPath "$INSTDIR"
 File /oname=autosnips-host.exe "${HOST_EXE}"
 File "${SOURCE}\LICENSE"
 File "${SOURCE}\THIRD-PARTY-LICENSES.txt"
 File "${SOURCE}\README.md"
 File "${SOURCE}\PRIVACY.md"
 File "${SOURCE}\START-HERE.md"
 ExecWait '"$INSTDIR\autosnips-host.exe" --stop-legacy'
 Sleep 300
 Delete "$SMSTARTUP\AutoSnips.lnk"
 ExecWait '"$INSTDIR\autosnips-host.exe" --register' $0
 ${If} $0 != 0
  MessageBox MB_OK|MB_ICONSTOP "The screenshot connection could not be registered. Setup did not finish. Please rerun setup."
  Abort
 ${EndIf}
 WriteUninstaller "$INSTDIR\Uninstall.exe"
 WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\AutoSnipsBrowser" "DisplayName" "NativeSnip Windows Screenshot Connection"
 WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\AutoSnipsBrowser" "DisplayVersion" "0.3.1"
 WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\AutoSnipsBrowser" "Publisher" "NativeSnip contributors"
 WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\AutoSnipsBrowser" "InstallLocation" "$INSTDIR"
 WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\AutoSnipsBrowser" "UninstallString" '$\"$INSTDIR\Uninstall.exe$\"'
 WriteRegStr HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\AutoSnipsBrowser" "DisplayIcon" "$INSTDIR\autosnips-host.exe"
 WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\AutoSnipsBrowser" "NoModify" 1
 WriteRegDWORD HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\AutoSnipsBrowser" "NoRepair" 1
 Delete "$SMPROGRAMS\AutoSnips\Uninstall screenshot connection.lnk"
 RMDir "$SMPROGRAMS\AutoSnips"
 CreateDirectory "$SMPROGRAMS\NativeSnip"
 CreateShortcut "$SMPROGRAMS\NativeSnip\Uninstall screenshot connection.lnk" "$INSTDIR\Uninstall.exe"
SectionEnd
Function un.onInit
 SetShellVarContext current
 SetRegView 64
FunctionEnd
Section "Uninstall"
 ExecWait '"$INSTDIR\autosnips-host.exe" --cancel'
 Sleep 600
 retry_delete:
 ClearErrors
 Delete "$INSTDIR\autosnips-host.exe"
 IfErrors 0 deletion_done
 MessageBox MB_RETRYCANCEL|MB_ICONEXCLAMATION "NativeSnip is still in use. Close any capture or folder-selection window and retry." IDRETRY retry_delete
 Abort
 deletion_done:
 DeleteRegKey HKCU "Software\Google\Chrome\NativeMessagingHosts\com.autosnips.capture"
 DeleteRegKey HKCU "Software\Chromium\NativeMessagingHosts\com.autosnips.capture"
 DeleteRegKey HKCU "Software\Microsoft\Edge\NativeMessagingHosts\com.autosnips.capture"
 DeleteRegKey HKCU "Software\Thorium\NativeMessagingHosts\com.autosnips.capture"
 DeleteRegKey HKCU "Software\Mozilla\NativeMessagingHosts\com.autosnips.capture"
 DeleteRegKey HKCU "Software\Microsoft\Windows\CurrentVersion\Uninstall\AutoSnipsBrowser"
 Delete "$INSTDIR\chromium-host.json"
 Delete "$INSTDIR\firefox-host.json"
 Delete "$INSTDIR\LICENSE"
 Delete "$INSTDIR\THIRD-PARTY-LICENSES.txt"
 Delete "$INSTDIR\README.md"
 Delete "$INSTDIR\PRIVACY.md"
 Delete "$INSTDIR\START-HERE.md"
 Delete "$INSTDIR\Uninstall.exe"
 RMDir "$INSTDIR"
 Delete "$SMPROGRAMS\NativeSnip\Uninstall screenshot connection.lnk"
 RMDir "$SMPROGRAMS\NativeSnip"
 ; Never remove user screenshots, settings, or diagnostic files.
SectionEnd
