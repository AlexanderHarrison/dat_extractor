use dat_tools::isoparser::ISODatFiles;
use dat_tools::dat::PrimitiveType;

use slippi_situation_parser::states::Character;
use glam::f32::Vec4;
use glam::Vec4Swizzles;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    let c = dat_tools::CharacterColour::Fox(dat_tools::FoxColour::Neutral);
    let mut data = dat_tools::get_fighter_data(&mut files, c).unwrap();

    for anim in data.animations.iter() {
        //if anim.name == "PlyFox5K_Share_ACTION_Appeal_figatree" {
        if &*anim.name == "PlyFox5K_Share_ACTION_SpecialAirNLoop_figatree" {
            data.skeleton.apply_animation(10.0, &anim);
            break;
        }
    }

    let bones = &*data.skeleton.bones;
    //data.skeleton.bone_tree_roots[0].inspect_each(&mut |b| println!("{}", b.index)); return;
    
    let mut b = 0;
    let mut m = 0;
    let mut p = 0;
    let mut v = 0;
    let mut mesh_index = 0;
    for bo in bones.iter() {
        b += 1;
        for mesh in bo.meshes.iter() {
            if 36 <= mesh_index && mesh_index <= 67 { 
                mesh_index += 1;
                continue;
            } else {
                mesh_index += 1;
            }
            m += 1;

            for pr in mesh.primitives.iter() {
                p += 1;

                v += pr.vertices.len();

                match pr.primitive_type {
                    PrimitiveType::Triangles => { }
                    PrimitiveType::TriangleStrip => { }
                    PrimitiveType::Quads => { }
                    p => panic!("{:?}", p)
                }
            }
        }
    }

    println!("{} bones", b);
    println!("{} meshes", m);
    println!("{} primitives", p);
    println!("{} vertices", v);
}
