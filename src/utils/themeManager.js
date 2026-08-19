/**
 * 主题管理器 - 整合莫奈主题提取和应用
 */

import { themeExtractor, applyTheme, extractAndApplyThemeFromAlbum } from './themeExtractor.js'

class ThemeManager {
    constructor() {
        this.currentTheme = null;
        this.isDarkMode = false;
        this.observers = [];

        // 检测系统主题偏好
        this.detectSystemTheme();
        this.setupSystemThemeListener();
    }

    /**
     * 检测系统主题偏好
     */
    detectSystemTheme() {
        if (window.matchMedia && window.matchMedia('(prefers-color-scheme: dark)').matches) {
            this.isDarkMode = true;
        }
    }

    /**
     * 监听系统主题变化
     */
    setupSystemThemeListener() {
        if (window.matchMedia) {
            const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
            mediaQuery.addEventListener('change', (e) => {
                this.isDarkMode = e.matches;
                if (this.currentTheme) {
                    this.applyCurrentTheme();
                }
                this.notifyObservers();
            });
        }
    }

    /**
     * 从专辑封面更新主题
     * @param {string} albumCoverUrl - 专辑封面URL
     */
    async updateThemeFromAlbum(albumCoverUrl) {
        try {
            console.log('正在从专辑封面提取主题...', albumCoverUrl);

            const theme = await extractAndApplyThemeFromAlbum(albumCoverUrl, this.isDarkMode);
            this.currentTheme = theme;

            console.log('主题提取成功:', {
                sourceColor: theme.sourceColor,
                isDark: this.isDarkMode,
                primaryColor: this.isDarkMode ? theme.darkTheme.primary : theme.lightTheme.primary
            });

            this.notifyObservers();
            return theme;
        } catch (error) {
            console.error('主题更新失败:', error);
            return null;
        }
    }

    /**
     * 切换明暗主题
     */
    toggleDarkMode() {
        this.isDarkMode = !this.isDarkMode;
        this.applyCurrentTheme();
        this.notifyObservers();
    }

    /**
     * 应用当前主题
     */
    applyCurrentTheme() {
        if (this.currentTheme) {
            applyTheme(this.currentTheme, this.isDarkMode);
        }
    }

    /**
     * 获取当前主题配色
     */
    getCurrentColors() {
        if (!this.currentTheme) return null;

        const theme = this.isDarkMode ? this.currentTheme.darkTheme : this.currentTheme.lightTheme;
        return {
            primary: theme.primary,
            primaryContainer: theme.primaryContainer,
            secondary: theme.secondary,
            background: theme.background,
            surface: theme.surface,
            onSurface: theme.onSurface,
            isDark: this.isDarkMode
        };
    }

    /**
     * 添加主题变化观察者
     * @param {Function} callback - 回调函数
     */
    addObserver(callback) {
        this.observers.push(callback);
    }

    /**
     * 移除观察者
     * @param {Function} callback - 回调函数
     */
    removeObserver(callback) {
        this.observers = this.observers.filter(obs => obs !== callback);
    }

    /**
     * 通知所有观察者
     */
    notifyObservers() {
        this.observers.forEach(callback => {
            try {
                callback(this.getCurrentColors());
            } catch (error) {
                console.error('主题观察者回调错误:', error);
            }
        });
    }

    /**
     * 预览主题（不实际应用）
     * @param {string} albumCoverUrl - 专辑封面URL
     * @returns {Promise<Object>} - 主题预览数据
     */
    async previewTheme(albumCoverUrl) {
        try {
            const theme = await themeExtractor.extractThemeFromImage(albumCoverUrl);
            return {
                light: {
                    primary: theme.lightTheme.primary,
                    background: theme.lightTheme.background,
                    surface: theme.lightTheme.surface
                },
                dark: {
                    primary: theme.darkTheme.primary,
                    background: theme.darkTheme.background,
                    surface: theme.darkTheme.surface
                },
                sourceColor: theme.sourceColor
            };
        } catch (error) {
            console.error('主题预览失败:', error);
            return null;
        }
    }

    /**
     * 重置为默认主题
     */
    resetToDefault() {
        const defaultTheme = themeExtractor.getDefaultTheme();
        this.currentTheme = defaultTheme;
        this.applyCurrentTheme();
        this.notifyObservers();
    }

    /**
     * 导出当前主题配置
     */
    exportTheme() {
        if (!this.currentTheme) return null;

        return {
            timestamp: Date.now(),
            isDarkMode: this.isDarkMode,
            sourceColor: this.currentTheme.sourceColor,
            lightTheme: this.currentTheme.lightTheme,
            darkTheme: this.currentTheme.darkTheme
        };
    }

    /**
     * 导入主题配置
     * @param {Object} themeData - 主题数据
     */
    importTheme(themeData) {
        try {
            this.currentTheme = {
                sourceColor: themeData.sourceColor,
                lightTheme: themeData.lightTheme,
                darkTheme: themeData.darkTheme,
                colors: [themeData.sourceColor]
            };
            this.isDarkMode = themeData.isDarkMode || false;
            this.applyCurrentTheme();
            this.notifyObservers();
            return true;
        } catch (error) {
            console.error('主题导入失败:', error);
            return false;
        }
    }
}

// 导出单例
export const themeManager = new ThemeManager();

// 导出便捷函数
export {
    themeExtractor,
    applyTheme,
    extractAndApplyThemeFromAlbum
};
