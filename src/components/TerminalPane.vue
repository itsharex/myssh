<template>
  <div class="terminal-pane" :style="paneStyles">
    <div class="terminal-output" ref="terminalOutput">
      <div
        v-for="(line, index) in outputLines"
        :key="index"
        class="terminal-line"
      >
        <span class="line-prompt" v-if="line.type === 'input'">{{ line.prompt }}</span>
        <span :class="['line-content', line.type]">{{ line.content }}</span>
      </div>
      <div v-if="server.connected" class="terminal-line">
        <span class="line-prompt">{{ currentPrompt }}</span>
        <input
          v-model="currentInput"
          @keydown.enter="handleCommand"
          @keydown.up.prevent="handleHistoryUp"
          @keydown.down.prevent="handleHistoryDown"
          @keydown.tab.prevent="handleTabComplete"
          @keydown="handleKeyDown"
          class="terminal-input"
          ref="terminalInput"
          autofocus
          autocapitalize="none"
          autocorrect="off"
          spellcheck="false"
          autocomplete="off"
          inputmode="text"
        />
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, watch, nextTick, onMounted, computed } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'

const props = defineProps({
  server: Object,
  theme: String,
  fontSize: Number,
  fontFamily: String,
  isRecording: Boolean
})

const emit = defineEmits(['record'])

const terminalOutput = ref(null)
const terminalInput = ref(null)

const outputLines = ref([
  { type: 'output', content: '欢迎使用 MySSH 终端', prompt: '' }
])

const currentInput = ref('')
const currentPrompt = ref('$ ')
const commandHistory = ref([])
const historyIndex = ref(-1)
const promptInfo = ref({ username: '', hostname: '', isRoot: false, currentDir: '~' })
const currentWorkingDir = ref('~') // 跟踪当前工作目录

const paneStyles = computed(() => ({
  '--terminal-font-size': `${props.fontSize}px`,
  '--terminal-font-family': props.fontFamily
}))

watch(() => props.server.connected, async (connected) => {
  if (connected) {
    outputLines.value.push({
      type: 'output',
      content: `已连接到 ${props.server.host}:${props.server.port}`,
      prompt: ''
    })
    // 获取用户信息和主机名
    await fetchPromptInfo()
    scrollToBottom()
  } else {
    // 断开连接时重置提示符
    currentPrompt.value = '$ '
    promptInfo.value = { username: '', hostname: '', isRoot: false, currentDir: '~' }
  }
})

onMounted(async () => {
  if (props.server.connected) {
    // 如果已经连接，获取提示符信息
    await fetchPromptInfo()
    nextTick(() => {
      terminalInput.value?.focus()
    })
  }
})

function scrollToBottom() {
  nextTick(() => {
    if (terminalOutput.value) {
      terminalOutput.value.scrollTop = terminalOutput.value.scrollHeight
    }
  })
}

// 获取提示符信息（用户名、主机名、是否root、当前目录）
async function fetchPromptInfo() {
  try {
    const { executeSshCommand } = await import('@/api/ssh')
    
    // 获取用户名
    const userResult = await executeSshCommand({
      serverId: props.server.id,
      command: 'whoami'
    })
    const username = userResult.output.trim() || props.server.username || 'user'
    
    // 获取主机名
    const hostnameResult = await executeSshCommand({
      serverId: props.server.id,
      command: 'hostname'
    })
    const hostname = hostnameResult.output.trim() || props.server.host || 'localhost'
    
    // 检查是否是 root 用户
    const idResult = await executeSshCommand({
      serverId: props.server.id,
      command: 'id -u'
    })
    const userId = parseInt(idResult.output.trim()) || 1000
    const isRoot = userId === 0 || username.toLowerCase() === 'root'
    
    // 获取当前目录
    const pwdResult = await executeSshCommand({
      serverId: props.server.id,
      command: 'pwd'
    })
    const fullPath = pwdResult.output.trim() || '~'
    
    promptInfo.value = {
      username,
      hostname,
      isRoot,
      currentDir: '~' // 临时值，会在 updateCurrentDirFromPath 中更新
    }
    
    // 更新当前目录（会同时更新 promptInfo.currentDir 和 currentWorkingDir）
    updateCurrentDirFromPath(fullPath)
  } catch (error) {
    console.error('获取提示符信息失败:', error)
    // 使用默认值
    promptInfo.value = {
      username: props.server.username || 'user',
      hostname: props.server.host || 'localhost',
      isRoot: false,
      currentDir: '~'
    }
    updatePrompt()
  }
}

