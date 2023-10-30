use dat_tools::isoparser::ISODatFiles;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    let stage_dat = files.read_file("GrPs1.dat").unwrap();
    let stage_dat = dat_tools::dat::HSDRawFile::new(&stage_dat);
    let models = dat_tools::dat::extract_stage(&stage_dat).unwrap().1;

    let mut i = 0;
    for model in models {
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

    //for r in stage_dat.roots.iter() {
    //    //println!("{}", r.root_string);
    //    if r.root_string == "coll_data" {
    //        let coll = r.hsd_struct;
    //    }
    //}

    //let scene = dat_tools::dat::extract_scene(&mesh_dat).unwrap();
}
