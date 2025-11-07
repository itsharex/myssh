<template>
  <div class="server-list" :class="{ collapsed: isCollapsed }">
    <!-- 展开状态 -->
    <template v-if="!isCollapsed">
      <div class="server-list-header">
        <h3>服务器列表</h3>
        <div class="header-actions">
          <button class="add-btn" @click="showAddDialog = true" title="添加服务器">
            <span class="icon">+</span>
          </button>
          <button class="collapse-btn" @click="toggleCollapse" title="收起">◀</button>
        </div>
      </div>
      
      <div class="server-items">
      <div
        v-for="server in servers"
        :key="server.id"
        :class="['server-item', { active: activeServerId === server.id, connected: server.connected }]"
        @click="selectServer(server.id)"
      >
        <div class="server-info">
          <div class="server-status">
            <span :class="['status-dot', server.connected ? 'connected' : 'disconnected']"></span>
          </div>
          <div class="server-details">
            <div class="server-name">{{ server.name }}</div>
            <div class="server-address">{{ server.host }}:{{ server.port }}</div>
          </div>
        </div>
        <div class="server-actions">
          <button
            v-if="!server.connected"
            @click.stop="handleConnect(server.id)"
            class="action-btn"
            title="连接"
          >
            ▶
          </button>
          <button
            v-else
            @click.stop="handleDisconnect(server.id)"
            class="action-btn"
            title="断开"
          >
            ⏸
          </button>
          <button
            @click.stop="handleDelete(server.id)"
            class="action-btn delete"
            title="删除"
          >
            ×
          </button>
        </div>
      </div>
    </div>
    </template>

    <!-- 收起状态 -->
    <div v-else class="server-list-collapsed">
      <button class="expand-btn" @click="toggleCollapse" title="展开服务器列表">
        🖥️
      </button>
    </div>

    <!-- 添加服务器对话框 -->
    <div v-if="showAddDialog" class="dialog-overlay" @click.self="closeDialog">
      <div class="dialog" @keydown.esc="closeDialog" @keydown.enter.prevent="handleAddServer">
        <div class="dialog-header">
          <h3>添加服务器</h3>
          <button @click="closeDialog" class="close-btn">×</button>
        </div>
        <div class="dialog-body">
          <div class="form-group">
            <label>服务器名称</label>
            <input 
              ref="nameInputRef"
              v-model="newServer.name" 
              type="text" 
              placeholder="例如: 生产服务器"
              @keydown.enter.prevent="hostInputRef?.focus()"
            />
          </div>
          <div class="form-group">
            <label>主机地址 <span class="required">*</span></label>
            <input 
              ref="hostInputRef"
              v-model="newServer.host" 
              type="text" 
              placeholder="192.168.1.100 或 example.com" 
              :class="{ error: hostError }"
              @blur="() => validateHost(newServer.host)"
              @input="hostError = ''"
              @keydown.enter.prevent="portInputRef?.focus()"
            />
            <span v-if="hostError" class="error-message">{{ hostError }}</span>
          </div>
          <div class="form-group">
            <label>端口</label>
            <input 
              ref="portInputRef"
              v-model.number="newServer.port" 
              type="number" 
              placeholder="22" 
              min="1"
              max="65535"
              :class="{ error: portError }"
              @blur="() => validatePort(newServer.port)"
              @input="portError = ''"
              @keydown.enter.prevent="usernameInputRef?.focus()"
            />
            <span v-if="portError" class="error-message">{{ portError }}</span>
          </div>
          <div class="form-group">
            <label>用户名 <span class="required">*</span></label>
            <input 
              ref="usernameInputRef"
              v-model="newServer.username" 
              type="text" 
              placeholder="root"
              @keydown.enter.prevent="passwordInputRef?.focus()"
            />
          </div>
          <div class="form-group">
            <label>密码</label>
            <input 
              ref="passwordInputRef"
              v-model="newServer.password" 
              type="password" 
              placeholder="密码或密钥路径"
              @keydown.enter.prevent="handleAddServer"
            />
          </div>
        </div>
        <div class="dialog-footer">
          <button @click="closeDialog">取消</button>
          <button @click="handleAddServer" class="primary">添加</button>
        </div>
      </div>
    </div>

    <!-- 确认删除对话框 -->
    <ConfirmDialog
      v-model:visible="showDeleteConfirm"
      title="删除服务器"
      :message="deleteConfirmMessage"
      confirm-text="删除"
      type="danger"
      @confirm="confirmDelete"
    />
  </div>
</template>

