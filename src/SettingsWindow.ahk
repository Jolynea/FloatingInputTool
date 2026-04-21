#Requires AutoHotkey v2.0

#Include Constants.ahk

class SettingsWindowController {
    __New(getThemeCallback, config, onSaveCallback) {
        this.getThemeCallback := getThemeCallback
        this.config := config
        this.onSaveCallback := onSaveCallback
        this.gui := ""
        this.targetFileEdit := ""
        this.hotkeyEdit := ""
        this.themeDropdown := ""
        this.currentTheme := ""
    }

    BuildGui() {
        this.gui := Gui("+OwnDialogs", AppConstants.AppName " Settings")
        this.gui.MarginX := 12
        this.gui.MarginY := 12
        this.gui.SetFont("s10", "Segoe UI")

        this.targetFileLabel := this.gui.Add("Text", "xm", "Target markdown file")
        this.targetFileEdit := this.gui.Add("Edit", "xm w460", this.config["targetFilePath"])

        this.hotkeyLabel := this.gui.Add("Text", "xm y+12", "Global hotkey")
        this.hotkeyEdit := this.gui.Add("Edit", "xm w220", this.config["hotkey"])
        this.hotkeyHelp := this.gui.Add("Text", "x+8 yp+3", "Example: ^!Space")

        this.themeLabel := this.gui.Add("Text", "xm y+12", "Theme")
        this.themeDropdown := this.gui.Add(
            "DropDownList",
            "xm w220 Choose1",
            ["follow-system", "theme-dark", "theme-white"]
        )

        this.saveButton := this.gui.Add("Button", "xm y+18 w100 Default", "Save")
        this.cancelButton := this.gui.Add("Button", "x+8 w100", "Cancel")

        this.saveButton.OnEvent("Click", (*) => this.Save())
        this.cancelButton.OnEvent("Click", (*) => this.Hide())
        this.gui.OnEvent("Close", (*) => this.Hide())
        this.gui.OnEvent("Escape", (*) => this.Hide())
    }

    EnsureGui() {
        if !IsObject(this.gui) {
            this.BuildGui()
            this.ApplyTheme(this.getThemeCallback.Call())
        }
    }

    Show(config := "") {
        this.EnsureGui()

        if IsObject(config) {
            this.UpdateConfig(config)
        }

        this.ApplyTheme(this.getThemeCallback.Call())
        this.targetFileEdit.Value := this.config["targetFilePath"]
        this.hotkeyEdit.Value := this.config["hotkey"]
        this.themeDropdown.Choose(this.ThemeModeToIndex(this.config["themeMode"]))
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

    ApplyTheme(theme) {
        this.currentTheme := theme
        if !IsObject(this.gui) {
            return
        }

        this.gui.BackColor := theme["windowBg"]
        this.targetFileLabel.SetFont("c" theme["titleText"], "Segoe UI")
        this.hotkeyLabel.SetFont("c" theme["titleText"], "Segoe UI")
        this.hotkeyHelp.SetFont("c" theme["mutedText"], "Segoe UI")
        this.themeLabel.SetFont("c" theme["titleText"], "Segoe UI")

        this.targetFileEdit.SetFont("c" theme["bodyText"], "Segoe UI")
        this.hotkeyEdit.SetFont("c" theme["bodyText"], "Segoe UI")
        this.themeDropdown.SetFont("c" theme["bodyText"], "Segoe UI")

        this.targetFileEdit.Opt("Background" theme["panelBg"])
        this.hotkeyEdit.Opt("Background" theme["panelBg"])
        this.themeDropdown.Opt("Background" theme["panelBg"])

        this.saveButton.SetFont("s10 Bold c" theme["saveText"], "Segoe UI")
        this.cancelButton.SetFont("s10 c" theme["bodyText"], "Segoe UI")
        this.saveButton.Opt("Background" theme["saveBg"])
        this.cancelButton.Opt("Background" theme["panelBg"])
    }

    Save() {
        this.EnsureGui()
        targetFilePath := Trim(this.targetFileEdit.Value)
        hotkeyValue := Trim(this.hotkeyEdit.Value)
        themeMode := this.IndexToThemeMode(this.themeDropdown.Value)

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
            "hotkey", hotkeyValue,
            "themeMode", themeMode
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

    ThemeModeToIndex(themeMode) {
        switch themeMode {
            case AppConstants.ThemeModeDark:
                return 2
            case AppConstants.ThemeModeWhite:
                return 3
            default:
                return 1
        }
    }

    IndexToThemeMode(index) {
        switch index {
            case 2:
                return AppConstants.ThemeModeDark
            case 3:
                return AppConstants.ThemeModeWhite
            default:
                return AppConstants.ThemeModeSystem
        }
    }
}
