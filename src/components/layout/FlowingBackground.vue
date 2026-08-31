<template>
    <div ref="rootRef" class="flowing-background" aria-hidden="true">
        <canvas ref="canvasRef" class="flowing-canvas"></canvas>
        <div ref="fallbackRef" class="flowing-fallback"></div>
    </div>
</template>

<script setup>
import { onBeforeUnmount, onMounted, ref, watch } from 'vue'
import { useAppSettingsStore } from '@/stores/appSettingsStore.js'
import { updateWebglDiagnostics } from '@/services/webglDiagnostics.js'

const props = defineProps({
    cover: {
        type: String,
        default: ''
    },
    bands: {
        type: Object,
        default: () => ({ bass: 0, mid: 0, treble: 0, level: 0 })
    }
})

const settingsStore = useAppSettingsStore()

const rootRef = ref(null)
const canvasRef = ref(null)
const fallbackRef = ref(null)

const DEFAULT_PALETTE = [
    [60, 15, 20],
    [120, 20, 25],
    [190, 40, 45],
    [230, 120, 110],
    [245, 200, 190]
]
const BLOB_N = 5
const QUALITY_SCALE = 0.5
const BASE_FLOW_SPEED = 2.4
const AUDIO_RESPONSE_SECONDS = 0.9
const AUDIO_RELEASE_SECONDS = 1.6
const TAU = Math.PI * 2

let canvas = null
let fallback = null
let gl = null
let program = null
let uniforms = null
let resizeObserver = null
let animationFrame = 0
let lastTime = 0
let shaderTime = Math.random() * 200
let animationGeneration = 0
let blobs = []
let coverRequestId = 0
let hasLoadedCoverPalette = false
let reducedMotion = false
let contextLostHandler = null
const blobValues = new Float32Array(BLOB_N * 4)
const currentPalette = DEFAULT_PALETTE.map((color) => [...color])
const targetPalette = DEFAULT_PALETTE.map((color) => [...color])
const smoothedBands = { bass: 0, mid: 0, treble: 0, level: 0 }

const VERTEX_SHADER = `
attribute vec2 aPos;
void main() { gl_Position = vec4(aPos, 0.0, 1.0); }
`

const FRAGMENT_SHADER = `
precision mediump float;
uniform vec2 uRes;
uniform float uTime;
uniform float uBass;
uniform float uMid;
uniform float uTreble;
uniform float uLevel;
uniform vec3 uBase;
uniform vec3 uCol[5];
uniform vec4 uBlob[5];

float hash(vec2 p) {
  p = fract(p * vec2(234.34, 435.345));
  p += dot(p, p + 34.23);
  return fract(p.x * p.y);
}

float vnoise(vec2 p) {
  vec2 i = floor(p);
  vec2 f = fract(p);
  vec2 u = f * f * (3.0 - 2.0 * f);
  return mix(mix(hash(i), hash(i + vec2(1.0, 0.0)), u.x),
             mix(hash(i + vec2(0.0, 1.0)), hash(i + vec2(1.0, 1.0)), u.x), u.y);
}

void main() {
  vec2 uv = gl_FragCoord.xy / uRes;
  float aspect = uRes.x / uRes.y;
  float t = uTime * 0.028;
  vec2 warp = vec2(
    vnoise(uv * 1.7 + vec2(t, 0.0)),
    vnoise(uv * 1.7 + vec2(4.7, 9.1) - t * 0.8)
  ) - 0.5;
  vec2 suv = uv + warp * (0.045 + uTreble * 0.008);
  vec2 p = vec2(suv.x * aspect, suv.y);

  vec3 colSum = uBase * 0.5;
  float wSum = 0.5;
  for (int i = 0; i < 5; i++) {
    vec2 bp = vec2(uBlob[i].x * aspect, uBlob[i].y);
    vec2 dv = p - bp;
    float sig = uBlob[i].z;
    float w = exp(-dot(dv, dv) / (2.0 * sig * sig)) * uBlob[i].w;
    colSum += uCol[i] * w;
    wSum += w;
  }
  vec3 col = colSum / wSum;
  float shade = vnoise(uv * 1.05 + vec2(0.0, t * 0.55));
  col *= 0.86 + 0.28 * shade;
  col *= 1.0 + uLevel * 0.035;
  float d = length(uv - 0.5);
  col *= 1.0 - 0.28 * smoothstep(0.42, 0.95, d);
  col += (hash(gl_FragCoord.xy) - 0.5) * 0.012;
  gl_FragColor = vec4(col, 1.0);
}
`

