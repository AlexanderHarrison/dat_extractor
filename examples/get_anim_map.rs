use dat_tools::isoparser::ISODatFiles;
use slp_parser::CharacterColour;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();

    //let character = CharacterColour::Peach(slp_parser::character_colours::PeachColour::Blue);
    let character = CharacterColour::CaptainFalcon(
        slp_parser::character_colours::CaptainFalconColour::Neutral
    );
    let data = dat_tools::get_fighter_data(&mut files, character).unwrap();

    for (i, a) in data.animations.iter().enumerate() {
        println!("{:3} {}", i, dat_tools::dat::demangle_anim_name(&*a.name).unwrap());
    }
}
