// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod ssh;
mod file;
mod monitor;
mod server;
mod ai;
mod db;

use tauri::Manager;

fn main() {
  tauri::Builder::default()
    .invoke_handler(tauri::generate_handler![
      // SSH 相关命令
      ssh::connect_ssh_server,
      ssh::disconnect_ssh_server,
      ssh::execute_ssh_command,
      ssh::reconnect_terminal,
      ssh::complete_command,
      // 文件管理相关命令
      file::list_remote_directory,
      file::upload_file,
      file::download_file,
      file::create_directory,
      file::delete_files,
      file::rename_file,
      file::change_file_mode,
      // 系统监控相关命令
      monitor::get_system_monitor,
      // 服务器配置 CRUD 命令
      server::get_servers,
      server::save_server,
      server::update_server,
      server::delete_server,
      server::get_server,
      // AI 助手相关命令
      ai::chat_with_ai,
      ai::get_ai_quick_actions,
    ])
    .setup(|app| {
      // 初始化数据库
      db::get_db();
      // 执行数据库迁移
      let db = db::get_db();
      let conn = db.lock().unwrap();
      db::migrate_database(&conn).unwrap_or_else(|e| {
        eprintln!("数据库迁移失败: {}", e);
      });
      Ok(())
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