function rgbToHsl(r, g, b) {
    r /= 255
    g /= 255
    b /= 255
    const max = Math.max(r, g, b)
    const min = Math.min(r, g, b)
    const lightness = (max + min) / 2
    if (max === min) return [0, 0, lightness]
    const delta = max - min
    const saturation = lightness > 0.5
        ? delta / (2 - max - min)
        : delta / (max + min)
    let hue
    if (max === r) hue = (g - b) / delta + (g < b ? 6 : 0)
    else if (max === g) hue = (b - r) / delta + 2
    else hue = (r - g) / delta + 4
    return [hue * 60, saturation, lightness]
}

function hslToRgb(h, s, l) {
    h = ((h % 360) + 360) % 360 / 360
    const q = l < 0.5 ? l * (1 + s) : l + s - l * s
    const p = 2 * l - q
    const hueToRgb = (value) => {
        value = ((value % 1) + 1) % 1
        if (value < 1 / 6) return p + (q - p) * 6 * value
        if (value < 1 / 2) return q
        if (value < 2 / 3) return p + (q - p) * (2 / 3 - value) * 6
        return p
    }
    return [hueToRgb(h + 1 / 3), hueToRgb(h), hueToRgb(h - 1 / 3)].map((value) => Math.round(value * 255))
}

function getThemePalette() {
    const primary = getComputedStyle(document.documentElement)
        .getPropertyValue('--primary-color')
        .split(',')
        .map((value) => Number(value.trim()))

    if (primary.length !== 3 || primary.some((value) => !Number.isFinite(value))) {
        return DEFAULT_PALETTE.map((color) => [...color])
    }

    const [hue, saturation, lightness] = rgbToHsl(...primary)
    const clampLightness = (value) => Math.max(0.12, Math.min(0.88, value))
    const clampSaturation = (value) => Math.max(0.18, Math.min(0.9, value))

    return [
        hslToRgb(hue, clampSaturation(saturation * 0.82), clampLightness(lightness * 0.34)),
        hslToRgb(hue, clampSaturation(saturation * 0.92), clampLightness(lightness * 0.58)),
        primary.map((value) => Math.round(value)),
        hslToRgb(hue, clampSaturation(saturation * 0.86), clampLightness(lightness * 1.12)),
        hslToRgb(hue, clampSaturation(saturation * 0.72), clampLightness(lightness * 1.32))
    ]
}

function medianCut(pixels, count) {
    let boxes = [pixels]
    while (boxes.length < count) {
        let boxIndex = 0
        let maxVariance = -1
        for (let index = 0; index < boxes.length; index++) {
            if (boxes[index].length < 2) continue
            const min = [255, 255, 255]
            const max = [0, 0, 0]
            for (const pixel of boxes[index]) {
                for (let channel = 0; channel < 3; channel++) {
                    min[channel] = Math.min(min[channel], pixel[channel])
                    max[channel] = Math.max(max[channel], pixel[channel])
                }
            }
            const variance = Math.max(max[0] - min[0], max[1] - min[1], max[2] - min[2])
            if (variance > maxVariance) {
                maxVariance = variance
                boxIndex = index
            }
        }
        if (maxVariance <= 0) break
        const box = boxes.splice(boxIndex, 1)[0]
        const min = [255, 255, 255]
        const max = [0, 0, 0]
        for (const pixel of box) {
            for (let channel = 0; channel < 3; channel++) {
                min[channel] = Math.min(min[channel], pixel[channel])
                max[channel] = Math.max(max[channel], pixel[channel])
            }
        }
        let splitChannel = 0
        let splitRange = -1
        for (let channel = 0; channel < 3; channel++) {
            if (max[channel] - min[channel] > splitRange) {
                splitRange = max[channel] - min[channel]
                splitChannel = channel
            }
        }
        box.sort((a, b) => a[splitChannel] - b[splitChannel])
        const middle = box.length >> 1
        boxes.push(box.slice(0, middle), box.slice(middle))
    }
    return boxes.map((box) => {
        const color = box.reduce((sum, pixel) => sum.map((value, index) => value + pixel[index]), [0, 0, 0])
        const size = box.length || 1
        return { rgb: color.map((value) => Math.round(value / size)), weight: box.length }
    })
}

