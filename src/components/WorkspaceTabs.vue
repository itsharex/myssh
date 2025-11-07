<template>
  <div class="workspace-tabs">
    <!-- 服务器标签页 -->
    <div v-if="connectedServers.length > 0" class="server-tabs-container">
      <div class="server-tabs-header">
        <div
          v-for="server in connectedServers"
          :key="server.id"
          :class="['server-tab', { active: activeServerId === server.id }]"
          @click="selectServer(server.id)"
        >
          <span class="server-tab-icon">🖥️</span>
          <span class="server-tab-name">{{ server.name }}</span>
          <span class="server-tab-status" :class="{ connected: server.connected }">
            {{ server.connected ? '●' : '○' }}
          </span>
          <button
            @click.stop="closeServerTab(server.id)"
            class="server-tab-close"
            title="关闭"
          >
            ×
          </button>
        </div>
      </div>

      <!-- 工作区内容 -->
      <div
        v-for="server in connectedServers"
        :key="server.id"
        v-show="activeServerId === server.id"
        class="workspace-container"
        :class="{ resizing: isResizing && activeServerId === server.id }"
      >
        <!-- 主内容区域：终端和文件管理（上下结构） -->
        <div class="main-content-section" :style="{ width: getMainContentWidth(server.id) }">
          <!-- 左侧区域：终端（上）+ 文件管理（下） -->
          <div class="left-section" :style="getLeftSectionStyle(server.id)">
            <!-- 终端（上） -->
            <div class="terminal-section" :style="getTerminalStyle(server.id)">
              <TerminalTab :server="server" :tab="null" :ref="el => setTerminalRef(server.id, el)" />
            </div>

            <!-- 水平分割器（终端和文件管理器之间） -->
            <div
              v-if="!getFilemanagerCollapsed(server.id)"
              class="resizer horizontal-resizer"
              @mousedown="startResize('horizontal-terminal', $event, server.id)"
            ></div>

            <!-- 文件管理器（下） -->
            <div class="filemanager-section" :class="{ collapsed: getFilemanagerCollapsed(server.id) }" :style="getFilemanagerStyle(server.id)">
              <!-- 展开状态 -->
              <template v-if="!getFilemanagerCollapsed(server.id)">
                <div class="section-header">
                  <h3>📁 文件管理</h3>
                  <div class="section-actions">
                    <button @click="refreshFileManager(server.id)" class="action-btn" title="刷新">🔄</button>
                    <button @click="toggleFilemanager(server.id)" class="action-btn" title="收起">▼</button>
                  </div>
                </div>
                <FileManagerTab :server="server" :tab="null" />
              </template>
              <!-- 收起状态 -->
              <div v-else class="filemanager-collapsed">
                <button @click="toggleFilemanager(server.id)" class="expand-filemanager-btn" title="展开文件管理">
                  📁
                </button>
              </div>
            </div>
          </div>

          <!-- 垂直分割器（左侧和右侧之间） -->
          <div
            v-if="getRightPanelVisible(server.id)"
            class="resizer vertical-resizer"
            @mousedown="startResize('vertical-right', $event, server.id)"
          ></div>

          <!-- 右侧区域：AI助手或监控（交替显示） -->
          <div v-if="getRightPanelVisible(server.id)" class="right-section" :style="{ width: getRightPanelWidth(server.id) + 'px' }">
            <!-- AI助手 -->
            <div v-if="getRightPanelType(server.id) === 'ai'" class="ai-chat-section">
              <div class="section-header">
                <h3>🤖 AI助手</h3>
                <div class="section-actions">
                  <button @click="toggleAIChat(server.id)" class="action-btn" title="收起">◀</button>
                </div>
              </div>
              <AIChatTab :server="server" :tab="null" />
            </div>

            <!-- 系统监控 -->
            <div v-else-if="getRightPanelType(server.id) === 'monitor'" class="monitor-section">
              <div class="section-header">
                <h3>📊 系统监控</h3>
                <div class="section-actions">
                  <button @click="toggleMonitorAutoRefresh(server.id)" class="action-btn" :class="{ active: getMonitorAutoRefresh(server.id) }" title="自动刷新">
                    {{ getMonitorAutoRefresh(server.id) ? '⏸' : '▶' }}
                  </button>
                  <button @click="refreshMonitor(server.id)" class="action-btn" title="刷新">🔄</button>
                  <button @click="toggleMonitor(server.id)" class="action-btn" title="收起">◀</button>
                </div>
              </div>
              <MonitorTab :server="server" :auto-refresh="getMonitorAutoRefresh(server.id)" :tab="null" />
            </div>
          </div>
        </div>

        <!-- 右侧按钮栏（一直显示在最右侧） -->
        <div class="right-panel-collapsed">
          <button @click="openAIChat(server.id)" class="expand-btn" :class="{ active: getRightPanelType(server.id) === 'ai' }" title="展开AI助手">🤖</button>
          <button @click="openMonitor(server.id)" class="expand-btn" :class="{ active: getRightPanelType(server.id) === 'monitor' }" title="展开监控">📊</button>
        </div>
      </div>
    </div>
    <div v-else class="empty-state">
      <div class="empty-content">
        <div class="empty-icon">🖥️</div>
        <h3>欢迎使用 MySSH</h3>
        <p>请从左侧添加并连接服务器以打开工作区</p>
      </div>
    </div>
  </div>
