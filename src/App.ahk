#Requires AutoHotkey v2.0

#Include Config.ahk
#Include Constants.ahk
#Include FileWriter.ahk
#Include InputWindow.ahk
#Include NoteFormatter.ahk
#Include SettingsWindow.ahk
#Include TrayController.ahk

class FloatingInputToolApp {
    __New() {
        this.configStore := AppConfigStore()
        this.config := this.configStore.Load()
        this.noteFormatter := NoteFormatter()
        this.fileWriter := FileWriter()
        this.inputWindow := InputWindowController(this.HandleSubmit.Bind(this))
        this.settingsWindow := SettingsWindowController(this.config, this.HandleSettingsSave.Bind(this))
        this.trayController := TrayController(
            this.config["hotkey"],
            this.ToggleInputWindow.Bind(this),
            this.ShowSettingsWindow.Bind(this)
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
        hotkeyChanged := previousHotkey != nextConfig["hotkey"]

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
        this.settingsWindow.UpdateConfig(this.config)
        TrayTip(AppConstants.AppName, "Settings saved.", 1)
    }
}
