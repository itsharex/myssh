<template>
  <div class="terminal-tab" :class="`theme-${currentTheme}`" :style="terminalStyles">
    <!-- 终端工具栏 -->
    <div class="terminal-toolbar">
      <div class="toolbar-left">
        <button @click="showSettings = !showSettings" class="toolbar-btn" title="设置">⚙️</button>
        <button @click="toggleSplit('horizontal')" class="toolbar-btn" title="水平分屏">⇄</button>
        <button @click="toggleSplit('vertical')" class="toolbar-btn" title="垂直分屏">⇅</button>
        <button @click="handleClear" class="toolbar-btn" title="清屏">清屏</button>
        <button @click="handleReconnect" class="toolbar-btn" title="重连">重连</button>
      </div>
      <div class="toolbar-right">
        <button 
          @click="toggleRecording" 
          class="toolbar-btn" 
          :class="{ recording: isRecording }"
          :title="isRecording ? '停止录制' : '开始录制'"
        >
          {{ isRecording ? '⏹️ 录制中' : '⏺️ 录制' }}
        </button>
        <button 
          v-if="recordings.length > 0"
          @click="showRecordings = !showRecordings" 
          class="toolbar-btn"
          title="回放"
        >
          ▶️ 回放
        </button>
      </div>
    </div>

    <!-- 设置面板 -->
    <div v-if="showSettings" class="settings-panel">
      <div class="settings-section">
        <label>主题配色</label>
        <select v-model="currentTheme" class="settings-select">
          <option v-for="theme in themes" :key="theme.id" :value="theme.id">{{ theme.name }}</option>
        </select>
      </div>
      <div class="settings-section">
        <label>字体大小: {{ fontSize }}px</label>
        <input 
          type="range" 
          v-model.number="fontSize" 
          min="10" 
          max="24" 
          class="settings-slider"
        />
      </div>
      <div class="settings-section">
        <label>字体族</label>
        <select v-model="fontFamily" class="settings-select">
          <option value="'Consolas', 'Monaco', 'Courier New', monospace">Consolas</option>
          <option value="'Courier New', monospace">Courier New</option>
          <option value="'Monaco', monospace">Monaco</option>
          <option value="'Fira Code', monospace">Fira Code</option>
        </select>
      </div>
      <button @click="showSettings = false" class="settings-close">关闭</button>
    </div>

    <!-- 录制列表 -->
    <div v-if="showRecordings" class="recordings-panel">
      <div class="recordings-header">
        <h4>录制回放</h4>
        <button @click="showRecordings = false" class="close-btn">×</button>
      </div>
      <div class="recordings-list">
        <div 
          v-for="recording in recordings" 
          :key="recording.id"
          class="recording-item"
          @click="playRecording(recording)"
        >
          <div class="recording-info">
            <div class="recording-name">{{ recording.name }}</div>
            <div class="recording-meta">{{ formatDate(recording.startTime) }} - {{ recording.duration }}秒</div>
          </div>
          <div class="recording-actions">
            <button @click.stop="deleteRecording(recording.id)" class="delete-btn" title="删除">×</button>
          </div>
        </div>
        <div v-if="recordings.length === 0" class="empty-recordings">暂无录制</div>
      </div>
    </div>

    <!-- 终端容器（支持分屏） -->
    <div class="terminal-container" ref="terminalContainer">
      <div 
        v-if="!isSplit"
        class="terminal-pane"
        :class="{ 'recording': isRecording }"
      >
        <TerminalPane 
          :server="server"
          :theme="currentTheme"
          :font-size="fontSize"
          :font-family="fontFamily"
          :is-recording="isRecording"
          @record="handleRecord"
          ref="mainPane"
        />
      </div>
      <div v-else class="split-container" :class="`split-${splitDirection}`">
        <div class="terminal-pane" :class="{ 'recording': isRecording }">
          <TerminalPane 
            :server="server"
            :theme="currentTheme"
            :font-size="fontSize"
            :font-family="fontFamily"
            :is-recording="isRecording"
            @record="handleRecord"
            ref="pane1"
          />
        </div>
        <div class="split-divider" @dblclick="closeSplit"></div>
        <div class="terminal-pane" :class="{ 'recording': isRecording }">
          <TerminalPane 
            :server="server"
            :theme="currentTheme"
            :font-size="fontSize"
            :font-family="fontFamily"
            :is-recording="isRecording"
            @record="handleRecord"
            ref="pane2"
          />
        </div>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import TerminalPane from './TerminalPane.vue'
import { info, error as showError, success } from '@/utils/toast'

const props = defineProps({
  tab: Object,
  server: Object
})

