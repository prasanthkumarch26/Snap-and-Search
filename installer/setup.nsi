; ══════════════════════════════════════════════════════════════════
;  Snap and Search — NSIS Installer Script
;  Requires: NSIS 3.x  (https://nsis.sourceforge.io/)
;  Build:    makensis installer\setup.nsi
; ══════════════════════════════════════════════════════════════════

!define APP_NAME        "Snap and Search"
!define APP_EXE         "SnapAndSearch.exe"
!define APP_VERSION     "1.0.0"
!define PUBLISHER       "Prasanth Kumar"
!define INSTALL_DIR     "$PROGRAMFILES64\SnapAndSearch"
!define UNINSTALL_REG   "Software\Microsoft\Windows\CurrentVersion\Uninstall\SnapAndSearch"
!define OUTPUT_FILE     "installer\SnapAndSearch-Setup.exe"

; ── Metadata ──────────────────────────────────────────────────────
Name            "${APP_NAME} ${APP_VERSION}"
OutFile         "${OUTPUT_FILE}"
InstallDir      "${INSTALL_DIR}"
InstallDirRegKey HKLM "${UNINSTALL_REG}" "InstallLocation"
RequestExecutionLevel admin
SetCompressor   /SOLID lzma

; ── Modern UI ─────────────────────────────────────────────────────
!include "MUI2.nsh"

!define MUI_ABORTWARNING
!define MUI_ICON   "assets\icon.ico"
!define MUI_UNICON "assets\icon.ico"

; Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "LICENSE"
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES

!insertmacro MUI_LANGUAGE "English"

; ── Install Section ───────────────────────────────────────────────
Section "Install"
    SetOutPath "$INSTDIR"

    ; Copy the main executable
    File "target\release\SnapAndSearch.exe"

    ; Write uninstaller
    WriteUninstaller "$INSTDIR\Uninstall.exe"

    ; Add/Remove Programs entry
    WriteRegStr   HKLM "${UNINSTALL_REG}" "DisplayName"          "${APP_NAME}"
    WriteRegStr   HKLM "${UNINSTALL_REG}" "DisplayVersion"        "${APP_VERSION}"
    WriteRegStr   HKLM "${UNINSTALL_REG}" "Publisher"             "${PUBLISHER}"
    WriteRegStr   HKLM "${UNINSTALL_REG}" "InstallLocation"       "$INSTDIR"
    WriteRegStr   HKLM "${UNINSTALL_REG}" "UninstallString"       "$INSTDIR\Uninstall.exe"
    WriteRegDWORD HKLM "${UNINSTALL_REG}" "NoModify"              1
    WriteRegDWORD HKLM "${UNINSTALL_REG}" "NoRepair"              1

    ; Start Menu shortcut
    CreateDirectory "$SMPROGRAMS\${APP_NAME}"
    CreateShortcut  "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk" "$INSTDIR\${APP_EXE}"
    CreateShortcut  "$SMPROGRAMS\${APP_NAME}\Uninstall.lnk"   "$INSTDIR\Uninstall.exe"

    ; Launch app after install
    Exec '"$INSTDIR\${APP_EXE}"'
SectionEnd

; ── Uninstall Section ─────────────────────────────────────────────
Section "Uninstall"
    ; Kill the running process
    ExecWait 'taskkill /F /IM "${APP_EXE}"'

    ; Remove files
    Delete "$INSTDIR\${APP_EXE}"
    Delete "$INSTDIR\Uninstall.exe"
    RMDir  "$INSTDIR"

    ; Remove Start Menu shortcuts
    Delete "$SMPROGRAMS\${APP_NAME}\${APP_NAME}.lnk"
    Delete "$SMPROGRAMS\${APP_NAME}\Uninstall.lnk"
    RMDir  "$SMPROGRAMS\${APP_NAME}"

    ; Remove registry entries
    DeleteRegKey HKLM "${UNINSTALL_REG}"
    DeleteRegValue HKCU "SOFTWARE\Microsoft\Windows\CurrentVersion\Run" "SnapAndSearch"
SectionEnd
