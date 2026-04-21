#Requires AutoHotkey v2.0

#Include Constants.ahk

class InputWindowController {
    __New(onSubmitCallback) {
        this.onSubmitCallback := onSubmitCallback
        this.visible := false
        this.gui := ""
        this.noteEdit := ""
    }

    BuildGui() {
        this.gui := Gui("+AlwaysOnTop +ToolWindow +OwnDialogs", AppConstants.AppName)
        this.gui.MarginX := 12
        this.gui.MarginY := 12
        this.gui.SetFont("s10", "Segoe UI")

        this.gui.Add("Text", "xm", "Capture a fleeting note")
        this.noteEdit := this.gui.Add("Edit", "xm w420 r10 WantTab -Wrap")
        this.gui.Add("Text", "xm cGray", "Use Esc to hide. Notes are saved to the configured markdown file.")

        this.saveButton := this.gui.Add("Button", "xm w90 Default", "Save")
        this.cancelButton := this.gui.Add("Button", "x+8 w90", "Hide")

        this.saveButton.OnEvent("Click", (*) => this.Submit())
        this.cancelButton.OnEvent("Click", (*) => this.Hide())
        this.gui.OnEvent("Close", (*) => this.Hide())
        this.gui.OnEvent("Escape", (*) => this.Hide())
    }

    EnsureGui() {
        if !IsObject(this.gui) {
            this.BuildGui()
        }
    }

    Show() {
        this.EnsureGui()
        this.gui.Show("AutoSize Center")
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
}
