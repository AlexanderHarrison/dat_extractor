use dat_tools::isoparser::ISODatFiles;
use slp_parser::Character;

fn main() {
    let file = std::fs::File::open("C:\\Users\\Alex\\Documents\\Melee\\melee vanilla.iso").unwrap();
    //let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();

    let character = Character::Peach.neutral();
    let data = dat_tools::get_fighter_data(&mut files, character).unwrap();

    for (i, a) in data.action_table.iter().enumerate() {
        let name: &str = a.name.as_deref().map(dat_tools::dat::demangle_anim_name).flatten().unwrap_or("");
        println!("{:3} {}", i, name);
    }
}
