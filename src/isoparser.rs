use std::io::{Read, Seek, SeekFrom, self};
use std::fs::File;
use std::collections::HashMap;
use crate::dat::DatFile;

const OFFSET_FST_OFFSET: u64 = 0x424;

#[derive(Debug)]
pub enum ISOParseError {
    FileNotFound,
    InvalidISO,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DatFileLocation {
    pub start_offset: u64,
    pub size: usize,
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

    pub fn find_file(&self, name: &str) -> Option<DatFileLocation> {
        self.files.iter()
            .find(|(nm, _)| nm.as_ref() == name)
            .map(|(_, loc)| *loc)
    }

    /// ISOParseError::FileNotFound if name is not in file system
    /// If already loaded, nothing occurs
    pub fn load_file_by_name(&mut self, name: &str) -> Result<DatFile, ISOParseError> {
        use std::collections::hash_map::Entry;

        let location = self.find_file(name).ok_or(ISOParseError::FileNotFound)?;

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

    pub fn load_file(&self, location: DatFileLocation) -> DatFile {
        self.open_files[&location].clone()
    }

    pub fn extract_file(&mut self, location: DatFileLocation, save_path: &std::path::Path) -> Result<(), io::Error> {
        let dat = self.load_file(location);
        std::fs::write(save_path, dat.data)
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