</template>

<script setup>
import { ref, computed, watch, onMounted, onUnmounted } from 'vue'
import { useServerStore } from '@/stores/serverStore'
import TerminalTab from './TerminalTab.vue'
import FileManagerTab from './FileManagerTab.vue'
import MonitorTab from './MonitorTab.vue'
import AIChatTab from './AIChatTab.vue'
import { registerShortcut } from '@/utils/shortcuts'

const store = useServerStore()

// 当前活动的服务器ID - 与 store 同步
const activeServerId = computed({
  get: () => store.activeServerId,
  set: (value) => {
    store.activeServerId = value
  }
})

// 已连接的服务器列表
const connectedServers = computed(() => {
  return store.servers.filter(s => s.connected)
})

// 当前活动的服务器
const activeServer = computed(() => {
  if (!activeServerId.value) return null
  return store.servers.find(s => s.id === activeServerId.value)
})

// 每个服务器的尺寸配置
const serverSizes = ref({}) // { serverId: { width: number, terminalHeight: number, rightPanelType: 'ai' | 'monitor' | null, rightPanelWidth: number, filemanagerCollapsed: boolean } }

// 每个服务器的监控自动刷新状态
const serverMonitorAutoRefresh = ref({}) // { serverId: boolean }

// 每个服务器的终端引用
const terminalRefs = ref({}) // { serverId: TerminalTab }

// 区域尺寸 - 根据窗口大小计算默认比例
const getDefaultFilemanagerWidth = () => {
  // 默认左侧区域占窗口宽度的 35%，最小 350px，最大 600px
  const windowWidth = window.innerWidth
  const defaultWidth = Math.max(350, Math.min(600, windowWidth * 0.35))
  return Math.round(defaultWidth)
}

const getDefaultTerminalHeight = () => {
  // 默认终端占窗口高度的 50%，最小 200px，最大 60vh
  const windowHeight = window.innerHeight
  const defaultHeight = Math.max(200, Math.min(windowHeight * 0.5, 500))
  return Math.round(defaultHeight)
}

const getDefaultRightPanelWidth = () => {
  // 默认右侧面板占窗口宽度的 30%，最小 300px，最大 500px
  const windowWidth = window.innerWidth
  const defaultWidth = Math.max(300, Math.min(500, windowWidth * 0.3))
  return Math.round(defaultWidth)
}


// 获取左侧区域宽度（终端和文件管理器共用）
function getServerWidth(serverId) {
  if (!serverSizes.value[serverId]) {
    serverSizes.value[serverId] = {
      width: getDefaultFilemanagerWidth(),
      terminalHeight: getDefaultTerminalHeight(),
      rightPanelType: null,
      rightPanelWidth: getDefaultRightPanelWidth(),
      filemanagerCollapsed: false
    }
  }
  return serverSizes.value[serverId].width
}

