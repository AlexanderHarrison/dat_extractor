use glam::f32::{Quat, Mat4, Vec3, Vec4};
use dat_tools::isoparser::*;
use dat_tools::dat::*;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    let model_dat = files.read_file("PlMsNr.dat").unwrap();
    let parsed_model_dat = HSDRawFile::new(&model_dat);

    let root_jobj = parsed_model_dat.roots.iter()
        .find_map(|root| JOBJ::try_from_root_node(root))
        .ok_or(DatExtractError::InvalidDatFile).unwrap();

    println!("{}", dat_tools::dat::iter_joint_tree(root_jobj.hsd_struct.clone(), 0x08, 0x0C).count());
}
