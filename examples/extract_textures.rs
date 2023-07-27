use dat_tools::isoparser::ISODatFiles;
use dat_tools::CharacterColour;
//use slippi_situation_parser::states::Character;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    let character = CharacterColour::Fox(dat_tools::FoxColour::Neutral);

    let data = dat_tools::get_fighter_data(&mut files, character).unwrap();


    for (i, t) in data.textures.iter().enumerate() {
        lodepng::encode_file(
            format!("textures/texture{}.png", i), 
            &t.rgba_data,
            t.width,
            t.height,
            lodepng::ColorType::BGRA, // TODO
            8
        ).unwrap();
    }
}
