<template>
  <div class="log-viewer">
    <div class="log-header">
      <div class="log-tabs">
        <div
          v-for="logFile in openLogFiles"
          :key="logFile.id"
          :class="['log-tab', { active: activeLogId === logFile.id }]"
          @click="selectLog(logFile.id)"
        >
          <span class="log-tab-name">{{ logFile.name }}</span>
          <button @click.stop="closeLog(logFile.id)" class="log-tab-close">×</button>
        </div>
        <button @click="openLogFile" class="add-log-btn" title="打开日志文件">+</button>
      </div>
      <div class="log-toolbar">
        <input
          v-model="searchText"
          @input="handleSearch"
          class="search-input"
          placeholder="搜索日志..."
        />
        <button @click="toggleAutoScroll" class="toolbar-btn" :class="{ active: autoScroll }" title="自动滚动">
          {{ autoScroll ? '⏸️' : '▶️' }}
        </button>
        <button @click="clearLog" class="toolbar-btn" title="清空">清空</button>
        <button @click="exportLog" class="toolbar-btn" title="导出">导出</button>
      </div>
    </div>

    <div class="log-content" ref="logContent">
      <div
        v-for="(line, index) in filteredLines"
        :key="index"
        :class="['log-line', getLineClass(line)]"
        :data-index="index"
      >
        <span class="log-time">{{ formatTime(line.timestamp) }}</span>
        <span class="log-level" :class="line.level">{{ line.level }}</span>
        <span class="log-message">{{ highlightSearch(line.message) }}</span>
      </div>
      <div v-if="loading" class="log-loading">
        <div class="loading-spinner"></div>
        <span>加载中...</span>
      </div>
      <div v-if="filteredLines.length === 0 && !loading" class="log-empty">
        暂无日志
      </div>
    </div>

    <!-- 清空确认对话框 -->
    <ConfirmDialog
      v-model:visible="showClearConfirm"
      title="清空日志"
      message="确定要清空当前日志吗？"
      confirm-text="清空"
      @confirm="confirmClear"
    />
  </div>
</template>

<script setup>
import { ref, computed, watch, nextTick, onMounted, onUnmounted } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { open, save as saveFile } from '@tauri-apps/api/dialog'
import ConfirmDialog from './ConfirmDialog.vue'
import { error, success, info } from '@/utils/toast'

const props = defineProps({
  server: Object
})

const showClearConfirm = ref(false)

const openLogFiles = ref([])
const activeLogId = ref(null)
const logLines = ref([])
const searchText = ref('')
const autoScroll = ref(true)
const loading = ref(false)
const refreshInterval = ref(null)

// 当前活动的日志文件
const activeLog = computed(() => {
  return openLogFiles.value.find(f => f.id === activeLogId.value)
})

// 过滤后的日志行
const filteredLines = computed(() => {
  if (!searchText.value) {
    return logLines.value
  }
  const search = searchText.value.toLowerCase()
  return logLines.value.filter(line => 
    line.message.toLowerCase().includes(search) ||
    line.level.toLowerCase().includes(search)
  )
})

// 选择日志文件
function selectLog(logId) {
  activeLogId.value = logId
  loadLogContent()
}

// 打开日志文件
async function openLogFile() {
  try {
    // TODO: 调用 Tauri 选择远程日志文件
    // const filePath = await invoke('select_remote_log_file', { serverId: props.server.id })
    
    // 模拟打开日志文件
    const fileName = prompt('请输入日志文件路径:')
    if (!fileName) return
    
    const logFile = {
      id: Date.now().toString(),
      name: fileName.split('/').pop(),
      path: fileName,
      serverId: props.server.id
    }
    
    openLogFiles.value.push(logFile)
    activeLogId.value = logFile.id
    loadLogContent()
  } catch (error) {
    console.error('打开日志文件失败:', error)
  }
}

