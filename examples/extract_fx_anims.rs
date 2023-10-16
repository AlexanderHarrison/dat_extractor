use dat_tools::isoparser::ISODatFiles;
//use dat_tools::dat::PrimitiveType;
//
//use slp_parser::CharacterColour;
//use glam::f32::Vec4;
//use glam::Vec4Swizzles;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    
    let dat = files.read_file("EfFxData.dat").unwrap();
    let hsd_ef_dat = dat_tools::dat::HSDRawFile::new(&dat);
    let table = dat_tools::dat::EffectTable::new(hsd_ef_dat.roots[0].hsd_struct.clone());
    let anims = table.joint_anim(0).unwrap();

    println!("bones: {}", table.model(0).unwrap().bones.len());

    for transform in anims.transforms.iter() {
        for track in transform.tracks.iter() {
            println!("{:?}", track.track_type);
        }
    }

    //let c = CharacterColour::Peach(slp_parser::character_colours::PeachColour::Green);
    //let mut data = dat_tools::get_fighter_data(&mut files, c).unwrap();

    //for anim in data.animations.iter() {
    //    println!("{}", dat_tools::dat::demangle_anim_name(&*anim.name).unwrap());
    //    //if anim.name == "PlyFox5K_Share_ACTION_Appeal_figatree" {
    //    //if &*anim.name == "PlyFox5K_Share_ACTION_SpecialAirNLoop_figatree" {
    //    //    data.skeleton.apply_animation(10.0, &anim);
    //    //    //break;
    //    //}
    //}
    //return;

