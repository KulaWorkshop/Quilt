pub mod encoder;

use crate::lzrw3a::{self, CompressAction};

use std::fs::File;
use std::io::{BufRead, BufReader, Read, Seek, SeekFrom, Write};
use std::path::Path;

use byteorder::{LittleEndian, ReadBytesExt};
use flate2::read::ZlibDecoder;

pub struct Archive {
    pub ty: ArchiveType,
    reader: BufReader<File>,
    entries: Vec<FileEntry>,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ArchiveType {
    Pak,
    Kub,
}

pub struct ArchiveIterator<'a> {
    index: u32,
    len: u32,
    entries: &'a Vec<FileEntry>,
}

pub struct FileEntry {
    pub name: Option<String>,
    pub offset: u32,
    pub size: u32,
    buffer: Vec<u8>,
    ty: ArchiveType,
}

fn get_type(reader: &mut BufReader<File>) -> Result<ArchiveType, std::io::Error> {
    // seek to first file offset
    reader.seek(SeekFrom::Start(4))?;
    let offset = reader.read_u32::<LittleEndian>()?;
    reader.seek(SeekFrom::Start(offset as u64))?;

    // check compression identifier
    let check = reader.read_u16::<LittleEndian>()?;
    Ok(match check {
        0x9C78 => ArchiveType::Pak,
        _ => ArchiveType::Kub,
    })
}

impl Archive {
    pub fn open(mut reader: BufReader<File>) -> Result<Self, std::io::Error> {
        let ty = get_type(&mut reader)?;

        Ok(Self {
            reader,
            entries: Vec::new(),
            ty,
        })
    }

    pub fn entries(&mut self) -> Result<ArchiveIterator<'_>, std::io::Error> {
        let iterator = self.get_entries()?;
        Ok(iterator)
    }

    fn get_entries(&mut self) -> Result<ArchiveIterator<'_>, std::io::Error> {
        // read file count
        self.reader.seek(SeekFrom::Start(0))?;
        let file_count = self.reader.read_u32::<LittleEndian>()?;

        // read entries
        for i in 0..file_count {
            // read offset and size
            let offset = self.reader.read_u32::<LittleEndian>()?;
            let size = self.reader.read_u32::<LittleEndian>()?;

            // read filename
            let mut name: Option<String> = None;
            if self.ty == ArchiveType::Pak {
                self.reader
                    .seek(SeekFrom::Current((file_count * 8 - i * 4 - 8) as i64))?;
                let name_offset: u32 = self.reader.read_u32::<LittleEndian>()?;
                self.reader.seek(SeekFrom::Start(name_offset as u64))?;

                let mut name_buf = String::new();
                self.reader.read_line(&mut name_buf)?;
                name = Some(name_buf.trim_end().to_string());
            }

            // read buffer
            let mut buffer = vec![0u8; size as usize];
            self.reader.seek(SeekFrom::Start(offset as u64))?;
            self.reader.read_exact(&mut buffer)?;
            self.reader
                .seek(SeekFrom::Start((4 + (i + 1) * 8) as u64))?;

            // add entry
            self.entries.push(FileEntry {
                name,
                offset,
                size,
                buffer,
                ty: self.ty,
            });
        }

        // return iterator
        Ok(ArchiveIterator {
            index: 0,
            len: file_count,
            entries: &self.entries,
        })
    }
}

impl<'a> Iterator for ArchiveIterator<'a> {
    type Item = &'a FileEntry;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.len {
            let result = Some(&self.entries[self.index as usize]);
            self.index += 1;
            result
        } else {
            None
        }
    }
}

impl FileEntry {
    pub fn unpack(&self, path: &Path) -> Result<usize, Box<dyn std::error::Error>> {
        let mut buffer = Vec::<u8>::new();

        // decompress with lzrw3a
        if self.ty == ArchiveType::Kub {
            buffer = lzrw3a::compress(CompressAction::Decompress, &self.buffer)
                .ok_or("lzrw3a buffer returned null")?;
        }
        // decompress with zlib
        else {
            let mut decoder = ZlibDecoder::new(&self.buffer[..]);
            decoder.read_to_end(&mut buffer)?;
        }

        let mut file = File::create(path)?;
        file.write_all(&buffer)?;
        Ok(buffer.len())
    }
}