// 关闭日志文件
function closeLog(logId) {
  const index = openLogFiles.value.findIndex(f => f.id === logId)
  if (index > -1) {
    openLogFiles.value.splice(index, 1)
    if (activeLogId.value === logId) {
      activeLogId.value = openLogFiles.value.length > 0 ? openLogFiles.value[0].id : null
    }
  }
}

// 加载日志内容
async function loadLogContent() {
  if (!activeLog.value || !props.server.connected) {
    return
  }
  
  loading.value = true
  
  try {
    // TODO: 调用 Tauri 读取日志文件
    // const result = await invoke('read_log_file', {
    //   serverId: props.server.id,
    //   filePath: activeLog.value.path,
    //   lines: 1000 // 读取最近1000行
    // })
    
    // 模拟日志数据
    const mockLines = Array.from({ length: 100 }, (_, i) => ({
      timestamp: Date.now() - (100 - i) * 1000,
      level: ['INFO', 'WARN', 'ERROR', 'DEBUG'][Math.floor(Math.random() * 4)],
      message: `这是第 ${i + 1} 条日志消息，包含一些示例内容`
    }))
    
    logLines.value = mockLines
    scrollToBottom()
  } catch (error) {
    console.error('加载日志失败:', error)
  } finally {
    loading.value = false
  }
}

// 实时监听日志
function startLogMonitoring() {
  if (!activeLog.value || !props.server.connected) {
    return
  }
  
  stopLogMonitoring()
  
  // TODO: 调用 Tauri 开始实时监听日志
  // await invoke('start_log_monitoring', {
  //   serverId: props.server.id,
  //   filePath: activeLog.value.path
  // })
  
  // 模拟实时日志
  refreshInterval.value = setInterval(() => {
    if (autoScroll.value) {
      const newLine = {
        timestamp: Date.now(),
        level: ['INFO', 'WARN', 'ERROR'][Math.floor(Math.random() * 3)],
        message: `实时日志消息 ${new Date().toLocaleTimeString()}`
      }
      logLines.value.push(newLine)
      scrollToBottom()
    }
  }, 2000)
}

// 停止监听
function stopLogMonitoring() {
  if (refreshInterval.value) {
    clearInterval(refreshInterval.value)
    refreshInterval.value = null
  }
}

// 搜索日志
function handleSearch() {
  // 搜索逻辑已在 computed 中处理
}

// 高亮搜索文本
function highlightSearch(text) {
  if (!searchText.value) {
    return text
  }
  const regex = new RegExp(`(${searchText.value})`, 'gi')
  return text.replace(regex, '<mark>$1</mark>')
}

// 获取日志行样式类
function getLineClass(line) {
  return {
    'log-info': line.level === 'INFO',
    'log-warn': line.level === 'WARN',
    'log-error': line.level === 'ERROR',
    'log-debug': line.level === 'DEBUG'
  }
}

// 切换自动滚动
function toggleAutoScroll() {
  autoScroll.value = !autoScroll.value
  if (autoScroll.value) {
    scrollToBottom()
  }
}

// 滚动到底部
function scrollToBottom() {
  nextTick(() => {
    const content = document.querySelector('.log-content')
    if (content) {
      content.scrollTop = content.scrollHeight
    }
  })
}

// 清空日志
function clearLog() {
  showClearConfirm.value = true
}

function confirmClear() {
  logLines.value = []
  success('日志已清空')
}

// 导出日志
async function exportLog() {
  if (!activeLog.value) {
    error('请先选择日志文件')
    return
  }
  
  try {
    const savePath = await saveFile({
      title: '导出日志',
      defaultPath: `${activeLog.value.name}_${new Date().toISOString().split('T')[0]}.log`
    })
    
    if (!savePath) return
    
    // TODO: 调用 Tauri 导出日志
    // await invoke('export_log_file', {
    //   serverId: props.server.id,
    //   remotePath: activeLog.value.path,
    //   localPath: savePath
    // })
    
    success(`日志已导出到: ${savePath}`)
  } catch (error) {
    console.error('导出日志失败:', error)
  }
}

