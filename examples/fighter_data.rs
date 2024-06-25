use dat_tools::isoparser::ISODatFiles;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    for c in slp_parser::Character::AS_LIST.iter() {
        let fi = dat_tools::get_fighter_data(&mut files, c.neutral()).unwrap();
    }
    //println!("{}", fi.attributes.shield_bone);
    
    //for a in fi.unwrap().action_table.iter() {
    //    if let Some(n) = a.name.as_deref() {
    //        if !n.contains("SpecialLw") && !n.contains("SpecialAirLw") {
    //            continue
    //        }
    //        println!("{}", n);

    //        if let Some(ref s) = a.subactions {
    //            println!("{:#?}", dat_tools::dat::parse_subactions(s));
    //        }
    //    } 
    //}
}