// 主题配置
const themes = [
  { id: 'dark', name: '深色主题', colors: { bg: '#1e1e1e', text: '#cccccc', prompt: '#007acc' } },
  { id: 'light', name: '浅色主题', colors: { bg: '#ffffff', text: '#000000', prompt: '#007acc' } },
  { id: 'solarized-dark', name: 'Solarized Dark', colors: { bg: '#002b36', text: '#839496', prompt: '#268bd2' } },
  { id: 'solarized-light', name: 'Solarized Light', colors: { bg: '#fdf6e3', text: '#657b83', prompt: '#268bd2' } },
  { id: 'monokai', name: 'Monokai', colors: { bg: '#272822', text: '#f8f8f2', prompt: '#66d9ef' } },
  { id: 'dracula', name: 'Dracula', colors: { bg: '#282a36', text: '#f8f8f2', prompt: '#bd93f9' } }
]

const currentTheme = ref('dark')
const fontSize = ref(13)
const fontFamily = ref("'Consolas', 'Monaco', 'Courier New', monospace")
const showSettings = ref(false)

// 分屏
const isSplit = ref(false)
const splitDirection = ref('horizontal') // 'horizontal' | 'vertical'

// 录制/回放
const isRecording = ref(false)
const showRecordings = ref(false)
const recordings = ref([])
const currentRecording = ref(null)

// 终端样式
const terminalStyles = computed(() => {
  const theme = themes.find(t => t.id === currentTheme.value) || themes[0]
  return {
    '--terminal-bg': theme.colors.bg,
    '--terminal-text': theme.colors.text,
    '--terminal-prompt': theme.colors.prompt,
    '--terminal-font-size': `${fontSize.value}px`,
    '--terminal-font-family': fontFamily.value
  }
})

// 加载保存的设置
function loadSettings() {
  const saved = localStorage.getItem(`terminal-settings-${props.server?.id}`)
  if (saved) {
    try {
      const settings = JSON.parse(saved)
      currentTheme.value = settings.theme || 'dark'
      fontSize.value = settings.fontSize || 13
      fontFamily.value = settings.fontFamily || "'Consolas', 'Monaco', 'Courier New', monospace"
    } catch (e) {
      console.error('Failed to load terminal settings:', e)
    }
  }
  
  // 加载录制列表
  const savedRecordings = localStorage.getItem(`terminal-recordings-${props.server?.id}`)
  if (savedRecordings) {
    try {
      recordings.value = JSON.parse(savedRecordings)
    } catch (e) {
      console.error('Failed to load recordings:', e)
    }
  }
}

// 保存设置
function saveSettings() {
  const settings = {
    theme: currentTheme.value,
    fontSize: fontSize.value,
    fontFamily: fontFamily.value
  }
  localStorage.setItem(`terminal-settings-${props.server?.id}`, JSON.stringify(settings))
}

// 监听设置变化
watch([currentTheme, fontSize, fontFamily], () => {
  saveSettings()
})

// 切换分屏
function toggleSplit(direction) {
  if (isSplit.value && splitDirection.value === direction) {
    isSplit.value = false
  } else {
    isSplit.value = true
    splitDirection.value = direction
  }
}

function closeSplit() {
  isSplit.value = false
}

// 录制功能
function toggleRecording() {
  if (isRecording.value) {
    stopRecording()
  } else {
    startRecording()
  }
}

function startRecording() {
  isRecording.value = true
  currentRecording.value = {
    id: Date.now().toString(),
    name: `录制_${new Date().toLocaleString('zh-CN')}`,
    startTime: Date.now(),
    events: []
  }
}

function stopRecording() {
  if (currentRecording.value) {
    currentRecording.value.duration = Math.floor((Date.now() - currentRecording.value.startTime) / 1000)
    recordings.value.push(currentRecording.value)
    saveRecordings()
    currentRecording.value = null
  }
  isRecording.value = false
}

function handleRecord(event) {
  if (isRecording.value && currentRecording.value) {
    currentRecording.value.events.push({
      type: event.type, // 'input' | 'output'
      content: event.content,
      timestamp: Date.now() - currentRecording.value.startTime
    })
  }
}

function saveRecordings() {
  localStorage.setItem(`terminal-recordings-${props.server?.id}`, JSON.stringify(recordings.value))
}

function deleteRecording(id) {
  recordings.value = recordings.value.filter(r => r.id !== id)
  saveRecordings()
}

async function playRecording(recording) {
  showRecordings.value = false
  // TODO: 调用 Tauri 回放录制
  // await invoke('play_terminal_recording', { 
  //   serverId: props.server.id, 
  //   recording: recording 
  // })
  info(`回放录制: ${recording.name}\n（将调用 Tauri 实现）`)
}

function formatDate(timestamp) {
  return new Date(timestamp).toLocaleString('zh-CN')
}

