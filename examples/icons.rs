use dat_tools::isoparser::ISODatFiles;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();

    let mut im = 0;
    for t in dat_tools::extract_stock_icons(&mut files).unwrap().iter() {
        println!("textures/texture{:03}.png", im);
        lodepng::encode_file(
            format!("textures/texture{:03}.png", im), 
            t,
            24,
            24,
            lodepng::ColorType::RGBA, // TODO
            8
        ).unwrap();
        im += 1;
    }

    //let dat = files.read_file("IfAll.dat").unwrap();
    //let hsd_if_dat = dat_tools::dat::HSDRawFile::new(&dat);

    //for r in hsd_if_dat.roots.iter() {
    //    println!("{}", r.root_string);
    //}

    //let root = hsd_if_dat.roots.iter()
    //    .find(|r| r.root_string == "Stc_scemdls")
    //    .unwrap().hsd_struct.clone();
    //
    //let mat_anims = root.get_reference(0x00).get_reference(0x08);
    //let mat_anim_j = mat_anims.get_reference(0x00);

    //fn recurse(s: &dat_tools::dat::HSDStruct<'_>, im: &mut usize) {
    //    let a = s.get_reference(0x08);
    //    let tex_anim = a.get_reference(0x08);
    //    let im_buffers = tex_anim.get_reference(0x0C);
    //    let tlut_buffers = tex_anim.get_reference(0x10);

    //    for i in 0..(im_buffers.len() / 4) {
    //        let image = im_buffers.get_reference(i * 4);
    //        let tlut = tlut_buffers.try_get_reference(i * 4)
    //            .map(dat_tools::dat::TLUT::new);
    //        let t = dat_tools::dat::decode_image(image, tlut);

    //        println!("textures/texture{:03}.png", im);
    //        lodepng::encode_file(
    //            format!("textures/texture{:03}.png", im), 
    //            &t.rgba_data,
    //            t.width,
    //            t.height,
    //            lodepng::ColorType::BGRA, // TODO
    //            8
    //        ).unwrap();
    //        *im += 1;
    //    }

    //    if let Some(ref c) = s.try_get_reference(0x00) {
    //        recurse(c, im);
    //    }
    //}

}
