use dat_tools::isoparser::ISODatFiles;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    let stage_dat = files.read_file("GrNLa.dat").unwrap();
    let stage_dat = dat_tools::dat::HSDRawFile::new(&stage_dat);
    let model = dat_tools::dat::extract_stage(&stage_dat).unwrap();

    for (i, t) in model.textures.iter().enumerate() {
        lodepng::encode_file(
            format!("textures/texture{}.png", i), 
            &t.rgba_data,
            t.width,
            t.height,
            lodepng::ColorType::BGRA, // TODO
            8
        ).unwrap();
    }

    //for r in stage_dat.roots.iter() {
    //    //println!("{}", r.root_string);
    //    if r.root_string == "coll_data" {
    //        let coll = r.hsd_struct;
    //    }
    //}

    //let scene = dat_tools::dat::extract_scene(&mesh_dat).unwrap();
}
