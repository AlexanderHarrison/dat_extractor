use dat_tools::isoparser::ISODatFiles;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    let fi = dat_tools::get_fighter_data(&mut files, slp_parser::CharacterColour::Marth(slp_parser::character_colours::MarthColour::Red));
    
    for a in fi.unwrap().action_table.iter() {
        if let Some(ref s) = a.subactions {
            println!("{:?}", a.name.as_deref());
            let mut i = 0usize;
            while i < s.len() {
                dat_tools::dat::parse_next_subaction(&s[i..]);
                i += dat_tools::dat::subaction_size(dat_tools::dat::subaction_cmd(s[i])) as usize;
            }
        }
    }
}
