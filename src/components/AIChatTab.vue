<template>
  <div class="ai-chat-tab">
    <div class="chat-container" v-if="server.connected">
      <!-- 对话消息列表 -->
      <div class="chat-messages" ref="messagesContainer">
        <div
          v-for="(message, index) in messages"
          :key="index"
          :class="['message', message.role]"
        >
          <div class="message-avatar">
            <span v-if="message.role === 'user'">👤</span>
            <span v-else>🤖</span>
          </div>
          <div class="message-content">
            <div class="message-header">
              <span class="message-role">{{ message.role === 'user' ? '我' : 'AI助手' }}</span>
              <span class="message-time">{{ formatTime(message.timestamp) }}</span>
            </div>
            <div class="message-text" v-html="formatMessage(message.content)"></div>
          </div>
        </div>
        <div v-if="isLoading" class="message assistant">
          <div class="message-avatar">🤖</div>
          <div class="message-content">
            <div class="message-header">
              <span class="message-role">AI助手</span>
            </div>
            <div class="message-text">
              <div class="typing-indicator">
                <span></span>
                <span></span>
                <span></span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- 输入区域 -->
      <div class="chat-input-area">
        <div class="input-toolbar">
          <button
            @click="clearChat"
            class="toolbar-btn"
            title="清空对话"
            :disabled="messages.length === 0"
          >
            🗑️
          </button>
          <button
            @click="showQuickActions = !showQuickActions"
            class="toolbar-btn"
            title="快捷操作"
          >
            ⚡
          </button>
        </div>
        
        <!-- 快捷操作面板 -->
        <div v-if="showQuickActions" class="quick-actions">
          <div class="quick-action-item" @click="insertQuickAction('查看系统状态')">
            📊 查看系统状态
          </div>
          <div class="quick-action-item" @click="insertQuickAction('查看磁盘使用情况')">
            💾 查看磁盘使用情况
          </div>
          <div class="quick-action-item" @click="insertQuickAction('查看网络连接')">
            🌐 查看网络连接
          </div>
          <div class="quick-action-item" @click="insertQuickAction('查看进程列表')">
            🔍 查看进程列表
          </div>
          <div class="quick-action-item" @click="insertQuickAction('查看日志文件')">
            📝 查看日志文件
          </div>
        </div>

        <div class="input-wrapper">
          <textarea
            v-model="inputText"
            @keydown.enter.exact.prevent="handleSend"
            @keydown.shift.enter.exact="handleNewLine"
            class="chat-input"
            placeholder="输入您的问题，AI将帮助您进行运维操作..."
            rows="3"
            ref="inputRef"
          ></textarea>
          <button
            @click="handleSend"
            class="send-btn"
            :disabled="!inputText.trim() || isLoading"
            title="发送 (Enter)"
          >
            {{ isLoading ? '⏳' : '📤' }}
          </button>
        </div>
      </div>
    </div>

    <div v-else class="chat-disconnected">
      <div class="disconnected-content">
        <div class="disconnected-icon">💬</div>
        <h3>未连接服务器</h3>
        <p>请先连接服务器以使用AI助手</p>
      </div>
    </div>

    <!-- 清空确认对话框 -->
    <ConfirmDialog
      v-model:visible="showClearConfirm"
      title="清空对话"
      message="确定要清空所有对话记录吗？"
      confirm-text="清空"
      @confirm="confirmClear"
    />
  </div>
</template>

<script setup>
import { ref, computed, watch, nextTick } from 'vue'
import ConfirmDialog from './ConfirmDialog.vue'
import { success } from '@/utils/toast'

const props = defineProps({
  server: {
    type: Object,
    required: true
  },
  tab: {
    type: Object,
    default: null
  }
})

const showClearConfirm = ref(false)

const messages = ref([])
const inputText = ref('')
const isLoading = ref(false)
const showQuickActions = ref(false)
const messagesContainer = ref(null)
const inputRef = ref(null)

// 监听服务器变化，清空对话
watch(() => props.server?.id, () => {
  messages.value = []
  inputText.value = ''
  isLoading.value = false
})

