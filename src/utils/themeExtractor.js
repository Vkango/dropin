/**
 * 基于图像的莫奈主题色彩提取器
 * 参考 Material Theme Builder 的设计理念
 * 从音乐专辑图等图像中提取主题色并生成亮色/暗色主题
 */

class MonetThemeExtractor {
    constructor() {
        this.canvas = document.createElement('canvas');
        this.ctx = this.canvas.getContext('2d');
    }

    /**
     * 从图像URL提取主题色彩
     * @param {string} imageUrl - 图像URL
     * @returns {Promise<Object>} - 包含亮色和暗色主题的对象
     */
    async extractThemeFromImage(imageUrl) {
        try {
            const img = await this.loadImage(imageUrl);
            const dominantColors = this.extractDominantColors(img);
            const primaryColor = this.selectPrimaryColor(dominantColors);

            return {
                sourceColor: primaryColor,
                lightTheme: this.generateLightTheme(primaryColor),
                darkTheme: this.generateDarkTheme(primaryColor),
                colors: dominantColors
            };
        } catch (error) {
            console.error('主题提取失败:', error);
            return this.getDefaultTheme();
        }
    }

    /**
     * 加载图像
     */
    loadImage(url) {
        return new Promise((resolve, reject) => {
            const img = new Image();
            img.crossOrigin = 'anonymous';
            img.onload = () => resolve(img);
            img.onerror = reject;
            img.src = url;
        });
    }

    /**
     * 提取主要颜色
     */
    extractDominantColors(img, sampleSize = 64) {
        this.canvas.width = sampleSize;
        this.canvas.height = sampleSize;
        this.ctx.drawImage(img, 0, 0, sampleSize, sampleSize);

        const imageData = this.ctx.getImageData(0, 0, sampleSize, sampleSize);
        const pixels = imageData.data;

        // 收集所有像素颜色
        const colors = [];
        for (let i = 0; i < pixels.length; i += 4) {
            const r = pixels[i];
            const g = pixels[i + 1];
            const b = pixels[i + 2];
            const a = pixels[i + 3];

            // 跳过透明像素
            if (a < 128) continue;

            colors.push({ r, g, b });
        }

        // 使用K-means聚类算法提取主要颜色
        return this.kMeansClustering(colors, 6);
    }

    /**
     * K-means聚类提取主要颜色
     */
    kMeansClustering(colors, k = 6, maxIterations = 10) {
        if (colors.length === 0) return [];

        // 初始化聚类中心
        let centroids = [];
        for (let i = 0; i < k; i++) {
            const randomIndex = Math.floor(Math.random() * colors.length);
            centroids.push({ ...colors[randomIndex] });
        }

        for (let iteration = 0; iteration < maxIterations; iteration++) {
            // 分配颜色到最近的聚类中心
            const clusters = Array(k).fill().map(() => []);

            colors.forEach(color => {
                let minDistance = Infinity;
                let closestCentroid = 0;

                centroids.forEach((centroid, index) => {
                    const distance = this.colorDistance(color, centroid);
                    if (distance < minDistance) {
                        minDistance = distance;
                        closestCentroid = index;
                    }
                });

                clusters[closestCentroid].push(color);
            });

            // 更新聚类中心
            const newCentroids = clusters.map(cluster => {
                if (cluster.length === 0) return centroids[0];

                const avgR = cluster.reduce((sum, c) => sum + c.r, 0) / cluster.length;
                const avgG = cluster.reduce((sum, c) => sum + c.g, 0) / cluster.length;
                const avgB = cluster.reduce((sum, c) => sum + c.b, 0) / cluster.length;

                return { r: Math.round(avgR), g: Math.round(avgG), b: Math.round(avgB) };
            });

            // 检查收敛
            let converged = true;
            for (let i = 0; i < k; i++) {
                if (this.colorDistance(centroids[i], newCentroids[i]) > 1) {
                    converged = false;
                    break;
                }
            }

            centroids = newCentroids;
            if (converged) break;
        }

        // 按颜色饱和度排序，选择最有活力的颜色
        return centroids
            .map(color => ({
                ...color,
                saturation: this.getSaturation(color),
                hex: this.rgbToHex(color)
            }))
            .sort((a, b) => b.saturation - a.saturation);
    }

