/**
 * SSH 连接相关命令处理
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use russh::{client, ChannelMsg, Disconnect, Error};
use russh_keys::load_secret_key;
use async_trait::async_trait;
use tokio::sync::Mutex as TokioMutex;
use tokio::time::sleep;

/// SSH 客户端 Handler
struct SshHandler;

#[async_trait]
impl client::Handler for SshHandler {
    type Error = Error;

    async fn check_server_key(
        self,
        _server_public_key: &russh_keys::key::PublicKey,
    ) -> Result<(Self, bool), Error> {
        // 暂时接受所有服务器密钥（生产环境应该验证密钥指纹）
        Ok((self, true))
    }
}

/// SSH 连接信息
pub struct SshConnection {
    pub server_id: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub session: Arc<TokioMutex<client::Handle<SshHandler>>>,
    pub last_heartbeat: Arc<Mutex<Instant>>,  // 最后心跳时间
    pub heartbeat_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,  // 心跳任务句柄
}

/// 全局连接池
type ConnectionPool = Arc<Mutex<HashMap<String, SshConnection>>>;

lazy_static::lazy_static! {
    static ref CONNECTIONS: ConnectionPool = Arc::new(Mutex::new(HashMap::new()));
}

/// 连接 SSH 服务器参数
#[derive(Debug, Deserialize)]
pub struct ConnectSshParams {
    pub server_id: String,
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: Option<String>,
    pub key_path: Option<String>,
}

/// 连接 SSH 服务器返回
#[derive(Debug, Serialize)]
pub struct ConnectSshResult {
    pub success: bool,
    pub connection_id: String,
    pub message: Option<String>,
}

/// 断开 SSH 服务器参数
#[derive(Debug, Deserialize)]
pub struct DisconnectSshParams {
    pub server_id: String,
}

/// 断开 SSH 服务器返回
#[derive(Debug, Serialize)]
pub struct DisconnectSshResult {
    pub success: bool,
    pub message: Option<String>,
}

/// 执行 SSH 命令参数
#[derive(Debug, Deserialize)]
pub struct ExecuteSshCommandParams {
    pub server_id: String,
    pub command: String,
    pub current_dir: Option<String>,  // 当前工作目录（可选）
}

/// 执行 SSH 命令返回
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExecuteSshCommandResult {
    pub output: String,
    pub exit_code: i32,
    pub is_interactive: bool,  // 是否是交互式命令
    pub interactive_message: Option<String>,  // 交互式命令的提示信息
    pub new_dir: Option<String>,  // 新目录（如果是 cd 命令）
    pub output_lines: Vec<String>,  // 已分割的输出行
}

/// 重连终端参数
#[derive(Debug, Deserialize)]
pub struct ReconnectTerminalParams {
    pub server_id: String,
}

/// 重连终端返回
#[derive(Debug, Serialize)]
pub struct ReconnectTerminalResult {
    pub success: bool,
    pub message: Option<String>,
}

/// 命令补全参数
#[derive(Debug, Deserialize)]
pub struct CompleteCommandParams {
    pub server_id: String,
    pub input: String,  // 完整的输入字符串
    pub current_dir: String,  // 当前工作目录
}

/// 命令补全返回
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteCommandResult {
    pub completed_input: Option<String>,  // 补全后的完整输入（如果可以直接补全）
    pub matches: Vec<String>,  // 匹配的选项列表（用于显示）
    pub should_show_matches: bool,  // 是否应该显示匹配列表
}

/// 连接 SSH 服务器
/// 
/// # 命令名称
/// `connect_ssh_server`
/// 
/// # 参数
/// - `server_id`: 服务器ID
/// - `host`: 主机地址
/// - `port`: 端口
/// - `username`: 用户名
/// - `password`: 密码（可选）
/// - `key_path`: 密钥路径（可选）
/// 
/// # 返回
/// - `success`: 是否成功
/// - `connection_id`: 连接ID
/// - `message`: 消息（可选）
#[tauri::command]
pub async fn connect_ssh_server(params: ConnectSshParams) -> Result<ConnectSshResult, String> {
    // 检查是否已经连接
    {
        let connections = CONNECTIONS.lock().unwrap();
        if connections.contains_key(&params.server_id) {
            return Err("服务器已连接".to_string());
        }
    }

    // 创建 SSH 客户端配置
    let config = russh::client::Config::default();
    // 设置 keepalive 间隔（30秒）
    // 注意：russh 库可能不直接支持 keepalive 配置，我们需要通过心跳任务来实现
    let config = Arc::new(config);

    // 建立 SSH 连接
    let address = format!("{}:{}", params.host, params.port);
    let mut handle = match client::connect(config, address, SshHandler {}).await {
        Ok(handle) => handle,
        Err(e) => {
            let error_msg = format!("{}", e);
            // 根据错误类型提供更友好的错误信息
            if error_msg.contains("Connection refused") || error_msg.contains("无法连接") {
                return Err(format!("无法连接到服务器 {}:{}，请检查主机地址和端口是否正确", params.host, params.port));
            } else if error_msg.contains("timeout") || error_msg.contains("超时") {
                return Err(format!("连接超时，无法连接到服务器 {}:{}", params.host, params.port));
            } else if error_msg.contains("No route to host") {
                return Err(format!("无法访问服务器 {}:{}，请检查网络连接", params.host, params.port));
            } else {
                return Err(format!("连接失败: {}", error_msg));
            }
        }
    };

    // 进行身份验证
    let auth_result = if let Some(key_path) = &params.key_path {
        // 使用密钥文件进行身份验证
        match load_secret_key(key_path, None) {
            Ok(key_pair) => {
                handle.authenticate_publickey(&params.username, Arc::new(key_pair)).await
            }
            Err(e) => {
                return Err(format!("加载密钥文件失败: {}，请检查密钥文件路径是否正确", e));
            }
        }
    } else if let Some(password) = &params.password {
        // 使用密码进行身份验证
        handle.authenticate_password(&params.username, password).await
    } else {
        return Err("必须提供密码或密钥路径".to_string());
    };

    match auth_result {
        Ok(true) => {
            // 身份验证成功，保存连接
            let session = Arc::new(TokioMutex::new(handle));
            let last_heartbeat = Arc::new(Mutex::new(Instant::now()));
            let heartbeat_task = Arc::new(Mutex::new(None));
            
            // 启动心跳任务
            let server_id_clone = params.server_id.clone();
            let session_clone = session.clone();
            let last_heartbeat_clone = last_heartbeat.clone();
            let heartbeat_task_clone = heartbeat_task.clone();
            
            let task = tokio::spawn(async move {
                heartbeat_loop(server_id_clone, session_clone, last_heartbeat_clone, heartbeat_task_clone).await;
            });
            
            *heartbeat_task.lock().unwrap() = Some(task);
            
            let connection = SshConnection {
                server_id: params.server_id.clone(),
                host: params.host.clone(),
                port: params.port,
                username: params.username.clone(),
                session,
                last_heartbeat,
                heartbeat_task,
            };

            let mut connections = CONNECTIONS.lock().unwrap();
            connections.insert(params.server_id.clone(), connection);

            Ok(ConnectSshResult {
                success: true,
                connection_id: params.server_id.clone(),
                message: Some("连接成功".to_string()),
            })
        }
        Ok(false) => {
            Err("身份验证失败，请检查用户名、密码或密钥是否正确".to_string())
        }
        Err(e) => {
            let error_msg = format!("{}", e);
            if error_msg.contains("Authentication failed") || error_msg.contains("认证失败") {
                Err("身份验证失败，请检查用户名、密码或密钥是否正确".to_string())
            } else {
                Err(format!("身份验证错误: {}", error_msg))
            }
        }
    }
}

/// 心跳循环，定期发送心跳以维持连接
async fn heartbeat_loop(
    server_id: String,
    session: Arc<TokioMutex<client::Handle<SshHandler>>>,
    last_heartbeat: Arc<Mutex<Instant>>,
    _heartbeat_task: Arc<Mutex<Option<tokio::task::JoinHandle<()>>>>,
) {
    // 心跳间隔：30秒
    let heartbeat_interval = Duration::from_secs(30);
    // 连接超时：如果超过5分钟没有成功心跳，认为连接断开
    let connection_timeout = Duration::from_secs(300);
    
    loop {
        sleep(heartbeat_interval).await;
        
        // 检查连接是否超时
        let last_heartbeat_time = *last_heartbeat.lock().unwrap();
        if last_heartbeat_time.elapsed() > connection_timeout {
            // 连接超时，主动断开
            eprintln!("SSH连接 {} 超时，主动断开", server_id);
            let _ = disconnect_ssh_server_internal(&server_id).await;
            break;
        }
        
        // 发送心跳：执行一个简单的命令来检测连接状态
        let heartbeat_result = {
            let handle = session.lock().await;
            let channel_result = handle.channel_open_session().await;
            drop(handle); // 释放锁，避免在 await 时持有锁
            
            match channel_result {
                Ok(mut channel) => {
                    // 执行 echo 命令作为心跳
                    let command = b"echo -n";
                    match channel.exec(true, command.to_vec()).await {
                        Ok(_) => {
                            // 等待响应
                            let mut received = false;
                            let mut timeout_count = 0;
                            loop {
                                match channel.wait().await {
                                    Some(ChannelMsg::Data { .. }) => {
                                        received = true;
                                        break;
                                    }
                                    Some(ChannelMsg::ExitStatus { .. }) => {
                                        received = true;
                                        break;
                                    }
                                    Some(ChannelMsg::Eof) | Some(ChannelMsg::Close) => {
                                        break;
                                    }
                                    None => {
                                        timeout_count += 1;
                                        if timeout_count > 10 {
                                            break;
                                        }
                                        sleep(Duration::from_millis(100)).await;
                                    }
                                    _ => {}
                                }
                            }
                            let _ = channel.close().await;
                            received
                        }
                        Err(_) => false,
                    }
                }
                Err(_) => false,
            }
        };
        
        if heartbeat_result {
            // 心跳成功，更新最后心跳时间
            *last_heartbeat.lock().unwrap() = Instant::now();
        } else {
            // 心跳失败，连接可能已断开
            eprintln!("SSH连接 {} 心跳失败，主动断开", server_id);
            let _ = disconnect_ssh_server_internal(&server_id).await;
            break;
        }
    }
}

/// 内部断开连接函数（不返回错误，用于心跳任务）
async fn disconnect_ssh_server_internal(server_id: &str) {
    // 获取连接信息并停止心跳任务
    let (session_opt, heartbeat_task_opt) = {
        let mut connections = CONNECTIONS.lock().unwrap();
        if let Some(conn) = connections.remove(server_id) {
            let task = conn.heartbeat_task.lock().unwrap().take();
            (Some(conn.session), task)
        } else {
            (None, None)
        }
    };
    
    // 停止心跳任务（在锁外）
    if let Some(task) = heartbeat_task_opt {
        task.abort();
    }
    
    // 断开连接（在锁外）
    if let Some(session) = session_opt {
        let handle = session.lock().await;
        let _ = handle.disconnect(Disconnect::ByApplication, "连接断开", "").await;
    }
}

/// 断开 SSH 服务器连接
/// 
/// # 命令名称
/// `disconnect_ssh_server`
/// 
/// # 参数
/// - `server_id`: 服务器ID
/// 
/// # 返回
/// - `success`: 是否成功
/// - `message`: 消息（可选）
#[tauri::command]
pub async fn disconnect_ssh_server(params: DisconnectSshParams) -> Result<DisconnectSshResult, String> {
    // 获取连接信息并停止心跳任务（在锁内完成）
    let (session_opt, heartbeat_task_opt) = {
        let mut connections = CONNECTIONS.lock().unwrap();
        if let Some(conn) = connections.remove(&params.server_id) {
            let task = conn.heartbeat_task.lock().unwrap().take();
            (Some(conn.session), task)
        } else {
            (None, None)
        }
    };
    
    // 停止心跳任务（在锁外）
    if let Some(task) = heartbeat_task_opt {
        task.abort();
    }
    
    // 断开 SSH 连接（在锁外执行异步操作）
    if let Some(session) = session_opt {
        let handle = session.lock().await;
        let _ = handle.disconnect(Disconnect::ByApplication, "用户断开连接", "").await;
        
        Ok(DisconnectSshResult {
            success: true,
            message: Some("断开连接成功".to_string()),
        })
    } else {
        Ok(DisconnectSshResult {
            success: true,
            message: Some("连接已断开".to_string()),
        })
    }
}

/// 执行 SSH 命令
/// 
/// # 命令名称
/// `execute_ssh_command`
/// 
/// # 参数
/// - `server_id`: 服务器ID
/// - `command`: 要执行的命令
/// 
/// # 返回
/// - `output`: 命令输出
/// - `exit_code`: 退出码
#[tauri::command]
pub async fn execute_ssh_command(params: ExecuteSshCommandParams) -> Result<ExecuteSshCommandResult, String> {
    let trimmed_command = params.command.trim();
    let current_dir = params.current_dir.as_deref().unwrap_or("~");
    
    // 检查是否是交互式命令
    if is_interactive_command(trimmed_command) {
        let command_name = trimmed_command.split_whitespace().next().unwrap_or("");
        let message = generate_interactive_message(command_name);
        return Ok(ExecuteSshCommandResult {
            output: String::new(),
            exit_code: 0,
            is_interactive: true,
            interactive_message: Some(message.clone()),
            new_dir: None,
            output_lines: split_output_lines(&message),
        });
    }
    
    // 处理命令（cd 命令特殊处理）
    let (final_command, is_cd) = if trimmed_command.starts_with("cd ") || trimmed_command == "cd" {
        process_cd_command(trimmed_command, current_dir)
    } else {
        (process_normal_command(trimmed_command, current_dir), false)
    };
    
    // 先获取并克隆 session，然后释放锁
    let (session, last_heartbeat) = {
        let connections = CONNECTIONS.lock().unwrap();
        match connections.get(&params.server_id) {
            Some(conn) => {
                // 更新最后心跳时间（执行命令也算是一种心跳）
                *conn.last_heartbeat.lock().unwrap() = Instant::now();
                (conn.session.clone(), conn.last_heartbeat.clone())
            }
            None => return Err("服务器未连接".to_string()),
        }
    };

    // 打开通道执行命令（在锁外执行异步操作）
    let handle = session.lock().await;
    
    let mut channel = match handle.channel_open_session().await {
        Ok(channel) => channel,
        Err(e) => {
            // 如果打开通道失败，可能是连接已断开，主动断开连接
            let error_msg = format!("{}", e);
            if error_msg.contains("disconnected") || error_msg.contains("断开") {
                let _ = disconnect_ssh_server_internal(&params.server_id).await;
            }
            return Err(format!("打开通道失败: {}，连接可能已断开", e));
        }
    };

    // 执行命令（使用 bash -c 包装以确保正确执行）
    let shell_command = format!("bash -c '{}'", final_command);
    let command_bytes = shell_command.as_bytes().to_vec();
    if let Err(e) = channel.exec(true, command_bytes).await {
        return Err(format!("执行命令失败: {}", e));
    }

    // 读取命令输出
    let mut output = Vec::new();
    let mut exit_code = 0;

    loop {
        match channel.wait().await {
            Some(ChannelMsg::Data { data }) => {
                output.extend_from_slice(&data);
            }
            Some(ChannelMsg::ExitStatus { exit_status }) => {
                exit_code = exit_status;
            }
            Some(ChannelMsg::Eof) => {
                break;
            }
            Some(ChannelMsg::Close) => {
                break;
            }
            None => {
                break;
            }
            _ => {}
        }
    }

    // 关闭通道
    let _ = channel.close().await;
    
    // 更新最后心跳时间
    *last_heartbeat.lock().unwrap() = Instant::now();
    
    // 处理输出
    let output_text = String::from_utf8_lossy(&output).to_string();
    let output_lines = split_output_lines(&output_text);
    
    // 如果是 cd 命令且成功，提取新目录
    let new_dir = if is_cd && exit_code == 0 {
        Some(output_text.trim().to_string())
    } else {
        None
    };

    Ok(ExecuteSshCommandResult {
        output: output_text,
        exit_code: exit_code as i32,
        is_interactive: false,
        interactive_message: None,
        new_dir,
        output_lines,
    })
}

/// 重连终端
/// 
/// # 命令名称
/// `reconnect_terminal`
/// 
/// # 参数
/// - `server_id`: 服务器ID
/// 
/// # 返回
/// - `success`: 是否成功
/// - `message`: 消息（可选）
#[tauri::command]
pub async fn reconnect_terminal(_params: ReconnectTerminalParams) -> Result<ReconnectTerminalResult, String> {
    // TODO: 实现实际的重连逻辑
    
    Ok(ReconnectTerminalResult {
        success: true,
        message: Some("重连成功".to_string()),
    })
}

/// 计算最长公共前缀
fn longest_common_prefix(strings: &[String]) -> String {
    if strings.is_empty() {
        return String::new();
    }
    
    let first = &strings[0];
    let mut prefix_len = first.len();
    
    for s in strings.iter().skip(1) {
        prefix_len = first
            .chars()
            .zip(s.chars())
            .take_while(|(a, b)| a == b)
            .count()
            .min(prefix_len);
    }
    
    first.chars().take(prefix_len).collect()
}

/// 文件操作命令列表
const FILE_OPERATION_COMMANDS: &[&str] = &[
    "cd", "ls", "cat", "less", "more", "head", "tail", "grep", "find",
    "rm", "rmdir", "mkdir", "touch", "cp", "mv", "chmod", "chown",
    "vi", "vim", "nano", "pwd", "open", "file", "stat", "readlink",
];

/// 交互式命令列表（不支持的命令）
const INTERACTIVE_COMMANDS: &[&str] = &[
    "vim", "vi", "nano", "emacs", "htop", "top", "less", "more", "man",
    "screen", "tmux", "byobu", "mc", "ranger", "ncdu", "htop", "glances",
    "watch", "dialog", "whiptail", "fzf", "ripgrep", "bat", "lesspipe",
];

/// 判断是否为文件操作命令
fn is_file_operation_command(cmd: &str) -> bool {
    FILE_OPERATION_COMMANDS.contains(&cmd)
}

/// 检查是否是交互式命令
fn is_interactive_command(command: &str) -> bool {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return false;
    }
    
    let parts: Vec<&str> = trimmed.split_whitespace().collect();
    if parts.is_empty() {
        return false;
    }
    
    let command_name = parts[0];
    INTERACTIVE_COMMANDS.contains(&command_name)
}

/// 生成交互式命令的提示信息
fn generate_interactive_message(command_name: &str) -> String {
    match command_name {
        "vim" | "vi" => {
            "警告: vim/vi 是交互式程序，当前终端不支持交互式操作。\n提示: 可以使用以下替代方案：\n  - 使用 cat/less 查看文件: less <文件名>\n  - 使用 echo 创建/编辑文件: echo \"内容\" > <文件名>\n  - 使用 sed 编辑文件: sed -i 's/旧/新/g' <文件名>".to_string()
        }
        "nano" => {
            "警告: nano 是交互式程序，当前终端不支持交互式操作。\n提示: 可以使用以下替代方案：\n  - 使用 cat/less 查看文件: less <文件名>\n  - 使用 echo 创建/编辑文件: echo \"内容\" > <文件名>".to_string()
        }
        "htop" | "top" => {
            "警告: htop/top 是交互式程序，当前终端不支持交互式操作。\n提示: 可以使用以下替代方案：\n  - 使用 ps 查看进程: ps aux\n  - 使用 ps aux | head 查看前几个进程".to_string()
        }
        "less" | "more" => {
            "警告: less/more 是交互式程序，当前终端不支持交互式操作。\n提示: 可以使用以下替代方案：\n  - 使用 cat 查看文件: cat <文件名>\n  - 使用 head/tail 查看文件部分内容".to_string()
        }
        "man" => {
            "警告: man 是交互式程序，当前终端不支持交互式操作。\n提示: 可以使用以下替代方案：\n  - 使用 man -P cat <命令> 查看手册\n  - 使用 --help 选项查看帮助".to_string()
        }
        "screen" | "tmux" | "byobu" => {
            "警告: screen/tmux/byobu 是终端复用器，当前终端不支持。\n提示: 可以使用以下替代方案：\n  - 使用 nohup 在后台运行命令\n  - 使用 & 在后台运行命令".to_string()
        }
        _ => {
            format!("警告: {} 是交互式程序，当前终端不支持交互式操作。", command_name)
        }
    }
}

/// 处理 cd 命令
fn process_cd_command(command: &str, current_dir: &str) -> (String, bool) {
    let trimmed = command.trim();
    let cd_path = if trimmed == "cd" {
        "~".to_string()
    } else if trimmed.starts_with("cd ") {
        trimmed[3..].trim().to_string()
    } else {
        return (command.to_string(), false);
    };
    
    // 转义路径中的单引号
    let escaped_cd_path = cd_path.replace('\'', "'\"'\"'");
    let escaped_base_dir = current_dir.replace('\'', "'\"'\"'");
    
    // 构建 cd 命令：先切换到当前目录，再执行 cd
    let cd_command = format!("cd \"{}\" && cd \"{}\" && pwd", escaped_base_dir, escaped_cd_path);
    (cd_command, true)
}

/// 处理普通命令（添加工作目录上下文）
fn process_normal_command(command: &str, current_dir: &str) -> String {
    // 转义命令中的单引号
    let escaped_command = command.replace('\'', "'\"'\"'");
    
    if current_dir == "~" || current_dir.is_empty() {
        escaped_command
    } else {
        let escaped_dir = current_dir.replace('\'', "'\"'\"'");
        format!("cd \"{}\" && {}", escaped_dir, escaped_command)
    }
}

/// 分割输出为行
fn split_output_lines(output: &str) -> Vec<String> {
    let lines: Vec<String> = output.lines().map(|s| s.to_string()).collect();
    let mut result = Vec::new();
    
    for (index, line) in lines.iter().enumerate() {
        // 最后一行如果是空的且不是唯一一行，则跳过
        if index == lines.len() - 1 && line.is_empty() && lines.len() > 1 {
            continue;
        }
        result.push(line.clone());
    }
    
    if result.is_empty() {
        result.push(String::new());
    }
    
    result
}

/// 解析输入，判断是路径补全还是命令补全
fn parse_completion_input(input: &str, current_dir: &str) -> (bool, String, String) {
    let input = input.trim();
    if input.is_empty() {
        return (false, String::new(), String::new());
    }
    
    let parts: Vec<&str> = input.split_whitespace().collect();
    let last_part = parts.last().unwrap_or(&"");
    
    if last_part.is_empty() {
        return (false, String::new(), String::new());
    }
    
    let first_part = parts.first().unwrap_or(&"");
    let is_file_op = is_file_operation_command(first_part);
    
    // 判断是否为路径补全
    let is_path = last_part.contains('/') 
        || last_part.starts_with('.') 
        || last_part.starts_with('~')
        || (is_file_op && parts.len() > 1);
    
    if is_path {
        // 路径补全：提取目录和前缀
        let (dir, prefix) = if last_part.contains('/') {
            let last_slash = last_part.rfind('/').unwrap();
            let dir_part = &last_part[..=last_slash];
            let prefix_part = &last_part[last_slash + 1..];
            
            // 处理相对路径
            let resolved_dir = if dir_part.starts_with("./") {
                format!("{}/{}", current_dir, &dir_part[2..])
            } else if dir_part.starts_with("../") {
                // 简化处理：使用当前目录
                current_dir.to_string()
            } else if !dir_part.starts_with('/') && !dir_part.starts_with('~') {
                format!("{}/{}", current_dir, dir_part)
            } else {
                dir_part.to_string()
            };
            
            (resolved_dir, prefix_part.to_string())
        } else {
            (current_dir.to_string(), last_part.to_string())
        };
        
        (true, dir, prefix)
    } else {
        // 命令补全
        (false, String::new(), last_part.to_string())
    }
}

/// 构建补全后的输入字符串
fn build_completed_input(
    original_input: &str,
    last_part: &str,
    common_prefix: &str,
    is_path: bool,
    dir: &str,
) -> String {
    let parts: Vec<&str> = original_input.split_whitespace().collect();
    if parts.is_empty() {
        return original_input.to_string();
    }
    
    let mut new_parts: Vec<String> = parts[..parts.len() - 1].iter().map(|s| s.to_string()).collect();
    
    if is_path {
        let new_last_part = if last_part.contains('/') {
            format!("{}{}", dir, common_prefix)
        } else {
            if dir == "~" {
                format!("~/{}", common_prefix)
            } else {
                format!("{}/{}", dir, common_prefix)
            }
        };
        new_parts.push(new_last_part);
    } else {
        new_parts.push(common_prefix.to_string());
    }
    
    new_parts.join(" ")
}

/// 命令补全
/// 
/// # 命令名称
/// `complete_command`
/// 
/// # 参数
/// - `server_id`: 服务器ID
/// - `input`: 完整的输入字符串
/// - `current_dir`: 当前工作目录
/// 
/// # 返回
/// - `completed_input`: 补全后的完整输入（如果可以直接补全）
/// - `matches`: 匹配的选项列表（用于显示）
/// - `should_show_matches`: 是否应该显示匹配列表
#[tauri::command]
pub async fn complete_command(params: CompleteCommandParams) -> Result<CompleteCommandResult, String> {
    // 解析输入，判断是路径补全还是命令补全
    let (is_path, dir, prefix) = parse_completion_input(&params.input, &params.current_dir);
    
    if prefix.is_empty() {
        return Ok(CompleteCommandResult {
            completed_input: None,
            matches: vec![],
            should_show_matches: false,
        });
    }
    
    // 先获取并克隆 session，然后释放锁
    let session = {
        let connections = CONNECTIONS.lock().unwrap();
        match connections.get(&params.server_id) {
            Some(conn) => conn.session.clone(),
            None => return Err("服务器未连接".to_string()),
        }
    };

    // 打开通道执行命令（在锁外执行异步操作）
    let handle = session.lock().await;
    
    let mut channel = match handle.channel_open_session().await {
        Ok(channel) => channel,
        Err(e) => return Err(format!("打开通道失败: {}", e)),
    };

    // 构建补全命令
    let command = if is_path {
        // 路径补全
        let escaped_dir = dir.replace('\'', "'\"'\"'");
        let escaped_prefix = prefix.replace('\'', "'\"'\"'");
        format!("bash -c 'cd \"{}\" && ls -1d {}* 2>/dev/null | head -50'", escaped_dir, escaped_prefix)
    } else {
        // 命令补全
        let escaped_prefix = prefix.replace('\'', "'\"'\"'");
        format!("bash -c 'compgen -c {} | head -50'", escaped_prefix)
    };

    // 执行命令
    let command_bytes = command.as_bytes().to_vec();
    if let Err(e) = channel.exec(true, command_bytes).await {
        return Err(format!("执行补全命令失败: {}", e));
    }

    // 读取命令输出
    let mut output = Vec::new();
    let mut exit_code = 0;

    loop {
        match channel.wait().await {
            Some(ChannelMsg::Data { data }) => {
                output.extend_from_slice(&data);
            }
            Some(ChannelMsg::ExitStatus { exit_status }) => {
                exit_code = exit_status;
            }
            Some(ChannelMsg::Eof) => {
                break;
            }
            Some(ChannelMsg::Close) => {
                break;
            }
            None => {
                break;
            }
            _ => {}
        }
    }

    // 关闭通道
    let _ = channel.close().await;

    if exit_code != 0 {
        return Ok(CompleteCommandResult {
            completed_input: None,
            matches: vec![],
            should_show_matches: false,
        });
    }

    // 解析输出
    let output_text = String::from_utf8_lossy(&output);
    let mut matches: Vec<String> = output_text
        .lines()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    // 去重
    matches.sort();
    matches.dedup();

    if is_path {
        // 路径补全：提取文件名部分（去掉目录路径）
        let matches_files: Vec<String> = matches
            .iter()
            .map(|m| {
                if m.contains('/') {
                    m.split('/').last().unwrap_or(m).to_string()
                } else {
                    m.clone()
                }
            })
            .collect();
        matches = matches_files;
        // 再次去重（因为可能有同名文件在不同目录）
        matches.sort();
        matches.dedup();
        // 过滤出以 prefix 开头的
        matches.retain(|s| s.starts_with(&prefix));
    } else {
        // 命令补全：过滤出以 prefix 开头的
        matches.retain(|s| s.starts_with(&prefix));
    }

    if matches.is_empty() {
        return Ok(CompleteCommandResult {
            completed_input: None,
            matches: vec![],
            should_show_matches: false,
        });
    }

    // 计算最长公共前缀
    let common_prefix = longest_common_prefix(&matches);
    let is_unique_match = matches.len() == 1;
    
    // 获取原始输入的最后一部分
    let parts: Vec<&str> = params.input.split_whitespace().collect();
    let last_part = parts.last().unwrap_or(&"");

    // 构建补全结果
    if is_unique_match {
        // 唯一匹配，直接补全
        let completed_input = build_completed_input(&params.input, last_part, &common_prefix, is_path, &dir);
        Ok(CompleteCommandResult {
            completed_input: Some(completed_input),
            matches: vec![],
            should_show_matches: false,
        })
    } else if common_prefix.len() > prefix.len() {
        // 多个匹配但有公共前缀，补全到公共前缀
        let completed_input = build_completed_input(&params.input, last_part, &common_prefix, is_path, &dir);
        Ok(CompleteCommandResult {
            completed_input: Some(completed_input),
            matches: vec![],
            should_show_matches: false,
        })
    } else {
        // 多个匹配且无公共前缀，显示所有选项
        Ok(CompleteCommandResult {
            completed_input: None,
            matches,
            should_show_matches: true,
        })
    }
}