<script setup>
import { ref, computed, watch, nextTick } from 'vue'
import { useServerStore } from '@/stores/serverStore'
import ConfirmDialog from './ConfirmDialog.vue'
import { error, success } from '@/utils/toast'
import { registerShortcut } from '@/utils/shortcuts'

const store = useServerStore()
const servers = computed(() => store.servers)
const activeServerId = computed(() => store.activeServerId)
const showAddDialog = ref(false)
const isCollapsed = ref(false)

// 删除确认对话框
const showDeleteConfirm = ref(false)
const deleteConfirmMessage = ref('')
const pendingDeleteServerId = ref(null)

// 输入框引用
const nameInputRef = ref(null)
const hostInputRef = ref(null)
const portInputRef = ref(null)
const usernameInputRef = ref(null)
const passwordInputRef = ref(null)

// 注册全局快捷键
registerShortcut('n', () => {
  if (!showAddDialog.value) {
    showAddDialog.value = true
  }
}, { ctrl: true })

// 监听对话框打开，自动聚焦第一个输入框
watch(showAddDialog, async (visible) => {
  if (visible) {
    await nextTick()
    // 优先聚焦主机地址（必填项）
    if (hostInputRef.value) {
      hostInputRef.value.focus()
    } else if (nameInputRef.value) {
      nameInputRef.value.focus()
    }
  }
})

// 从 localStorage 加载收起状态
const loadCollapseState = () => {
  const saved = localStorage.getItem('serverListCollapsed')
  if (saved !== null) {
    isCollapsed.value = JSON.parse(saved)
  }
}

// 保存收起状态
const saveCollapseState = () => {
  localStorage.setItem('serverListCollapsed', JSON.stringify(isCollapsed.value))
}

// 切换收起/展开
function toggleCollapse() {
  isCollapsed.value = !isCollapsed.value
  saveCollapseState()
}

// 初始化时加载状态
loadCollapseState()

const newServer = ref({
  name: '',
  host: '',
  port: 22,
  username: '',
  password: ''
})

const hostError = ref('')
const portError = ref('')

