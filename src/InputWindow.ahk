#Requires AutoHotkey v2.0

#Include Constants.ahk

class InputWindowController {
    __New(getThemeCallback, onSubmitCallback) {
        this.getThemeCallback := getThemeCallback
        this.onSubmitCallback := onSubmitCallback
        this.visible := false
        this.gui := ""
        this.noteEdit := ""
        this.currentTheme := ""
        this.hideIconIsPicture := false
    }

    BuildGui() {
        this.gui := Gui("+AlwaysOnTop -Caption +ToolWindow +Border +OwnDialogs", AppConstants.AppName)
        this.gui.MarginX := 14
        this.gui.MarginY := 12
        this.gui.SetFont("s10", "Segoe UI")

        this.titleText := this.gui.Add("Text", "x14 y8 w240 h20 BackgroundTrans", "Fleeting Note")
        this.closeText := this.gui.Add("Text", "x470 y6 w20 h24 Center BackgroundTrans", "X")
        this.noteEdit := this.gui.Add("Edit", "x14 y28 w476 r9 WantTab")
        this.timestampText := this.gui.Add("Text", "x16 y244 w180 h20 BackgroundTrans", FormatTime(, "yyyy-MM-dd HH:mm"))
        this.hideIcon := this.CreateHideControl()
        this.saveButton := this.gui.Add("Button", "x396 y236 w94 h32 Default", "Save")

        this.titleText.OnEvent("Click", (*) => this.BeginWindowDrag())
        this.closeText.OnEvent("Click", (*) => this.CloseWithoutSaving())
        this.saveButton.OnEvent("Click", (*) => this.Submit())
        this.gui.OnEvent("Close", (*) => this.CloseWithoutSaving())
        this.gui.OnEvent("Escape", (*) => this.Hide())
    }

    CreateHideControl() {
        this.hideIconIsPicture := false

        if FileExist(AppConstants.GhostIconPath) {
            try {
                control := this.gui.Add("Picture", "x350 y233 w28 h28 BackgroundTrans", AppConstants.GhostIconPath)
                this.hideIconIsPicture := true
            } catch {
                control := this.gui.Add("Text", "x350 y233 w28 h28 Center BackgroundTrans", "G")
            }
        } else {
            control := this.gui.Add("Text", "x350 y233 w28 h28 Center BackgroundTrans", "G")
        }

        control.OnEvent("Click", (*) => this.Hide())
        return control
    }

    EnsureGui() {
        if !IsObject(this.gui) {
            this.BuildGui()
            this.ApplyTheme(this.getThemeCallback.Call())
        }
    }

    Show() {
        this.EnsureGui()
        this.ApplyTheme(this.getThemeCallback.Call())
        this.timestampText.Value := FormatTime(, "yyyy-MM-dd HH:mm")
        this.gui.Show("w504 h274 Center")
        WinSetAlwaysOnTop(1, "ahk_id " this.gui.Hwnd)
        this.noteEdit.Focus()
        this.visible := true
    }

    Hide() {
        if IsObject(this.gui) {
            this.gui.Hide()
        }
        this.visible := false
    }

    CloseWithoutSaving() {
        this.EnsureGui()
        this.noteEdit.Value := ""
        this.Hide()
    }

    IsVisible() {
        return this.visible
    }

    Submit() {
        this.EnsureGui()
        noteText := this.noteEdit.Value
        if Trim(noteText, " `t`r`n") = "" {
            SoundBeep(1500, 80)
            return
        }

        try {
            this.onSubmitCallback.Call(noteText)
        } catch Error as err {
            MsgBox("Unable to save the note.`n`n" err.Message, AppConstants.AppName, "Iconx")
            return
        }

        this.noteEdit.Value := ""
        this.Hide()
    }

    ApplyTheme(theme) {
        this.currentTheme := theme
        if !IsObject(this.gui) {
            return
        }

        this.gui.BackColor := theme["windowBg"]

        this.titleText.SetFont("s10 Bold c" theme["titleText"], "Segoe UI")
        this.closeText.SetFont("s16 c" theme["titleText"], "Segoe UI")
        this.timestampText.SetFont("s9 c" theme["mutedText"], "Segoe UI")

        this.noteEdit.SetFont("s10 c" theme["bodyText"], "Segoe UI")
        this.noteEdit.Opt("Background" theme["panelBg"])

        if !this.hideIconIsPicture {
            this.hideIcon.SetFont("s12 c" theme["hideText"], "Segoe UI")
        }
        this.saveButton.SetFont("s10 Bold c" theme["saveText"], "Segoe UI")
        this.saveButton.Opt("Background" theme["saveBg"])
    }

    BeginWindowDrag() {
        if IsObject(this.gui) {
            PostMessage(0xA1, 2,,, "ahk_id " this.gui.Hwnd)
        }
    }
}