// 获取左侧区域样式
function getLeftSectionStyle(serverId) {
  // 如果右侧面板关闭，左侧区域占满整个主内容区域
  if (!getRightPanelVisible(serverId)) {
    return { width: '100%' }
  }
  // 如果右侧面板打开，使用固定宽度
  return { width: getServerWidth(serverId) + 'px' }
}

// 获取文件管理器收起状态
function getFilemanagerCollapsed(serverId) {
  if (!serverSizes.value[serverId]) {
    serverSizes.value[serverId] = {
      width: getDefaultFilemanagerWidth(),
      terminalHeight: getDefaultTerminalHeight(),
      rightPanelType: null,
      rightPanelWidth: getDefaultRightPanelWidth(),
      filemanagerCollapsed: false
    }
  }
  return serverSizes.value[serverId].filemanagerCollapsed || false
}

// 切换文件管理器收起状态
function toggleFilemanager(serverId) {
  if (!serverSizes.value[serverId]) {
    serverSizes.value[serverId] = {
      width: getDefaultFilemanagerWidth(),
      terminalHeight: getDefaultTerminalHeight(),
      rightPanelType: null,
      rightPanelWidth: getDefaultRightPanelWidth(),
      filemanagerCollapsed: false
    }
  }
  const config = serverSizes.value[serverId]
  config.filemanagerCollapsed = !config.filemanagerCollapsed
  saveServerSizes()
}

// 获取终端样式
function getTerminalStyle(serverId) {
  const isFilemanagerCollapsed = getFilemanagerCollapsed(serverId)
  
  if (isFilemanagerCollapsed) {
    // 如果文件管理收起，终端占满整个高度
    return { height: '100%' }
  }
  // 如果文件管理展开，使用固定高度
  return { height: getTerminalHeight(serverId) + 'px' }
}

// 获取文件管理器样式
function getFilemanagerStyle(serverId) {
  const terminalHeight = getTerminalHeight(serverId)
  const isCollapsed = getFilemanagerCollapsed(serverId)
  
  if (isCollapsed) {
    // 收起时，高度固定为按钮栏高度
    return { height: '40px' }
  }
  // 展开时，占满剩余高度
  return { height: `calc(100% - ${terminalHeight}px - 4px)` }
}

// 获取终端高度
function getTerminalHeight(serverId) {
  if (!serverSizes.value[serverId]) {
    serverSizes.value[serverId] = {
      width: getDefaultFilemanagerWidth(),
      terminalHeight: getDefaultTerminalHeight(),
      rightPanelType: null,
      rightPanelWidth: getDefaultRightPanelWidth(),
      filemanagerCollapsed: false
    }
  }
  const config = serverSizes.value[serverId]
  return config.terminalHeight || getDefaultTerminalHeight()
}

// 获取主内容区域宽度
function getMainContentWidth(serverId) {
  const buttonBarWidth = 40 // 右侧按钮栏固定宽度
  if (!getRightPanelVisible(serverId)) {
    // 如果右侧面板关闭，主内容区域占满到按钮栏左侧
    return `calc(100% - ${buttonBarWidth}px)`
  }
  // 如果右侧面板打开，主内容区域占满到右侧面板左侧
  return `calc(100% - ${getRightPanelWidth(serverId)}px - ${buttonBarWidth}px - 4px)`
}

// 获取右侧面板是否可见
function getRightPanelVisible(serverId) {
  if (!serverSizes.value[serverId]) {
    serverSizes.value[serverId] = {
      width: getDefaultFilemanagerWidth(),
      terminalHeight: getDefaultTerminalHeight(),
      rightPanelType: null,
      rightPanelWidth: getDefaultRightPanelWidth(),
      filemanagerCollapsed: false
    }
  }
  return serverSizes.value[serverId].rightPanelType !== null
}

// 获取右侧面板类型
function getRightPanelType(serverId) {
  if (!serverSizes.value[serverId]) {
    serverSizes.value[serverId] = {
      width: getDefaultFilemanagerWidth(),
      terminalHeight: getDefaultTerminalHeight(),
      rightPanelType: null,
      rightPanelWidth: getDefaultRightPanelWidth(),
      filemanagerCollapsed: false
    }
  }
  return serverSizes.value[serverId].rightPanelType
}

