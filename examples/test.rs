use dat_tools::isoparser::ISODatFiles;
use dat_tools::dat::{AnimationFrame, PrimitiveType};

use slippi_situation_parser::states::Character;
use glam::f32::Vec4;
use glam::Vec4Swizzles;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    let c = dat_tools::CharacterColour::Fox(dat_tools::FoxColour::Neutral);
    let data = dat_tools::get_fighter_data(&mut files, c).unwrap();
    let model = data.model;

    let mut frame = AnimationFrame::new_t_pose(&model);
    for anim in data.animations.iter() {
        //if anim.name == "PlyFox5K_Share_ACTION_Appeal_figatree" {
        if &*anim.name == "PlyFox5K_Share_ACTION_SpecialAirNLoop_figatree" {
            anim.frame_at(10.0, &mut frame, &model);
            break;
        }
    }

    //for m in model.inv_world_transforms.iter() {
    //    println!("{:.4}", m.x_axis.x);
    //    println!("{:.4}", m.x_axis.y);
    //    println!("{:.4}", m.x_axis.z);
    //    println!("{:.4}", m.x_axis.w);

    //    println!("{:.4}", m.y_axis.x);
    //    println!("{:.4}", m.y_axis.y);
    //    println!("{:.4}", m.y_axis.z);
    //    println!("{:.4}", m.y_axis.w);

    //    println!("{:.4}", m.z_axis.x);
    //    println!("{:.4}", m.z_axis.y);
    //    println!("{:.4}", m.z_axis.z);
    //    println!("{:.4}", m.z_axis.w);

    //    println!("{:.4}", m.w_axis.x);
    //    println!("{:.4}", m.w_axis.y);
    //    println!("{:.4}", m.w_axis.z);
    //    println!("{:.4}", m.w_axis.w);
    //}
    //return;

    //let bones = &*data.skeleton.bones;
    ////data.skeleton.bone_tree_roots[0].inspect_each(&mut |b| println!("{}", b.index)); return;
    //
    //let mut b = 0;
    //let mut m = 0;
    //let mut p = 0;
    //let mut v = 0;
    //let mut mesh_index = 0;
    //for bo in bones.iter() {
    //    b += 1;
    //    for mesh in bo.meshes.iter() {
    //        if 36 <= mesh_index && mesh_index <= 67 { 
    //            mesh_index += 1;
    //            continue;
    //        } else {
    //            mesh_index += 1;
    //        }
    //        m += 1;

    //        for pr in mesh.primitives.iter() {
    //            p += 1;

    //            v += pr.vertices.len();

    //            match pr.primitive_type {
    //                PrimitiveType::Triangles => { }
    //                PrimitiveType::TriangleStrip => { }
    //                PrimitiveType::Quads => { }
    //                p => panic!("{:?}", p)
    //            }
    //        }
    //    }
    //}

    //let frame = dat_tools::dat::AnimationFrame::new_t_pose(&model);
    frame.obj(&model);

    //for anim in frame.animated_bind_transforms.iter() {
    //    println!("{:.4}", anim.x_axis[0].abs());
    //    println!("{:.4}", anim.x_axis[1].abs());
    //    println!("{:.4}", anim.x_axis[2].abs());
    //    println!("{:.4}", anim.x_axis[3].abs());
    //    println!("{:.4}", anim.y_axis[0].abs());
    //    println!("{:.4}", anim.y_axis[1].abs());
    //    println!("{:.4}", anim.y_axis[2].abs());
    //    println!("{:.4}", anim.y_axis[3].abs());
    //    println!("{:.4}", anim.z_axis[0].abs());
    //    println!("{:.4}", anim.z_axis[1].abs());
    //    println!("{:.4}", anim.z_axis[2].abs());
    //    println!("{:.4}", anim.z_axis[3].abs());
    //    println!("{:.4}", anim.w_axis[0].abs());
    //    println!("{:.4}", anim.w_axis[1].abs());
    //    println!("{:.4}", anim.w_axis[2].abs());
    //    println!("{:.4}", anim.w_axis[3].abs());
    //}

    //println!("{} bones", model.bones.len());
    //println!("{} primitives", model.primitives.len());
    //println!("{} vertices", model.vertices.len());
}
