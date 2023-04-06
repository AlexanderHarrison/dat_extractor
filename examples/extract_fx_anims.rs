use dat_tools::isoparser::ISODatFiles;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    let base_dat = files.read_file("PlFx.dat").unwrap();
    let anim_dat = files.read_file("PlFxAJ.dat").unwrap();
    let fighter_data = dat_tools::dat::parse_fighter_data(&base_dat).unwrap();
    let anims = dat_tools::dat::extract_anims(&fighter_data, &anim_dat.data).unwrap();
    for anim in anims {
        use std::io::Write;

        let name = format!("anims/{}.dat", anim.name);
        let mut f = std::fs::File::create(name).unwrap();
        f.write_all(anim.data).unwrap();
    }
}
