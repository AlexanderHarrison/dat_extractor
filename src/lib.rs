pub mod dat;
pub mod isoparser;

use dat::FighterData;
use slippi_situation_parser::states::Character;
use isoparser::{ISOParseError, ISODatFiles};

pub fn parse_string(bytes: &[u8]) -> Option<&str> {
    let terminator = bytes.iter().position(|b| *b == 0)?;
    let str_bytes = &bytes[..terminator];
    std::str::from_utf8(str_bytes).ok()
}

pub fn open_iso(path: &str) -> Result<ISODatFiles, ISOParseError> {
    let file = std::fs::File::open(path).map_err(|_| ISOParseError::FileNotFound)?;
    ISODatFiles::new(file)
}

pub fn get_fighter_data(
    files: &mut ISODatFiles, 
    character_colour: CharacterColour,
) -> Result<FighterData, ISOParseError> {
    let character = character_colour.character();
    let data_filename = character_data_filename(character);
    let anim_filename = character_animation_filename(character);
    let model_filename = character_model_filename(character_colour);

    let base_dat = files.load_file_by_name(data_filename)?;
    let anim_dat = files.load_file_by_name(anim_filename)?;
    let model_dat = files.load_file_by_name(model_filename)?;

    dat::parse_fighter_data(&base_dat, &anim_dat, &model_dat)
        .ok_or(ISOParseError::InvalidISO)
}

#[derive(Copy, Clone, Debug)]
pub enum CharacterColour {
    Fox(FoxColour)
}

#[derive(Copy, Clone, Debug)]
pub enum FoxColour {
    Neutral = 0,
    Lavender = 1,
    Green = 2,
    Orange = 3,
}

pub const fn character_model_filename(character: CharacterColour) -> &'static str {
    use CharacterColour::*;
    match character {
        Fox(FoxColour::Neutral) => "PlFxNr.dat",
        Fox(FoxColour::Lavender) => "PlFxLa.dat",
        Fox(FoxColour::Green) => "PlFxGr.dat",
        Fox(FoxColour::Orange) => "PlFxOr.dat",
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

impl CharacterColour {
    pub fn character(self) -> Character {
        match self {
            CharacterColour::Fox(..) => Character::Fox,
        }
    }
}