// 滚动到底部
function scrollToBottom() {
  nextTick(() => {
    if (messagesContainer.value) {
      messagesContainer.value.scrollTop = messagesContainer.value.scrollHeight
    }
  })
}

// 格式化时间
function formatTime(timestamp) {
  const date = new Date(timestamp)
  const hours = date.getHours().toString().padStart(2, '0')
  const minutes = date.getMinutes().toString().padStart(2, '0')
  return `${hours}:${minutes}`
}

// 格式化消息内容（支持代码高亮）
function formatMessage(content) {
  // 简单的代码块处理
  let formatted = content
    .replace(/`([^`]+)`/g, '<code>$1</code>')
    .replace(/\n/g, '<br>')
  return formatted
}

// 发送消息
async function handleSend() {
  if (!inputText.value.trim() || isLoading.value) return

  const userMessage = {
    role: 'user',
    content: inputText.value.trim(),
    timestamp: Date.now()
  }

  messages.value.push(userMessage)
  const question = inputText.value.trim()
  inputText.value = ''
  showQuickActions.value = false
  scrollToBottom()

  // 开始加载
  isLoading.value = true
  scrollToBottom()

  try {
    // TODO: 调用 Tauri API 与 AI 服务交互
    // const response = await invoke('chat_with_ai', {
    //   serverId: props.server.id,
    //   question: question,
    //   history: messages.value.slice(0, -1)
    // })

    // 模拟AI响应（临时）
    await new Promise(resolve => setTimeout(resolve, 1000))
    const aiResponse = generateMockResponse(question)

    messages.value.push({
      role: 'assistant',
      content: aiResponse,
      timestamp: Date.now()
    })
  } catch (error) {
    console.error('AI对话错误:', error)
    messages.value.push({
      role: 'assistant',
      content: '抱歉，AI服务暂时不可用，请稍后重试。',
      timestamp: Date.now()
    })
  } finally {
    isLoading.value = false
    scrollToBottom()
  }
}

// 生成模拟响应（临时，后续替换为真实AI调用）
function generateMockResponse(question) {
  const lowerQuestion = question.toLowerCase()
  
  if (lowerQuestion.includes('系统') || lowerQuestion.includes('状态')) {
    return `根据当前系统监控数据：
- CPU使用率：正常
- 内存使用率：正常
- 磁盘使用率：正常

系统运行状态良好。如需查看详细数据，请查看系统监控面板。`
  } else if (lowerQuestion.includes('磁盘')) {
    return `磁盘使用情况：
- 总容量：100GB
- 已使用：50GB
- 可用空间：50GB
- 使用率：50%

磁盘空间充足，无需清理。`
  } else if (lowerQuestion.includes('网络')) {
    return `网络连接状态：
- 连接数：正常
- 带宽使用：正常
- 延迟：正常

网络连接稳定。`
  } else if (lowerQuestion.includes('进程')) {
    return `主要进程：
1. systemd - 系统服务管理
2. sshd - SSH服务
3. nginx - Web服务器

所有关键进程运行正常。`
  } else {
    return `我理解您的问题："${question}"

这是一个很好的运维问题。建议您：
1. 查看系统监控面板获取实时数据
2. 使用终端执行相关命令
3. 查看日志文件获取详细信息