// 验证主机地址格式
function validateHost(host) {
  if (!host || host.trim() === '') {
    hostError.value = '主机地址不能为空'
    return false
  }
  
  const trimmedHost = host.trim()
  
  // IPv4 地址格式校验
  const ipv4Regex = /^((25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)\.){3}(25[0-5]|2[0-4][0-9]|[01]?[0-9][0-9]?)$/
  
  // 域名格式校验（支持子域名）
  const domainRegex = /^([a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?\.)+[a-zA-Z]{2,}$|^[a-zA-Z0-9]([a-zA-Z0-9\-]{0,61}[a-zA-Z0-9])?$/
  
  // 支持 localhost
  if (trimmedHost === 'localhost') {
    hostError.value = ''
    return true
  }
  
  // 校验 IPv4 或域名
  if (ipv4Regex.test(trimmedHost) || domainRegex.test(trimmedHost)) {
    hostError.value = ''
    return true
  } else {
    hostError.value = '请输入有效的IP地址或域名（例如: 192.168.1.100 或 example.com）'
    return false
  }
}

// 验证端口范围
function validatePort(port) {
  if (!port || port === '') {
    portError.value = '端口不能为空'
    return false
  }
  
  const portNum = Number(port)
  if (isNaN(portNum) || portNum < 1 || portNum > 65535) {
    portError.value = '端口必须在 1-65535 之间'
    return false
  }
  
  portError.value = ''
  return true
}

function selectServer(serverId) {
  store.activeServerId = serverId
  const server = store.servers.find(s => s.id === serverId)
  // 如果服务器已连接，确保工作区切换到该服务器
  if (server && server.connected) {
    // 触发工作区切换（通过 store 的 activeServerId）
    // WorkspaceTabs 会监听这个变化
  }
}

async function handleConnect(serverId) {
  await store.connectServer(serverId)
  // 连接成功后，工作区会自动显示该服务器的工作区
  // 不需要手动创建标签页，因为工作区会显示所有连接的服务器
}

async function handleDisconnect(serverId) {
  await store.disconnectServer(serverId)
}

function closeDialog() {
  showAddDialog.value = false
  // 清除错误信息
  hostError.value = ''
  portError.value = ''
}

function handleDelete(serverId) {
  const server = store.servers.find(s => s.id === serverId)
  if (server) {
    pendingDeleteServerId.value = serverId
    deleteConfirmMessage.value = `确定要删除服务器 "${server.name || server.host}" 吗？\n此操作不可恢复！`
    showDeleteConfirm.value = true
  }
}

function confirmDelete() {
  if (pendingDeleteServerId.value) {
    store.removeServer(pendingDeleteServerId.value)
    success('服务器已删除')
    pendingDeleteServerId.value = null
  }
}

function handleAddServer() {
  // 清除之前的错误信息
  hostError.value = ''
  portError.value = ''
  
  // 验证主机地址
  if (!validateHost(newServer.value.host)) {
    hostInputRef.value?.focus()
    return
  }
  
  // 验证端口
  if (!validatePort(newServer.value.port)) {
    portInputRef.value?.focus()
    return
  }
  
  // 验证用户名
  if (!newServer.value.username || newServer.value.username.trim() === '') {
    error('请填写用户名')
    usernameInputRef.value?.focus()
    return
  }
  
  // 添加服务器
  store.addServer({
    ...newServer.value,
    host: newServer.value.host.trim(),
    username: newServer.value.username.trim()
  })
  
  success('服务器已添加')
  
  // 重置表单
  newServer.value = {
    name: '',
    host: '',
    port: 22,
    username: '',
    password: ''
  }
  hostError.value = ''
  portError.value = ''
  showAddDialog.value = false
}
</script>

<style scoped>
.server-list {
  width: 360px;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
  display: flex;
  flex-direction: column;
  height: 100vh;
  transition: width 0.3s;
  flex-shrink: 0;
}

.server-list.collapsed {
  width: 40px;
}

.server-list-header {
  padding: 16px 20px;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.header-actions {
  display: flex;
  gap: 8px;
  align-items: center;
}

.collapse-btn {
  width: 28px;
  height: 28px;
  padding: 0;
  font-size: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
  background: transparent;
  border: 1px solid var(--border-color);
  border-radius: 4px;
  color: var(--text-primary);
  cursor: pointer;
  transition: background 0.2s;
}

.collapse-btn:hover {
  background: var(--bg-hover);
}

/* 收起状态 */
.server-list-collapsed {
  width: 40px;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: flex-start;
  background: var(--bg-secondary);
  border-right: 1px solid var(--border-color);
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

.server-list-header h3 {
  font-size: 18px;
  font-weight: 600;
}

.add-btn {
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 4px;
  font-size: 22px;
  line-height: 1;
}

.server-items {
  flex: 1;
  overflow-y: auto;
  padding: 12px;
}

.server-item {
  padding: 14px 16px;
  margin-bottom: 6px;
  border-radius: 6px;
  cursor: pointer;
  transition: background 0.2s;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.server-item:hover {
  background: var(--bg-hover);
}

.server-item.active {
  background: var(--bg-active);
}

.server-item.connected .server-name {
  color: var(--success-color);
}

.server-info {
  display: flex;
  align-items: center;
  flex: 1;
  min-width: 0;
}

.server-status {
  margin-right: 12px;
}

.status-dot {
  width: 10px;
  height: 10px;
  border-radius: 50%;
  display: inline-block;
}

.status-dot.connected {
  background: var(--success-color);
  box-shadow: 0 0 4px var(--success-color);
}

.status-dot.disconnected {
  background: var(--text-disabled);
}

.server-details {
  flex: 1;
  min-width: 0;
}

.server-name {
  font-size: 15px;
  font-weight: 500;
  margin-bottom: 4px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.server-address {
  font-size: 13px;
  color: var(--text-secondary);
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.server-actions {
  display: flex;
  gap: 6px;
  opacity: 0;
  transition: opacity 0.2s;
}

.server-item:hover .server-actions {
  opacity: 1;
}

.action-btn {
  width: 28px;
  height: 28px;
  padding: 0;
  font-size: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.action-btn.delete:hover {
  background: var(--error-color);
  color: white;
}

/* 对话框样式 */
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
  z-index: 1000;
}

.dialog {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  width: 480px;
  max-width: 90vw;
  box-shadow: 0 4px 20px rgba(0, 0, 0, 0.3);
}

.dialog-header {
  padding: 16px;
  border-bottom: 1px solid var(--border-color);
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.dialog-header h3 {
  font-size: 16px;
  font-weight: 600;
}

.close-btn {
  width: 24px;
  height: 24px;
  padding: 0;
  font-size: 20px;
  line-height: 1;
}

.dialog-body {
  padding: 16px;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 6px;
}

.form-group input {
  width: 100%;
}

.form-group input.error {
  border-color: var(--error-color);
}

.error-message {
  display: block;
  font-size: 11px;
  color: var(--error-color);
  margin-top: 4px;
}

.required {
  color: var(--error-color);
}

.dialog-footer {
  padding: 16px;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.dialog-footer button.primary {
  background: var(--accent-color);
  color: white;
}

.dialog-footer button.primary:hover {
  background: var(--accent-hover);
}
</style>

