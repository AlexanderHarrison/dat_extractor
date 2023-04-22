use dat_tools::isoparser::ISODatFiles;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    let base_dat = files.read_file("PlFx.dat").unwrap();
    let anim_dat = files.read_file("PlFxAJ.dat").unwrap();
    let fighter_data = dat_tools::dat::parse_fighter_data(&base_dat).unwrap();
    let anims = dat_tools::dat::extract_anim_dat_files(&fighter_data, &anim_dat.data).unwrap();
    for anim in anims {
        //use std::io::Write;

        //let name = format!("anims/{}.dat", anim.name);
        //let mut f = std::fs::File::create(name).unwrap();
        //f.write_all(anim.data).unwrap();

        if anim.name == "PlyFox5K_Share_ACTION_Appeal_figatree" {
            let anim = dat_tools::dat::extract_anim_from_dat_file(anim);
            for transform in anim.transforms.iter() {
                for track in transform.tracks.iter() {
                    for key in track.keys.iter() {
                        println!("{}", key.interpolation as u8);
                    }
                }
            }

            break;
        }
    }
}
