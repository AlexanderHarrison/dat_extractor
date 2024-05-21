use glam::f32::{Quat, Mat4, Vec3, Vec4};
use dat_tools::isoparser::*;
use dat_tools::dat::*;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    let dat = files.read_file("EfCoData.dat").unwrap();
    let file = HSDRawFile::new(&dat);

    let ef = EffectTable::new(file.roots[0].hsd_struct.clone());

    let count = (ef.hsd_struct.len() - 0x08) / 0x14;
    for i in 0..count {
        let effect_model = ef.hsd_struct.get_embedded_struct(0x08 + 0x14 * i, 0x14);

        println!("{i}:");
        let j = effect_model.try_get_reference(0x8);
        if let Some(j) = j {
            println!("  joint anim {}", j.iter_joint_tree(0, 4).count());
        }

        let m = effect_model.try_get_reference(0xC);
        if let Some(m) = m {
            println!("  material anim {}", m.clone().iter_joint_tree(0, 4).count());
            for (idx, a) in m.iter_joint_tree(0, 4).enumerate() {
                println!(
                    "    {}: mat_anim {}, aobj {}, tex anim {}",
                    idx,
                    a.try_get_reference(0).is_some(),
                    a.try_get_reference(4).is_some(),
                    a.try_get_reference(8).is_some(),
                )
            }
        }

        let s = effect_model.try_get_reference(0x10);
        if let Some(s) = s {
            println!("  shape anim {}", s.iter_joint_tree(0, 4).count());
        }
    }
}
