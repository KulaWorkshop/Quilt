use std::{
    fs::File,
    io::{BufReader, BufWriter, Read, Seek, SeekFrom, Write},
    path::Path,
};

use byteorder::{LittleEndian, WriteBytesExt};
use flate2::{Compression, bufread::ZlibEncoder};

use crate::{
    archive::ArchiveType,
    lzrw3a::{self, CompressAction},
};

pub struct ArchiveEncoder {
    ty: ArchiveType,
    entries: Vec<FileEntry>,
}

struct FileEntry {
    name: Option<String>,
    buffer: Vec<u8>,
}

impl ArchiveEncoder {
    pub fn new(ty: ArchiveType) -> Self {
        Self {
            ty,
            entries: Vec::new(),
        }
    }

    pub fn add(&mut self, path: &Path) -> Result<(), std::io::Error> {
        let mut file = File::open(path)?;

        let entry: FileEntry = match self.ty {
            ArchiveType::Pak => {
                let reader = BufReader::new(file);
                let mut encoder = ZlibEncoder::new(reader, Compression::default());
                let mut buffer = Vec::new();
                encoder.read_to_end(&mut buffer)?;

                let name = path.file_name().unwrap();

                FileEntry {
                    name: Some(String::from(name.to_str().unwrap())),
                    buffer,
                }
            }
            ArchiveType::Kub => {
                let mut buffer = Vec::new();
                file.read_to_end(&mut buffer)?;

                FileEntry {
                    name: None,
                    buffer: lzrw3a::compress(CompressAction::Compress, &buffer)
                        .expect("Compression failed"),
                }
            }
        };

        self.entries.push(entry);
        Ok(())
    }

    pub fn pack(&self, path: &Path) -> Result<(), std::io::Error> {
        // open file
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // write file count
        writer.write_u32::<LittleEndian>(self.entries.len() as u32)?;

        // write entries
        for _ in 0..self.entries.len() {
            writer.write_u64::<LittleEndian>(0)?;

            // write filename offset blanks
            if self.ty == ArchiveType::Pak {
                writer.write_u32::<LittleEndian>(0)?;
            }
        }

        // write filenames
        if self.ty == ArchiveType::Pak {
            let file_count = self.entries.len();
            for (i, e) in self.entries.iter().enumerate() {
                let name_offset = writer.stream_position()? as u32;
                let name = e.name.clone().unwrap() + "\n\0";
                writer.write_all(name.as_bytes())?;
                writer.seek(SeekFrom::Start(((4 + file_count * 8) + 4 * i) as u64))?;
                writer.write_u32::<LittleEndian>(name_offset)?;
                writer.seek(SeekFrom::End(0))?;
            }

            let padding = (4 - (writer.stream_position()? % 4)) % 4;
            for _ in 0..padding {
                writer.write_u8(0)?;
            }
        }

        for (i, e) in self.entries.iter().enumerate() {
            let offset = writer.stream_position()? as u32;
            let size = e.buffer.len() as u32;

            writer.write_all(&e.buffer)?;
            writer.seek(SeekFrom::Start((4 + (i * 8)) as u64))?;
            writer.write_u32::<LittleEndian>(offset)?;
            writer.write_u32::<LittleEndian>(size)?;
            writer.seek(SeekFrom::End(0))?;
        }

        Ok(())
    }
}
