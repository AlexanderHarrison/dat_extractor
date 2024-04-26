use dat_tools::isoparser::ISODatFiles;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    let fi = dat_tools::get_fighter_data(&mut files, slp_parser::CharacterColour::Marth(slp_parser::character_colours::MarthColour::Red));
    
    for a in fi.unwrap().action_table.iter() {
        if let Some(n) = a.name.as_deref() {
            if !n.contains("Guard") {
                continue
            }

            if let Some(ref s) = a.subactions {
                println!("{:#?}", dat_tools::dat::parse_subactions(s));
            }
        } 
    }
}
