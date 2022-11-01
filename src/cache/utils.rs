use log::{error, info};
use std::path::PathBuf;
use std::{fs, time};
use std::sync::mpsc::Receiver;
use std::thread;

use super::filedata::delete_file;
use super::metadata::Metadata;
use crate::cache::filedata::FileData;

pub fn run_writer(receiver: Receiver<FileData>) {
    thread::spawn(move || loop {

        match receiver.recv() {

            Ok(filedata) => {
                if filedata.get_path().as_path().is_file() { info!("File already exists. Not writing"); }
                else if !FileData::write_file(&filedata) { info!("Failed to write FileData."); }
            },
            Err(_) => { error!("Failed to receive cache file"); },
        }
    });
}

static SLEEP_TIME: u64 = 30;

pub fn run_cleaner(cache_dir: PathBuf) {
    thread::spawn(move || loop {
        clean_folders(cache_dir.clone());
        thread::sleep(time::Duration::from_secs(SLEEP_TIME));
    });
}

fn clean_folders(path: PathBuf) {
    if let Ok(entry) = fs::read_dir(path.as_path()) {
        for dir_entry in entry.flatten() {
            let dir_entry_path = dir_entry.path();
            if dir_entry_path.is_file() {
                if let Ok(metadata) = Metadata::parse_file(&dir_entry_path) {
                    if metadata.ttl_check() { delete_file(dir_entry_path); }
                }
            }
        }
    }
}