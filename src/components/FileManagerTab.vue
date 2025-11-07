<template>
  <div class="file-manager-tab">
    <div class="file-manager-header">
      <div class="path-bar">
        <button @click="handleGoHome" class="path-btn" title="首页">🏠</button>
        <button @click="handleGoUp" class="path-btn" title="上一级">↑</button>
        <input
          v-model="currentPath"
          @keydown.enter="handleNavigate"
          class="path-input"
          placeholder="/"
        />
        <button @click="handleRefresh" class="path-btn" title="刷新">🔄</button>
      </div>
      <div class="file-actions">
        <input
          ref="searchInputRef"
          v-model="searchText"
          @input="handleSearch"
          class="search-input"
          placeholder="搜索文件... (Ctrl+F)"
        />
        <button @click="handleUpload" class="action-btn" title="上传文件">📤 上传</button>
        <button @click="handleDownload" class="action-btn" title="下载文件">📥 下载</button>
        <button @click="handleNewFolder" class="action-btn" title="新建文件夹">📁 新建</button>
        <button 
          v-if="selectedFiles.length > 0"
          @click="handleRename" 
          class="action-btn" 
          title="重命名"
        >
          ✏️ 重命名
        </button>
        <button 
          v-if="selectedFiles.length > 0"
          @click="handleChmod" 
          class="action-btn" 
          title="权限"
        >
          🔒 权限
        </button>
        <button @click="handleDelete" class="action-btn delete" title="删除">🗑️ 删除</button>
        <button 
          v-if="selectedFiles.length > 0"
          @click="handleSelectAll" 
          class="action-btn" 
          title="全选/取消全选"
        >
          {{ selectedFiles.length === filteredFiles.length ? '☐ 全选' : '☑ 取消全选' }}
        </button>
      </div>
    </div>
    
    <div class="file-list-container">
      <div class="file-list-header">
        <div class="file-col name">名称</div>
        <div class="file-col size">大小</div>
        <div class="file-col type">类型</div>
        <div class="file-col permissions">权限</div>
        <div class="file-col date">修改时间</div>
      </div>
      <div class="file-list" v-if="filteredFiles.length > 0">
        <div
          v-for="file in filteredFiles"
          :key="file.path || file.name"
          :class="['file-item', { selected: isFileSelected(file) }]"
          @click="handleFileClick(file, $event)"
          @dblclick="handleFileDoubleClick(file)"
          @contextmenu.prevent="handleContextMenu(file, $event)"
        >
          <div class="file-col name">
            <input
              v-if="file.editing"
              v-model="file.newName"
              @keydown.enter="confirmRename(file)"
              @keydown.esc="cancelRename(file)"
              @blur="cancelRename(file)"
              class="rename-input"
              ref="renameInput"
            />
            <template v-else>
              <span class="file-icon">{{ getFileIcon(file) }}</span>
              <span class="file-name">{{ file.name }}</span>
            </template>
          </div>
          <div class="file-col size">{{ formatSize(file.size) }}</div>
          <div class="file-col type">{{ file.type }}</div>
          <div class="file-col permissions">
            {{ file.permissions || '---' }}
          </div>
          <div class="file-col date">{{ formatDate(file.modified) }}</div>
        </div>
      </div>
      <div v-else-if="loading" class="empty-files loading">
        <div class="loading-spinner"></div>
        <p>加载中...</p>
      </div>
      <div v-else-if="error" class="empty-files error">
        <p>❌ {{ error }}</p>
        <button @click="loadFiles" class="retry-btn">重试</button>
      </div>
      <div 
        v-else 
        class="empty-files upload-zone"
        @drop="handleDrop"
        @dragover.prevent
        @dragenter.prevent
        :class="{ 'drag-over': isDragOver }"
        @dragenter="isDragOver = true"
        @dragleave="isDragOver = false"
      >
        <div class="upload-content">
          <div class="upload-icon">📤</div>
          <p class="upload-text">目录为空</p>
          <p class="upload-hint">点击"上传"按钮或拖拽文件到此处上传</p>
          <button @click="handleUpload" class="upload-btn">选择文件上传</button>
        </div>
      </div>
    </div>

    <!-- 右键菜单 -->
    <div 
      v-if="contextMenu.show"
      class="context-menu"
      :style="{ left: contextMenu.x + 'px', top: contextMenu.y + 'px' }"
      @click.stop
    >
      <div class="context-menu-item" @click="handleContextRename">
        ✏️ 重命名
      </div>
      <div class="context-menu-item" @click="handleContextChmod">
        🔒 权限设置
      </div>
      <div class="context-menu-item" @click="handleContextPreview">
        👁️ 预览
      </div>
      <div class="context-menu-item" @click="handleContextCompare">
        🔍 对比
      </div>
      <div class="context-menu-divider"></div>
      <div class="context-menu-item" @click="handleContextDownload">
        📥 下载
      </div>
      <div class="context-menu-item" @click="handleContextDelete">
        🗑️ 删除
      </div>
    </div>

    <!-- 文件预览对话框 -->
    <FilePreview
      v-if="previewFile"
      :show="!!previewFile"
      :file="previewFile"
      :server="server"
      @close="previewFile = null"
      @compare="handleCompareFromPreview"
      @download="handleDownloadFromPreview"
    />

    <!-- 文件对比对话框 -->
    <FileCompare
      v-if="compareFiles.left && compareFiles.right"
      :show="!!compareFiles.left && !!compareFiles.right"
      :left-file="compareFiles.left"
      :right-file="compareFiles.right"
      :server="server"
      @close="compareFiles = { left: null, right: null }"
    />

    <!-- 权限设置对话框 -->
    <div v-if="showChmodDialog" class="chmod-dialog-overlay" @click.self="showChmodDialog = false">
      <div class="chmod-dialog">
        <div class="dialog-header">
          <h3>设置文件权限</h3>
          <button @click="showChmodDialog = false" class="close-btn">×</button>
        </div>
        <div class="dialog-body">
          <div class="chmod-info">
            <p>文件: {{ chmodFile?.name }}</p>
            <p>当前权限: {{ chmodFile?.permissions || '---' }}</p>
          </div>
          <div class="chmod-input">
            <label>八进制权限 (如: 755):</label>
            <input v-model="chmodValue" type="text" placeholder="755" maxlength="3" />
          </div>
          <div class="chmod-bits">
            <div class="chmod-group">
              <label>所有者 (Owner)</label>
              <div class="chmod-checkboxes">
                <label><input type="checkbox" v-model="chmodBits.owner.read" /> 读</label>
                <label><input type="checkbox" v-model="chmodBits.owner.write" /> 写</label>
                <label><input type="checkbox" v-model="chmodBits.owner.execute" /> 执行</label>
              </div>
            </div>
            <div class="chmod-group">
              <label>组 (Group)</label>
              <div class="chmod-checkboxes">
                <label><input type="checkbox" v-model="chmodBits.group.read" /> 读</label>
                <label><input type="checkbox" v-model="chmodBits.group.write" /> 写</label>
                <label><input type="checkbox" v-model="chmodBits.group.execute" /> 执行</label>
              </div>
            </div>
            <div class="chmod-group">
              <label>其他 (Others)</label>
              <div class="chmod-checkboxes">
                <label><input type="checkbox" v-model="chmodBits.others.read" /> 读</label>
                <label><input type="checkbox" v-model="chmodBits.others.write" /> 写</label>
                <label><input type="checkbox" v-model="chmodBits.others.execute" /> 执行</label>
              </div>
            </div>
          </div>
        </div>
        <div class="dialog-footer">
          <button @click="showChmodDialog = false">取消</button>
          <button @click="confirmChmod" class="primary">确定</button>
        </div>
      </div>
    </div>

    <!-- 删除确认对话框 -->
    <ConfirmDialog
      v-model:visible="showDeleteConfirm"
      title="删除文件"
      :message="deleteConfirmMessage"
      confirm-text="删除"
      type="danger"
      @confirm="confirmDelete"
    />
  </div>
