use crate::dat::{FighterData, HSDRawFile, Stream};

pub struct AnimDatFile<'a> {
    pub data: &'a [u8],
    pub name: &'a str,
}

#[derive(Debug)]
pub enum AnimParseError {
    InvalidAnimFile,
    CharacterMismatch
}

/// Pass in the raw data of the animations file - Pl*AJ.dat
/// That is necessary (for now).
pub fn extract_anims<'a>(fighter_data: &'a FighterData, aj_data: &'a [u8]) -> Result<Vec<AnimDatFile<'a>>, AnimParseError> {
    let aj_hsd = HSDRawFile::open(Stream::new(aj_data));
    if aj_hsd.roots.len() == 0 { return Err(AnimParseError::InvalidAnimFile) }
    let s = &aj_hsd.roots[0].root_string;

    // 'PlyFox5K_Share_ACTION_Wait1_figatree' -> 'Fox'
    let anim_char_name: &str = s.strip_prefix("Ply")
        .and_then(|s| s.find("K_").map(|p| &s[..p-1]))
        .ok_or(AnimParseError::InvalidAnimFile)?;

    if anim_char_name != &*fighter_data.character_name { return Err(AnimParseError::CharacterMismatch); }

    let mut animations: Vec<AnimDatFile> = Vec::with_capacity(fighter_data.fighter_actions.len());

    for action in &fighter_data.fighter_actions {
        let offset = action.animation_offset;
        let size = action.animation_size;
        let data = &aj_data[offset..offset+size];

        if let Some(name) = &action.name {
            if animations.iter().find(|a| a.name == &**name).is_none() {
                animations.push(AnimDatFile {
                    data,
                    name,
                })
            }
        }
    }

    Ok(animations)
}
