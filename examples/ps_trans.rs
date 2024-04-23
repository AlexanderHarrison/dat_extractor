use dat_tools::isoparser::*;
use dat_tools::dat::*;

// ALL HAIL https://discord.com/channels/159510174892163074/636000118495117323/1165560454974812180

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    let stage_dat = files.read_file("GrPs.dat").unwrap();
    let stage_dat = HSDRawFile::new(&stage_dat);

    //let mut texture_cache = std::collections::HashMap::with_capacity(64);
    let mut textures: Vec<Texture> = Vec::with_capacity(64);
    for r in stage_dat.roots.iter() {
        //if r.root_string.ends_with("CMPR_image") {
            println!("{}", r.root_string);
        //}
        //check(1, &mut texture_cache, &mut textures, r.hsd_struct.clone());
    }

    let mut i = 0;
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

fn check(
    i: usize,
    cache: &mut std::collections::HashMap<*const u8, u16>,
    textures: &mut Vec<Texture>,
    s: HSDStruct
) {
    println!("{} 0x{:x} {}", " ".repeat(i), s.len(), s.reference_count());
    if s.len() == 0x5c {
        let tobj = TOBJ::new(s.clone());

        if let Some(data_ptr) = tobj.image_buffer().map(|b| b.as_ptr()) {
            use std::collections::hash_map::Entry;
            match cache.entry(data_ptr) {
                Entry::Occupied(_) => (),
                Entry::Vacant(entry) => {
                    let texture = tobj.texture().unwrap();
                    let texture_idx = textures.len() as _;
                    textures.push(texture);
                    entry.insert(texture_idx);
                }
            }
        }
    }

    for (_, r) in s.get_references().borrow().iter() {
        if r.data.as_ptr() != s.data.as_ptr() {
            check(i+1, cache, textures, r.clone());
        }
    }
}
