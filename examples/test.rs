use dat_tools::isoparser::ISODatFiles;
use dat_tools::dat::PrimitiveType;

use slippi_situation_parser::states::Character;
use glam::f32::Vec4;
use glam::Vec4Swizzles;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    let mut data = dat_tools::get_fighter_data(&mut files, Character::Fox).unwrap();

    for anim in data.animations.iter() {
        //if anim.name == "PlyFox5K_Share_ACTION_Appeal_figatree" {
        if &*anim.name == "PlyFox5K_Share_ACTION_SpecialAirNLoop_figatree" {
            data.skeleton.apply_animation(10.0, &anim);
            break;
        }
    }

    let bones = &*data.skeleton.bones;
    //data.skeleton.bone_tree_roots[0].inspect_each(&mut |b| println!("{}", b.index)); return;
    
    let mut i = 1;
    for mesh in bones[0].meshes.iter() {
        for p in mesh.primitives.iter() {
            let mut points = Vec::with_capacity(p.vertices.len());

            for v in p.vertices.iter() {
                let t = Vec4::from((v.pos, 1.0));

                let awt = bones[0].animated_world_transform(&bones);
                let t2 = awt * t;
                                 
                let pos = if v.weights.x == 1.0 { // good
                    let t = bones[v.bones.x as usize].animated_world_transform(&bones) * t2;
                    t.xyz()
                } else if v.weights != Vec4::ZERO {
                    let v1 = (bones[v.bones.x as usize].animated_bind_matrix(bones) * v.weights.x) * t;
                    let v2 = (bones[v.bones.y as usize].animated_bind_matrix(bones) * v.weights.y) * t;
                    let v3 = (bones[v.bones.z as usize].animated_bind_matrix(bones) * v.weights.z) * t;
                    let v4 = (bones[v.bones.w as usize].animated_bind_matrix(bones) * v.weights.w) * t;
                    (v1 + v2 + v3 + v4).xyz()
                } else {
                    t2.xyz()
                };
                
                points.push(pos);
            }

            match p.primitive_type {
                PrimitiveType::Triangles => {
                    for t in points.chunks_exact(3) {
                        println!("    [{}f32, {}f32, {}f32],", t[0].x, t[0].y, t[0].z);
                        println!("    [{}f32, {}f32, {}f32],", t[2].x, t[2].y, t[2].z);
                        println!("    [{}f32, {}f32, {}f32],", t[1].x, t[1].y, t[1].z);

                        //println!("    [{}, {}, {}],", i, i+1, i+2);
                        i += 3;
                    }
                }
                PrimitiveType::TriangleStrip => {
                    for j in 2..points.len() {
                        if j & 1 == 1 {
                            println!("    [{}f32, {}f32, {}f32],", points[j-2].x, points[j-2].y, points[j-2].z);
                            println!("    [{}f32, {}f32, {}f32],", points[j-1].x, points[j-1].y, points[j-1].z);
                            println!("    [{}f32, {}f32, {}f32],", points[j].x, points[j].y, points[j].z);
                        } else {
                            println!("    [{}f32, {}f32, {}f32],", points[j-1].x, points[j-1].y, points[j-1].z);
                            println!("    [{}f32, {}f32, {}f32],", points[j-2].x, points[j-2].y, points[j-2].z);
                            println!("    [{}f32, {}f32, {}f32],", points[j].x, points[j].y, points[j].z);
                        }
                        //println!("    [{}, {}, {}],", i+1, i, i+2);
                        i += 1;
                    }
                    i += 2;
                }
                PrimitiveType::Quads => {
                    for t in points.chunks_exact(4) {
                        println!("    [{}f32, {}f32, {}f32],", t[1].x, t[1].y, t[1].z);
                        println!("    [{}f32, {}f32, {}f32],", t[0].x, t[0].y, t[0].z);
                        println!("    [{}f32, {}f32, {}f32],", t[2].x, t[2].y, t[2].z);

                        println!("    [{}f32, {}f32, {}f32],", t[3].x, t[3].y, t[3].z);
                        println!("    [{}f32, {}f32, {}f32],", t[2].x, t[2].y, t[2].z);
                        println!("    [{}f32, {}f32, {}f32],", t[0].x, t[0].y, t[0].z);

                        //println!("    [{}, {}, {}],", i, i+1, i+2);
                        //println!("    [{}, {}, {}],", i+2, i+3, i);
                        i += 4;
                    }
                }
                p => panic!("{:?}", p)
            }
        }
    }
}
