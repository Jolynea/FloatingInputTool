#Requires AutoHotkey v2.0

#Include Config.ahk
#Include Constants.ahk
#Include FileWriter.ahk
#Include InputWindow.ahk
#Include NoteFormatter.ahk
#Include SettingsWindow.ahk
#Include ThemeManager.ahk
#Include TrayController.ahk

class FloatingInputToolApp {
    __New() {
        this.configStore := AppConfigStore()
        this.config := this.configStore.Load()
        this.noteFormatter := NoteFormatter()
        this.fileWriter := FileWriter()
        this.inputWindow := InputWindowController(
            this.GetCurrentTheme.Bind(this),
            this.HandleSubmit.Bind(this)
        )
        this.settingsWindow := SettingsWindowController(
            this.GetCurrentTheme.Bind(this),
            this.config,
            this.HandleSettingsSave.Bind(this)
        )
        this.trayController := TrayController(
            this.config["hotkey"],
            this.config["themeMode"],
            this.ToggleInputWindow.Bind(this),
            this.ShowSettingsWindow.Bind(this),
            this.HandleThemeModeChange.Bind(this)
        )
    }

    Run() {
        this.trayController.Initialize()
    }

    ToggleInputWindow(*) {
        if this.inputWindow.IsVisible() {
            this.inputWindow.Hide()
        } else {
            this.inputWindow.Show()
        }
    }

    ShowSettingsWindow(*) {
        this.settingsWindow.Show(this.config)
    }

    HandleSubmit(noteText) {
        noteBlock := this.noteFormatter.Format(noteText)
        this.fileWriter.PrependToFile(this.config["targetFilePath"], noteBlock)
        TrayTip(AppConstants.AppName, "Note captured successfully.", 1)
    }

    HandleSettingsSave(nextConfig) {
        previousHotkey := this.config["hotkey"]
        previousThemeMode := this.config["themeMode"]
        hotkeyChanged := previousHotkey != nextConfig["hotkey"]
        themeChanged := previousThemeMode != nextConfig["themeMode"]

        if hotkeyChanged {
            this.trayController.UpdateHotkey(nextConfig["hotkey"])
        }

        try {
            this.configStore.Save(nextConfig)
        } catch Error as err {
            if hotkeyChanged {
                this.trayController.UpdateHotkey(previousHotkey)
            }

            throw err
        }

        this.config := nextConfig
        if themeChanged {
            this.trayController.UpdateThemeMode(this.config["themeMode"])
            this.ApplyThemeToWindows()
        }
        this.settingsWindow.UpdateConfig(this.config)
        TrayTip(AppConstants.AppName, "Settings saved.", 1)
    }

    HandleThemeModeChange(themeMode) {
        nextConfig := Map(
            "targetFilePath", this.config["targetFilePath"],
            "hotkey", this.config["hotkey"],
            "themeMode", themeMode
        )

        this.configStore.Save(nextConfig)
        this.config := nextConfig
        this.settingsWindow.UpdateConfig(this.config)
        this.trayController.UpdateThemeMode(themeMode)
        this.ApplyThemeToWindows()
        TrayTip(AppConstants.AppName, "Theme updated.", 1)
    }

    GetCurrentTheme() {
        return ThemeManager.GetTheme(this.config["themeMode"])
    }

    ApplyThemeToWindows() {
        theme := this.GetCurrentTheme()
        this.inputWindow.ApplyTheme(theme)
        this.settingsWindow.ApplyTheme(theme)
    }
}