function rgbToLab(r, g, b) {
    const linear = (value) => {
        value /= 255
        return value > 0.04045 ? Math.pow((value + 0.055) / 1.055, 2.4) : value / 12.92
    }
    r = linear(r)
    g = linear(g)
    b = linear(b)
    let x = (r * 0.4124 + g * 0.3576 + b * 0.1805) / 0.95047
    let y = r * 0.2126 + g * 0.7152 + b * 0.0722
    let z = (r * 0.0193 + g * 0.1192 + b * 0.9505) / 1.08883
    const transform = (value) => value > 0.008856 ? Math.cbrt(value) : 7.787 * value + 16 / 116
    x = transform(x)
    y = transform(y)
    z = transform(z)
    return [116 * y - 16, 500 * (x - y), 200 * (y - z)]
}

const labDistance = (a, b) => Math.hypot(a[0] - b[0], a[1] - b[1], a[2] - b[2])

function extractPalette(image, count = 5) {
    const size = 48
    const sample = document.createElement('canvas')
    sample.width = size
    sample.height = size
    const context = sample.getContext('2d', { willReadFrequently: true })
    context.drawImage(image, 0, 0, size, size)
    const pixels = context.getImageData(0, 0, size, size).data
    const colors = []
    for (let index = 0; index < pixels.length; index += 4) {
        colors.push([pixels[index], pixels[index + 1], pixels[index + 2]])
    }

    const clusters = medianCut(colors, 16)
    clusters.forEach((cluster) => {
        cluster.hsl = rgbToHsl(...cluster.rgb)
        cluster.lab = rgbToLab(...cluster.rgb)
        cluster.score = Math.sqrt(cluster.weight) * (0.35 + cluster.hsl[1])
    })
    const sorted = [...clusters].sort((a, b) => b.score - a.score)
    const chosen = sorted[0] ? [sorted[0]] : []
    while (chosen.length < count && chosen.length < sorted.length) {
        let best = null
        let bestValue = -1
        for (const cluster of sorted) {
            if (chosen.includes(cluster)) continue
            const minDistance = Math.min(...chosen.map((selected) => labDistance(cluster.lab, selected.lab)))
            const value = minDistance * (0.5 + cluster.score / sorted[0].score)
            if (value > bestValue) {
                bestValue = value
                best = cluster
            }
        }
        if (!best) break
        chosen.push(best)
    }

    const dominantHue = sorted[0]?.hsl[0] || 0
    const result = chosen.map((cluster) => {
        let [hue, saturation, lightness] = cluster.hsl
        if (saturation < 0.1) {
            hue = dominantHue
            saturation = 0.35
        }
        saturation = Math.min(1, saturation * 1.55 + 0.1)
        lightness = Math.max(0.12, Math.min(0.78, lightness))
        return hslToRgb(hue, saturation, lightness)
    })
    result.sort((a, b) => rgbToHsl(...a)[2] - rgbToHsl(...b)[2])
    while (result.length < count) result.push(DEFAULT_PALETTE[result.length % DEFAULT_PALETTE.length])
    return result.slice(0, count)
}

function createShader(type, source) {
    const shader = gl.createShader(type)
    gl.shaderSource(shader, source)
    gl.compileShader(shader)
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        console.warn('流沙背景 shader 编译失败:', gl.getShaderInfoLog(shader))
        gl.deleteShader(shader)
        return null
    }
    return shader
}