// 获取右侧面板宽度
function getRightPanelWidth(serverId) {
  if (!serverSizes.value[serverId]) {
    serverSizes.value[serverId] = {
      width: getDefaultFilemanagerWidth(),
      terminalHeight: getDefaultTerminalHeight(),
      rightPanelType: null,
      rightPanelWidth: getDefaultRightPanelWidth(),
      filemanagerCollapsed: false
    }
  }
  const config = serverSizes.value[serverId]
  return config.rightPanelWidth || getDefaultRightPanelWidth()
}

// 打开AI助手（如果当前是监控，则切换到AI助手）
function openAIChat(serverId) {
  if (!serverSizes.value[serverId]) {
    serverSizes.value[serverId] = {
      width: getDefaultFilemanagerWidth(),
      terminalHeight: getDefaultTerminalHeight(),
      rightPanelType: null,
      rightPanelWidth: getDefaultRightPanelWidth()
    }
  }
  const config = serverSizes.value[serverId]
  // 如果当前是监控，切换到AI助手；如果当前是AI助手，关闭；如果当前是null，打开AI助手
  if (config.rightPanelType === 'monitor') {
    config.rightPanelType = 'ai'
  } else if (config.rightPanelType === 'ai') {
    config.rightPanelType = null
  } else {
    config.rightPanelType = 'ai'
  }
  if (!config.rightPanelWidth) {
    config.rightPanelWidth = getDefaultRightPanelWidth()
  }
  saveServerSizes()
}

// 切换AI助手（关闭）
function toggleAIChat(serverId) {
  if (!serverSizes.value[serverId]) {
    serverSizes.value[serverId] = {
      width: getDefaultFilemanagerWidth(),
      terminalHeight: getDefaultTerminalHeight(),
      rightPanelType: null,
      rightPanelWidth: getDefaultRightPanelWidth()
    }
  }
  const config = serverSizes.value[serverId]
  if (config.rightPanelType === 'ai') {
    config.rightPanelType = null
  } else {
    config.rightPanelType = 'ai'
  }
  saveServerSizes()
}

// 打开监控（如果当前是AI助手，则切换到监控）
function openMonitor(serverId) {
  if (!serverSizes.value[serverId]) {
    serverSizes.value[serverId] = {
      width: getDefaultFilemanagerWidth(),
      terminalHeight: getDefaultTerminalHeight(),
      rightPanelType: null,
      rightPanelWidth: getDefaultRightPanelWidth()
    }
  }
  const config = serverSizes.value[serverId]
  // 如果当前是AI助手，切换到监控；如果当前是监控，关闭；如果当前是null，打开监控
  if (config.rightPanelType === 'ai') {
    config.rightPanelType = 'monitor'
  } else if (config.rightPanelType === 'monitor') {
    config.rightPanelType = null
  } else {
    config.rightPanelType = 'monitor'
  }
  if (!config.rightPanelWidth) {
    config.rightPanelWidth = getDefaultRightPanelWidth()
  }
  saveServerSizes()
}

// 切换监控（关闭）
function toggleMonitor(serverId) {
  if (!serverSizes.value[serverId]) {
    serverSizes.value[serverId] = {
      width: getDefaultFilemanagerWidth(),
      terminalHeight: getDefaultTerminalHeight(),
      rightPanelType: null,
      rightPanelWidth: getDefaultRightPanelWidth()
    }
  }
  const config = serverSizes.value[serverId]
  if (config.rightPanelType === 'monitor') {
    config.rightPanelType = null
  } else {
    config.rightPanelType = 'monitor'
  }
  saveServerSizes()
}

// 获取服务器的监控自动刷新状态
function getMonitorAutoRefresh(serverId) {
  if (serverMonitorAutoRefresh.value[serverId] === undefined) {
    serverMonitorAutoRefresh.value[serverId] = true
  }
  return serverMonitorAutoRefresh.value[serverId]
}

// 设置终端引用
function setTerminalRef(serverId, el) {
  if (el) {
    terminalRefs.value[serverId] = el
  }
}

