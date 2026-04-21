#Requires AutoHotkey v2.0

#Include Constants.ahk

class AppConfigStore {
    Load() {
        this.EnsureConfigDirectory()

        config := Map()
        config["targetFilePath"] := IniRead(
            AppConstants.ConfigFilePath,
            "capture",
            "target_file_path",
            this.GetDefaultTargetFilePath()
        )
        config["hotkey"] := IniRead(
            AppConstants.ConfigFilePath,
            "capture",
            "hotkey",
            AppConstants.DefaultHotkey
        )
        config["themeMode"] := IniRead(
            AppConstants.ConfigFilePath,
            "appearance",
            "theme_mode",
            AppConstants.DefaultThemeMode
        )

        return config
    }

    Save(config) {
        this.EnsureConfigDirectory()

        IniWrite(config["targetFilePath"], AppConstants.ConfigFilePath, "capture", "target_file_path")
        IniWrite(config["hotkey"], AppConstants.ConfigFilePath, "capture", "hotkey")
        IniWrite(config["themeMode"], AppConstants.ConfigFilePath, "appearance", "theme_mode")
    }

    EnsureConfigDirectory() {
        if !DirExist(AppConstants.AppDataDirectory) {
            DirCreate(AppConstants.AppDataDirectory)
        }
    }

    GetDefaultTargetFilePath() {
        return A_MyDocuments "\" AppConstants.DefaultTargetFileName
    }
}
