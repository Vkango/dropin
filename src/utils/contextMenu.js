import { reactive } from 'vue'

// 通用右键菜单单例：任何组件通过 openContextMenu() 弹出菜单，
// 返回 Promise，在用户选中某项（或菜单被关闭）时决议。
export const contextMenuState = reactive({
  open: false,
  // 每次打开自增，作为 AnimatePresence 的 key，保证重复打开时重新播放入场动画
  session: 0,
  x: 0,
  y: 0,
  items: [],
  minWidth: 210
})

let pendingResolve = null

export function openContextMenu(options = {}) {
  const { x = 0, y = 0, items = [], minWidth = 210 } = options
  // 若已有菜单打开，先以“未选择”决议旧的 Promise
  if (pendingResolve) pendingResolve(null)
  contextMenuState.session += 1
  contextMenuState.x = x
  contextMenuState.y = y
  contextMenuState.items = items
  contextMenuState.minWidth = minWidth
  contextMenuState.open = true
  return new Promise((resolve) => {
    pendingResolve = resolve
  })
}

export function selectContextMenuItem(item) {
  if (!contextMenuState.open || !item) return
  finish({ id: item.id, item })
}

export function closeContextMenu() {
  if (!contextMenuState.open) return
  finish(null)
}

function finish(value) {
  contextMenuState.open = false
  const resolve = pendingResolve
  pendingResolve = null
  resolve?.(value)
}
