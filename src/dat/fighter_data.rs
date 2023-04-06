use crate::dat::{HSDStruct, DatFile};

pub struct FighterData {
    pub character_name: Box<str>,
    pub fighter_actions: Vec<FighterAction>,
}

pub struct FighterAction {
    pub name: Option<Box<str>>,        // get => _s.GetReference<HSD_String>(0x00);
    pub animation_offset: usize, // get => _s.GetInt32(0x04)
    pub animation_size: usize, // get => _s.GetInt32(0x08);
}

/// None if not a fighter dat file.
/// Filename should be "PlFx.dat" or the like.
pub fn parse_fighter_data(fighter_dat: &DatFile) -> Option<FighterData> {
    let mut actions = Vec::new();

    let stream = crate::dat::Stream::new(&fighter_dat.data);
    let hsdfile = crate::dat::HSDRawFile::open(stream);

    let fighter_root = &hsdfile.roots[0];
    let name = fighter_root.root_string;
    let hsd_struct = &fighter_root.hsd_struct;

    if !name.starts_with("ftData") || name.contains("Copy") {
        return None;
    }

    let action_table_struct = hsd_struct.get_reference(0x0C);

    for i in 0..(action_table_struct.len() / 0x18) {
        let s = action_table_struct.get_embedded_struct(i * 0x18, 0x18);
        let action = parse_fighter_action(s)?;
        actions.push(action);
    }

    Some(FighterData {
        character_name: name.strip_prefix("ftData").unwrap().to_string().into_boxed_str(),
        fighter_actions: actions
    })
}

fn parse_fighter_action(hsd_struct: HSDStruct) -> Option<FighterAction> {
    let name = if let Some(str_buffer) = hsd_struct.get_buffer(0x00) {
        Some(crate::parse_string(str_buffer)?.to_string().into_boxed_str())
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