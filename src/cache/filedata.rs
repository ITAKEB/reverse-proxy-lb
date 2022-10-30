use super::metadata::Metadata;
use std::path::{PathBuf, Path};
use std::io::{prelude::*, BufReader};
use std::io::SeekFrom;
use uuid::Uuid;
use std::fs::{self, File};

#[derive(Debug)]
pub struct FileData {

    pub path: PathBuf,
    pub metadata: Metadata,
    pub content_data: Vec<u8>,
}

impl FileData {

    pub fn default(ttl: u64, content_length: u64, path: PathBuf, content_data: Vec<u8>, content_type: Option<String>) -> Result<Self, String> {

        if !check_valid_path(&path) { return Err(format!("Path no valido al internar crear FileData. path: {path:?} ")) }

        let metadata = match Metadata::default(ttl, content_length, content_type) {

            Ok(x) => { x },
            _ => { return Err("falló al crear el metadata. :(".to_string()) },
        };

        Ok(FileData {

            path,
            metadata,
            content_data,
        })
    }

    pub fn parse_file(path: PathBuf, metadata: Metadata) -> Result<FileData, String> {

        let file = match File::open(path.as_path()) {

            Ok(x) => { x },
            Err(e) => { return Err(e.to_string()) },
        };
        let mut reader = BufReader::new(file);
        reader.seek(SeekFrom::Start(metadata.get_size())).unwrap();

        let mut content_data = vec![0; metadata.content_length as usize];

        match reader.read_exact(&mut content_data) {

            Ok(_) => {

                Ok(FileData {

                    path,
                    metadata,
                    content_data,
                })
            },

            Err(_) => { Err("Failed to read cache file.".to_string()) },
        }
    }

    pub fn write_file(&self) -> bool {

        let uuid = Uuid::new_v4();
        let mut written_path = self.path.clone();
        written_path.set_extension(uuid.to_string());

        let parent = match self.get_path().parent() {

            Some(x) => { x },
            None => { return false; },
        };

        match fs::create_dir_all(parent) {

            Ok(_) => {},
            Err(_) => { return false; },
        }

        let mut file = match File::create(&written_path) {

            Ok(file) => { file },
            Err(_) => { return false; },
        };

        let content_type = self.format_content_type();
        let header = FileData::generate_header(self, content_type);

        let mut index = 0;

        while index < header.len() {

            match file.write(&header[index..]) {

                Ok(bytes_written) => {
                    index += bytes_written;
                    file.flush().unwrap();
                },
                Err(_) => { return false; },
            }
        }

        let content = self.content_data.as_slice();

        index = 0;

        while index < content.len() {

            match file.write(&content[index..]) {

                Ok(bytes_written) => {
                    index += bytes_written;
                    file.flush().unwrap();
                },
                Err(_) => { return false; },
            }
        }

        std::fs::rename(&written_path.as_path(), &self.path.as_path()).unwrap();

        true
    }

    pub fn generate_header(&self, content_type: Vec<u8>) -> Vec<u8> {

        let ts = self.metadata.creation_date.as_secs().to_le_bytes().to_vec();
        let ttl = self.metadata.ttl.as_secs().to_le_bytes().to_vec();
        let length = self.metadata.content_length.to_le_bytes().to_vec();

        [content_type, ts, ttl, length].concat()
    }

    pub fn get_path(&self) -> &PathBuf {

        &self.path
    }

    pub fn get_content(&self) -> &Vec<u8> {

        &self.content_data
    }

    pub fn format_content_type(&self) -> Vec<u8> {

        match &self.metadata.get_content_type() {

            Some(ct) => {

                let mut ct = ct.clone().trim().to_string();
                ct.push('\n');
                ct.as_bytes().to_vec()
            },
            None => {

                let mut ct = String::new();
                ct.push('\n');
                ct.as_bytes().to_vec()
            },
        }
    }
}

pub fn check_valid_path(path: &Path) -> bool {

    !path.is_dir()
}

pub fn create_file_path(cache_folder: &Path, file_route: String) -> PathBuf {

    let mut path = cache_folder.to_path_buf();
    if file_route.starts_with('/') { path.push(file_route.strip_prefix('/').unwrap()); } else { path.push(file_route); }

    path
}

pub fn delete_file(path: PathBuf) -> bool {

    match fs::remove_file(path) {

        Ok(_) => { true },
        Err(_) => { false },
    }
}