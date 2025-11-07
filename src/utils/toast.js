// Toast 工具函数
let toastInstance = null

export function setToastInstance(instance) {
  toastInstance = instance
}

export function showToast(message, type = 'info', duration = 3000) {
  if (toastInstance) {
    return toastInstance.show(message, type, duration)
  }
  // 降级到 alert
  console.warn('Toast instance not initialized, falling back to alert')
  alert(message)
}

export function success(message, duration = 3000) {
  if (toastInstance) {
    return toastInstance.success(message, duration)
  }
  console.warn('Toast instance not initialized, falling back to alert')
  alert(message)
}

export function error(message, duration = 5000) {
  if (toastInstance) {
    return toastInstance.error(message, duration)
  }
  console.warn('Toast instance not initialized, falling back to alert')
  alert(message)
}

export function warning(message, duration = 4000) {
  if (toastInstance) {
    return toastInstance.warning(message, duration)
  }
  console.warn('Toast instance not initialized, falling back to alert')
  alert(message)
}

export function info(message, duration = 3000) {
  if (toastInstance) {
    return toastInstance.info(message, duration)
  }
  console.warn('Toast instance not initialized, falling back to alert')
  alert(message)
}

