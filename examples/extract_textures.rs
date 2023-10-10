use dat_tools::isoparser::ISODatFiles;
use slp_parser::CharacterColour;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();

    //let stage_dat = files.read_file("GrOp.dat").unwrap();
    //let hsd_stage_dat = dat_tools::dat::HSDRawFile::new(&stage_dat);
    //let models = dat_tools::dat::extract_stage(&hsd_stage_dat).unwrap();

    let character = CharacterColour::Fox(slp_parser::character_colours::FoxColour::Neutral);
    //let character = CharacterColour::Peach(slp_parser::character_colours::PeachColour::Neutral);
    //let fighter_dat = files.read_file("PlFx.dat").unwrap();
    //let fighter_dat = dat_tools::dat::HSDRawFile::new(&fighter_dat);
    //let model_dat = files.read_file("PlFxGr.dat").unwrap();
    //let model_dat = dat_tools::dat::HSDRawFile::new(&model_dat);
    //let model = dat_tools::dat::extract_character_model(&fighter_dat, &model_dat).unwrap();

    let data = dat_tools::get_fighter_data(&mut files, character).unwrap();

    let mut i = 0;
    for t in data.model.textures.iter() {
        println!("textures/texture{:02}.png", i);
        lodepng::encode_file(
            format!("textures/texture{:02}.png", i), 
            &t.rgba_data,
            t.width,
            t.height,
            lodepng::ColorType::BGRA, // TODO
            8
        ).unwrap();
        i += 1;
    }

    for t in data.articles.iter() {
        if let Some(ref model) = t.model {
            for t in model.textures.iter() {
                println!("textures/texture{:02}.png", i);
                lodepng::encode_file(
                    format!("textures/texture{:02}.png", i), 
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
}