//    let bones = &*data.skeleton.bones;
//    //data.skeleton.bone_tree_roots[0].inspect_each(&mut |b| println!("{}", b.index)); return;
//    
//    let mut i = 1;
//    for mesh in bones[0].meshes.iter() {
//        for p in mesh.primitives.iter() {
//            let mut points = Vec::with_capacity(p.vertices.len());
//
//            for v in p.vertices.iter() {
//                let t = Vec4::from((v.pos, 1.0));
//
//                let awt = bones[0].animated_world_transform(&bones);
//                let t2 = awt * t;
//                                 
//                let pos = if v.weights.x == 1.0 { // good
//                    let t = bones[v.bones.x as usize].animated_world_transform(&bones) * t2;
//                    t.xyz()
//                } else if v.weights != Vec4::ZERO {
//                    let v1 = (bones[v.bones.x as usize].animated_bind_matrix(bones) * v.weights.x) * t;
//                    let v2 = (bones[v.bones.y as usize].animated_bind_matrix(bones) * v.weights.y) * t;
//                    let v3 = (bones[v.bones.z as usize].animated_bind_matrix(bones) * v.weights.z) * t;
//                    let v4 = (bones[v.bones.w as usize].animated_bind_matrix(bones) * v.weights.w) * t;
//                    (v1 + v2 + v3 + v4).xyz()
//                } else {
//                    t2.xyz()
//                };
//                
//                points.push(pos);
//            }
//
//            match p.primitive_type {
//                PrimitiveType::Triangles => {
//                    for t in points.chunks_exact(3) {
//                        println!("v {} {} {}", t[0].x, t[0].y, t[0].z);
//                        println!("v {} {} {}", t[1].x, t[1].y, t[1].z);
//                        println!("v {} {} {}", t[2].x, t[2].y, t[2].z);
//
//                        println!("f {} {} {}", i, i+1, i+2);
//                        i += 3;
//                    }
//                }
//                PrimitiveType::TriangleStrip => {
//                    println!("v {} {} {}", points[0].x, points[0].y, points[0].z);
//                    println!("v {} {} {}", points[1].x, points[1].y, points[1].z);
//
//                    for p in &points[2..] {
//                        println!("v {} {} {}", p.x, p.y, p.z);
//
//                        println!("f {} {} {}", i, i+1, i+2);
//                        i += 1;
//                    }
//                    i += 2;
//                }
//                PrimitiveType::Quads => {
//                    for t in points.chunks_exact(4) {
//                        println!("v {} {} {}", t[0].x, t[0].y, t[0].z);
//                        println!("v {} {} {}", t[1].x, t[1].y, t[1].z);
//                        println!("v {} {} {}", t[2].x, t[2].y, t[2].z);
//                        println!("v {} {} {}", t[3].x, t[3].y, t[3].z);
//
//                        println!("f {} {} {} {3}", i, i+1, i+2, i+3);
//                        i += 4;
//                    }
//                }
//                p => panic!("{:?}", p)
//            }
//        }
//    }
//
//    //let mut bone_index = 0;
//
//    //let mut i = 1;
//    //let mut mesh_index = 0;
//    //let mut bone_to_obj = move |bone: &Bone| {
//    //    bone_index += 1;
//    //    for mesh in bone.meshes.iter() {
//    //        // skip low poly mesh
//    //        if 36 <= mesh_index && mesh_index <= 67 { 
//    //            println!("{}", bone_index);
//    //            mesh_index += 1;
//    //            continue;
//    //        } else {
//    //            mesh_index += 1;
//    //        }
//
//    //        continue;
//
//    //        for p in mesh.primitives.iter() {
//    //            let mut points = Vec::with_capacity(p.vertices.len());
//
//    //            for v in p.vertices.iter() {
//    //                let t = Vec4::from((v.pos, 1.0));
//
//    //                let awt = bone.animated_world_transform(&bones);
//    //                let t2 = awt * t;
//    //                                 
//    //                let pos = if v.weights.x == 1.0 { // good
//    //                    let t = bones[v.bones.x as usize].animated_world_transform(&bones) * t2;
//    //                    t.xyz()
//    //                } else if v.weights != Vec4::ZERO {
//    //                    let v1 = (bones[v.bones.x as usize].animated_bind_matrix(bones) * v.weights.x) * t;
//    //                    let v2 = (bones[v.bones.y as usize].animated_bind_matrix(bones) * v.weights.y) * t;
//    //                    let v3 = (bones[v.bones.z as usize].animated_bind_matrix(bones) * v.weights.z) * t;
//    //                    let v4 = (bones[v.bones.w as usize].animated_bind_matrix(bones) * v.weights.w) * t;
//    //                    (v1 + v2 + v3 + v4).xyz()
//    //                } else {
//    //                    t2.xyz()
//    //                };
//    //                
//    //                points.push(pos);
//    //            }
//
//    //            match p.primitive_type {
//    //                PrimitiveType::Triangles => {
//    //                    for t in points.chunks_exact(3) {
//    //                        println!("v {} {} {}", t[0].x, t[0].y, t[0].z);
//    //                        println!("v {} {} {}", t[1].x, t[1].y, t[1].z);
//    //                        println!("v {} {} {}", t[2].x, t[2].y, t[2].z);
//
//    //                        println!("f {} {} {}", i, i+1, i+2);
//    //                        i += 3;
//    //                    }
//    //                }
//    //                PrimitiveType::TriangleStrip => {
//    //                    println!("v {} {} {}", points[0].x, points[0].y, points[0].z);
//    //                    println!("v {} {} {}", points[1].x, points[1].y, points[1].z);
//
//    //                    for p in &points[2..] {
//    //                        println!("v {} {} {}", p.x, p.y, p.z);
//
//    //                        println!("f {} {} {}", i, i+1, i+2);
//    //                        i += 1;
//    //                    }
//    //                    i += 2;
//    //                }
//    //                PrimitiveType::Quads => {
//    //                    for t in points.chunks_exact(4) {
//    //                        println!("v {} {} {}", t[0].x, t[0].y, t[0].z);
//    //                        println!("v {} {} {}", t[1].x, t[1].y, t[1].z);
//    //                        println!("v {} {} {}", t[2].x, t[2].y, t[2].z);
//    //                        println!("v {} {} {}", t[3].x, t[3].y, t[3].z);
//
//    //                        println!("f {} {} {} {3}", i, i+1, i+2, i+3);
//    //                        i += 4;
//    //                    }
//    //                }
//    //                p => panic!("{:?}", p)
//    //            }
//    //        }
//    //    }
//    //};
//
//    //for root in data.skeleton.bone_tree_roots.iter() {
//    //    root.inspect_high_poly_bones(&data.skeleton.bones, &mut bone_to_obj)
//    //}
}
