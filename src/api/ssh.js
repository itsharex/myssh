/**
 * SSH 连接相关 API
 */

import { invoke } from '@tauri-apps/api/tauri'

/**
 * 连接 SSH 服务器
 * @param {Object} params - 连接参数
 * @param {string} params.serverId - 服务器ID
 * @param {string} params.host - 主机地址
 * @param {number} params.port - 端口
 * @param {string} params.username - 用户名
 * @param {string} [params.password] - 密码（可选）
 * @param {string} [params.keyPath] - 密钥路径（可选）
 * @returns {Promise<{success: boolean, connectionId: string}>}
 */
export async function connectSshServer(params) {
  try {
    const result = await invoke('connect_ssh_server', {
      params: {
        server_id: params.serverId,
        host: params.host,
        port: params.port,
        username: params.username,
        password: params.password || null,
        key_path: params.keyPath || null
      }
    })
    return result
  } catch (error) {
    console.error('连接SSH服务器失败:', error)
    // Tauri 错误可能是字符串或对象
    let errorMessage = '连接服务器失败'
    if (typeof error === 'string') {
      errorMessage = error
    } else if (error?.message) {
      errorMessage = error.message
    } else if (error?.toString) {
      errorMessage = error.toString()
    }
    throw new Error(errorMessage)
  }
}

/**
 * 断开 SSH 服务器连接
 * @param {string} serverId - 服务器ID
 * @returns {Promise<{success: boolean}>}
 */
export async function disconnectSshServer(serverId) {
  try {
    const result = await invoke('disconnect_ssh_server', {
      params: {
        server_id: serverId
      }
    })
    return result
  } catch (error) {
    console.error('断开SSH服务器连接失败:', error)
    let errorMessage = '断开连接失败'
    if (typeof error === 'string') {
      errorMessage = error
    } else if (error?.message) {
      errorMessage = error.message
    } else if (error?.toString) {
      errorMessage = error.toString()
    }
    throw new Error(errorMessage)
  }
}

/**
 * 执行 SSH 命令
 * @param {Object} params - 命令参数
 * @param {string} params.serverId - 服务器ID
 * @param {string} params.command - 要执行的命令
 * @param {string} [params.currentDir] - 当前工作目录（可选）
 * @returns {Promise<{output: string, exitCode: number, isInteractive: boolean, interactiveMessage?: string, newDir?: string, outputLines: string[]}>}
 */
export async function executeSshCommand(params) {
  try {
    const result = await invoke('execute_ssh_command', {
      params: {
        server_id: params.serverId,
        command: params.command,
        current_dir: params.currentDir || null
      }
    })
    return result
  } catch (error) {
    console.error('执行SSH命令失败:', error)
    let errorMessage = '执行命令失败'
    if (typeof error === 'string') {
      errorMessage = error
    } else if (error?.message) {
      errorMessage = error.message
    } else if (error?.toString) {
      errorMessage = error.toString()
    }
    throw new Error(errorMessage)
  }
}

/**
 * 重连终端
 * @param {string} serverId - 服务器ID
 * @returns {Promise<{success: boolean}>}
 */
export async function reconnectTerminal(serverId) {
  try {
    const result = await invoke('reconnect_terminal', {
      params: {
        server_id: serverId
      }
    })
    return result
  } catch (error) {
    console.error('重连终端失败:', error)
    let errorMessage = '重连失败'
    if (typeof error === 'string') {
      errorMessage = error
    } else if (error?.message) {
      errorMessage = error.message
    } else if (error?.toString) {
      errorMessage = error.toString()
    }
    throw new Error(errorMessage)
  }
}

/**
 * 命令补全
 * @param {Object} params - 补全参数
 * @param {string} params.serverId - 服务器ID
 * @param {string} params.input - 完整的输入字符串
 * @param {string} params.currentDir - 当前工作目录
 * @returns {Promise<{completedInput?: string, matches: string[], shouldShowMatches: boolean}>}
 */
export async function completeCommand(params) {
  try {
    const result = await invoke('complete_command', {
      params: {
        server_id: params.serverId,
        input: params.input,
        current_dir: params.currentDir
      }
    })
    return result
  } catch (error) {
    console.error('命令补全失败:', error)
    let errorMessage = '补全失败'
    if (typeof error === 'string') {
      errorMessage = error
    } else if (error?.message) {
      errorMessage = error.message
    } else if (error?.toString) {
      errorMessage = error.toString()
    }
    throw new Error(errorMessage)
  }
}

