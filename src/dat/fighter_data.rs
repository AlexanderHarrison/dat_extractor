use crate::dat::{
    HSDStruct, DatFile, Model,
    HSDRawFile, Animation, extract_anims, extract_character_model
};
use crate::parse_string;

#[derive(Debug, Clone)]
pub struct FighterData {
    pub character_name: Box<str>,
    pub animations: Box<[Animation]>,
    pub model: Model,

    pub attributes: FighterAttributes,
}

#[derive(Debug, Clone, Copy)]
pub struct FighterAttributes {
    pub shield_bone: u16,
    pub shield_size: f32,
}

#[derive(Debug, Clone)]
pub struct FighterAction {
    pub name: Option<Box<str>>,        // get => _s.GetReference<HSD_String>(0x00);
    pub animation_offset: usize, // get => _s.GetInt32(0x04)
    pub animation_size: usize, // get => _s.GetInt32(0x08);
}

// SBM_FighterData.cs
#[derive(Debug, Clone)]
pub struct FighterDataRoot<'a> {
    pub hsd_struct: HSDStruct<'a>
}

impl<'a> FighterDataRoot<'a> {
    pub fn new(hsd_struct: HSDStruct<'a>) -> Self {
        Self { hsd_struct }
    }

    pub fn attributes(&self) -> FighterAttributes {
        let centre_bubble = self.hsd_struct.get_reference(0x34);
        FighterAttributes {
            shield_bone: centre_bubble.get_u32(0x00) as u16,
            shield_size: centre_bubble.get_f32(0x04),
        }
    }
}

/// None if not a fighter dat file.
/// Filename should be "PlFx.dat" or the like.
pub fn parse_fighter_data(fighter_dat: &DatFile, anim_dat: &DatFile, model_dat: &DatFile) -> Option<FighterData> {
    let fighter_hsdfile = HSDRawFile::new(fighter_dat);

    let fighter_root_node = &fighter_hsdfile.roots[0];
    let name = fighter_root_node.root_string;
    if !name.starts_with("ftData") || name.contains("Copy") {
        return None;
    }

    let fighter_data_root = FighterDataRoot::new(fighter_root_node.hsd_struct.clone());
    let attributes = fighter_data_root.attributes();

    let actions = parse_actions(&fighter_hsdfile)?;
    let animations = extract_anims(anim_dat, actions).ok()?;

    let parsed_model_dat = HSDRawFile::new(model_dat);
    let model = extract_character_model(&fighter_hsdfile, &parsed_model_dat).ok()?;

    Some(FighterData {
        character_name: name.strip_prefix("ftData").unwrap().to_string().into_boxed_str(),
        animations,
        model,

        attributes,
    })
}

//pub fn get_high_poly_mesh_jobj<'a>(fighter_hsd: &HSDRawFile<'a>) -> JOBJ<'a> {
pub fn get_high_poly_bone_indicies<'a>(fighter_hsd: &HSDRawFile<'a>) -> &'a [u8] {
    let fighter_root = &fighter_hsd.roots[0];
    let lookup_tables = fighter_root.hsd_struct.get_reference(0x08);
    let costume_table = lookup_tables.get_array(0x10, 0x04).next().unwrap();
    let high_poly_table = costume_table.get_array(0x08, 0x00).next().unwrap();
    let jobj_table = high_poly_table.get_array(0x08, 0x04).next().unwrap();
    let count = jobj_table.get_i32(0x00) as usize;
    &jobj_table.get_buffer(0x04)[..count]
    //high_poly_table.get_i32(0x00)
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