    /**
     * 计算颜色距离
     */
    colorDistance(color1, color2) {
        const dr = color1.r - color2.r;
        const dg = color1.g - color2.g;
        const db = color1.b - color2.b;
        return Math.sqrt(dr * dr + dg * dg + db * db);
    }

    /**
     * 计算颜色饱和度
     */
    getSaturation(color) {
        const { r, g, b } = color;
        const max = Math.max(r, g, b);
        const min = Math.min(r, g, b);
        return max === 0 ? 0 : (max - min) / max;
    }

    /**
     * 将封面主色轻量混入明/暗中性色，避免背景过于饱和。
     */
    blendColors(color, neutral, colorWeight) {
        const neutralWeight = 1 - colorWeight;
        return {
            r: Math.round(color.r * colorWeight + neutral.r * neutralWeight),
            g: Math.round(color.g * colorWeight + neutral.g * neutralWeight),
            b: Math.round(color.b * colorWeight + neutral.b * neutralWeight)
        };
    }

    /**
     * 选择主色调
     */
    selectPrimaryColor(colors) {
        if (colors.length === 0) return { r: 136, g: 208, b: 236 }; // 默认蓝色

        // 选择饱和度最高且不太暗的颜色
        const suitableColors = colors.filter(color => {
            const brightness = (color.r * 299 + color.g * 587 + color.b * 114) / 1000;
            return color.saturation > 0.3 && brightness > 50 && brightness < 200;
        });

        return suitableColors.length > 0 ? suitableColors[0] : colors[0];
    }

    /**
     * 生成亮色主题
     */
    generateLightTheme(primaryColor) {
        const hsl = this.rgbToHsl(primaryColor);
        const primary = this.hslToRgb({ h: hsl.h, s: 0.7, l: 0.45 });
        const background = this.blendColors(primary, { r: 250, g: 250, b: 250 }, 0.1);
        const surface = this.blendColors(primary, { r: 255, g: 255, b: 255 }, 0.15);

        return {
            // 主色系
            primary,
            primaryContainer: this.hslToRgb({ h: hsl.h, s: 0.4, l: 0.9 }),
            onPrimary: { r: 255, g: 255, b: 255 },
            onPrimaryContainer: this.hslToRgb({ h: hsl.h, s: 0.8, l: 0.15 }),

            // 次要色系
            secondary: this.hslToRgb({ h: (hsl.h + 60) % 360, s: 0.5, l: 0.5 }),
            secondaryContainer: this.hslToRgb({ h: (hsl.h + 60) % 360, s: 0.3, l: 0.92 }),
            onSecondary: { r: 255, g: 255, b: 255 },
            onSecondaryContainer: this.hslToRgb({ h: (hsl.h + 60) % 360, s: 0.6, l: 0.2 }),

            // 第三色系
            tertiary: this.hslToRgb({ h: (hsl.h + 120) % 360, s: 0.5, l: 0.5 }),
            tertiaryContainer: this.hslToRgb({ h: (hsl.h + 120) % 360, s: 0.3, l: 0.92 }),
            onTertiary: { r: 255, g: 255, b: 255 },
            onTertiaryContainer: this.hslToRgb({ h: (hsl.h + 120) % 360, s: 0.6, l: 0.2 }),

            // 背景色系
            background,
            onBackground: { r: 28, g: 27, b: 31 },
            surface,
            onSurface: { r: 28, g: 27, b: 31 },
            surfaceVariant: { r: 231, g: 224, b: 236 },
            onSurfaceVariant: { r: 73, g: 69, b: 79 },

            // 轮廓线
            outline: { r: 121, g: 116, b: 126 },
            outlineVariant: { r: 202, g: 182, b: 224 },

            // 错误色系
            error: { r: 186, g: 26, b: 26 },
            errorContainer: { r: 255, g: 218, b: 214 },
            onError: { r: 255, g: 255, b: 255 },
            onErrorContainer: { r: 65, g: 0, b: 2 },

            // 表面色调
            surfaceTint: this.hslToRgb({ h: hsl.h, s: 0.7, l: 0.45 }),
            inverseSurface: { r: 49, g: 48, b: 51 },
            inverseOnSurface: { r: 244, g: 239, b: 244 },
            inversePrimary: this.hslToRgb({ h: hsl.h, s: 0.6, l: 0.8 }),

            // 阴影
            shadow: { r: 0, g: 0, b: 0 },
            scrim: { r: 0, g: 0, b: 0 }
        };
    }

