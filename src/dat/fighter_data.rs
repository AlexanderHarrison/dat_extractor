use crate::dat::{
    HSDStruct, DatFile, Stream, Model,
    HSDRawFile, Animation, extract_anims, extract_mesh::extract_model,
    textures::{Texture, extract_textures},
};
use crate::parse_string;

#[derive(Debug, Clone)]
pub struct FighterData {
    pub character_name: Box<str>,
    pub animations: Box<[Animation]>,
    pub textures: Box<[Texture]>,
    pub model: Model,
}

#[derive(Debug, Clone)]
pub struct FighterAction {
    pub name: Option<Box<str>>,        // get => _s.GetReference<HSD_String>(0x00);
    pub animation_offset: usize, // get => _s.GetInt32(0x04)
    pub animation_size: usize, // get => _s.GetInt32(0x08);
}

/// None if not a fighter dat file.
/// Filename should be "PlFx.dat" or the like.
pub fn parse_fighter_data(fighter_dat: &DatFile, anim_dat: &DatFile, model_dat: &DatFile) -> Option<FighterData> {
    let stream = Stream::new(&fighter_dat.data);
    let fighter_hsdfile = HSDRawFile::open(stream);

    let name = fighter_hsdfile.roots[0].root_string;
    if !name.starts_with("ftData") || name.contains("Copy") {
        return None;
    }

    let actions = parse_actions(&fighter_hsdfile)?;
    
    let animations = extract_anims(anim_dat, actions).unwrap(); // TODO
                                                                
    let parsed_model_dat = HSDRawFile::new(model_dat);
    let (jobjs, model) = extract_model(&parsed_model_dat).ok()?;

    let textures = extract_textures(&*jobjs);

    Some(FighterData {
        character_name: name.strip_prefix("ftData").unwrap().to_string().into_boxed_str(),
        animations,
        textures,
        model
    })
}

pub fn parse_actions(fighter_hsd: &HSDRawFile) -> Option<Box<[FighterAction]>> {
    let mut actions = Vec::new();

    let fighter_root = &fighter_hsd.roots[0];
    let hsd_struct = &fighter_root.hsd_struct;

    let action_table_struct = hsd_struct.get_reference(0x0C);

    for i in 0..(action_table_struct.len() / 0x18) {
        let s = action_table_struct.get_embedded_struct(i * 0x18, 0x18);
        let action = parse_fighter_action(s)?;
        actions.push(action);
    }

    Some(actions.into_boxed_slice())
}

fn parse_fighter_action(hsd_struct: HSDStruct) -> Option<FighterAction> {
    let name = if let Some(str_buffer) = hsd_struct.try_get_buffer(0x00) {
        Some(parse_string(str_buffer)?.to_string().into_boxed_str())
    } else {
        None
    };

    let animation_offset = hsd_struct.get_i32(0x04) as usize;
    let animation_size = hsd_struct.get_i32(0x08) as usize;

    Some(FighterAction {
        name,
        animation_offset,
        animation_size,
    })
}
