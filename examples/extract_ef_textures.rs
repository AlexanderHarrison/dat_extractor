use dat_tools::isoparser::ISODatFiles;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    let dat = files.read_file("EfCoData.dat").unwrap();
    let hsd_ef_dat = dat_tools::dat::HSDRawFile::new(&dat);

    let table = dat_tools::dat::EffectTable::new(hsd_ef_dat.roots[0].hsd_struct.clone());
    //table.texture_bank().textures();
    let models = table.models();

    //for r in hsd_ef_dat.roots.iter() {
    //    println!("{}", r.root_string);
    //}

    let mut i = 0;
    let model = &models[13]; // shield stuff
    //let model = &models[26]; // blast zone stuff???
    //let model = &models[30]; // yoshi egg
    //for model in models {
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
    //}

}
