use dat_tools::isoparser::ISODatFiles;
//use dat_tools::CharacterColour;
//use slippi_situation_parser::states::Character;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    //let character = CharacterColour::Fox(dat_tools::FoxColour::Neutral);

    //let stage_dat = files.read_file("GrOp.dat").unwrap();
    //let hsd_stage_dat = dat_tools::dat::HSDRawFile::new(&stage_dat);
    //let models = dat_tools::dat::extract_stage(&hsd_stage_dat).unwrap();

    //let data = dat_tools::get_fighter_data(&mut files, character).unwrap();

    let mut i = 0;
    for model in models {
        for t in model.textures.iter() {
            println!("textures/texture{}.png", i);
            lodepng::encode_file(
                format!("textures/texture{}.png", i), 
                &t.rgba_data,
                t.width,
                t.height,
                lodepng::ColorType::BGRA, // TODO
                8
            ).unwrap();
            i += 1;
        }
    }

}
