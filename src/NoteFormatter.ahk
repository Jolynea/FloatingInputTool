#Requires AutoHotkey v2.0

class NoteFormatter {
    Format(noteText) {
        normalizedText := StrReplace(noteText, "`r`n", "`n")
        normalizedText := StrReplace(normalizedText, "`r", "`n")
        normalizedText := RTrim(normalizedText, "`n")

        lines := StrSplit(normalizedText, "`n")
        timestamp := FormatTime(, "yyyy-MM-dd HH:mm")
        noteBlock := "> [!fleeting] " timestamp "`r`n"

        for line in lines {
            noteBlock .= (line = "" ? ">" : "> " line) "`r`n"
        }

        noteBlock .= "`r`n"
        return noteBlock
    }
}
