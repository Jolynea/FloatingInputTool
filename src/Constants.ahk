#Requires AutoHotkey v2.0

class AppConstants {
    static AppName => "FloatingInputTool"
    static ConfigDirectoryName => "FloatingInputTool"
    static ConfigFileName => "config.ini"
    static DefaultHotkey => "^!Space"
    static DefaultTargetFileName => "fleeting-notes.md"

    static AppDataDirectory {
        get => A_AppData "\" AppConstants.ConfigDirectoryName
    }

    static ConfigFilePath {
        get => AppConstants.AppDataDirectory "\" AppConstants.ConfigFileName
    }
}