// 拖动状态
const isResizing = ref(false)
const resizeType = ref(null) // 'horizontal' | 'vertical'
const currentResizeServerId = ref(null)
const startX = ref(0)
const startY = ref(0)
const startLeftWidth = ref(0)
const startTerminalHeight = ref(0)
const startRightPanelWidth = ref(0)

// 监听服务器连接状态
watch(() => store.servers, (servers) => {
  const connected = servers.filter(s => s.connected)
  if (connected.length > 0) {
    // 如果有连接的服务器，且当前没有活动服务器或活动服务器未连接，自动选择第一个已连接的
    if (!activeServerId.value || !connected.find(s => s.id === activeServerId.value)) {
      const firstConnected = connected[0]
      if (firstConnected) {
        store.activeServerId = firstConnected.id
    // 初始化服务器尺寸
    if (!serverSizes.value[firstConnected.id]) {
      serverSizes.value[firstConnected.id] = {
        width: getDefaultFilemanagerWidth(),
        terminalHeight: getDefaultTerminalHeight(),
        rightPanelType: null,
        rightPanelWidth: getDefaultRightPanelWidth(),
        filemanagerCollapsed: false
      }
    }
        // 初始化监控自动刷新
        if (serverMonitorAutoRefresh.value[firstConnected.id] === undefined) {
          serverMonitorAutoRefresh.value[firstConnected.id] = true
        }
      }
    }
  } else {
    store.activeServerId = null
  }
}, { deep: true })

// 监听 store 的 activeServerId 变化，确保同步
watch(() => store.activeServerId, (newId) => {
  if (newId) {
    // 初始化服务器尺寸（如果还没有）
    if (!serverSizes.value[newId]) {
      serverSizes.value[newId] = {
        width: getDefaultFilemanagerWidth(),
        terminalHeight: getDefaultTerminalHeight(),
        rightPanelType: null,
        rightPanelWidth: getDefaultRightPanelWidth(),
        filemanagerCollapsed: false
      }
    }
    // 初始化监控自动刷新（如果还没有）
    if (serverMonitorAutoRefresh.value[newId] === undefined) {
      serverMonitorAutoRefresh.value[newId] = true
    }
  }
})

// 选择服务器（点击标签页时）
function selectServer(serverId) {
  store.activeServerId = serverId
  // 初始化服务器尺寸（如果还没有）
  if (!serverSizes.value[serverId]) {
    serverSizes.value[serverId] = {
      width: getDefaultFilemanagerWidth(),
      terminalHeight: getDefaultTerminalHeight(),
      rightPanelType: null,
      rightPanelWidth: getDefaultRightPanelWidth(),
      filemanagerCollapsed: false
    }
  }
  // 初始化监控自动刷新（如果还没有）
  if (serverMonitorAutoRefresh.value[serverId] === undefined) {
    serverMonitorAutoRefresh.value[serverId] = true
  }
}

// 关闭服务器标签页
function closeServerTab(serverId) {
  // 断开服务器连接
  store.disconnectServer(serverId)
  // 如果关闭的是当前活动的服务器，切换到其他服务器
  if (activeServerId.value === serverId) {
    const remaining = connectedServers.value.filter(s => s.id !== serverId)
    if (remaining.length > 0) {
      store.activeServerId = remaining[0].id
    } else {
      store.activeServerId = null
    }
  }
}

function startResize(type, event, serverId) {
  if (!serverId) return
  
  isResizing.value = true
  resizeType.value = type
  currentResizeServerId.value = serverId
  startX.value = event.clientX
  startY.value = event.clientY
  
  const sizes = serverSizes.value[serverId] || {
    width: getDefaultFilemanagerWidth(),
    terminalHeight: getDefaultTerminalHeight(),
    rightPanelType: null,
    rightPanelWidth: getDefaultRightPanelWidth(),
    filemanagerCollapsed: false
  }
  startLeftWidth.value = sizes.width
  startTerminalHeight.value = sizes.terminalHeight || getDefaultTerminalHeight()
  startRightPanelWidth.value = sizes.rightPanelWidth || getDefaultRightPanelWidth()
  
  event.preventDefault()
  document.addEventListener('mousemove', handleResize)
  document.addEventListener('mouseup', stopResize)
}

