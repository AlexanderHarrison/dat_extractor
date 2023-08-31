use dat_tools::isoparser::ISODatFiles;
use dat_tools::dat::{AnimationFrame, PrimitiveType};

use slippi_situation_parser::states::Character;
use glam::f32::Vec4;
use glam::Vec4Swizzles;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();

    let fs = [
        ("Ca", "C.Falcon"),
        ("Cl", "CLink, Young Link"),
        ("Dk", "Donkey Kong"),
        ("Dr", "Dr.Mario"),
        ("Fc", "Falco"),
        ("Fe", "Fire Emblem, Roy"),
        ("Fx", "Fox"),
        ("Gn", "Ganondorf"),
        ("Gw", "Game n Watch"),
        ("Kb", "Kirby"),
        ("Kp", "Koopa, Bowser"),
        ("Lg", "Luigi"),
        ("Lk", "Link"),
        ("Mr", "Mario"),
        ("Ms", "Marth"),
        ("Mt", "Mewtwo"),
        ("Nn", "Nana, Ice Climbers"),
        ("Ns", "Ness"),
        ("Pc", "Pichu"),
        ("Pe", "Peach"),
        ("Pk", "Pikachu"),
        ("Pr", "JigglyPuff"),
        ("Sk", "Sheik"),
        ("Ss", "Samus"),
        ("Ys", "Yoshi"),
        ("Zd", "Zelda"),
    ];

    for (f,ch) in fs {
        let fc = format!("Pl{}.dat", f);
        let fr = files.read_file(&fc).unwrap();
        let hsd = dat_tools::dat::HSDRawFile::new(&fr);
        println!("{}: {:?}", ch, dat_tools::dat::get_high_poly_bone_indicies(&hsd));
        //let fighter_root = &hsd.roots[0];
        //let hsd_struct = &fighter_root.hsd_struct;
        //let c = hsd_struct.get_reference(0x34);
        //println!("{}: \t{}", ch, c.get_f32(0x04))
    }
}
