<template>
  <div v-if="visible" class="dialog-overlay" @click.self="handleCancel">
    <div class="dialog confirm-dialog">
      <div class="dialog-header">
        <h3>{{ title }}</h3>
        <button @click="handleCancel" class="close-btn">×</button>
      </div>
      <div class="dialog-body">
        <p class="confirm-message">{{ message }}</p>
      </div>
      <div class="dialog-footer">
        <button @click="handleCancel" class="cancel-btn">取消</button>
        <button @click="handleConfirm" class="confirm-btn" :class="{ danger: type === 'danger' }">
          {{ confirmText }}
        </button>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, watch } from 'vue'

const props = defineProps({
  visible: {
    type: Boolean,
    default: false
  },
  title: {
    type: String,
    default: '确认操作'
  },
  message: {
    type: String,
    required: true
  },
  confirmText: {
    type: String,
    default: '确认'
  },
  type: {
    type: String,
    default: 'default', // 'default' | 'danger'
    validator: (value) => ['default', 'danger'].includes(value)
  }
})

const emit = defineEmits(['update:visible', 'confirm', 'cancel'])

function handleConfirm() {
  emit('confirm')
  emit('update:visible', false)
}

function handleCancel() {
  emit('cancel')
  emit('update:visible', false)
}

// 监听 ESC 键
watch(() => props.visible, (visible) => {
  if (visible) {
    const handleEsc = (e) => {
      if (e.key === 'Escape') {
        handleCancel()
      }
    }
    document.addEventListener('keydown', handleEsc)
    return () => {
      document.removeEventListener('keydown', handleEsc)
    }
  }
})
</script>

<style scoped>
/* 对话框遮罩层 */
.dialog-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 10000;
}

/* 对话框容器 */
.dialog {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  width: 480px;
  max-width: 90vw;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
  max-height: 90vh;
  overflow: hidden;
}

.confirm-dialog {
  width: 400px;
  max-width: 90vw;
}

/* 对话框头部 */
.dialog-header {
  padding: 16px;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
  flex-shrink: 0;
}

.dialog-header h3 {
  margin: 0;
  font-size: 16px;
  font-weight: 600;
  color: var(--text-primary);
}

.close-btn {
  width: 24px;
  height: 24px;
  padding: 0;
  font-size: 20px;
  line-height: 1;
  background: transparent;
  border: none;
  color: var(--text-secondary);
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 3px;
  transition: all 0.2s;
}

.close-btn:hover {
  background: var(--bg-hover);
  color: var(--text-primary);
}

/* 对话框内容 */
.dialog-body {
  padding: 16px;
  flex: 1;
  overflow-y: auto;
}

.confirm-message {
  font-size: 14px;
  color: var(--text-primary);
  line-height: 1.6;
  margin: 0;
  white-space: pre-line;
}

/* 对话框底部 */
.dialog-footer {
  padding: 16px;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: flex-end;
  gap: 8px;
  flex-shrink: 0;
}

.cancel-btn {
  padding: 6px 16px;
  background: transparent;
  border: 1px solid var(--border-color);
  border-radius: 3px;
  color: var(--text-primary);
  cursor: pointer;
  transition: all 0.2s;
}

.cancel-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent-color);
}

.confirm-btn {
  padding: 6px 16px;
  background: var(--accent-color);
  border: 1px solid var(--accent-color);
  border-radius: 3px;
  color: white;
  cursor: pointer;
  transition: all 0.2s;
}

.confirm-btn:hover {
  background: var(--accent-hover);
  border-color: var(--accent-hover);
}

.confirm-btn.danger {
  background: var(--error-color);
  border-color: var(--error-color);
}

.confirm-btn.danger:hover {
  background: #d65e4a;
  border-color: #d65e4a;
}
</style>