function initWebgl() {
    gl = canvas.getContext('webgl', {
        antialias: false,
        depth: false,
        stencil: false,
        alpha: false,
        powerPreference: settingsStore.state.gpuMode === 'high-performance'
            ? 'high-performance'
            : settingsStore.state.gpuMode === 'compatibility' ? 'default' : 'low-power',
        preserveDrawingBuffer: false
    })
    if (!gl) {
        updateWebglDiagnostics({ status: 'unavailable', error: 'WebGL context creation failed', contextLost: false })
        return false
    }

    const debugInfo = gl.getExtension('WEBGL_debug_renderer_info')
    updateWebglDiagnostics({
        status: 'ready',
        renderer: debugInfo ? gl.getParameter(debugInfo.UNMASKED_RENDERER_WEBGL) : gl.getParameter(gl.RENDERER),
        vendor: debugInfo ? gl.getParameter(debugInfo.UNMASKED_VENDOR_WEBGL) : gl.getParameter(gl.VENDOR),
        version: gl.getParameter(gl.VERSION),
        shadingLanguageVersion: gl.getParameter(gl.SHADING_LANGUAGE_VERSION),
        context: 'webgl',
        error: '',
        contextLost: false
    })

    const vertex = createShader(gl.VERTEX_SHADER, VERTEX_SHADER)
    const highp = gl.getShaderPrecisionFormat?.(gl.FRAGMENT_SHADER, gl.HIGH_FLOAT)
    const fragmentSource = highp?.precision > 0
        ? FRAGMENT_SHADER.replace('precision mediump float;', 'precision highp float;')
        : FRAGMENT_SHADER
    const fragment = createShader(gl.FRAGMENT_SHADER, fragmentSource)
    if (!vertex || !fragment) {
        updateWebglDiagnostics({ status: 'shader-failed', error: 'WebGL shader compilation failed' })
        return false
    }

    program = gl.createProgram()
    gl.attachShader(program, vertex)
    gl.attachShader(program, fragment)
    gl.linkProgram(program)
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
        console.warn('流沙背景 shader 链接失败:', gl.getProgramInfoLog(program))
        updateWebglDiagnostics({ status: 'shader-failed', error: gl.getProgramInfoLog(program) || 'WebGL shader linking failed' })
        return false
    }
    gl.useProgram(program)

    const buffer = gl.createBuffer()
    gl.bindBuffer(gl.ARRAY_BUFFER, buffer)
    gl.bufferData(gl.ARRAY_BUFFER, new Float32Array([-1, -1, 3, -1, -1, 3]), gl.STATIC_DRAW)
    const position = gl.getAttribLocation(program, 'aPos')
    gl.enableVertexAttribArray(position)
    gl.vertexAttribPointer(position, 2, gl.FLOAT, false, 0, 0)

    uniforms = {
        res: gl.getUniformLocation(program, 'uRes'),
        time: gl.getUniformLocation(program, 'uTime'),
        bass: gl.getUniformLocation(program, 'uBass'),
        mid: gl.getUniformLocation(program, 'uMid'),
        treble: gl.getUniformLocation(program, 'uTreble'),
        level: gl.getUniformLocation(program, 'uLevel'),
        base: gl.getUniformLocation(program, 'uBase'),
        colors: gl.getUniformLocation(program, 'uCol[0]'),
        blobs: gl.getUniformLocation(program, 'uBlob[0]')
    }
    contextLostHandler = (event) => {
        event.preventDefault()
        stopAnimation()
        updateWebglDiagnostics({ status: 'context-lost', error: 'WebGL context lost', contextLost: true })
        showFallback()
    }
    canvas.addEventListener('webglcontextlost', contextLostHandler)
    return true
}

function releaseWebgl() {
    stopAnimation()
    if (contextLostHandler) canvas?.removeEventListener('webglcontextlost', contextLostHandler)
    if (gl && program) gl.deleteProgram(program)
    contextLostHandler = null
    gl = null
    program = null
    uniforms = null
}

