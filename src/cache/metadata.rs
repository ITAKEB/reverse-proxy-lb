use std::time::{SystemTime, Duration, UNIX_EPOCH};
use std::fs::File;
use std::path::Path;
use std::io::{prelude::*, BufReader};

pub const METADATA_SPLIT_SIZE: usize = std::mem::size_of::<u64>();

#[derive(Debug)]
pub struct Metadata {

    pub creation_date: Duration,
    pub ttl: Duration,
    pub content_length: u64,
    pub content_type: Option<String>,
}

impl Metadata {

    pub fn default(ttl: u64, content_length: u64, content_type: Option<String>) -> Result<Self, String> {

        let date = match SystemTime::now().duration_since(UNIX_EPOCH) {

            Ok(x) => { x },
            Err(e) => { return Err(e.to_string()) },
        };

        Ok(Metadata {

            creation_date: date,
            ttl: Duration::from_secs(ttl),
            content_length,
            content_type,
        })
    }

    pub fn ttl_check(&self) -> bool {

        match SystemTime::now().duration_since(UNIX_EPOCH) {

            Ok(x) => { x >= self.creation_date + self.ttl },
            Err(_) => { false },
        }
    }

    pub fn parse_file(path: &Path) -> Result<Self, String> {

        let file = match File::open(path){

            Ok(file) => { file },
            Err(x) => { return Err(x.to_string()) },
        };

        let mut bf_reader = BufReader::new(file);
        let mut buffer = [0u8; std::mem::size_of::<u64>() * 3]; //metadata_split_size
        let mut content_type = String::new();

        match bf_reader.read_line(&mut content_type) {

            Ok(_) => {},
            Err(e) => { return Err(e.to_string()) },
        }

        match bf_reader.read_exact(&mut buffer) {

            Ok(_) => {},
            Err(e) => { return Err(e.to_string()) },
        }

        Ok(Self::parse_buffer(&buffer, content_type))
    }

    pub fn parse_buffer(buffer: &[u8], content_type: String) -> Self {

        assert!(buffer.len() == std::mem::size_of::<u64>() * 3);

        let content_type = if content_type.is_empty() { None} else { Some(content_type.trim().to_string()) };

        let (date, ttl_secs, content_length) = parse_tools(buffer);

        Metadata {
            creation_date: date,
            ttl: ttl_secs,
            content_length,
            content_type,
        }
    }

    pub fn get_size(&self) -> u64 {

        match &self.content_type {

            Some(ct) => { ((METADATA_SPLIT_SIZE * 3) + ct.as_bytes().len() + 1) as u64 },
            None => { ((METADATA_SPLIT_SIZE * 3) + 1) as u64 },
        }
    }

    pub fn get_creation_date(&self) -> Duration {

        self.creation_date
    }

    pub fn get_ttl_time(&self) -> Duration {

        self.ttl
    }

    pub fn get_content_length(&self) -> u64 {

        self.content_length
    }

    pub fn get_content_type(&self) -> &Option<String> {

        &self.content_type
    }
}

pub fn parse_tools(buffer: &[u8]) -> (Duration, Duration, u64) {

    let (a, b) = buffer.split_at(METADATA_SPLIT_SIZE);
    let (b, c) = b.split_at(METADATA_SPLIT_SIZE);

    let date_parse = Duration::from_secs(u64::from_le_bytes(a.try_into().unwrap()));
    let ttl_parse = Duration::from_secs(u64::from_le_bytes(b.try_into().unwrap()));
    let cl_parse = u64::from_le_bytes(c.try_into().unwrap());

    (date_parse, ttl_parse, cl_parse)
}
