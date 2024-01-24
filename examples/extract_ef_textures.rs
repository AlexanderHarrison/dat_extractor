use dat_tools::isoparser::ISODatFiles;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    //let dat = files.read_file("EfCoData.dat").unwrap();
    let dat = files.read_file("EfPeData.dat").unwrap();
    let hsd_ef_dat = dat_tools::dat::HSDRawFile::new(&dat);

    let table = dat_tools::dat::EffectTable::new(hsd_ef_dat.roots[0].hsd_struct.clone());
    //let textures = table.texture_bank().unwrap().textures();
    let models = table.models();
    //let textures = table.hidden_mat_animation_textures();
    //let models = table.hidden_animation_textures();
    //println!("{}", models.len());

    //for r in hsd_ef_dat.roots.iter() {
    //    println!("{}", r.root_string);
    //}

    println!("{}", models.len());
    let mut i = 0;
    //let model = &models[0]; // shield stuff
    //let model = &models[26]; // blast zone stuff???
    //let model = &models[30]; // yoshi egg
    for model in models.iter() {
        let textures = &model.textures;
        for t in textures.iter() {
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