function mulberry32(seed) {
    return () => {
        seed |= 0
        seed = seed + 0x6D2B79F5 | 0
        let value = Math.imul(seed ^ seed >>> 15, 1 | seed)
        value = value + Math.imul(value ^ value >>> 7, 61 | value) ^ value
        return ((value ^ value >>> 14) >>> 0) / 4294967296
    }
}

function makeBlobs(palette) {
    const random = mulberry32((Math.random() * 1e9) | 0)
    blobs = palette.slice(0, BLOB_N).map(() => ({
        cx: 0.15 + random() * 0.7,
        cy: 0.18 + random() * 0.64,
        rx: 0.1 + random() * 0.26,
        ry: 0.09 + random() * 0.22,
        wx: 0.04 + random() * 0.06,
        wy: 0.035 + random() * 0.05,
        direction: random() < 0.5 ? -1 : 1,
        flowSpeed: 0.75 + random() * 0.5,
        audioInfluence: random() < 0.7 ? 0.65 + random() * 0.35 : 0,
        audioBand: Math.floor(random() * 3),
        audioDirection: random() < 0.5 ? -1 : 1,
        audioTurn: 0,
        phaseX: random() * TAU,
        phaseY: random() * TAU,
        breathePhase: random() * TAU,
        weightPhase: random() * TAU,
        sig: 0.12 + random() * 0.11,
        weight: 0.85 + random() * 0.5
    }))

    for (let index = 0; index < BLOB_N; index++) {
        currentPalette[index] = [...(palette[index] || DEFAULT_PALETTE[index])]
        targetPalette[index] = [...currentPalette[index]]
    }
    uploadPalette()
}

function uploadPalette() {
    const colors = new Float32Array(BLOB_N * 3)
    currentPalette.forEach((color, index) => {
        colors[index * 3] = color[0] / 255
        colors[index * 3 + 1] = color[1] / 255
        colors[index * 3 + 2] = color[2] / 255
    })
    gl.uniform3fv(uniforms.colors, colors)
    const dark = currentPalette[0] || DEFAULT_PALETTE[0]
    gl.uniform3f(uniforms.base, dark[0] / 255 * 0.45, dark[1] / 255 * 0.45, dark[2] / 255 * 0.45)
}

function setPalette(palette, immediate = false) {
    for (let index = 0; index < BLOB_N; index++) {
        const nextColor = palette[index] || DEFAULT_PALETTE[index]
        targetPalette[index] = [...nextColor]
        if (immediate) currentPalette[index] = [...nextColor]
    }
}

function updatePalette(delta) {
    const amount = 1 - Math.exp(-delta / 0.9)
    let changed = false
    for (let index = 0; index < BLOB_N; index++) {
        for (let channel = 0; channel < 3; channel++) {
            const current = currentPalette[index][channel]
            const target = targetPalette[index][channel]
            const next = current + (target - current) * amount
            currentPalette[index][channel] = next
            changed ||= Math.abs(target - next) > 0.1
        }
    }
    if (changed || currentPalette.some((color, index) => color.some((value, channel) => value !== targetPalette[index][channel]))) {
        uploadPalette()
    }
}

function readBands() {
    const value = props.bands || {}
    return {
        bass: Number.isFinite(Number(value.bass)) ? Math.max(0, Math.min(1, Number(value.bass))) : 0,
        mid: Number.isFinite(Number(value.mid)) ? Math.max(0, Math.min(1, Number(value.mid))) : 0,
        treble: Number.isFinite(Number(value.treble)) ? Math.max(0, Math.min(1, Number(value.treble))) : 0,
        level: Number.isFinite(Number(value.level)) ? Math.max(0, Math.min(1, Number(value.level))) : 0
    }
}

