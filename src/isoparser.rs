use std::io::{Write, Read, Seek, SeekFrom, self};
use std::fs::File;
use std::collections::HashMap;
use crate::dat::DatFile;
use std::rc::Rc;

const OFFSET_FST_OFFSET: u64 = 0x424;

#[derive(Debug)]
pub enum ISOParseError {
    FileNotFound,
    InvalidISO,
    ReplacementFileTooLarge,
    WriteError(std::io::Error),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DatFileLocation {
    header_offset: u64,
    start_offset: u64,
    size: usize,
}

#[derive(Debug)]
pub struct ISODatFiles {
    pub iso: File,
    pub files: HashMap<Box<str>, DatFileLocation>,
    pub open_files: HashMap<DatFileLocation, DatFile>,
}

impl ISODatFiles {
    pub fn new(mut rawiso: File) -> Result<Self, ISOParseError> {
        let iso = &mut rawiso;
        iso.seek(SeekFrom::Start(OFFSET_FST_OFFSET)).map_err(|_| ISOParseError::InvalidISO)?;
        let fst_offset = read_u32(iso)? as u64;
        iso.seek(SeekFrom::Start(fst_offset + 0x8)).map_err(|_| ISOParseError::InvalidISO)?;
        let entry_count = read_u32(iso)? as u64;
        let string_table_offset = fst_offset + entry_count * 0xC;
        let entry_start_offset = fst_offset + 0xC;
        let mut iso_dat_files = HashMap::new();

        read_files(iso, entry_start_offset, string_table_offset, string_table_offset, &mut iso_dat_files)?;

        Ok(ISODatFiles {
            iso: rawiso,
            files: iso_dat_files,
            open_files: HashMap::new(),
        })
    }

    fn find_file(&self, name: &str) -> Option<DatFileLocation> {
        self.files.iter()
            .find(|(nm, _)| nm.as_ref() == name)
            .map(|(_, loc)| *loc)
    }

    /// ISOParseError::FileNotFound if name is not in file system
    pub fn read_file(&mut self, name: &str) -> Result<DatFile, ISOParseError> {
        let location = self.find_file(name).ok_or(ISOParseError::FileNotFound)?;

        use std::collections::hash_map::Entry;
        let dat = match self.open_files.entry(location) {
            Entry::Occupied(entry) => {
                entry.get().clone()
            },
            Entry::Vacant(entry) => {
                self.iso.seek(SeekFrom::Start(location.start_offset)).map_err(|_| ISOParseError::InvalidISO)?;
                let mut buf = vec![0; location.size];
                self.iso.read_exact(&mut buf).map_err(|_| ISOParseError::InvalidISO)?;
                entry.insert(DatFile {
                    filename: name.to_string().into_boxed_str().into(), 
                    data: buf.into_boxed_slice().into(),
                }).clone()
            }
        };

        Ok(dat)
    }

    pub fn extract_file(&mut self, name: &str, save_path: &std::path::Path) -> Result<(), io::Error> {
        let dat = self.read_file(name).map_err(<ISOParseError as Into<io::Error>>::into)?;
        std::fs::write(save_path, dat.data)
    }

    pub fn write_file(&mut self, file: &str, source: Rc<[u8]>) -> Result<(), ISOParseError> {
        let mut dst = self.files[file];

        if dst.size > source.len() {
            return Err(ISOParseError::ReplacementFileTooLarge);
        }

        //if self.iso.options.G

        dst.size = source.len();

        if let Some(f) = self.open_files.get_mut(&dst) {
            f.data = source.clone();
        }

        // write data
        self.iso.seek(SeekFrom::Start(dst.start_offset)).map_err(|_| ISOParseError::InvalidISO)?;
        self.iso.write_all(&source).map_err(|e| ISOParseError::WriteError(e))?;

        // write file size in header
        self.iso.seek(SeekFrom::Start(dst.header_offset + 0x8)).map_err(|_| ISOParseError::InvalidISO)?;
        let file_size = (source.len() as u32).to_be_bytes();
        self.iso.write_all(&file_size).map_err(|e| ISOParseError::WriteError(e))?;

        Ok(())
    }
}

fn read_files(
    iso: &mut File,
    start_offset: u64, 
    end_offset: u64, 
    string_table_offset: u64,
    files: &mut HashMap<Box<str>, DatFileLocation>,
) -> Result<(), ISOParseError> {
    let mut offset = start_offset;

    while offset < end_offset {
        iso.seek(SeekFrom::Start(offset)).map_err(|_| ISOParseError::InvalidISO)?;

        let mut buf = [0; 0xC];
        iso.read_exact(&mut buf).map_err(|_| ISOParseError::InvalidISO)?;

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

            files.insert(filename, DatFileLocation {
                header_offset: offset,
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
    iso.read_exact(&mut buf).map_err(|_| ISOParseError::InvalidISO)?;
    Ok(u32::from_be_bytes(buf))
}

fn read_filename(mut iso: &File, filename_offset: u64) -> Result<Box<str>, ISOParseError> {
    let return_offset = iso.stream_position().map_err(|_| ISOParseError::InvalidISO)?;

    iso.seek(SeekFrom::Start(filename_offset)).map_err(|_| ISOParseError::InvalidISO)?;

    let s = {
        use io::BufRead;
        let mut buf = Vec::new();
        let mut bufreader = io::BufReader::new(&mut iso);
        bufreader.read_until(0, &mut buf).map_err(|_| ISOParseError::InvalidISO)?;
        buf.pop(); // remove null byte

        // no safe version
        unsafe { std::str::from_boxed_utf8_unchecked(buf.into_boxed_slice()) }
    };

    iso.seek(SeekFrom::Start(return_offset)).map_err(|_| ISOParseError::InvalidISO)?;

    Ok(s)
}

impl Into<io::Error> for ISOParseError {
    fn into(self) -> io::Error {
        match self {
            ISOParseError::FileNotFound => io::Error::from(io::ErrorKind::NotFound),
            ISOParseError::InvalidISO => io::Error::from(io::ErrorKind::InvalidData),
            ISOParseError::ReplacementFileTooLarge => io::Error::from(io::ErrorKind::InvalidData),
            ISOParseError::WriteError(e) => e,
        }
    }
}

impl std::ops::Drop for ISODatFiles {
    fn drop(&mut self) {
        let _ = self.iso.sync_all(); // sometimes permission error on windows???
        self.iso.flush().unwrap();
    }
}