</template>

<script setup>
import { ref, watch, onMounted, computed, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/tauri'
import { open, save as saveFile } from '@tauri-apps/api/dialog'
import FilePreview from './FilePreview.vue'
import FileCompare from './FileCompare.vue'
import ConfirmDialog from './ConfirmDialog.vue'
import { error as showError, success, warning, info } from '@/utils/toast'

const props = defineProps({
  tab: Object,
  server: Object
})

const currentPath = ref('/')
const files = ref([])
const selectedFiles = ref([]) // 存储选中文件的完整路径
const loading = ref(false)
const error = ref(null)
const isDragOver = ref(false)
const searchText = ref('')
const contextMenu = ref({ show: false, x: 0, y: 0, file: null })
const previewFile = ref(null)
const compareFiles = ref({ left: null, right: null })
const showChmodDialog = ref(false)
const chmodFile = ref(null)
const chmodValue = ref('')
const chmodBits = ref({
  owner: { read: false, write: false, execute: false },
  group: { read: false, write: false, execute: false },
  others: { read: false, write: false, execute: false }
})

// 删除确认对话框
const showDeleteConfirm = ref(false)
const deleteConfirmMessage = ref('')
const pendingDeleteFiles = ref([])

// 搜索框引用（用于快捷键聚焦）
const searchInputRef = ref(null)
const renameInput = ref(null)

// 过滤后的文件列表
const filteredFiles = computed(() => {
  if (!searchText.value) {
    return files.value
  }
  const search = searchText.value.toLowerCase()
  return files.value.filter(file => 
    file.name.toLowerCase().includes(search) ||
    (file.path && file.path.toLowerCase().includes(search))
  )
})

watch(() => props.server.connected, (connected) => {
  if (connected) {
    loadFiles()
  } else {
    files.value = []
    selectedFiles.value = []
  }
})

watch(() => currentPath.value, () => {
  if (props.server.connected) {
    loadFiles()
  }
})

onMounted(() => {
  if (props.server.connected) {
    loadFiles()
  }
})

async function loadFiles() {
  if (!props.server.connected) {
    return
  }

  loading.value = true
  error.value = null

  try {
    // TODO: 调用 Tauri 获取远程目录文件列表
    // 接口: invoke('list_remote_directory', { serverId: props.server.id, path: currentPath.value })
    // 返回格式: { files: [{ name, type, size, modified, path }] }
    
    // 临时模拟数据，实际应该调用 Tauri
    const mockFiles = [
      { name: '..', type: 'directory', size: 0, modified: new Date(), path: getParentPath() },
      { name: 'home', type: 'directory', size: 0, modified: new Date(), path: currentPath.value + '/home' },
      { name: 'etc', type: 'directory', size: 0, modified: new Date(), path: currentPath.value + '/etc' },
      { name: 'var', type: 'directory', size: 0, modified: new Date(), path: currentPath.value + '/var' },
      { name: 'readme.txt', type: 'file', size: 1024, modified: new Date(), path: currentPath.value + '/readme.txt' },
      { name: 'config.json', type: 'file', size: 2048, modified: new Date(), path: currentPath.value + '/config.json' }
    ]

    // 实际调用应该是：
    // const result = await invoke('list_remote_directory', {
    //   serverId: props.server.id,
    //   path: currentPath.value
    // })
    // files.value = result.files || []

    files.value = mockFiles
  } catch (err) {
    error.value = err.message || '获取文件列表失败'
    console.error('加载文件列表失败:', err)
  } finally {
    loading.value = false
  }
}

function getParentPath() {
  const parts = currentPath.value.split('/').filter(p => p)
  if (parts.length > 0) {
    parts.pop()
    return '/' + parts.join('/')
  }
  return '/'
}

function getFileIcon(file) {
  if (file.type === 'directory') {
    return '📁'
  }
  const ext = file.name.split('.').pop()?.toLowerCase()
  const iconMap = {
    'txt': '📄',
    'json': '📋',
    'js': '📜',
    'py': '🐍',
    'jpg': '🖼️',
    'png': '🖼️',
    'zip': '📦',
    'tar': '📦'
  }
  return iconMap[ext] || '📄'
}

function formatSize(size) {
  if (size === 0) return '-'
  const units = ['B', 'KB', 'MB', 'GB']
  let unitIndex = 0
  let fileSize = size
  while (fileSize >= 1024 && unitIndex < units.length - 1) {
    fileSize /= 1024
    unitIndex++
  }
  return `${fileSize.toFixed(2)} ${units[unitIndex]}`
}

function formatDate(date) {
  if (!date) return '-'
  return new Date(date).toLocaleString('zh-CN')
}

function handleFileClick(file, event) {
  if (file.name === '..' || file.editing) return
  
  // Ctrl/Cmd + 点击多选
  if (event.ctrlKey || event.metaKey) {
    toggleFileSelection(file)
  } else if (event.shiftKey) {
    // Shift + 点击范围选择
    handleRangeSelection(file)
  } else {
    // 普通点击：单选或切换选择
    if (!isFileSelected(file)) {
      selectedFiles.value = []
      toggleFileSelection(file)
    }
  }
}

function toggleFileSelection(file) {
  const filePath = file.path || (currentPath.value.endsWith('/') ? currentPath.value + file.name : currentPath.value + '/' + file.name)
  const index = selectedFiles.value.indexOf(filePath)
  if (index > -1) {
    selectedFiles.value.splice(index, 1)
  } else {
    selectedFiles.value.push(filePath)
  }
}

function handleRangeSelection(file) {
  const currentIndex = filteredFiles.value.findIndex(f => 
    (f.path || (currentPath.value.endsWith('/') ? currentPath.value + f.name : currentPath.value + '/' + f.name)) === 
    (file.path || (currentPath.value.endsWith('/') ? currentPath.value + file.name : currentPath.value + '/' + file.name))
  )
  
  if (currentIndex === -1) return
  
  // 找到最后一个选中的文件
  let lastSelectedIndex = -1
  for (let i = filteredFiles.value.length - 1; i >= 0; i--) {
    if (isFileSelected(filteredFiles.value[i])) {
      lastSelectedIndex = i
      break
    }
  }
  
  if (lastSelectedIndex === -1) {
    toggleFileSelection(file)
    return
  }
  
  const start = Math.min(lastSelectedIndex, currentIndex)
  const end = Math.max(lastSelectedIndex, currentIndex)
  
  for (let i = start; i <= end; i++) {
    const f = filteredFiles.value[i]
    if (f.name !== '..' && !isFileSelected(f)) {
      toggleFileSelection(f)
    }
  }
}

function handleSelectAll() {
  if (selectedFiles.value.length === filteredFiles.value.filter(f => f.name !== '..').length) {
    selectedFiles.value = []
  } else {
    selectedFiles.value = filteredFiles.value
      .filter(f => f.name !== '..')
      .map(f => f.path || (currentPath.value.endsWith('/') ? currentPath.value + f.name : currentPath.value + '/' + f.name))
  }
}

function isFileSelected(file) {
  const filePath = file.path || (currentPath.value.endsWith('/') ? currentPath.value + file.name : currentPath.value + '/' + file.name)
  return selectedFiles.value.includes(filePath)
}

function handleFileDoubleClick(file) {
  if (file.type === 'directory') {
    if (file.name === '..') {
      handleGoUp()
    } else {
      // 使用文件的完整路径
      currentPath.value = file.path || (currentPath.value.endsWith('/') ? currentPath.value + file.name : currentPath.value + '/' + file.name)
      // loadFiles 会在 watch currentPath 时自动调用
    }
  } else {
    // 预览文件
    previewFile.value = file
  }
}

// 右键菜单
function handleContextMenu(file, event) {
  if (file.name === '..') return
  contextMenu.value = {
    show: true,
    x: event.clientX,
    y: event.clientY,
    file: file
  }
  
  // 点击其他地方关闭菜单
  setTimeout(() => {
    document.addEventListener('click', closeContextMenu, { once: true })
  }, 0)
}

function closeContextMenu() {
  contextMenu.value.show = false
}

function handleContextRename() {
  if (contextMenu.value.file) {
    handleRenameFile(contextMenu.value.file)
  }
  closeContextMenu()
}

function handleContextChmod() {
  if (contextMenu.value.file) {
    handleChmodFile(contextMenu.value.file)
  }
  closeContextMenu()
}

function handleContextPreview() {
  if (contextMenu.value.file) {
    previewFile.value = contextMenu.value.file
  }
  closeContextMenu()
}

function handleContextCompare() {
  if (contextMenu.value.file) {
    compareFiles.value.left = contextMenu.value.file
    // 提示选择第二个文件
    info('请选择要对比的第二个文件')
  }
  closeContextMenu()
}

function handleContextDownload() {
  if (contextMenu.value.file) {
    selectedFiles.value = [contextMenu.value.file.path || (currentPath.value.endsWith('/') ? currentPath.value + contextMenu.value.file.name : currentPath.value + '/' + contextMenu.value.file.name)]
    handleDownload()
  }
  closeContextMenu()
}

function handleContextDelete() {
  if (contextMenu.value.file) {
    selectedFiles.value = [contextMenu.value.file.path || (currentPath.value.endsWith('/') ? currentPath.value + contextMenu.value.file.name : currentPath.value + '/' + contextMenu.value.file.name)]
    handleDelete()
  }
  closeContextMenu()
}

// 重命名
function handleRename() {
  if (selectedFiles.value.length === 1) {
    const file = files.value.find(f => {
      const filePath = f.path || (currentPath.value.endsWith('/') ? currentPath.value + f.name : currentPath.value + '/' + f.name)
      return filePath === selectedFiles.value[0]
    })
    if (file) {
      handleRenameFile(file)
    }
  } else {
    warning('请选择一个文件进行重命名')
  }
}

function handleRenameFile(file) {
  file.editing = true
  file.newName = file.name
  nextTick(() => {
    const input = renameInput.value
    if (input && input.length > 0) {
      input[0].focus()
      input[0].select()
    }
  })
}

async function confirmRename(file) {
  if (!file.newName || file.newName.trim() === '') {
    cancelRename(file)
    return
  }
  
  if (file.newName === file.name) {
    cancelRename(file)
    return
  }
  
  try {
    loading.value = true
    const oldPath = file.path || (currentPath.value.endsWith('/') ? currentPath.value + file.name : currentPath.value + '/' + file.name)
    const newPath = currentPath.value.endsWith('/') 
      ? currentPath.value + file.newName.trim()
      : currentPath.value + '/' + file.newName.trim()
    
    // TODO: 调用 Tauri 重命名文件
    // await invoke('rename_remote_file', {
    //   serverId: props.server.id,
    //   oldPath: oldPath,
    //   newPath: newPath
    // })
    
    console.log('重命名文件:', oldPath, '->', newPath)
    info(`准备重命名: ${oldPath} -> ${newPath}\n（将调用 Tauri 实现）`)
    
    file.name = file.newName.trim()
    success('文件重命名成功')
    file.path = newPath
    file.editing = false
    await loadFiles()
  } catch (err) {
    error.value = err.message || '重命名失败'
    showError('重命名失败: ' + error.value)
    cancelRename(file)
  } finally {
    loading.value = false
  }
}

function cancelRename(file) {
  file.editing = false
  file.newName = ''
}

// 权限设置
function handleChmod() {
  if (selectedFiles.value.length === 1) {
    const file = files.value.find(f => {
      const filePath = f.path || (currentPath.value.endsWith('/') ? currentPath.value + f.name : currentPath.value + '/' + f.name)
      return filePath === selectedFiles.value[0]
    })
    if (file) {
      handleChmodFile(file)
    }
  } else {
    warning('请选择一个文件进行权限设置')
  }
}

function handleChmodFile(file) {
  chmodFile.value = file
  chmodValue.value = file.permissions ? file.permissions.replace(/[^0-7]/g, '') : '644'
  
  // 解析权限位
  if (chmodValue.value.length === 3) {
    const owner = parseInt(chmodValue.value[0])
    const group = parseInt(chmodValue.value[1])
    const others = parseInt(chmodValue.value[2])
    
    chmodBits.value.owner.read = (owner & 4) !== 0
    chmodBits.value.owner.write = (owner & 2) !== 0
    chmodBits.value.owner.execute = (owner & 1) !== 0
    
    chmodBits.value.group.read = (group & 4) !== 0
    chmodBits.value.group.write = (group & 2) !== 0
    chmodBits.value.group.execute = (group & 1) !== 0
    
    chmodBits.value.others.read = (others & 4) !== 0
    chmodBits.value.others.write = (others & 2) !== 0
    chmodBits.value.others.execute = (others & 1) !== 0
  }
  
  showChmodDialog.value = true
}

// 监听权限位变化，更新八进制值
watch(chmodBits, (newBits) => {
  const owner = (newBits.owner.read ? 4 : 0) + (newBits.owner.write ? 2 : 0) + (newBits.owner.execute ? 1 : 0)
  const group = (newBits.group.read ? 4 : 0) + (newBits.group.write ? 2 : 0) + (newBits.group.execute ? 1 : 0)
  const others = (newBits.others.read ? 4 : 0) + (newBits.others.write ? 2 : 0) + (newBits.others.execute ? 1 : 0)
  chmodValue.value = `${owner}${group}${others}`
}, { deep: true })

// 监听八进制值变化，更新权限位
watch(chmodValue, (newValue) => {
  if (newValue.length === 3 && /^[0-7]{3}$/.test(newValue)) {
    const owner = parseInt(newValue[0])
    const group = parseInt(newValue[1])
    const others = parseInt(newValue[2])
    
    chmodBits.value.owner.read = (owner & 4) !== 0
    chmodBits.value.owner.write = (owner & 2) !== 0
    chmodBits.value.owner.execute = (owner & 1) !== 0
    
    chmodBits.value.group.read = (group & 4) !== 0
    chmodBits.value.group.write = (group & 2) !== 0
    chmodBits.value.group.execute = (group & 1) !== 0
    
    chmodBits.value.others.read = (others & 4) !== 0
    chmodBits.value.others.write = (others & 2) !== 0
    chmodBits.value.others.execute = (others & 1) !== 0
  }
})

async function confirmChmod() {
  if (!chmodFile.value || !/^[0-7]{3}$/.test(chmodValue.value)) {
    showError('请输入有效的权限值（三位八进制数，如：755）')
    return
  }
  
  try {
    loading.value = true
    error.value = null
    const filePath = chmodFile.value.path || (currentPath.value.endsWith('/') ? currentPath.value + chmodFile.value.name : currentPath.value + '/' + chmodFile.value.name)
    
    // TODO: 调用 Tauri 设置文件权限
    // await invoke('set_file_permissions', {
    //   serverId: props.server.id,
    //   filePath: filePath,
    //   permissions: parseInt(chmodValue.value, 8)
    // })
    
    console.log('设置文件权限:', filePath, '->', chmodValue.value)
    info(`准备设置权限: ${filePath} -> ${chmodValue.value}\n（将调用 Tauri 实现）`)
    
    chmodFile.value.permissions = chmodValue.value
    success('权限设置成功')
    showChmodDialog.value = false
    await loadFiles()
  } catch (err) {
    error.value = err.message || '设置权限失败'
    showError('设置权限失败: ' + error.value)
  } finally {
    loading.value = false
  }
}

// 搜索
function handleSearch() {
  // 搜索逻辑已在 computed 中处理
}

// 文件对比
function handleCompareFromPreview(file) {
  compareFiles.value.left = previewFile.value
  compareFiles.value.right = file
  previewFile.value = null
}

function handleDownloadFromPreview(file) {
  selectedFiles.value = [file.path || (currentPath.value.endsWith('/') ? currentPath.value + file.name : currentPath.value + '/' + file.name)]
  handleDownload()
  previewFile.value = null
}

function handleNavigate() {
  // loadFiles 会在 watch currentPath 时自动调用
}

function handleGoHome() {
  currentPath.value = '/'
  // loadFiles 会在 watch currentPath 时自动调用
}

function handleGoUp() {
  const parts = currentPath.value.split('/').filter(p => p)
  if (parts.length > 0) {
    parts.pop()
    currentPath.value = '/' + parts.join('/')
  } else {
    currentPath.value = '/'
  }
  // loadFiles 会在 watch currentPath 时自动调用
}

function handleRefresh() {
  loadFiles()
}

async function uploadFiles(filePaths) {
  if (!props.server.connected) {
    showError('请先连接服务器')
    return
  }

  if (!filePaths || filePaths.length === 0) {
    return
  }

  loading.value = true
  error.value = null

  try {
    // TODO: 调用 Tauri 上传文件
    // 接口: invoke('upload_files', { 
    //   serverId: props.server.id, 
    //   localPaths: filePaths,
    //   remotePath: currentPath.value 
    // })
    
    // 实际调用应该是：
    // for (const localPath of filePaths) {
    //   await invoke('upload_file', {
    //     serverId: props.server.id,
    //     localPath: localPath,
    //     remotePath: currentPath.value
    //   })
    // }

    // 临时提示，实际应该调用 Tauri
    console.log('上传文件:', filePaths, '到:', currentPath.value)
    info(`准备上传 ${filePaths.length} 个文件到 ${currentPath.value}\n（将调用 Tauri 实现）`)

    // 上传成功后刷新文件列表
    success(`成功上传 ${filePaths.length} 个文件`)
    await loadFiles()
  } catch (err) {
    error.value = err.message || '文件上传失败'
    showError('上传失败: ' + error.value)
    console.error('文件上传失败:', err)
  } finally {
    loading.value = false
  }
}

async function handleUpload() {
  try {
    // 使用 Tauri 文件选择对话框
    const selected = await open({
      multiple: true,
      title: '选择要上传的文件'
    })

    if (!selected) {
      return // 用户取消选择
    }

    const filePaths = Array.isArray(selected) ? selected : [selected]
    await uploadFiles(filePaths)
  } catch (err) {
    console.error('选择文件失败:', err)
  }
}

async function handleDrop(event) {
  isDragOver.value = false
  
  if (!props.server.connected) {
    alert('请先连接服务器')
    return
  }

  // 从拖拽事件中获取文件路径
  // 注意：在浏览器环境中，需要使用 DataTransfer API
  // 但在 Tauri 中，可能需要使用不同的方式
  // 这里先实现基本逻辑，实际需要根据 Tauri 的拖拽 API 调整
  
  const files = event.dataTransfer?.files
  if (!files || files.length === 0) {
    return
  }

  // TODO: 在 Tauri 中，需要将 File 对象转换为路径
  // 可能需要使用 Tauri 的文件系统 API 或拖拽 API
  // 这里先提示用户使用上传按钮
  info('拖拽上传功能需要 Tauri 支持\n请使用"上传"按钮选择文件')
  
  // 实际实现应该是：
  // const filePaths = []
  // for (const file of files) {
  //   // 获取文件路径（需要 Tauri API）
  //   const path = await getFilePath(file)
  //   filePaths.push(path)
  // }
  // await uploadFiles(filePaths)
}

async function handleDownload() {
  if (selectedFiles.value.length === 0) {
    warning('请先选择要下载的文件')
    return
  }
  
  if (!props.server.connected) {
    showError('请先连接服务器')
    return
  }

  try {
    let savePath

    if (selectedFiles.value.length === 1) {
      // 单个文件：使用文件保存对话框
      const fileName = selectedFiles.value[0].split('/').pop()
      savePath = await saveFile({
        title: '选择保存位置',
        defaultPath: fileName
      })
    } else {
      // 多个文件：使用文件夹选择对话框
      savePath = await open({
        directory: true,
        multiple: false,
        title: '选择保存文件夹'
      })
    }

    if (!savePath) {
      return // 用户取消选择
    }

    loading.value = true
    error.value = null

    // TODO: 调用 Tauri 下载文件
    // 接口: invoke('download_files', {
    //   serverId: props.server.id,
    //   remotePaths: selectedFiles.value,
    //   localPath: savePath
    // })

    // 实际调用应该是：
    // if (selectedFiles.value.length === 1) {
    //   // 单个文件直接保存
    //   await invoke('download_file', {
    //     serverId: props.server.id,
    //     remotePath: selectedFiles.value[0],
    //     localPath: savePath
    //   })
    // } else {
    //   // 多个文件保存到文件夹
    //   for (const remotePath of selectedFiles.value) {
    //     const fileName = remotePath.split('/').pop()
    //     const localFilePath = savePath + '/' + fileName
    //     await invoke('download_file', {
    //       serverId: props.server.id,
    //       remotePath: remotePath,
    //       localPath: localFilePath
    //     })
    //   }
    // }

    // 临时提示，实际应该调用 Tauri
    console.log('下载文件:', selectedFiles.value, '到:', savePath)
    info(`准备下载 ${selectedFiles.value.length} 个文件到 ${savePath}\n（将调用 Tauri 实现）`)

    // 下载成功后清空选择
    success(`成功下载 ${selectedFiles.value.length} 个文件`)
    selectedFiles.value = []
  } catch (err) {
    error.value = err.message || '文件下载失败'
    showError('下载失败: ' + error.value)
    console.error('文件下载失败:', err)
  } finally {
    loading.value = false
  }
}

async function handleNewFolder() {
  if (!props.server.connected) {
    showError('请先连接服务器')
    return
  }

  // 使用简单的输入对话框（可以后续改为自定义对话框）
  const name = prompt('请输入文件夹名称:')
  if (!name || !name.trim()) {
    return
  }

  try {
    loading.value = true
    error.value = null

    const folderPath = currentPath.value.endsWith('/') 
      ? currentPath.value + name.trim()
      : currentPath.value + '/' + name.trim()

    // TODO: 调用 Tauri 创建文件夹
    // 接口: invoke('create_directory', {
    //   serverId: props.server.id,
    //   path: folderPath
    // })

    // 实际调用应该是：
    // await invoke('create_directory', {
    //   serverId: props.server.id,
    //   path: folderPath
    // })

    // 临时提示，实际应该调用 Tauri
    console.log('创建文件夹:', folderPath)
    info(`准备创建文件夹: ${folderPath}\n（将调用 Tauri 实现）`)

    // 创建成功后刷新文件列表
    success('文件夹创建成功')
    await loadFiles()
  } catch (err) {
    error.value = err.message || '创建文件夹失败'
    showError('创建失败: ' + error.value)
    console.error('创建文件夹失败:', err)
  } finally {
    loading.value = false
  }
}

async function handleDelete() {
  if (selectedFiles.value.length === 0) {
    warning('请先选择要删除的文件')
    return
  }
  
  if (!props.server.connected) {
    showError('请先连接服务器')
    return
  }
  
  // 显示确认对话框
  pendingDeleteFiles.value = [...selectedFiles.value]
  deleteConfirmMessage.value = `确定要删除选中的 ${selectedFiles.value.length} 个文件/文件夹吗？\n此操作不可恢复！`
  showDeleteConfirm.value = true
}

function confirmDelete() {
  if (pendingDeleteFiles.value.length === 0) return
  
  performDelete()
}

async function performDelete() {

  try {
    loading.value = true
    error.value = null

    // TODO: 调用 Tauri 删除文件
    // 接口: invoke('delete_files', {
    //   serverId: props.server.id,
    //   paths: selectedFiles.value
    // })

    // 实际调用应该是：
    // await invoke('delete_files', {
    //   serverId: props.server.id,
    //   paths: selectedFiles.value
    // })

    // 临时提示，实际应该调用 Tauri
    console.log('删除文件:', pendingDeleteFiles.value)
    info(`准备删除 ${pendingDeleteFiles.value.length} 个文件\n（将调用 Tauri 实现）`)

    // 删除成功后刷新文件列表
    success(`成功删除 ${pendingDeleteFiles.value.length} 个文件`)
    selectedFiles.value = []
    await loadFiles()
  } catch (err) {
    error.value = err.message || '删除文件失败'
    showError('删除失败: ' + error.value)
    pendingDeleteFiles.value = []
    console.error('删除文件失败:', err)
  } finally {
    loading.value = false
  }
}
</script>

<style scoped>
.file-manager-tab {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: var(--bg-primary);
  overflow: hidden;
}

.file-manager-header {
  padding: 8px 12px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  flex-shrink: 0;
}

.path-bar {
  display: flex;
  align-items: center;
  gap: 8px;
  margin-bottom: 8px;
}

.path-btn {
  width: 32px;
  height: 28px;
  padding: 0;
  font-size: 14px;
  display: flex;
  align-items: center;
  justify-content: center;
}

.path-input {
  flex: 1;
  font-size: 12px;
  font-family: 'Consolas', monospace;
}

.file-actions {
  display: flex;
  gap: 8px;
}

.action-btn {
  font-size: 12px;
  padding: 4px 12px;
  display: flex;
  align-items: center;
  gap: 4px;
}

.action-btn.delete:hover {
  background: var(--error-color);
  color: white;
}

.file-list-container {
  flex: 1;
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.file-list-header {
  display: flex;
  padding: 8px 12px;
  background: var(--bg-secondary);
  border-bottom: 1px solid var(--border-color);
  font-size: 12px;
  font-weight: 600;
  color: var(--text-secondary);
}

.file-col {
  padding: 0 8px;
}

.file-col.name {
  flex: 1;
  min-width: 200px;
}

.file-col.size {
  width: 100px;
  text-align: right;
}

.file-col.type {
  width: 80px;
}

.file-col.permissions {
  width: 80px;
  font-family: 'Consolas', monospace;
  font-size: 11px;
}

.file-col.date {
  width: 180px;
}

.search-input {
  flex: 1;
  padding: 4px 8px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 3px;
  color: var(--text-primary);
  font-size: 12px;
  margin-right: 8px;
}

.rename-input {
  flex: 1;
  padding: 2px 4px;
  background: var(--bg-primary);
  border: 1px solid var(--accent-color);
  border-radius: 2px;
  color: var(--text-primary);
  font-size: 13px;
  font-family: inherit;
}

.context-menu {
  position: fixed;
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 4px;
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.3);
  z-index: 1000;
  min-width: 150px;
  padding: 4px 0;
}

.context-menu-item {
  padding: 8px 16px;
  cursor: pointer;
  font-size: 13px;
  color: var(--text-primary);
  transition: background 0.2s;
}

.context-menu-item:hover {
  background: var(--bg-hover);
}

.context-menu-divider {
  height: 1px;
  background: var(--border-color);
  margin: 4px 0;
}

.chmod-dialog-overlay {
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

.chmod-dialog {
  background: var(--bg-secondary);
  border: 1px solid var(--border-color);
  border-radius: 6px;
  width: 500px;
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
  margin: 0;
  font-size: 16px;
}

.dialog-body {
  padding: 16px;
}

.chmod-info {
  margin-bottom: 16px;
  padding: 12px;
  background: var(--bg-primary);
  border-radius: 4px;
}

.chmod-info p {
  margin: 4px 0;
  font-size: 13px;
}

.chmod-input {
  margin-bottom: 16px;
}

.chmod-input label {
  display: block;
  font-size: 12px;
  color: var(--text-secondary);
  margin-bottom: 6px;
}

.chmod-input input {
  width: 100%;
  padding: 6px 8px;
  background: var(--bg-primary);
  border: 1px solid var(--border-color);
  border-radius: 3px;
  color: var(--text-primary);
  font-size: 13px;
  font-family: 'Consolas', monospace;
}

.chmod-bits {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.chmod-group {
  padding: 12px;
  background: var(--bg-primary);
  border-radius: 4px;
}

.chmod-group label {
  display: block;
  font-size: 12px;
  font-weight: 600;
  color: var(--text-primary);
  margin-bottom: 8px;
}

.chmod-checkboxes {
  display: flex;
  gap: 16px;
}

.chmod-checkboxes label {
  display: flex;
  align-items: center;
  gap: 4px;
  font-weight: normal;
  font-size: 12px;
  cursor: pointer;
}

.chmod-checkboxes input[type="checkbox"] {
  width: auto;
  margin: 0;
}

.dialog-footer {
  padding: 16px;
  border-top: 1px solid var(--border-color);
  display: flex;
  justify-content: flex-end;
  gap: 8px;
}

.dialog-footer button {
  padding: 6px 16px;
  background: transparent;
  border: 1px solid var(--border-color);
  border-radius: 3px;
  color: var(--text-primary);
  cursor: pointer;
}

.dialog-footer button.primary {
  background: var(--accent-color);
  color: white;
  border-color: var(--accent-color);
}

.file-list {
  flex: 1;
  overflow-y: auto;
}

.file-item {
  display: flex;
  padding: 8px 12px;
  border-bottom: 1px solid var(--border-color);
  cursor: pointer;
  transition: background 0.2s;
  font-size: 13px;
}

.file-item:hover {
  background: var(--bg-hover);
}

.file-item.selected {
  background: var(--bg-active);
}

.file-item .file-col.name {
  display: flex;
  align-items: center;
  gap: 8px;
}

.file-icon {
  font-size: 16px;
  width: 20px;
  text-align: center;
}

.file-name {
  flex: 1;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
}

.empty-files {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  height: 100%;
  color: var(--text-secondary);
  gap: 12px;
}

.empty-files.loading {
  gap: 12px;
}

.loading-spinner {
  width: 32px;
  height: 32px;
  border: 3px solid var(--bg-tertiary);
  border-top-color: var(--accent-color);
  border-radius: 50%;
  animation: spin 0.8s linear infinite;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.empty-files.error {
  color: var(--error-color);
}

.upload-zone {
  border: 2px dashed var(--border-color);
  border-radius: 8px;
  margin: 20px;
  transition: all 0.3s;
  cursor: pointer;
}

.upload-zone:hover {
  border-color: var(--accent-color);
  background: var(--bg-hover);
}

.upload-zone.drag-over {
  border-color: var(--accent-color);
  background: var(--bg-active);
  border-style: solid;
}

.upload-content {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 12px;
  padding: 40px;
}

.upload-icon {
  font-size: 48px;
  margin-bottom: 8px;
}

.upload-text {
  font-size: 16px;
  color: var(--text-primary);
  margin: 0;
}

.upload-hint {
  font-size: 12px;
  color: var(--text-secondary);
  margin: 0;
}

.upload-btn {
  padding: 8px 20px;
  background: var(--accent-color);
  color: white;
  border-radius: 4px;
  font-size: 13px;
  margin-top: 8px;
  cursor: pointer;
  transition: background 0.2s;
}

.upload-btn:hover {
  background: var(--accent-hover);
}

.retry-btn {
  padding: 6px 16px;
  background: var(--accent-color);
  color: white;
  border-radius: 4px;
  font-size: 12px;
}

.retry-btn:hover {
  background: var(--accent-hover);
}
</style>