function updateBlobs(delta) {
    const target = readBands()
    for (const key of ['bass', 'mid', 'treble', 'level']) {
        const responseSeconds = target[key] > smoothedBands[key]
            ? AUDIO_RESPONSE_SECONDS
            : AUDIO_RELEASE_SECONDS
        const amount = 1 - Math.exp(-delta / responseSeconds)
        smoothedBands[key] += (target[key] - smoothedBands[key]) * amount
    }
    const audio = smoothedBands
    shaderTime += delta * BASE_FLOW_SPEED
    for (let index = 0; index < blobs.length; index++) {
        const blob = blobs[index]
        const band = audio[blob.audioBand === 0 ? 'bass' : blob.audioBand === 1 ? 'mid' : 'treble']
        const audioAmount = band * blob.audioInfluence
        const speed = blob.flowSpeed * (1 + audioAmount * 0.16)
        const turnTarget = blob.audioDirection * audioAmount * 0.24
        blob.audioTurn += (turnTarget - blob.audioTurn) * (1 - Math.exp(-delta / 1.2))
        // Integrate phase instead of multiplying the accumulated time by the
        // current audio-reactive speed. The latter re-applies a new speed to
        // the entire history whenever the audio level changes, which makes
        // the animation appear to accelerate over time.
        blob.phaseX = (blob.phaseX + delta * BASE_FLOW_SPEED * blob.wx * speed * blob.direction) % TAU
        blob.phaseY = (blob.phaseY + delta * BASE_FLOW_SPEED * blob.wy * speed * blob.direction) % TAU
        blob.breathePhase = (blob.breathePhase + delta * BASE_FLOW_SPEED * 0.9) % TAU
        blob.weightPhase = (blob.weightPhase + delta * BASE_FLOW_SPEED * 0.7) % TAU
        const phaseX = blob.phaseX
        const phaseY = blob.phaseY
        const localX = blob.rx * Math.sin(phaseX)
        const localY = blob.ry * Math.sin(phaseY)
        const cosTurn = Math.cos(blob.audioTurn)
        const sinTurn = Math.sin(blob.audioTurn)
        const x = blob.cx + localX * cosTurn - localY * sinTurn
        const y = blob.cy + localX * sinTurn + localY * cosTurn
        const breathe = 0.6 + 0.4 * Math.sin(blob.breathePhase)
        const sigma = blob.sig * (1 + 0.1 * audio.bass * breathe)
        const weight = blob.weight * (1 + 0.08 * audio.mid * Math.sin(blob.weightPhase))
        blobValues[index * 4] = Math.max(-0.2, Math.min(1.2, x))
        blobValues[index * 4 + 1] = Math.max(-0.2, Math.min(1.2, y))
        blobValues[index * 4 + 2] = sigma
        blobValues[index * 4 + 3] = weight
    }
    gl.uniform4fv(uniforms.blobs, blobValues)
    return audio
}

function resize() {
    const dpr = Math.min(window.devicePixelRatio || 1, 2)
    canvas.width = Math.max(2, Math.round(canvas.clientWidth * QUALITY_SCALE * dpr))
    canvas.height = Math.max(2, Math.round(canvas.clientHeight * QUALITY_SCALE * dpr))
    gl?.viewport(0, 0, canvas.width, canvas.height)
}

function draw(now) {
    if (!gl || !program || !uniforms) return
    const delta = Math.min(0.1, (now - lastTime) / 1000)
    lastTime = now
    updatePalette(delta)
    const audio = updateBlobs(delta)
    gl.uniform2f(uniforms.res, canvas.width, canvas.height)
    gl.uniform1f(uniforms.time, shaderTime)
    gl.uniform1f(uniforms.bass, audio.bass)
    gl.uniform1f(uniforms.mid, audio.mid)
    gl.uniform1f(uniforms.treble, audio.treble)
    gl.uniform1f(uniforms.level, audio.level)
    gl.drawArrays(gl.TRIANGLES, 0, 3)
}

function renderFrame(now, generation) {
    if (generation !== animationGeneration) return
    animationFrame = requestAnimationFrame((timestamp) => renderFrame(timestamp, generation))
    draw(now)
}

function startAnimation() {
    if (!animationFrame && !reducedMotion) {
        const generation = ++animationGeneration
        lastTime = performance.now()
        animationFrame = requestAnimationFrame((timestamp) => renderFrame(timestamp, generation))
    }
}

function stopAnimation() {
    animationGeneration++
    if (animationFrame) cancelAnimationFrame(animationFrame)
    animationFrame = 0
}