// 更新提示符
function updatePrompt() {
  const { username, hostname, isRoot, currentDir } = promptInfo.value
  const promptSymbol = isRoot ? '#' : '$'
  currentPrompt.value = `${username}@${hostname}:${currentDir}${promptSymbol} `
}

async function handleCommand() {
  const command = currentInput.value.trim()
  if (!command) {
    outputLines.value.push({
      type: 'input',
      prompt: currentPrompt.value,
      content: ''
    })
    currentInput.value = ''
    scrollToBottom()
    return
  }

  // 录制输入
  if (props.isRecording) {
    emit('record', { type: 'input', content: command })
  }

  // 添加到历史记录
  commandHistory.value.push(command)
  historyIndex.value = commandHistory.value.length

  // 显示输入的命令
  outputLines.value.push({
    type: 'input',
    prompt: currentPrompt.value,
    content: command
  })

  currentInput.value = ''

  // 调用 Tauri 执行命令
  await executeCommand(command)
  
  scrollToBottom()
}

async function executeCommand(command) {
  try {
    const { executeSshCommand } = await import('@/api/ssh')
    
    // 调用 Rust 后端执行命令（所有计算都在 Rust 端完成）
    const result = await executeSshCommand({
      serverId: props.server.id,
      command: command,
      currentDir: currentWorkingDir.value || '~'
    })
    
    // 处理交互式命令
    if (result.isInteractive && result.interactiveMessage) {
      // 显示交互式命令的提示信息（已按行分割）
      result.outputLines.forEach((line) => {
        outputLines.value.push({
          type: line.startsWith('警告:') ? 'error' : 'output',
          content: line,
          prompt: ''
        })
      })
      scrollToBottom()
      return
    }
    
    // 处理 cd 命令
    if (result.newDir) {
      // 更新当前工作目录
      updateCurrentDirFromPath(result.newDir)
      // 不显示 pwd 的输出，只显示空行
      outputLines.value.push({
        type: 'output',
        content: '',
        prompt: ''
      })
    } else {
      // 录制输出
      if (props.isRecording) {
        emit('record', { type: 'output', content: result.output })
      }

      // 显示命令输出（已按行分割，直接使用）
      result.outputLines.forEach((line) => {
        outputLines.value.push({
          type: result.exit_code === 0 ? 'output' : 'error',
          content: line,
          prompt: ''
        })
      })
    }
  } catch (error) {
    const serverName = props.server ? (props.server.name || `${props.server.host}:${props.server.port}`) : '未知服务器'
    let errorMessage = '执行命令失败'
    if (error instanceof Error) {
      errorMessage = error.message || errorMessage
    } else if (typeof error === 'string') {
      errorMessage = error
    } else if (error?.message) {
      errorMessage = error.message
    }
    // 错误信息也按行分割
    const errorLines = errorMessage.split('\n')
    errorLines.forEach((line) => {
      outputLines.value.push({
        type: 'error',
        content: line ? `${serverName}: ${line}` : '',
        prompt: ''
      })
    })
  }
}

function handleHistoryUp() {
  if (commandHistory.value.length === 0) return
  if (historyIndex.value > 0) {
    historyIndex.value--
    currentInput.value = commandHistory.value[historyIndex.value]
  }
}

function handleHistoryDown() {
  if (historyIndex.value < commandHistory.value.length - 1) {
    historyIndex.value++
    currentInput.value = commandHistory.value[historyIndex.value]
  } else {
    historyIndex.value = commandHistory.value.length
    currentInput.value = ''
  }
}

// 处理按键事件，防止自动大写
function handleKeyDown(event) {
  // 对于空格键后的字符，确保不会自动大写
  if (event.key === ' ' || (event.key.length === 1 && !event.ctrlKey && !event.metaKey && !event.altKey)) {
    // 允许正常输入，但阻止自动大写
    // 通过设置 inputmode="none" 和 autocapitalize="off" 已经处理了大部分情况
    // 这里作为额外保护
    return true
  }
}

