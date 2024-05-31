use glam::f32::{Quat, Mat4, Vec3, Vec4};
use dat_tools::isoparser::*;
use dat_tools::dat::*;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    let stage_dat = files.read_file("GrNLa.dat").unwrap();
    let parsed_stage_dat = dat_tools::dat::HSDRawFile::new(&stage_dat);

    let stage_root = parsed_stage_dat.roots.iter()
        .find(|root| root.root_string == "map_head")
        .ok_or(DatExtractError::InvalidDatFile).unwrap()
        .hsd_struct.clone();
    //let stage_root = MapHead::new(stage_root);

    let gobjs = stage_root.get_array(0x34, 8);

    fn print_mat_anim(m: HSDStruct, depth: usize)  {
        println!("{}MAT ANIM {}", " ".repeat(depth), m.iter_joint_list(0).count());
        //println!("{}MAT ANIM", " ".repeat(depth));

        //if let Some(c) = m.try_get_reference(4) {
        //    println!("{}AOBJ", " ".repeat(depth+1));
        //}

        //if let Some(c) = m.try_get_reference(8) {
        //    for t in c.iter_joint_list(0) {
        //        println!("{}TEX ANIM", " ".repeat(depth+1));
        //    }
        //}

        //if let Some(c) = m.try_get_reference(0) {
        //    print_mat_anim(c, depth);
        //}
    }

    fn print_joint(mj: HSDStruct, depth: usize)  {
        println!("{}JOINT", " ".repeat(depth));

        if let Some(c) = mj.try_get_reference(8) {
            print_mat_anim(c, depth+1);
        }

        if let Some(c) = mj.try_get_reference(0) {
            print_joint(c, depth+1);
        }

        if let Some(c) = mj.try_get_reference(4) {
            print_joint(c, depth);
        }
    }

    fn print_jobj(j: HSDStruct, depth: usize)  {
        println!("{}JOBJ", " ".repeat(depth));
        
        let jobj = JOBJ::new(j.clone());

        if let Some(c) = jobj.get_dobj() {
            println!("{}DOBJ {}", " ".repeat(depth+1), c.hsd_struct.iter_joint_list(4).count());
        }

        if let Some(c) = j.try_get_reference(8) {
            print_jobj(c, depth+1);
        }

        if let Some(c) = j.try_get_reference(0xC) {
            print_jobj(c, depth);
        }
    }

    for gobj in gobjs.take(5) {
        println!("GOBJ");
        if let Some(j) = gobj.try_get_reference(0x8) {
            for r in j.get_references().borrow().values() {
                print_joint(r.clone(), 1);
            }
        }

        print_jobj(gobj.get_reference(0), 1);

        //if let Some(j) = gobj.try_get_reference(0x4) {
        //    println!(" anim joint count: {}", j.len() / 4 - 1);
        //}
        //if let Some(j) = gobj.try_get_reference(0x8) {
        //    println!(" anim mat count: {}", j.len() / 4 - 1);
        //}
        //if let Some(j) = gobj.try_get_reference(0xC) {
        //    println!(" anim shape count: {}", j.len() / 4 - 1);
        //}
    }

    //for joint in stage_root.try_get_null_ptr_array(8).unwrap() {
    //    print_joint(joint, 0);
    //}
}
