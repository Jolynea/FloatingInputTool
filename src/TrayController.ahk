#Requires AutoHotkey v2.0

#Include Constants.ahk

class TrayController {
    __New(initialHotkey, onOpenInput, onOpenSettings) {
        this.currentHotkey := initialHotkey
        this.onOpenInput := onOpenInput
        this.onOpenSettings := onOpenSettings
        this.hotkeyHandler := (*) => this.onOpenInput.Call()
    }

    Initialize() {
        A_IconTip := AppConstants.AppName

        tray := A_TrayMenu
        tray.Delete()
        tray.Add("Open Input", (*) => this.onOpenInput.Call())
        tray.Add("Settings", (*) => this.onOpenSettings.Call())
        tray.Add()
        tray.Add("Reload", (*) => Reload())
        tray.Add("Exit", (*) => ExitApp())
        tray.Default := "Open Input"
        tray.ClickCount := 1

        this.UpdateHotkey(this.currentHotkey)
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
}
