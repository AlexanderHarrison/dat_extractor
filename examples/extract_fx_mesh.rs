use dat_tools::isoparser::ISODatFiles;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    let mesh_dat = files.read_file("PlMsNr.dat").unwrap();
    let mesh_dat = dat_tools::dat::HSDRawFile::open(mesh_dat.stream());
    let fighter_dat = files.read_file("PlMs.dat").unwrap();
    let fighter_dat = dat_tools::dat::HSDRawFile::open(fighter_dat.stream());
    let scene = dat_tools::dat::extract_character_model(&fighter_dat, &mesh_dat).unwrap();

    let mut i = 0;
        for t in scene.textures.iter() {
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