function formatTime(timestamp) {
  return new Date(timestamp).toLocaleTimeString('zh-CN')
}

watch(() => activeLogId.value, () => {
  loadLogContent()
  startLogMonitoring()
})

watch(() => props.server.connected, (connected) => {
  if (connected && activeLog.value) {
    startLogMonitoring()
  } else {
    stopLogMonitoring()
  }
})

onMounted(() => {
  if (props.server.connected && activeLog.value) {
    startLogMonitoring()
  }
})

onUnmounted(() => {
  stopLogMonitoring()
})
</script>

<style scoped>
.log-viewer {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
  overflow: hidden;
}

.log-header {
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.log-tabs {
  display: flex;
  align-items: center;
  padding: 4px 8px;
  overflow-x: auto;
  border-bottom: 1px solid var(--border-color);
}

.log-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 4px 12px;
  background: var(--bg-tertiary);
  border-right: 1px solid var(--border-color);
  cursor: pointer;
  white-space: nowrap;
  transition: background 0.2s;
}

.log-tab:hover {
  background: var(--bg-hover);
}

.log-tab.active {
  background: var(--bg-primary);
  border-bottom: 2px solid var(--accent-color);
}

.log-tab-name {
  font-size: 12px;
}

.log-tab-close {
  width: 16px;
  height: 16px;
  padding: 0;
  background: transparent;
  border: none;
  color: var(--text-primary);
  cursor: pointer;
  font-size: 14px;
  line-height: 1;
  opacity: 0.6;
}

.log-tab-close:hover {
  opacity: 1;
  color: var(--error-color);
}

.add-log-btn {
  padding: 4px 8px;
  background: transparent;
  border: 1px solid var(--border-color);
  border-radius: 3px;
  color: var(--text-primary);
  cursor: pointer;
  font-size: 14px;
}

.log-toolbar {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 8px;
}

.search-input {
  flex: 1;
  padding: 4px 8px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 3px;
  color: var(--text-primary);
  font-size: 12px;
}

.toolbar-btn {
  padding: 4px 8px;
  font-size: 12px;
  background: transparent;
  border: 1px solid var(--border-color);
  border-radius: 3px;
  color: var(--text-primary);
  cursor: pointer;
}

.toolbar-btn.active {
  background: var(--accent-color);
  color: white;
}

.log-content {
  flex: 1;
  overflow-y: auto;
  padding: 8px;
  font-family: 'Consolas', 'Monaco', 'Courier New', monospace;
  font-size: 12px;
  line-height: 1.6;
}

.log-line {
  display: flex;
  gap: 8px;
  margin-bottom: 2px;
  padding: 2px 4px;
  border-radius: 2px;
  transition: background 0.2s;
}

.log-line:hover {
  background: var(--bg-hover);
}

.log-time {
  color: var(--text-secondary);
  font-size: 11px;
  min-width: 80px;
}

.log-level {
  min-width: 50px;
  font-weight: 600;
  font-size: 11px;
}

.log-level.INFO {
  color: var(--success-color);
}

.log-level.WARN {
  color: var(--warning-color);
}

.log-level.ERROR {
  color: var(--error-color);
}

.log-level.DEBUG {
  color: var(--text-secondary);
}

.log-message {
  flex: 1;
  color: var(--text-primary);
  word-break: break-all;
}

.log-message :deep(mark) {
  background: var(--warning-color);
  color: var(--bg-primary);
  padding: 0 2px;
}

.log-loading {
  text-align: center;
  padding: 40px;
  color: var(--text-secondary);
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
}

.log-loading .loading-spinner {
  width: 16px;
  height: 16px;
  border: 2px solid var(--bg-tertiary);
  border-top-color: var(--accent-color);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.log-empty {
  text-align: center;
  padding: 40px;
  color: var(--text-secondary);
}
</style>