// Tab 键自动补全
async function handleTabComplete() {
  const input = currentInput.value.trim()
  if (!input) return
  
  try {
    const { completeCommand } = await import('@/api/ssh')
    
    // 调用 Rust 后端进行补全（所有计算都在 Rust 端完成）
    const result = await completeCommand({
      serverId: props.server.id,
      input: input,
      currentDir: currentWorkingDir.value || '~'
    })
    
    // 处理补全结果
    if (result.completedInput) {
      // 有补全结果，直接更新输入
      currentInput.value = result.completedInput
    } else if (result.shouldShowMatches && result.matches && result.matches.length > 0) {
      // 需要显示匹配列表
      outputLines.value.push({
        type: 'output',
        content: result.matches.join('  '),
        prompt: ''
      })
      scrollToBottom()
    }
    // 如果没有匹配或补全结果，静默处理（不做任何操作）
  } catch (error) {
    // 补全失败，静默处理
    console.error('命令补全失败:', error)
  }
}

function clear() {
  outputLines.value = []
}

// 更新当前目录（从路径字符串）
function updateCurrentDirFromPath(fullPath) {
  let currentDir = fullPath.trim() || '~'
  const { username, isRoot } = promptInfo.value
  
  // 处理 root 用户的目录
  if (isRoot) {
    if (currentDir === '/root') {
      currentDir = '/root'
    } else if (currentDir.startsWith('/root/')) {
      // 保持 /root/xxx 格式，显示完整路径
      currentDir = currentDir
    } else {
      // 其他路径保持原样
      currentDir = currentDir
    }
  } else {
    // 普通用户：将 /home/username 简化为 ~
    if (currentDir === `/home/${username}`) {
      currentDir = '~'
    } else if (currentDir.startsWith(`/home/${username}/`)) {
      currentDir = currentDir.replace(`/home/${username}`, '~')
    } else {
      // 其他路径保持原样
      currentDir = currentDir
    }
  }
  
  promptInfo.value.currentDir = currentDir
  currentWorkingDir.value = fullPath.trim() // 保存完整路径用于后续命令
  updatePrompt()
}

// 更新当前目录（通过执行 pwd 命令）
async function updateCurrentDir() {
  try {
    const { executeSshCommand } = await import('@/api/ssh')
    const pwdResult = await executeSshCommand({
      serverId: props.server.id,
      command: 'pwd'
    })
    const fullPath = pwdResult.output.trim() || '~'
    updateCurrentDirFromPath(fullPath)
  } catch (error) {
    console.error('更新当前目录失败:', error)
  }
}

// 暴露方法供父组件调用
defineExpose({
  clear
})
</script>

<style scoped>
.terminal-pane {
  height: 100%;
  width: 100%;
  background: var(--terminal-bg, var(--bg-primary));
  overflow: hidden;
}

.terminal-output {
  height: 100%;
  overflow-y: auto;
  padding: 12px;
  font-family: var(--terminal-font-family, 'Consolas', 'Monaco', 'Courier New', monospace);
  font-size: var(--terminal-font-size, 13px);
  line-height: 1.6;
  color: var(--terminal-text, var(--text-primary));
}

.terminal-line {
  display: flex;
  margin-bottom: 2px;
  word-break: break-all;
}

.line-prompt {
  color: var(--terminal-prompt, var(--accent-color));
  margin-right: 4px;
  user-select: none;
}

.line-content {
  flex: 1;
  white-space: pre-wrap; /* 保留换行和空格 */
  word-break: break-all; /* 长单词换行 */
}

.line-content.input {
  color: var(--terminal-text, var(--text-primary));
}

.line-content.output {
  color: var(--terminal-text, var(--text-primary));
}

.line-content.error {
  color: var(--error-color);
}

.terminal-input {
  flex: 1;
  background: transparent;
  border: none;
  outline: none;
  color: var(--terminal-text, var(--text-primary));
  font-family: inherit;
  font-size: inherit;
  padding: 0;
  margin: 0;
  text-transform: none !important; /* 禁用文本转换 */
  -webkit-text-transform: none; /* Safari 兼容 */
  -moz-text-transform: none; /* Firefox 兼容 */
}
</style>