    /**
     * 生成暗色主题
     */
    generateDarkTheme(primaryColor) {
        const hsl = this.rgbToHsl(primaryColor);
        const primary = this.hslToRgb({ h: hsl.h, s: 0.6, l: 0.8 });
        const background = this.blendColors(primary, { r: 16, g: 16, b: 18 }, 0.12);
        const surface = this.blendColors(primary, { r: 28, g: 28, b: 31 }, 0.16);

        return {
            // 主色系
            primary,
            primaryContainer: this.hslToRgb({ h: hsl.h, s: 0.7, l: 0.3 }),
            onPrimary: this.hslToRgb({ h: hsl.h, s: 0.8, l: 0.15 }),
            onPrimaryContainer: this.hslToRgb({ h: hsl.h, s: 0.4, l: 0.9 }),

            // 次要色系
            secondary: this.hslToRgb({ h: (hsl.h + 60) % 360, s: 0.4, l: 0.8 }),
            secondaryContainer: this.hslToRgb({ h: (hsl.h + 60) % 360, s: 0.5, l: 0.3 }),
            onSecondary: this.hslToRgb({ h: (hsl.h + 60) % 360, s: 0.6, l: 0.2 }),
            onSecondaryContainer: this.hslToRgb({ h: (hsl.h + 60) % 360, s: 0.3, l: 0.92 }),

            // 第三色系
            tertiary: this.hslToRgb({ h: (hsl.h + 120) % 360, s: 0.4, l: 0.8 }),
            tertiaryContainer: this.hslToRgb({ h: (hsl.h + 120) % 360, s: 0.5, l: 0.3 }),
            onTertiary: this.hslToRgb({ h: (hsl.h + 120) % 360, s: 0.6, l: 0.2 }),
            onTertiaryContainer: this.hslToRgb({ h: (hsl.h + 120) % 360, s: 0.3, l: 0.92 }),

            // 背景色系
            background,
            onBackground: { r: 230, g: 225, b: 229 },
            surface,
            onSurface: { r: 230, g: 225, b: 229 },
            surfaceVariant: { r: 73, g: 69, b: 79 },
            onSurfaceVariant: { r: 202, g: 182, b: 224 },

            // 轮廓线
            outline: { r: 147, g: 143, b: 153 },
            outlineVariant: { r: 73, g: 69, b: 79 },

            // 错误色系
            error: { r: 255, g: 180, b: 171 },
            errorContainer: { r: 147, g: 0, b: 10 },
            onError: { r: 105, g: 0, b: 5 },
            onErrorContainer: { r: 255, g: 218, b: 214 },

            // 表面色调
            surfaceTint: this.hslToRgb({ h: hsl.h, s: 0.6, l: 0.8 }),
            inverseSurface: { r: 230, g: 225, b: 229 },
            inverseOnSurface: { r: 49, g: 48, b: 51 },
            inversePrimary: this.hslToRgb({ h: hsl.h, s: 0.7, l: 0.45 }),

            // 阴影
            shadow: { r: 0, g: 0, b: 0 },
            scrim: { r: 0, g: 0, b: 0 }
        };
    }

    /**
     * 获取默认主题
     */
    getDefaultTheme() {
        const defaultPrimary = { r: 136, g: 208, b: 236 };
        return {
            sourceColor: defaultPrimary,
            lightTheme: this.generateLightTheme(defaultPrimary),
            darkTheme: this.generateDarkTheme(defaultPrimary),
            colors: [defaultPrimary]
        };
    }

    /**
     * 颜色转换工具函数
     */
    rgbToHex(color) {
        const toHex = (n) => {
            const hex = Math.round(n).toString(16);
            return hex.length === 1 ? '0' + hex : hex;
        };
        return `#${toHex(color.r)}${toHex(color.g)}${toHex(color.b)}`;
    }

