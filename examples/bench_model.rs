use dat_tools::isoparser::ISODatFiles;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    let c = dat_tools::CharacterColour::Fox(dat_tools::FoxColour::Neutral);

    let mut times = Vec::new();

    for _ in 0..10 {
        let t = std::time::Instant::now();
        let data = dat_tools::get_fighter_data(&mut files, c).unwrap();
        times.push(t.elapsed());
        println!("{}", data.character_name);
        let mut l = 0;
        let data = std::hint::black_box(data);
        l += data.animations.len();
        l += data.model.bones.len();
        l += data.model.bone_child_idx.len();
        l += data.model.base_transforms.len();
        l += data.model.inv_world_transforms.len();
        l += data.model.primitive_groups.len();
        l += data.model.textures.len();
        l += data.model.primitives.len();
        l += data.model.vertices.len();
        println!("{}", l);
    }

    let mut ave = 0.0;
    for t in times.iter() {
        ave += t.as_secs_f64() * 1000.0;
    }
    ave /= times.len() as f64;
    println!("{} msec", ave);
}
