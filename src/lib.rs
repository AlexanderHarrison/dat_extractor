pub mod dat;
pub mod isoparser;

use dat::FighterData;

use slippi_situation_parser::states::Character;
use isoparser::ISODatFiles;

pub fn parse_string(bytes: &[u8]) -> Option<&str> {
    let terminator = bytes.iter().position(|b| *b == 0)?;
    let str_bytes = &bytes[..terminator];
    std::str::from_utf8(str_bytes).ok()
}

#[derive(Copy, Clone, Debug)]
pub enum ISOError {
    FileNotFound,
    InvalidISO,
}

pub fn open_iso(path: &str) -> Result<ISODatFiles, ISOError> {
    let file = std::fs::File::open(path).map_err(|_| ISOError::FileNotFound)?;
    let iso = ISODatFiles::new(file).map_err(|_| ISOError::InvalidISO)?;

    Ok(iso)
}

pub fn get_fighter_data(
    files: &mut ISODatFiles, 
    character: Character
) -> Result<FighterData, ISOError> {
    let data_filename = character_data_filename(character);
    let anim_filename = character_animation_filename(character);
    let model_filename = character_model_filenames(character)[0];

    let base_dat = files.load_file_by_name(data_filename).map_err(|_| ISOError::InvalidISO)?;
    let anim_dat = files.load_file_by_name(anim_filename).map_err(|_| ISOError::InvalidISO)?;
    let model_dat = files.load_file_by_name(model_filename).map_err(|_| ISOError::InvalidISO)?;

    dat::parse_fighter_data(&base_dat, &anim_dat, &model_dat).ok_or(ISOError::InvalidISO)
}

pub const fn character_model_filenames(character: Character) -> &'static [&'static str] {
    match character {
        Character::Fox => &["PlFxNr.dat", "PlFxLa.dat", "PlFxGr.dat", "PlFxOr.dat"],
        _ => todo!(),
    }
}

pub const fn character_data_filename(character: Character) -> &'static str {
    match character {
        Character::Fox => "PlFx.dat",
        _ => todo!(),
    }
}

pub const fn character_animation_filename(character: Character) -> &'static str {
    match character {
        Character::Fox => "PlFxAJ.dat",
        _ => todo!(),
    }
}

pub const fn inner_character_prefix(character: Character) -> &'static str {
    match character {
        Character::Fox => "Fx",
        _ => todo!(),
    }
}