function handleResize(event) {
  if (!isResizing.value || !currentResizeServerId.value) return

  const serverId = currentResizeServerId.value
  
  if (!serverSizes.value[serverId]) {
    serverSizes.value[serverId] = {
      width: getDefaultFilemanagerWidth(),
      terminalHeight: getDefaultTerminalHeight(),
      rightPanelType: null,
      rightPanelWidth: getDefaultRightPanelWidth(),
      filemanagerCollapsed: false
    }
  }

  if (resizeType.value === 'vertical') {
    // 调整左侧区域宽度（终端和文件管理器共用）
    const deltaX = event.clientX - startX.value
    const newWidth = startLeftWidth.value + deltaX
    const minWidth = 300
    const maxWidth = window.innerWidth * 0.7
    serverSizes.value[serverId].width = Math.max(minWidth, Math.min(maxWidth, newWidth))
  } else if (resizeType.value === 'horizontal-terminal') {
    // 调整终端高度（终端和文件管理器之间的分割器）
    const deltaY = event.clientY - startY.value
    const newHeight = startTerminalHeight.value + deltaY
    const minHeight = 200
    const maxHeight = window.innerHeight * 0.7
    serverSizes.value[serverId].terminalHeight = Math.max(minHeight, Math.min(maxHeight, newHeight))
  } else if (resizeType.value === 'vertical-right') {
    // 调整右侧面板宽度（从左侧拖动，向右缩小，向左扩大）
    const deltaX = startX.value - event.clientX
    const newWidth = startRightPanelWidth.value + deltaX
    const minWidth = 300
    const maxWidth = window.innerWidth * 0.5
    serverSizes.value[serverId].rightPanelWidth = Math.max(minWidth, Math.min(maxWidth, newWidth))
  }
}

function stopResize() {
  isResizing.value = false
  resizeType.value = null
  currentResizeServerId.value = null
  document.removeEventListener('mousemove', handleResize)
  document.removeEventListener('mouseup', stopResize)
  
  // 保存所有服务器的尺寸
  saveServerSizes()
}

// 保存服务器尺寸到 localStorage
function saveServerSizes() {
  localStorage.setItem('serverSizes', JSON.stringify(serverSizes.value))
  localStorage.setItem('serverMonitorAutoRefresh', JSON.stringify(serverMonitorAutoRefresh.value))
}

// 从 localStorage 加载服务器尺寸
function loadServerSizes() {
  const savedSizes = localStorage.getItem('serverSizes')
  const savedAutoRefresh = localStorage.getItem('serverMonitorAutoRefresh')
  
  if (savedSizes) {
    try {
      serverSizes.value = JSON.parse(savedSizes)
    } catch (e) {
      console.error('Failed to parse server sizes:', e)
    }
  }
  
  if (savedAutoRefresh) {
    try {
      serverMonitorAutoRefresh.value = JSON.parse(savedAutoRefresh)
    } catch (e) {
      console.error('Failed to parse monitor auto refresh:', e)
    }
  }
}

// 窗口大小变化处理函数
let resizeHandler = null

onMounted(() => {
  // 从 localStorage 恢复尺寸
  loadServerSizes()
  
  // 注册全局快捷键
  // Ctrl/Cmd + W: 关闭当前服务器标签页
  registerShortcut('w', () => {
    if (activeServerId.value) {
      closeServerTab(activeServerId.value)
    }
  }, { ctrl: true })
  
  // Ctrl/Cmd + F: 聚焦到搜索框（在文件管理器中）
  registerShortcut('f', () => {
    // 这个功能需要在 FileManagerTab 中实现
  }, { ctrl: true })
  
  // 监听窗口大小变化
  resizeHandler = () => {
    // 窗口大小变化时，可以调整默认值（如果需要）
  }
  
  window.addEventListener('resize', resizeHandler)
  
  // 监听服务器连接，自动打开工作区
  watch(() => store.servers, (servers) => {
    const connected = servers.filter(s => s.connected)
    if (connected.length > 0 && !activeServerId.value) {
      activeServerId.value = connected[0].id
    }
  }, { immediate: true, deep: true })
})

