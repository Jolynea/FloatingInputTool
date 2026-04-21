#Requires AutoHotkey v2.0

class FileWriter {
    PrependToFile(targetFilePath, noteBlock) {
        this.EnsureDestinationExists(targetFilePath)

        existingContent := ""
        if FileExist(targetFilePath) {
            existingContent := FileRead(targetFilePath, "UTF-8")
        }

        output := noteBlock
        if existingContent != "" {
            output .= existingContent
        }

        outputFile := FileOpen(targetFilePath, "w", "UTF-8")
        if !IsObject(outputFile) {
            throw Error("Unable to open the target file for writing.")
        }

        outputFile.Write(output)
        outputFile.Close()
    }

    EnsureDestinationExists(targetFilePath) {
        SplitPath(targetFilePath, &fileName, &directoryPath)

        if directoryPath != "" && !DirExist(directoryPath) {
            DirCreate(directoryPath)
        }

        if !FileExist(targetFilePath) {
            FileAppend("", targetFilePath, "UTF-8")
        }
    }
}
