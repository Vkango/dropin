import { t } from '@/i18n/index.js'

// 指令引擎：解析层与执行层分离。
// 未来接入 Agent 时，只需让 LLM 把自然语言转成同构意图
// { type: 'command', name, args }，执行层无需改动。

export const parseInput = (input) => {
  const text = String(input || '').trim()
  if (!text) return { type: 'empty' }
  if (text.startsWith('/')) {
    const segments = text.slice(1).split(/\s+/).filter(Boolean)
    return {
      type: 'command',
      name: (segments[0] || '').toLowerCase(),
      args: segments.slice(1),
      argsText: segments.slice(1).join(' ')
    }
  }
  return { type: 'query', text }
}

const includes = (haystack, needle) =>
  String(haystack || '').toLowerCase().includes(String(needle || '').toLowerCase())

const pickMatch = (items, text, readLabel) => {
  const exact = items.find((item) => String(readLabel(item) || '').toLowerCase() === text.toLowerCase())
  if (exact) return [exact]
  return items.filter((item) => includes(readLabel(item), text))
}

export const searchSongs = (songs, text) => {
  const tokens = String(text || '').split(/\s+/).filter(Boolean)
  if (!tokens.length) return []
  return songs.filter((song) => tokens.every((token) => (
    includes(song.title, token)
    || includes(song.artist, token)
    || includes(song.album, token)
  )))
}

const shuffle = (list) => {
  const copy = list.slice()
  for (let i = copy.length - 1; i > 0; i -= 1) {
    const j = Math.floor(Math.random() * (i + 1))
    ;[copy[i], copy[j]] = [copy[j], copy[i]]
  }
  return copy
}

const songsResult = (tracks, title, extra = {}) => ({ kind: 'songs', tracks, title, ...extra })

const usageHint = (usage) => ({ kind: 'hint', message: t('home.command.missingArg', { usage }) })

const commands = [
  {
    name: 'help',
    descKey: 'home.command.helpDesc',
    run: async () => ({ kind: 'help' })
  },
  {
    name: 'artist',
    argKey: 'home.command.argName',
    descKey: 'home.command.artistDesc',
    valueSource: (ctx) => (ctx.artists || []).map((artist) => ({ label: artist.name, value: artist.name })),
    run: async (args, ctx) => {
      const name = args.join(' ').trim()
      if (!name) return usageHint('/artist')
      const matched = pickMatch(ctx.artists || [], name, (artist) => artist.name)
      const names = matched.map((artist) => artist.name)
      const tracks = ctx.tracks.filter((song) => names.includes(song.artist))
      return songsResult(tracks, names.join(' / ') || name)
    }
  },
  {
    name: 'album',
    argKey: 'home.command.argName',
    descKey: 'home.command.albumDesc',
    valueSource: (ctx) => (ctx.albums || []).map((album) => ({ label: album.title, value: album.title })),
    run: async (args, ctx) => {
      const name = args.join(' ').trim()
      if (!name) return usageHint('/album')
      const matched = pickMatch(ctx.albums || [], name, (album) => album.title)
      const titles = matched.map((album) => album.title)
      const tracks = ctx.tracks.filter((song) => titles.includes(song.album))
      return songsResult(tracks, titles.join(' / ') || name)
    }
  },
  {
    name: 'tag',
    argKey: 'home.command.argName',
    descKey: 'home.command.tagDesc',
    valueSource: (ctx) => (ctx.tags || []).map((tag) => ({ label: tag.label, value: tag.label })),
    run: async (args, ctx) => {
      const label = args.join(' ').trim()
      if (!label) return usageHint('/tag')
      const matched = pickMatch(ctx.tags || [], label, (tag) => tag.label)
      if (!matched.length) return songsResult([], label)
      const collected = []
      for (const tag of matched) {
        const tracks = await ctx.tracksByTag(tag.id)
        collected.push(...tracks)
      }
      const seen = new Set()
      const tracks = collected.filter((song) => !seen.has(song.id) && seen.add(song.id))
      return songsResult(tracks, matched.map((tag) => tag.label).join(' / ') || label)
    }
  },
  {
    name: 'random',
    argKey: 'home.command.argCount',
    descKey: 'home.command.randomDesc',
    run: async (args, ctx) => {
      const count = Math.max(1, Math.min(500, Number(args[0]) || 20))
      return songsResult(shuffle(ctx.tracks).slice(0, count), t('home.command.randomTitle', { count }))
    }
  },
  {
    name: 'recent',
    descKey: 'home.command.recentDesc',
    run: async (args, ctx) => {
      const byId = new Map(ctx.tracks.map((song) => [song.id, song]))
      const tracks = []
      const seen = new Set()
      for (const entry of ctx.history || []) {
        const song = byId.get(entry.trackId)
        if (song && !seen.has(song.id)) {
          seen.add(song.id)
          tracks.push(song)
        }
      }
      return songsResult(tracks, t('home.command.recentTitle'))
    }
  },
  {
    name: 'all',
    descKey: 'home.command.allDesc',
    run: async (args, ctx) => songsResult(ctx.tracks, t('home.command.allTitle'))
  },
  {
    name: 'play',
    argKey: 'home.command.argKeyword',
    descKey: 'home.command.playDesc',
    run: async (args, ctx) => {
      const keyword = args.join(' ').trim()
      if (!keyword) return usageHint('/play')
      return songsResult(searchSongs(ctx.tracks, keyword), keyword, { autoPlay: true })
    }
  }
]

export const listCommands = () => commands.map((command) => ({
  name: command.name,
  usage: `/${command.name}${command.argKey ? ` <${t(command.argKey)}>` : ''}`,
  description: t(command.descKey)
}))

const findCommand = (name) => commands.find((command) => command.name === name)

export const execute = async (input, ctx) => {
  const parsed = parseInput(input)
  if (parsed.type === 'empty') return { kind: 'idle' }
  if (parsed.type === 'query') {
    return songsResult(searchSongs(ctx.tracks, parsed.text), parsed.text)
  }
  const command = findCommand(parsed.name)
  if (!command) {
    const typing = commands.some((item) => item.name.startsWith(parsed.name))
    if (typing) return { kind: 'idle' }
    return { kind: 'error', message: t('home.command.unknown', { name: parsed.name }) }
  }
  return command.run(parsed.args, ctx)
}

// 输入联想：指令名补全 + 参数值补全（Minecraft 式）。
export const suggest = (input, ctx) => {
  const parsed = parseInput(input)
  if (parsed.type !== 'command') return []

  const raw = String(input || '').trim()
  const hasSpace = /\s/.test(raw.slice(1))
  if (!hasSpace) {
    return commands
      .filter((command) => command.name.startsWith(parsed.name))
      .map((command) => ({
        value: `/${command.name}${command.argKey ? ' ' : ''}`,
        usage: `/${command.name}${command.argKey ? ` <${t(command.argKey)}>` : ''}`,
        description: t(command.descKey)
      }))
  }

  const command = findCommand(parsed.name)
  if (!command?.valueSource) return []
  return (command.valueSource(ctx) || [])
    .filter((item) => includes(item.label, parsed.argsText))
    .slice(0, 8)
    .map((item) => ({
      value: `/${command.name} ${item.value}`,
      usage: item.label,
      description: t(command.descKey)
    }))
}