onUnmounted(() => {
  // 保存所有配置
  saveServerSizes()
  
  // 移除窗口大小监听
  if (resizeHandler) {
    window.removeEventListener('resize', resizeHandler)
  }
  
  stopResize()
})

function toggleMonitorAutoRefresh(serverId) {
  if (!serverId) return
  serverMonitorAutoRefresh.value[serverId] = !getMonitorAutoRefresh(serverId)
  saveServerSizes()
}

function refreshMonitor(serverId) {
  // MonitorTab 组件内部会处理刷新
}

function refreshFileManager(serverId) {
  // FileManagerTab 组件内部会处理刷新
}
</script>

<style scoped>
.workspace-tabs {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
}

.workspace-container {
  display: flex;
  flex-direction: row;
  height: 100%;
  overflow: hidden;
}

/* 主内容区域 */
.main-content-section {
  flex: 1;
  display: flex;
  flex-direction: row;
  overflow: hidden;
  min-width: 0;
}

/* 左侧区域（文件管理器和终端，上下结构） */
.left-section {
  min-width: 300px;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
  overflow: hidden;
  flex-shrink: 1;
  flex-grow: 1;
}

/* 右侧区域（AI助手或监控，交替显示） */
.right-section {
  min-width: 300px;
  max-width: 50vw;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
  overflow: hidden;
  flex-shrink: 0;
  height: 100%;
}

/* 系统监控区域（右侧，交替显示） */
.monitor-section {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-secondary);
  overflow: hidden;
  flex-shrink: 0;
}


.expand-monitor-btn {
  width: 100%;
  height: 100%;
  padding: 8px;
  font-size: 20px;
  background: transparent;
  border: none;
  color: var(--text-primary);
  cursor: pointer;
  transition: background 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.expand-monitor-btn:hover {
  background: var(--bg-hover);
}

/* 分割器 */
.resizer {
  background: var(--border-color);
  flex-shrink: 0;
  z-index: 10;
  transition: background 0.2s;
}

.resizer:hover {
  background: var(--accent-color);
}

.horizontal-resizer {
  height: 4px;
  width: 100%;
  cursor: row-resize;
  user-select: none;
  flex-shrink: 0;
}

.vertical-resizer {
  width: 4px;
  height: 100%;
  cursor: col-resize;
  user-select: none;
}

/* 拖动时的样式 */
.workspace-container.resizing {
  user-select: none;
}

.workspace-container.resizing .resizer {
  background: var(--accent-color);
}


/* 文件管理器（左侧下方，可调整高度，可收起） */
.filemanager-section {
  min-height: 200px;
  max-height: 70vh;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
  overflow: hidden;
  flex-shrink: 0;
  transition: height 0.3s;
}

.filemanager-section.collapsed {
  min-height: 40px;
  height: 40px;
}

/* 文件管理器收起状态 */
.filemanager-collapsed {
  height: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-secondary);
  border-top: 1px solid var(--border-color);
  flex-shrink: 0;
}

.expand-filemanager-btn {
  width: 100%;
  height: 100%;
  padding: 8px;
  font-size: 18px;
  background: transparent;
  border: none;
  color: var(--text-primary);
  cursor: pointer;
  transition: background 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
}

.expand-filemanager-btn:hover {
  background: var(--bg-hover);
}


/* 终端（左侧下方） */
.terminal-section {
  flex: 1;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
  overflow: hidden;
  min-width: 0;
  min-height: 0;
}

/* AI助手（右侧，交替显示） */
.ai-chat-section {
  height: 100%;
  display: flex;
  flex-direction: column;
  background: var(--bg-primary);
  overflow: hidden;
  flex-shrink: 0;
}

/* AI助手折叠状态 */
.ai-chat-collapsed {
  width: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-secondary);
  border-left: 1px solid var(--border-color);
  flex-shrink: 0;
}