function setFallback(palette) {
    if (!fallback) return
    const hex = (color) => `#${color.map((value) => value.toString(16).padStart(2, '0')).join('')}`
    const colors = palette.length ? palette : DEFAULT_PALETTE
    fallback.style.background = [
        `radial-gradient(circle at 25% 35%, ${hex(colors[2] || colors[0])} 0%, transparent 45%)`,
        `radial-gradient(circle at 75% 25%, ${hex(colors[4] || colors[1] || colors[0])} 0%, transparent 42%)`,
        `radial-gradient(circle at 65% 75%, ${hex(colors[3] || colors[1] || colors[0])} 0%, transparent 45%)`,
        `radial-gradient(circle at 30% 80%, ${hex(colors[1] || colors[0])} 0%, transparent 48%)`,
        `linear-gradient(160deg, ${hex(colors[0])}, ${hex(colors[2] || colors[0])})`
    ].join(',')
}

function showFallback() {
    canvas.style.display = 'none'
    fallback.style.display = 'block'
}

function showWebgl() {
    canvas.style.display = 'block'
    fallback.style.display = 'none'
}

function applyPalette(palette, { immediate = false } = {}) {
    setFallback(palette)
    if (!gl || !program) return
    setPalette(palette, immediate)
    if (immediate) uploadPalette()
    resize()
    showWebgl()
    if (reducedMotion) {
        draw(performance.now())
    } else {
        startAnimation()
    }
}

function loadCover() {
    const requestId = ++coverRequestId
    const image = new Image()
    image.onload = () => {
        if (requestId !== coverRequestId) return
        try {
            const palette = extractPalette(image)
            applyPalette(palette, { immediate: !hasLoadedCoverPalette })
            hasLoadedCoverPalette = true
        } catch (error) {
            console.warn('专辑封面取色失败:', error)
            applyPalette(getThemePalette(), { immediate: !hasLoadedCoverPalette })
        }
    }
    image.onerror = () => {
        if (requestId === coverRequestId) applyPalette(getThemePalette(), { immediate: !hasLoadedCoverPalette })
    }
    image.src = props.cover || '/assets/cover.jpg'
}

function handleVisibilityChange() {
    if (document.hidden) stopAnimation()
    else startAnimation()
}

onMounted(() => {
    canvas = canvasRef.value
    fallback = fallbackRef.value
    reducedMotion = window.matchMedia('(prefers-reduced-motion: reduce)').matches
    const initialPalette = getThemePalette()
    setFallback(initialPalette)
    showFallback()

    if (initWebgl()) {
        makeBlobs(initialPalette)
        resize()
        loadCover()
    } else {
        loadCover()
    }

    resizeObserver = new ResizeObserver(resize)
    resizeObserver.observe(rootRef.value)
    window.addEventListener('resize', resize)
    document.addEventListener('visibilitychange', handleVisibilityChange)
    if (gl) startAnimation()
})

watch(() => props.cover, loadCover)

onBeforeUnmount(() => {
    coverRequestId++
    stopAnimation()
    resizeObserver?.disconnect()
    window.removeEventListener('resize', resize)
    document.removeEventListener('visibilitychange', handleVisibilityChange)
    releaseWebgl()
})
</script>

<style scoped>
.flowing-background {
    position: absolute;
    inset: 0;
    overflow: hidden;
    background: #090807;
}

.flowing-canvas,
.flowing-fallback {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
}

.flowing-canvas {
    display: block;
}

.flowing-fallback {
    display: none;
    inset: -20%;
    filter: blur(60px);
    animation: flowing-background-drift 40s ease-in-out infinite alternate;
}

@keyframes flowing-background-drift {
    0% {
        transform: translate3d(-4%, -3%, 0) rotate(0deg) scale(1);
    }

    50% {
        transform: translate3d(3%, 2%, 0) rotate(4deg) scale(1.12);
    }

    100% {
        transform: translate3d(-2%, 4%, 0) rotate(-3deg) scale(1.05);
    }
}

@media (prefers-reduced-motion: reduce) {
    .flowing-fallback {
        animation: none;
    }
}
</style>
