<template>
  <TransitionGroup name="toast" tag="div" class="toast-container">
    <div
      v-for="toast in toasts"
      :key="toast.id"
      :class="['toast', `toast-${toast.type}`]"
    >
      <span class="toast-icon">{{ getIcon(toast.type) }}</span>
      <span class="toast-message">{{ toast.message }}</span>
      <button @click="removeToast(toast.id)" class="toast-close">×</button>
    </div>
  </TransitionGroup>
</template>

<script setup>
import { ref, onMounted, onUnmounted } from 'vue'

const toasts = ref([])
let toastId = 0

// Toast 类型
const ToastType = {
  SUCCESS: 'success',
  ERROR: 'error',
  WARNING: 'warning',
  INFO: 'info'
}

// 显示 Toast
function showToast(message, type = ToastType.INFO, duration = 3000) {
  const id = ++toastId
  const toast = {
    id,
    message,
    type,
    duration
  }
  
  toasts.value.push(toast)
  
  // 自动移除
  if (duration > 0) {
    setTimeout(() => {
      removeToast(id)
    }, duration)
  }
  
  return id
}

// 移除 Toast
function removeToast(id) {
  const index = toasts.value.findIndex(t => t.id === id)
  if (index > -1) {
    toasts.value.splice(index, 1)
  }
}

// 获取图标
function getIcon(type) {
  const icons = {
    success: '✓',
    error: '✕',
    warning: '⚠',
    info: 'ℹ'
  }
  return icons[type] || icons.info
}

// 导出方法供全局使用
defineExpose({
  show: showToast,
  success: (message, duration) => showToast(message, ToastType.SUCCESS, duration),
  error: (message, duration) => showToast(message, ToastType.ERROR, duration),
  warning: (message, duration) => showToast(message, ToastType.WARNING, duration),
  info: (message, duration) => showToast(message, ToastType.INFO, duration)
})
</script>

<style scoped>
.toast-container {
  position: fixed;
  top: 20px;
  right: 20px;
  z-index: 10000;
  display: flex;
  flex-direction: column;
  gap: 8px;
  pointer-events: none;
}

.toast {
  display: flex;
  align-items: center;
  gap: 10px;
  padding: 12px 16px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  min-width: 300px;
  max-width: 500px;
  pointer-events: auto;
  animation: slideIn 0.3s ease-out;
}

.toast-success {
  border-left: 4px solid var(--success-color);
}

.toast-error {
  border-left: 4px solid var(--error-color);
}

.toast-warning {
  border-left: 4px solid var(--warning-color);
}

.toast-info {
  border-left: 4px solid var(--accent-color);
}

.toast-icon {
  font-size: 18px;
  font-weight: bold;
  flex-shrink: 0;
}

.toast-success .toast-icon {
  color: var(--success-color);
}

.toast-error .toast-icon {
  color: var(--error-color);
}

.toast-warning .toast-icon {
  color: var(--warning-color);
}

.toast-info .toast-icon {
  color: var(--accent-color);
}

.toast-message {
  flex: 1;
  font-size: 14px;
  color: var(--text-primary);
  line-height: 1.4;
}

.toast-close {
  width: 20px;
  height: 20px;
  padding: 0;
  font-size: 16px;
  line-height: 1;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 3px;
  flex-shrink: 0;
  transition: all 0.2s;
}

.toast-close:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

@keyframes slideIn {
  from {
    transform: translateX(100%);
    opacity: 0;
  }
  to {
    transform: translateX(0);
    opacity: 1;
  }
}

.toast-enter-active,
.toast-leave-active {
  transition: all 0.3s ease;
}

.toast-enter-from {
  transform: translateX(100%);
  opacity: 0;
}

.toast-leave-to {
  transform: translateX(100%);
  opacity: 0;
}
</style>

