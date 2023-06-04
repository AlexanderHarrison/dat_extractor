use dat_tools::isoparser::ISODatFiles;
use glam::f32::{Mat4, Vec4};
use glam::Vec4Swizzles;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();

    let mesh = files.read_file("PlFxGr.dat").unwrap();
    let file = dat_tools::dat::HSDRawFile::from_dat_file(&mesh);
    let mut scene = dat_tools::dat::extract_scene(&file).unwrap();

    let base_dat = files.read_file("PlFx.dat").unwrap();
    let anim_dat = files.read_file("PlFxAJ.dat").unwrap();
    let fighter_data = dat_tools::dat::parse_fighter_data(&base_dat).unwrap();
    let anims = dat_tools::dat::extract_anim_dat_files(&fighter_data, &anim_dat.data).unwrap();
    for anim in anims {
        //use std::io::Write;

        //let name = format!("anims/{}.dat", anim.name);
        //let mut f = std::fs::File::create(name).unwrap();
        //f.write_all(anim.data).unwrap();

        //if anim.name == "PlyFox5K_Share_ACTION_Appeal_figatree" {
        //    let anim = dat_tools::dat::extract_anim_from_dat_file(anim);
        //    scene.skeleton.apply_animation(10.0, &anim);
        //    break;
        //}
    }

    let bones = &*scene.skeleton.bones;

    for bone in scene.skeleton.bones.iter() {
        if let Some(mesh) = &bone.mesh {
            for v in mesh.vertices.iter() {
                let t = Vec4::from((v.pos, 1.0)); // matches

                let awt = bone.animated_world_transform(&bones); // MATCHES!!!
                let t2 = awt * t; // good
                                 
                let pos = if v.weights.x == 1.0 {
                    let t = bones[v.bones.x as usize].animated_world_transform(&bones) * t2;
                    t.xyz()
                } else if v.weights != Vec4::ZERO {
                    let v1 = (bones[v.bones.x as usize].animated_bind_matrix(bones) * v.weights.x).transpose() * t;
                    let v2 = (bones[v.bones.y as usize].animated_bind_matrix(bones) * v.weights.y).transpose() * t;
                    let v3 = (bones[v.bones.z as usize].animated_bind_matrix(bones) * v.weights.z).transpose() * t;
                    let v4 = (bones[v.bones.w as usize].animated_bind_matrix(bones) * v.weights.w).transpose() * t;
                    (v1 + v2 + v3 + v4).xyz()
                } else {
                    t2.xyz()
                };
                
                println!("{}, {}, {}", pos.x, pos.y, pos.z);
            }
        }
    }
}
