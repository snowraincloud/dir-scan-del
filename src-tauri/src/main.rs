// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::fs::{self};
use std::path::PathBuf;


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
                            let mut size: u64 = 0;
                            if let Ok(entries) = fs::read_dir(&dir_path) {
                                for entry in entries {
                                    if let Ok(entry) = entry {
                                        size += match entry.metadata() {
                                            Ok(metadata) => metadata.len(),
                                            Err(_) => 0,
                                        };
                                            
                                    }
                                }
                            }
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

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![scan, del])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