/* 监控折叠状态 */
.monitor-collapsed {
  width: 40px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--bg-secondary);
  border-left: 1px solid var(--border-color);
  flex-shrink: 0;
}

/* 右侧按钮栏（一直显示在最右侧） */
.right-panel-collapsed {
  width: 40px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-start;
  background: var(--bg-secondary);
  border-left: 1px solid var(--border-color);
  flex-shrink: 0;
  padding: 8px 0;
  gap: 8px;
}

.expand-btn {
  width: 32px;
  height: 32px;
  padding: 0;
  font-size: 18px;
  background: transparent;
  border: none;
  color: var(--text-primary);
  cursor: pointer;
  transition: background 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
}

.expand-btn:hover {
  background: var(--bg-hover);
}

.expand-btn.active {
  background: var(--accent-color);
  color: white;
}

.expand-ai-btn {
  width: 100%;
  height: 100%;
  padding: 8px;
  font-size: 20px;
  background: transparent;
  border: none;
  color: var(--text-primary);
  cursor: pointer;
  transition: background 0.2s;
  display: flex;
  align-items: center;
  justify-content: center;
}

.expand-ai-btn:hover {
  background: var(--bg-hover);
}


/* 区域标题栏 */
.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 8px 12px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.section-header h3 {
  font-size: 13px;
  font-weight: 600;
  margin: 0;
  color: var(--text-primary);
}

.section-actions {
  display: flex;
  gap: 6px;
}

.action-btn {
  font-size: 11px;
  padding: 4px 8px;
  background: transparent;
  border: none;
  color: var(--text-primary);
  cursor: pointer;
  border-radius: 3px;
  transition: background 0.2s;
}

.action-btn:hover {
  background: var(--bg-hover);
}

.action-btn.active {
  background: var(--accent-color);
  color: white;
}

/* 空状态 */
.empty-state {
  display: flex;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-secondary);
}

.empty-content {
  text-align: center;
}

.empty-icon {
  font-size: 64px;
  margin-bottom: 16px;
}

.empty-content h3 {
  font-size: 20px;
  margin-bottom: 8px;
  color: var(--text-primary);
}

.empty-content p {
  font-size: 14px;
}

/* 服务器标签页 */
.server-tabs-container {
  display: flex;
  flex-direction: column;
  height: 100%;
}

.server-tabs-header {
  display: flex;
  align-items: center;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  padding: 0 8px;
  overflow-x: auto;
  min-height: 36px;
  flex-shrink: 0;
}

.server-tab {
  display: flex;
  align-items: center;
  gap: 6px;
  padding: 6px 12px;
  background: var(--bg-tertiary);
  border-right: 1px solid var(--border-color);
  cursor: pointer;
  user-select: none;
  white-space: nowrap;
  transition: background 0.2s;
  min-width: 120px;
}

.server-tab:hover {
  background: var(--bg-hover);
}

.server-tab.active {
  background: var(--bg-primary);
  border-bottom: 2px solid var(--accent-color);
}

.server-tab-icon {
  font-size: 14px;
}

.server-tab-name {
  font-size: 12px;
  flex: 1;
}

.server-tab-status {
  font-size: 10px;
  color: var(--text-disabled);
}

.server-tab-status.connected {
  color: var(--success-color);
}

.server-tab-close {
  margin-left: 4px;
  width: 16px;
  height: 16px;
  padding: 0;
  font-size: 14px;
  line-height: 1;
  opacity: 0.6;
  border-radius: 3px;
}

.server-tab-close:hover {
  opacity: 1;
  background: var(--error-color);
  color: white;
}

/* 响应式调整 */
@media (max-width: 1200px) {
  .filemanager-section {
    min-width: 200px;
  }
}

@media (max-width: 900px) {
  .left-section {
    width: 100% !important;
    min-width: unset;
    max-width: unset;
  }
  
  .filemanager-section {
    height: 40% !important;
    min-height: 200px;
  }
  
  .terminal-section {
    height: 60% !important;
  }
  
  .vertical-resizer {
    display: none;
  }
  
  .monitor-section {
    min-width: 250px;
  }
  
  .monitor-collapsed {
    width: 30px;
  }
}
</style>
