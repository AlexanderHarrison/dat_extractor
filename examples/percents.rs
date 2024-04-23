use dat_tools::isoparser::ISODatFiles;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();

    let icons = dat_tools::extract_percents(&mut files).unwrap();

    let mut im = 0;
    for i in icons.iter() {
        println!("textures/texture{:03}.png", im);
        lodepng::encode_file(
            format!("textures/texture{:03}.png", im), 
            &i.rgba_data,
            i.width as _,
            i.height as _,
            lodepng::ColorType::BGRA, // TODO
            8
        ).unwrap();
        im += 1;
    }
}
