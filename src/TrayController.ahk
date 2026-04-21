#Requires AutoHotkey v2.0

#Include Constants.ahk

class TrayController {
    __New(initialHotkey, initialThemeMode, onOpenInput, onOpenSettings, onThemeModeChange) {
        this.currentHotkey := initialHotkey
        this.currentThemeMode := initialThemeMode
        this.onOpenInput := onOpenInput
        this.onOpenSettings := onOpenSettings
        this.onThemeModeChange := onThemeModeChange
        this.hotkeyHandler := (*) => this.onOpenInput.Call()
    }

    Initialize() {
        A_IconTip := AppConstants.AppName

        tray := A_TrayMenu
        tray.Delete()
        tray.Add("Open Input", (*) => this.onOpenInput.Call())
        tray.Add("Settings", (*) => this.onOpenSettings.Call())
        tray.Add("Theme", this.BuildThemeMenu())
        tray.Add()
        tray.Add("Reload", (*) => Reload())
        tray.Add("Exit", (*) => ExitApp())
        tray.Default := "Open Input"
        tray.ClickCount := 1

        this.UpdateHotkey(this.currentHotkey)
        this.UpdateThemeMenuChecks()
    }

    BuildThemeMenu() {
        this.themeMenu := Menu()
        this.themeMenu.Add("follow-system", (*) => this.onThemeModeChange.Call(AppConstants.ThemeModeSystem))
        this.themeMenu.Add("theme-dark", (*) => this.onThemeModeChange.Call(AppConstants.ThemeModeDark))
        this.themeMenu.Add("theme-white", (*) => this.onThemeModeChange.Call(AppConstants.ThemeModeWhite))
        return this.themeMenu
    }

    UpdateHotkey(nextHotkey) {
        previousHotkey := this.currentHotkey

        if previousHotkey != "" {
            try Hotkey(previousHotkey, "Off")
        }

        try {
            HotIf((*) => true)
            Hotkey(nextHotkey, this.hotkeyHandler, "On")
        } catch Error as err {
            if previousHotkey != "" {
                HotIf((*) => true)
                Hotkey(previousHotkey, this.hotkeyHandler, "On")
            }

            throw Error("Unable to register hotkey '" nextHotkey "'. " err.Message)
        }

        this.currentHotkey := nextHotkey
    }

    UpdateThemeMode(themeMode) {
        this.currentThemeMode := themeMode
        this.UpdateThemeMenuChecks()
    }

    UpdateThemeMenuChecks() {
        if !IsObject(this.themeMenu) {
            return
        }

        this.themeMenu.Uncheck("follow-system")
        this.themeMenu.Uncheck("theme-dark")
        this.themeMenu.Uncheck("theme-white")

        switch this.currentThemeMode {
            case AppConstants.ThemeModeDark:
                this.themeMenu.Check("theme-dark")
            case AppConstants.ThemeModeWhite:
                this.themeMenu.Check("theme-white")
            default:
                this.themeMenu.Check("follow-system")
        }
    }
}