    rgbToHsl({ r, g, b }) {
        r /= 255;
        g /= 255;
        b /= 255;

        const max = Math.max(r, g, b);
        const min = Math.min(r, g, b);
        const diff = max - min;
        const sum = max + min;

        let h = 0;
        let s = 0;
        const l = sum / 2;

        if (diff !== 0) {
            s = l > 0.5 ? diff / (2 - sum) : diff / sum;

            switch (max) {
                case r:
                    h = ((g - b) / diff) + (g < b ? 6 : 0);
                    break;
                case g:
                    h = (b - r) / diff + 2;
                    break;
                case b:
                    h = (r - g) / diff + 4;
                    break;
            }
            h /= 6;
        }

        return { h: h * 360, s, l };
    }

    hslToRgb({ h, s, l }) {
        h /= 360;

        const hue2rgb = (p, q, t) => {
            if (t < 0) t += 1;
            if (t > 1) t -= 1;
            if (t < 1 / 6) return p + (q - p) * 6 * t;
            if (t < 1 / 2) return q;
            if (t < 2 / 3) return p + (q - p) * (2 / 3 - t) * 6;
            return p;
        };

        let r, g, b;

        if (s === 0) {
            r = g = b = l;
        } else {
            const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
            const p = 2 * l - q;
            r = hue2rgb(p, q, h + 1 / 3);
            g = hue2rgb(p, q, h);
            b = hue2rgb(p, q, h - 1 / 3);
        }

        return {
            r: Math.round(r * 255),
            g: Math.round(g * 255),
            b: Math.round(b * 255)
        };
    }

    hslToHex(hsl) {
        const rgb = this.hslToRgb(hsl);
        return this.rgbToHex(rgb);
    }
}

// 导出单例实例
export const themeExtractor = new MonetThemeExtractor();

/**
 * 应用主题到CSS变量
 * @param {Object} theme - 主题对象
 * @param {boolean} isDark - 是否为暗色主题
 */
export function applyTheme(theme, isDark = false) {
    const root = document.documentElement;
    const selectedTheme = isDark ? theme.darkTheme : theme.lightTheme;

    // 应用主题色到CSS变量（RGB格式）
    Object.entries(selectedTheme).forEach(([key, value]) => {
        const cssVar = `--md-sys-color-${key.replace(/([A-Z])/g, '-$1').toLowerCase()}`;
        const rgbValue = `${value.r}, ${value.g}, ${value.b}`;
        root.style.setProperty(cssVar, rgbValue);
    });

    // 设置便于使用的主色调变量
    root.style.setProperty('--primary-color', `${selectedTheme.primary.r}, ${selectedTheme.primary.g}, ${selectedTheme.primary.b}`);
    root.style.setProperty('--primary-hover-color', `${selectedTheme.primaryContainer.r}, ${selectedTheme.primaryContainer.g}, ${selectedTheme.primaryContainer.b}`);
    root.style.setProperty('--background-color', `${selectedTheme.background.r}, ${selectedTheme.background.g}, ${selectedTheme.background.b}`);
    root.style.setProperty('--surface-color', `${selectedTheme.surface.r}, ${selectedTheme.surface.g}, ${selectedTheme.surface.b}`);
    root.style.setProperty('--text-color', `${selectedTheme.onSurface.r}, ${selectedTheme.onSurface.g}, ${selectedTheme.onSurface.b}`);
    root.style.setProperty('--secondary-color', `${selectedTheme.secondary.r}, ${selectedTheme.secondary.g}, ${selectedTheme.secondary.b}`);
    root.style.setProperty('--outline-color', `${selectedTheme.outline.r}, ${selectedTheme.outline.g}, ${selectedTheme.outline.b}`);

    // 设置主题模式
    document.body.setAttribute('data-theme', isDark ? 'dark' : 'light');
}

/**
 * 便捷函数：从专辑封面提取并应用主题
 * @param {string} albumCoverUrl - 专辑封面URL
 * @param {boolean} isDark - 是否使用暗色主题
 * @returns {Promise<Object>} - 提取的主题对象
 */
export async function extractAndApplyThemeFromAlbum(albumCoverUrl, isDark = false) {
    try {
        const theme = await themeExtractor.extractThemeFromImage(albumCoverUrl);
        applyTheme(theme, isDark);
        return theme;
    } catch (error) {
        console.error('主题应用失败:', error);
        // 应用默认主题
        const defaultTheme = themeExtractor.getDefaultTheme();
        applyTheme(defaultTheme, isDark);
        return defaultTheme;
    }
}
