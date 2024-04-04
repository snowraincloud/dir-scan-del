// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::{self};
use std::path::{Path, PathBuf};

use std::sync::OnceLock;
use serde::Deserialize;



fn calculate_directory_size(path: &Path) -> u64 {
    let mut total_size = 0;

    if path.is_dir() {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let entry_path = entry.path();
                    if entry_path.is_dir() {
                        total_size += calculate_directory_size(&entry_path);
                    } else {
                        total_size += match entry_path.metadata() {
                            Ok(metadata) => metadata.len(),
                            Err(_) => 0,
                        };
                    }
                }
            }
        }
    } else {
        total_size += match path.metadata() {
            Ok(metadata) => metadata.len(),
            Err(_) => 0,
        };
    }

    total_size
}

fn scan_dir(directory: &PathBuf, target_directory: Vec<String>, all_size: &mut u64, res: &mut Vec<(String, u64, String)>)  {
    match fs::read_dir(directory) {
        Ok(entries) => {
            for entry in entries {
                if let Ok(entry) = entry {
                    let entry_path = entry.path();
                    if !entry_path.is_dir() {
                        continue;
                    }
                    if let Some(dir_name) = entry_path.file_name(){
                        if target_directory.contains(&dir_name.to_string_lossy().to_string()) {
                            let dir_path = entry_path.clone();
                            let size: u64 = calculate_directory_size(&dir_path);
                            *all_size += size;
                            res.push((entry_path.to_str().unwrap_or("Error dir").to_string(), size, "success".to_string()));
                        }else{
                            scan_dir(&entry_path, target_directory.clone(), all_size, res);
                        }
                    }
                }
            }
        }
        Err(err) => {
            println!("Error: {:?}", directory);
            res.push((directory.to_str().unwrap_or("Error dir").to_string(), 0, err.to_string()));
        }
    }
}

fn del_dir(target_directory: Vec<String>, res: &mut Vec<(String, String)>) {
    for dir in target_directory {
        let dir_path = PathBuf::from(dir);
        match fs::remove_dir_all(&dir_path) {
            Ok(_) => {
                res.push((dir_path.to_str().unwrap_or("Error dir").to_string(), "success".to_string()));
            }
            Err(err) => {
                res.push((dir_path.to_str().unwrap_or("Error dir").to_string(), err.to_string()));
            }
        }
    }
}

#[tauri::command]
fn scan(directory: String, target_directory: Vec<String>) -> Result<(Vec<(String, u64, String)>, u64), String> {
    let mut res: Vec<(String, u64, String)> = Vec::new();
    let mut all_size: u64 = 0;
    let dir_path = PathBuf::from(directory);
    scan_dir(&dir_path, target_directory, &mut all_size, &mut res);
    Ok((res, all_size))
}

#[tauri::command]
fn del(target_directory: Vec<String>) -> Result<Vec<(String, String)>, String> {
    let mut res: Vec<(String, String)> = Vec::new();
    del_dir(target_directory, &mut res);
    Ok(res)
}

static CONF:OnceLock<Config> = OnceLock::new();

#[tauri::command]
fn get_config() -> Result<(String, Vec<String>), String> {
    let config = CONF.get().unwrap();
    Ok((config.path.clone(), config.target_directory.clone()))
}

#[derive(Deserialize, Debug)]
struct Config {
    path: String,
    target_directory: Vec<String>,
}


fn main() {
    // 读取配置文件内容
    let content = fs::read_to_string("./conf.toml").expect("读取配置文件失败");
    // 解析为结构体
    let config: Config = toml::from_str(&content).expect("解析配置文件失败");
    CONF.set(config).unwrap();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![scan, del, get_config])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
