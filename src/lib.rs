pub mod dat;
pub mod isoparser;

use dat::FighterData;
use isoparser::{ISOParseError, ISODatFiles};
use slp_parser::{Stage, Character, CharacterColour, character_colours::*};

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

    let base_dat = files.read_file(data_filename)?;
    let anim_dat = files.read_file(anim_filename)?;
    let model_dat = files.read_file(model_filename)?;

    dat::parse_fighter_data(&base_dat, &anim_dat, &model_dat)
        .ok_or(ISOParseError::InvalidISO)
}

pub const fn stage_filename(stage: Stage) -> &'static str {
    match stage {
        Stage::FinalDestination => "GrNLa.dat",
        Stage::FountainOfDreams => "GrIz.dat",
        Stage::DreamLandN64     => "GrOp.dat",
        Stage::Battlefield      => "GrNBa.dat",
        Stage::PokemonStadium   => "GrPs.dat",
        Stage::YoshisStory      => "GrOy.dat",
        _ => todo!(),
    }
}

pub const fn character_model_filename(character: CharacterColour) -> &'static str {
    use CharacterColour::*;
    match character {
        Fox(FoxColour::Neutral) => "PlFxNr.dat",
        Fox(FoxColour::Blue) => "PlFxLa.dat",
        Fox(FoxColour::Green) => "PlFxGr.dat",
        Fox(FoxColour::Red) => "PlFxOr.dat",

        Falco(FalcoColour::Neutral) => "PlFcNr.dat",
        Falco(FalcoColour::Blue) => "PlFcBu.dat",
        Falco(FalcoColour::Green) => "PlFcGr.dat",
        Falco(FalcoColour::Red) => "PlFcRe.dat",

        Marth(MarthColour::Red) => "PlMsRe.dat",
        Marth(MarthColour::Neutral) => "PlMsBk.dat",
        Marth(MarthColour::Black) => "PlMsNr.dat",
        Marth(MarthColour::Green) => "PlMsGr.dat",
        Marth(MarthColour::White) => "PlMsWh.dat",

        _ => todo!(),
    }
}

pub const fn character_data_filename(character: Character) -> &'static str {
    match character {
        Character::Fox => "PlFx.dat",
        Character::Falco => "PlFc.dat",
        Character::Marth => "PlMs.dat",
        _ => todo!(),
    }
}

pub const fn character_effect_filename(character: Character) -> Option<&'static str> {
    match character {
        Character::Fox   => Some("EfFxData.dat"),
        Character::Falco => Some("EfFxData.dat"), // maybe??
        Character::Marth => Some("PlMsData.dat"),
        _ => todo!(),
    }
}

pub const fn character_animation_filename(character: Character) -> &'static str {
    match character {
        Character::Fox => "PlFxAJ.dat",
        Character::Falco => "PlFcAJ.dat",
        Character::Marth => "PlMsAJ.dat",
        _ => todo!(),
    }
}

pub const fn inner_character_prefix(character: Character) -> &'static str {
    match character {
        Character::Fox => "Fx",
        Character::Falco => "Fc",
        Character::Marth => "Ms",
        _ => todo!(),
    }
}
