#Requires AutoHotkey v2.0

class AppConstants {
    static AppName => "FloatingInputTool"
    static ConfigDirectoryName => "FloatingInputTool"
    static ConfigFileName => "config.ini"
    static DefaultHotkey => "^!Space"
    static DefaultTargetFileName => "fleeting-notes.md"
    static DefaultThemeMode => "system"
    static ThemeModeSystem => "system"
    static ThemeModeDark => "dark"
    static ThemeModeWhite => "white"
    static GhostIconPath => A_ScriptDir "\ghost-svgrepo-com.svg"

    static AppDataDirectory {
        get => A_AppData "\" AppConstants.ConfigDirectoryName
    }

    static ConfigFilePath {
        get => AppConstants.AppDataDirectory "\" AppConstants.ConfigFileName
    }
}
