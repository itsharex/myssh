// 全局快捷键管理
const shortcuts = new Map()

export function registerShortcut(key, handler, options = {}) {
  const { ctrl = false, shift = false, alt = false, meta = false, preventDefault = true } = options
  
  const keyHandler = (event) => {
    const ctrlPressed = event.ctrlKey || event.metaKey
    const shiftPressed = event.shiftKey
    const altPressed = event.altKey
    const metaPressed = event.metaKey
    
    if (
      ctrl === ctrlPressed &&
      shift === shiftPressed &&
      alt === altPressed &&
      (meta === metaPressed || (!meta && !metaPressed)) &&
      event.key.toLowerCase() === key.toLowerCase()
    ) {
      if (preventDefault) {
        event.preventDefault()
      }
      handler(event)
    }
  }
  
  shortcuts.set(`${key}-${ctrl}-${shift}-${alt}-${meta}`, keyHandler)
  document.addEventListener('keydown', keyHandler)
  
  return () => {
    document.removeEventListener('keydown', keyHandler)
    shortcuts.delete(`${key}-${ctrl}-${shift}-${alt}-${meta}`)
  }
}

export function unregisterShortcut(key, options = {}) {
  const { ctrl = false, shift = false, alt = false, meta = false } = options
  const keyHandler = shortcuts.get(`${key}-${ctrl}-${shift}-${alt}-${meta}`)
  if (keyHandler) {
    document.removeEventListener('keydown', keyHandler)
    shortcuts.delete(`${key}-${ctrl}-${shift}-${alt}-${meta}`)
  }
}

export function unregisterAllShortcuts() {
  shortcuts.forEach((handler) => {
    document.removeEventListener('keydown', handler)
  })
  shortcuts.clear()
}

