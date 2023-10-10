use dat_tools::isoparser::ISODatFiles;
use slp_parser::CharacterColour;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    //let character = CharacterColour::Fox(slp_parser::character_colours::FoxColour::Neutral);
    let character = CharacterColour::Peach(slp_parser::character_colours::PeachColour::Neutral);

    //let stage_dat = files.read_file("GrOp.dat").unwrap();
    //let hsd_stage_dat = dat_tools::dat::HSDRawFile::new(&stage_dat);
    //let models = dat_tools::dat::extract_stage(&hsd_stage_dat).unwrap();

    let data = dat_tools::get_fighter_data(&mut files, character).unwrap();

    for article in data.articles.into_iter() {
        let mut i = 0;
        //if let Some(ref model) = article.model {
        //    for t in model.textures.iter() {
        //        println!("textures/texture{:02}.png", i);
        //        lodepng::encode_file(
        //            format!("textures/texture{:02}.png", i), 
        //            &t.rgba_data,
        //            t.width,
        //            t.height,
        //            lodepng::ColorType::BGRA, // TODO
        //            8
        //        ).unwrap();
        //        i += 1;
        //    }
        //}

        for t in article.images.iter() {
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
