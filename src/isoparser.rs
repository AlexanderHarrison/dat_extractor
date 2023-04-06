use thiserror::Error;
use std::io::{Read, Seek, SeekFrom, self};
use std::fs::File;
use std::collections::HashMap;
use miniimm::MiniImmStr;
use crate::dat::DatFile;

const OFFSET_FST_OFFSET: u64 = 0x424;

#[derive(Error, Debug)]
pub enum ISOParseError {
    #[error("File not found")]
    FileNotFound,

    #[error("io error")]
    OtherIOErr(io::Error),
}

#[derive(Debug, Copy, Clone)]
pub struct DatFileLocation {
    pub start_offset: u64,
    pub size: usize,
}

#[derive(Debug)]
pub struct ISODatFiles {
    pub iso: File,
    pub files: HashMap<MiniImmStr, DatFileLocation>,
}

impl ISODatFiles {
    pub fn new(mut rawiso: File) -> Result<Self, ISOParseError> {
        let iso = &mut rawiso;
        iso.seek(SeekFrom::Start(OFFSET_FST_OFFSET)).map_err(|e| e.into())?;
        let fst_offset = read_u32(iso)? as u64;
        iso.seek(SeekFrom::Start(fst_offset + 0x8)).map_err(|e| e.into())?;
        let entry_count = read_u32(iso)? as u64;
        let string_table_offset = fst_offset + entry_count * 0xC;

        let entry_start_offset = fst_offset + 0xC;

        let mut iso_dat_files = HashMap::new();

        read_files(iso, entry_start_offset, string_table_offset, string_table_offset, &mut iso_dat_files)?;

        Ok(ISODatFiles {
            iso: rawiso,
            files: iso_dat_files,
        })
    }

    pub fn find_file(&self, name: &str) -> Option<DatFileLocation> {
        self.files.iter()
            .find(|(nm, _)| nm.as_str() == name)
            .map(|(_, loc)| *loc)
    }

    /// ISOParseError::FileNotFound if name is not in file system
    pub fn read_file(&mut self, name: &str) -> Result<DatFile, ISOParseError> {
        self.files.iter()
            .find(|(nm, _)| nm.as_str() == name)
            .map(|(filename, loc)| {
                self.iso.seek(SeekFrom::Start(loc.start_offset)).map_err(|e| e.into())?;
                let mut buf = vec![0; loc.size];
                self.iso.read_exact(&mut buf).map_err(|e| e.into())?;
                Ok(DatFile {
                    filename: filename.clone(), 
                    data: buf.into_boxed_slice(),
                })
            }).unwrap_or(Err(ISOParseError::FileNotFound))
    }

    pub fn extract_file(&mut self, name: &str, path: &std::path::Path) -> Result<(), ISOParseError> {
        let dat = self.read_file(name)?;
        std::fs::write(path, dat.data).map_err(|e| e.into())
    }
}

fn read_files(
    iso: &mut File,
    start_offset: u64, 
    end_offset: u64, 
    string_table_offset: u64,
    files: &mut HashMap<MiniImmStr, DatFileLocation>,
) -> Result<(), ISOParseError> {
    let mut offset = start_offset;

    while offset < end_offset {
        iso.seek(SeekFrom::Start(offset)).map_err(|e| e.into())?;

        let mut buf = [0; 0xC];
        iso.read_exact(&mut buf).map_err(|e| e.into())?;

        let is_folder = buf[0] == 1;
        if !is_folder {
            // ignore folder structures for now.

            let file_offset = u32::from_be_bytes(buf[0x4..0x8].try_into().unwrap());
            let file_size = u32::from_be_bytes(buf[0x8..0xC].try_into().unwrap());

            let mut filename_offset_buf = [0; 4];
            filename_offset_buf[1] = buf[1];
            filename_offset_buf[2] = buf[2];
            filename_offset_buf[3] = buf[3];
            let filename_offset = u32::from_be_bytes(filename_offset_buf) as u64;
            let filename = read_filename(iso, string_table_offset + filename_offset)?;
            let filename = MiniImmStr::from_string(filename);

            files.insert(filename, DatFileLocation {
                start_offset: file_offset as _,
                size: file_size as _,
            });
        }

        offset += 0xC;
    }

    Ok(())
}

fn read_u32(iso: &mut File) -> Result<u32, ISOParseError> {
    let mut buf = [0; 4];
    iso.read_exact(&mut buf).map_err(|e| e.into())?;
    Ok(u32::from_be_bytes(buf))
}

fn read_filename(mut iso: &File, filename_offset: u64) -> Result<String, ISOParseError> {
    let return_offset = iso.stream_position().map_err(|e| e.into())?;

    iso.seek(SeekFrom::Start(filename_offset)).map_err(|e| e.into())?;

    let s = {
        use io::BufRead;
        let mut buf = Vec::new();
        let mut bufreader = io::BufReader::new(&mut iso);
        bufreader.read_until(0, &mut buf).map_err(|e| e.into())?;
        buf.pop(); // remove null byte

        // no safe version
        unsafe { std::str::from_boxed_utf8_unchecked(buf.into_boxed_slice()) }
    };

    iso.seek(SeekFrom::Start(return_offset)).map_err(|e| e.into())?;

    Ok(s.to_string())
}

impl Into<ISOParseError> for io::Error {
    fn into(self) -> ISOParseError {
        match self.kind() {
            io::ErrorKind::NotFound => ISOParseError::FileNotFound,
            _ => ISOParseError::OtherIOErr(self),
        }
    }
}
