#Requires AutoHotkey v2.0

#Include Constants.ahk

class SettingsWindowController {
    __New(config, onSaveCallback) {
        this.config := config
        this.onSaveCallback := onSaveCallback
        this.gui := ""
        this.targetFileEdit := ""
        this.hotkeyEdit := ""
    }

    BuildGui() {
        this.gui := Gui("+OwnDialogs", AppConstants.AppName " Settings")
        this.gui.MarginX := 12
        this.gui.MarginY := 12
        this.gui.SetFont("s10", "Segoe UI")

        this.gui.Add("Text", "xm", "Target markdown file")
        this.targetFileEdit := this.gui.Add("Edit", "xm w460", this.config["targetFilePath"])

        this.gui.Add("Text", "xm y+12", "Global hotkey")
        this.hotkeyEdit := this.gui.Add("Edit", "xm w220", this.config["hotkey"])
        this.gui.Add("Text", "x+8 yp+3 cGray", "Example: ^!Space")

        this.saveButton := this.gui.Add("Button", "xm y+18 w90 Default", "Save")
        this.cancelButton := this.gui.Add("Button", "x+8 w90", "Cancel")

        this.saveButton.OnEvent("Click", (*) => this.Save())
        this.cancelButton.OnEvent("Click", (*) => this.Hide())
        this.gui.OnEvent("Close", (*) => this.Hide())
        this.gui.OnEvent("Escape", (*) => this.Hide())
    }

    EnsureGui() {
        if !IsObject(this.gui) {
            this.BuildGui()
        }
    }

    Show(config := "") {
        this.EnsureGui()

        if IsObject(config) {
            this.UpdateConfig(config)
        }

        this.targetFileEdit.Value := this.config["targetFilePath"]
        this.hotkeyEdit.Value := this.config["hotkey"]
        this.gui.Show("AutoSize")
        this.targetFileEdit.Focus()
    }

    Hide() {
        if IsObject(this.gui) {
            this.gui.Hide()
        }
    }

    UpdateConfig(config) {
        this.config := config
    }

    Save() {
        this.EnsureGui()
        targetFilePath := Trim(this.targetFileEdit.Value)
        hotkeyValue := Trim(this.hotkeyEdit.Value)

        if targetFilePath = "" {
            MsgBox("Target markdown file path is required.", AppConstants.AppName, "Icon!")
            return
        }

        if hotkeyValue = "" {
            hotkeyValue := AppConstants.DefaultHotkey
        }

        if !RegExMatch(targetFilePath, "\.md$", &_) {
            targetFilePath .= ".md"
        }

        nextConfig := Map(
            "targetFilePath", targetFilePath,
            "hotkey", hotkeyValue
        )

        try {
            this.onSaveCallback.Call(nextConfig)
        } catch Error as err {
            MsgBox("Unable to save settings.`n`n" err.Message, AppConstants.AppName, "Iconx")
            return
        }

        this.UpdateConfig(nextConfig)
        this.Hide()
    }
}
