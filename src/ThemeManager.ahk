#Requires AutoHotkey v2.0

#Include Constants.ahk

class ThemeManager {
    static ResolveThemeMode(themeMode) {
        if themeMode = AppConstants.ThemeModeSystem {
            return ThemeManager.DetectSystemThemeMode()
        }

        return themeMode
    }

    static DetectSystemThemeMode() {
        try {
            appsUseLightTheme := RegRead(
                "HKEY_CURRENT_USER\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize",
                "AppsUseLightTheme"
            )
            return appsUseLightTheme = 0 ? AppConstants.ThemeModeDark : AppConstants.ThemeModeWhite
        } catch {
            return AppConstants.ThemeModeWhite
        }
    }

    static GetTheme(themeMode) {
        resolvedMode := ThemeManager.ResolveThemeMode(themeMode)
        return resolvedMode = AppConstants.ThemeModeDark
            ? ThemeManager.BuildDarkTheme(themeMode)
            : ThemeManager.BuildWhiteTheme(themeMode)
    }

    static BuildWhiteTheme(themeMode) {
        theme := Map()
        theme["mode"] := AppConstants.ThemeModeWhite
        theme["requestedMode"] := themeMode
        theme["windowBg"] := "F8F8FF"
        theme["panelBg"] := "F4F2F7"
        theme["panelBorder"] := "D5D4DE"
        theme["titleText"] := "9E9E9E"
        theme["bodyText"] := "333333"
        theme["mutedText"] := "9E9E9E"
        theme["saveBg"] := "EDEBF1"
        theme["saveText"] := "333333"
        theme["hideText"] := "9E9E9E"
        theme["buttonShadow"] := "C8C6D0"
        theme["surfaceText"] := "333333"
        return theme
    }

    static BuildDarkTheme(themeMode) {
        theme := Map()
        theme["mode"] := AppConstants.ThemeModeDark
        theme["requestedMode"] := themeMode
        theme["windowBg"] := "0C0C0C"
        theme["panelBg"] := "18181E"
        theme["panelBorder"] := "3A3845"
        theme["titleText"] := "9E9E9E"
        theme["bodyText"] := "AAA7B2"
        theme["mutedText"] := "9E9E9E"
        theme["saveBg"] := "797979"
        theme["saveText"] := "F7F7FE"
        theme["hideText"] := "9E9E9E"
        theme["buttonShadow"] := "1A1A20"
        theme["surfaceText"] := "AAA7B2"
        return theme
    }
}
