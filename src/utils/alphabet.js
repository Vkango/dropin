import { pinyin } from 'pinyin-pro'

export const ALPHABET = 'ABCDEFGHIJKLMNOPQRSTUVWXYZ'.split('')
export const ALL_INITIAL = ''
export const OTHER_INITIAL = '#'

export const alphabetOptions = [
  { value: OTHER_INITIAL, label: OTHER_INITIAL },
  ...ALPHABET.map((letter) => ({ value: letter, label: letter }))
]

const initialRank = new Map([
  [OTHER_INITIAL, 0],
  ...ALPHABET.map((letter, index) => [letter, index + 1])
])
const collator = new Intl.Collator('zh-Hans-CN', { numeric: true, sensitivity: 'base' })

const textValue = (value) => String(value ?? '').trim()

export const getInitial = (value) => {
  const text = textValue(value)
  if (!text) return OTHER_INITIAL

  const firstCharacter = Array.from(text)[0]
  if (/^[a-z]$/i.test(firstCharacter)) return firstCharacter.toUpperCase()

  if (/^[\u3400-\u9fff]$/.test(firstCharacter)) {
    const firstPinyin = pinyin(firstCharacter, { pattern: 'first', toneType: 'none' })
      .charAt(0)
      .toUpperCase()
    if (/^[A-Z]$/.test(firstPinyin)) return firstPinyin
  }

  return OTHER_INITIAL
}

export const getPinyinSortKey = (value) => {
  const text = textValue(value)
  if (!text) return ''

  return pinyin(text, { toneType: 'none' })
    .replace(/\s+/g, '')
    .toLocaleLowerCase()
}

export const compareByPinyin = (left, right) => {
  const leftKey = getPinyinSortKey(left)
  const rightKey = getPinyinSortKey(right)
  return collator.compare(leftKey, rightKey)
}

export const filterByInitial = (items, selectedInitial, getLabel) => {
  if (!selectedInitial) return [...items]
  return items.filter((item) => getInitial(getLabel(item)) === selectedInitial)
}

export const sortByInitial = (items, getLabel) => {
  return items.map(item => {
    const label = getLabel(item)
    return { item, rank: initialRank.get(getInitial(label)) ?? 0, key: getPinyinSortKey(label) }
  }).sort((left, right) => left.rank - right.rank || collator.compare(left.key, right.key))
    .map(entry => entry.item)
}

export const groupByInitial = (items, getLabel) => {
  const sortedItems = sortByInitial(items, getLabel)
  const groups = new Map()

  sortedItems.forEach((item) => {
    const initial = getInitial(getLabel(item))
    if (!groups.has(initial)) groups.set(initial, [])
    groups.get(initial).push(item)
  })

  return [...groups.entries()].map(([initial, groupedItems]) => ({
    initial,
    items: groupedItems
  }))
}

export const getAvailableInitials = (items, getLabel) => {
  return [...new Set(items.map((item) => getInitial(getLabel(item))))]
}