function handleClear() {
  // 由 TerminalPane 处理
}

async function handleReconnect() {
  const serverName = props.server ? (props.server.name || `${props.server.host}:${props.server.port}`) : '未知服务器'
  
  try {
    // 调用 Tauri API 重连终端
    const { reconnectTerminal } = await import('@/api/ssh')
    await reconnectTerminal(props.server.id)
    success(`${serverName} 重连成功`)
  } catch (err) {
    console.error('重连失败:', err)
    let errorMessage = '重连失败'
    if (err instanceof Error) {
      errorMessage = err.message || errorMessage
    } else if (typeof err === 'string') {
      errorMessage = err
    } else if (err?.message) {
      errorMessage = err.message
    }
    showError(`${serverName}: ${errorMessage}`)
  }
}

// 暴露方法供父组件调用
defineExpose({
  clearTerminal: handleClear,
  reconnect: handleReconnect
})

onMounted(() => {
  loadSettings()
})

watch(() => props.server?.id, () => {
  loadSettings()
})
</script>

<style scoped>
.terminal-tab {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--terminal-bg, var(--bg-primary));
  overflow: hidden;
}

.terminal-toolbar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 6px 12px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.toolbar-left,
.toolbar-right {
  display: flex;
  gap: 6px;
}

.toolbar-btn {
  padding: 4px 8px;
  font-size: 12px;
  background: transparent;
  border: 1px solid var(--border-color);
  border-radius: 3px;
  color: var(--text-primary);
  cursor: pointer;
  transition: all 0.2s;
}

.toolbar-btn:hover {
  background: var(--bg-hover);
  border-color: var(--accent-color);
}

.toolbar-btn.recording {
  background: var(--error-color);
  color: white;
  border-color: var(--error-color);
}

.settings-panel {
  position: absolute;
  top: 40px;
  right: 12px;
  z-index: 100;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  padding: 16px;
  min-width: 250px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
}

.settings-section {
  margin-bottom: 16px;
}

.settings-section label {
  display: block;
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 6px;
}

.settings-select {
  width: 100%;
  padding: 6px 8px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 3px;
  color: var(--text-primary);
  font-size: 12px;
}

.settings-slider {
  width: 100%;
}

.settings-close {
  width: 100%;
  padding: 6px;
  background: var(--accent-color);
  color: white;
  border: none;
  border-radius: 3px;
  cursor: pointer;
}

.recordings-panel {
  position: absolute;
  top: 40px;
  right: 12px;
  z-index: 100;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  width: 350px;
  max-height: 400px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  display: flex;
  flex-direction: column;
}

.recordings-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  border-bottom: 1px solid var(--border-color);
}

.recordings-header h4 {
  margin: 0;
  font-size: 14px;
}

.close-btn {
  width: 24px;
  height: 24px;
  padding: 0;
  background: transparent;
  border: none;
  color: var(--text-primary);
  cursor: pointer;
  font-size: 18px;
  line-height: 1;
}

.recordings-list {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
}

.recording-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px;
  margin-bottom: 4px;
  background: var(--bg-primary);
  border-radius: 4px;
  cursor: pointer;
  transition: background 0.2s;
}

.recording-item:hover {
  background: var(--bg-hover);
}

.recording-info {
  flex: 1;
}

.recording-name {
  font-size: 13px;
  color: var(--text-primary);
  margin-bottom: 2px;
}

.recording-meta {
  font-size: 11px;
  color: var(--text-secondary);
}

.delete-btn {
  width: 20px;
  height: 20px;
  padding: 0;
  background: transparent;
  border: none;
  color: var(--error-color);
  cursor: pointer;
  font-size: 16px;
  line-height: 1;
}

.empty-recordings {
  text-align: center;
  padding: 20px;
  color: var(--text-secondary);
  font-size: 12px;
}

.terminal-container {
  flex: 1;
  overflow: hidden;
  position: relative;
  min-height: 0;
}

.terminal-pane {
  height: 100%;
  width: 100%;
}

.terminal-pane.recording {
  border: 2px solid var(--error-color);
  box-sizing: border-box;
}

.split-container {
  display: flex;
  height: 100%;
  width: 100%;
}

.split-container.split-horizontal {
  flex-direction: row;
}

.split-container.split-vertical {
  flex-direction: column;
}

.split-container .terminal-pane {
  flex: 1;
  min-width: 0;
  min-height: 0;
}

.split-divider {
  width: 4px;
  height: 100%;
  background: var(--border-color);
  cursor: col-resize;
  flex-shrink: 0;
}

.split-container.split-vertical .split-divider {
  width: 100%;
  height: 4px;
  cursor: row-resize;
}

.split-divider:hover {
  background: var(--accent-color);
}
</style>