如需具体操作步骤，请告诉我更多细节。`
  }
}

// 换行处理
function handleNewLine() {
  inputText.value += '\n'
}

// 清空对话
function clearChat() {
  showClearConfirm.value = true
}

function confirmClear() {
  messages.value = []
  success('对话已清空')
}

// 插入快捷操作
function insertQuickAction(action) {
  inputText.value = action
  showQuickActions.value = false
  nextTick(() => {
    inputRef.value?.focus()
  })
}
</script>

<style scoped>
.ai-chat-tab {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
}

.chat-container {
  display: flex;
  flex-direction: column;
  height: 100%;
  overflow: hidden;
}

.chat-messages {
  flex: 1;
  overflow-y: auto;
  padding: 16px;
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.message {
  display: flex;
  gap: 12px;
  animation: fadeIn 0.3s;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.message.user {
  flex-direction: row-reverse;
}

.message-avatar {
  width: 32px;
  height: 32px;
  border-radius: 50%;
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 18px;
  flex-shrink: 0;
  background: var(--bg-tertiary);
}

.message.user .message-avatar {
  background: var(--accent-color);
}

.message-content {
  flex: 1;
  max-width: 80%;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.message.user .message-content {
  align-items: flex-end;
}

.message-header {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 11px;
  color: var(--text-secondary);
}

.message.user .message-header {
  flex-direction: row-reverse;
}

.message-role {
  font-weight: 600;
  color: var(--text-primary);
}

.message-time {
  font-size: 10px;
}

.message-text {
  padding: 10px 14px;
  border-radius: 8px;
  background: var(--bg-secondary);
  color: var(--text-primary);
  line-height: 1.6;
  word-wrap: break-word;
}

.message.user .message-text {
  background: var(--accent-color);
  color: white;
}

.message-text code {
  background: var(--bg-tertiary);
  padding: 2px 6px;
  border-radius: 3px;
  font-family: 'Courier New', monospace;
  font-size: 0.9em;
}

.message.user .message-text code {
  background: rgba(255, 255, 255, 0.2);
  color: white;
}

.typing-indicator {
  display: flex;
  gap: 4px;
  padding: 10px 14px;
}

.typing-indicator span {
  width: 8px;
  height: 8px;
  border-radius: 50%;
  background: var(--text-secondary);
  animation: typing 1.4s infinite;
}

.typing-indicator span:nth-child(2) {
  animation-delay: 0.2s;
}

.typing-indicator span:nth-child(3) {
  animation-delay: 0.4s;
}

@keyframes typing {
  0%, 60%, 100% {
    transform: translateY(0);
    opacity: 0.5;
  }
  30% {
    transform: translateY(-10px);
    opacity: 1;
  }
}

.chat-input-area {
  border-top: 1px solid var(--border-color);
  background: var(--bg-secondary);
  padding: 12px;
}

.input-toolbar {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
}

.toolbar-btn {
  padding: 4px 8px;
  font-size: 12px;
  background: transparent;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
}

.toolbar-btn:hover:not(:disabled) {
  background: var(--bg-hover);
  border-color: var(--accent-color);
}

.toolbar-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.quick-actions {
  display: flex;
  flex-wrap: wrap;
  gap: 8px;
  margin-bottom: 8px;
  padding: 8px;
  background: var(--bg-tertiary);
  border-radius: 6px;
}

.quick-action-item {
  padding: 6px 12px;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  font-size: 12px;
  cursor: pointer;
  transition: all 0.2s;
}

.quick-action-item:hover {
  background: var(--bg-hover);
  border-color: var(--accent-color);
  transform: translateY(-1px);
}

.input-wrapper {
  display: flex;
  gap: 8px;
  align-items: flex-end;
}

.chat-input {
  flex: 1;
  padding: 10px 12px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  color: var(--text-primary);
  font-size: 13px;
  font-family: inherit;
  resize: none;
  outline: none;
  transition: border-color 0.2s;
}

.chat-input:focus {
  border-color: var(--accent-color);
}

.chat-input::placeholder {
  color: var(--text-disabled);
}

.send-btn {
  width: 40px;
  height: 40px;
  padding: 0;
  font-size: 18px;
  background: var(--accent-color);
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  transition: all 0.2s;
  flex-shrink: 0;
}

.send-btn:hover:not(:disabled) {
  background: var(--accent-hover);
  transform: scale(1.05);
}

.send-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.chat-disconnected {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-secondary);
}

.disconnected-content {
  text-align: center;
}

.disconnected-icon {
  font-size: 64px;
  margin-bottom: 16px;
}

.disconnected-content h3 {
  font-size: 20px;
  margin-bottom: 8px;
  color: var(--text-primary);
}

.disconnected-content p {
  font-size: 14px;
}
</style>

