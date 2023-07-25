use crate::dat::{HSDRawFile, JOBJ, DatExtractError};
use glam::f32::{Mat4, Vec3, Vec4, Vec2};
use glam::u32::UVec4;

#[derive(Copy, Clone, Default, Debug)]
pub struct Bone {
    pub parent: Option<u16>,
    pub child_start: u16,
    pub child_len: u16, // zero if none

    pub prim_start: u16,
    pub prim_len: u16, // zero if none
}

#[derive(Copy, Clone, Debug)]
pub enum PrimitiveType {
    //Points = 0xB8,
    //Lines = 0xA8,
    //LineStrip = 0xB0,
    Triangles = 0x90,
    TriangleStrip = 0x98,
    //TriangleFan = 0xA0,
    Quads = 0x80
}


#[derive(Copy, Clone, Debug)]
pub struct Primitive {
    pub primitive_type: PrimitiveType,
    pub vert_start: u32,
    pub vert_len: u16,
}

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos: Vec3,
    pub uv: Vec2,
    pub normal: Vec3,
    pub weights: Vec4,
    pub bones: UVec4,
}

#[derive(Debug, Clone)]
pub struct Model {
    pub bones: Box<[Bone]>,
    pub bone_child_idx: Box<[u16]>,
    pub base_transforms: Box<[Mat4]>,
    pub inv_world_transforms: Box<[Mat4]>,

    pub primitives: Box<[Primitive]>,
    pub vertices: Box<[Vertex]>,
}

/// returns the model's jobjs and the extracted model
pub fn extract_model<'a>(parsed_model_dat: &HSDRawFile<'a>) -> Result<(Box<[JOBJ<'a>]>, Model), DatExtractError> {
    let mut bones = Vec::with_capacity(128);
    let mut bone_child_idx = Vec::with_capacity(256);
    let mut bone_jobjs = Vec::with_capacity(128);

    // set child + parent idx --------------------------------------------------------
    fn set_bone_idx<'a, 'b>(
        bone_jobjs: &'a mut Vec<JOBJ<'b>>, 
        bone_child_idx: &'a mut Vec<u16>, 
        bones: &'a mut Vec<Bone>, 
        parent: Option<u16>,
        jobj: JOBJ<'b>
    ) -> u16 {
        bone_jobjs.push(jobj.clone());
        let bone_idx = bones.len() as _;
        bones.push(Bone::default());

        let child_start = bone_child_idx.len() as _;
        let mut child_len = 0;

        for child_jobj in jobj.children() {
            child_len += 1;
            let child_idx = set_bone_idx(bone_jobjs, bone_child_idx, bones, Some(bone_idx), child_jobj);
            bone_child_idx.push(child_idx)
        }

        bones[bone_idx as usize] = Bone {
            parent,
            child_start,
            child_len,

            // set later
            prim_start: 0,
            prim_len: 0,
        };
        bone_idx
    }

    let root_jobj = parsed_model_dat.roots.iter()
        .find_map(|root| JOBJ::try_from_root_node(root))
        .ok_or(DatExtractError::InvalidDatFile)?;
    
    for jobj in root_jobj.siblings() {
        set_bone_idx(&mut bone_jobjs, &mut bone_child_idx, &mut bones, None, jobj);
    }

    // get primitives / vertices ------------------------------------------------------
    let mut primitives = Vec::with_capacity(256);
    let mut vertices = Vec::with_capacity(8192);

    let mut dobj_idx = 0;
    for (i, jobj) in bone_jobjs.iter().enumerate() {
        let prim_start = primitives.len() as _;
        let mut prim_len = 0;

        if let Some(dobj) = jobj.get_dobj() {
            for dobj in dobj.siblings() {
                if 36 <= dobj_idx && dobj_idx <= 67 { // skip low poly mesh
                    dobj_idx += 1;
                    continue;
                }

                dobj_idx += 1;
                for pobj in dobj.get_pobj().siblings() {
                    prim_len += pobj.decode_primitives(&mut primitives, &mut vertices, &bone_jobjs);
                }
            }
        }

        let bone = &mut bones[i];
        bone.prim_start = prim_start;
        bone.prim_len = prim_len;
    }

    // get transforms ------------------------------------------------------
    let mut base_transforms = Vec::with_capacity(bones.len());
    let mut world_transforms = Vec::with_capacity(bones.len());

    for (i, jobj) in bone_jobjs.iter().enumerate() {
        let base_transform = jobj.transform();
        base_transforms.push(base_transform);

        let world_transform = match bones[i].parent {
            Some(p_i) => world_transforms[p_i as usize] * base_transform,
            None => base_transform
        };

        world_transforms.push(world_transform)
    }

    let mut inv_world_transforms = world_transforms;
    for t in inv_world_transforms.iter_mut() {
        *t = t.inverse();
    }

    // construct model ------------------------------------------------
    let model = Model {
        bones: bones.into_boxed_slice(),
        bone_child_idx: bone_child_idx.into_boxed_slice(),
        base_transforms: base_transforms.into_boxed_slice(),
        inv_world_transforms: inv_world_transforms.into_boxed_slice(),
        primitives: primitives.into_boxed_slice(),
        vertices: vertices.into_boxed_slice(),
    };

    Ok((bone_jobjs.into_boxed_slice(), model))
}

//impl Model {
//    pub fn obj(&self) {
//        let bones = &*self.bones;
//
//        let mut i = 1;
//        let mut mesh_index = 0;
//        let mut bone_to_obj = move |bone: &Bone| {
//            for mesh in bone.meshes.iter() {
//                // skip low poly mesh
//                if 36 <= mesh_index && mesh_index <= 67 { 
//                    mesh_index += 1;
//                    continue;
//                } else {
//                    mesh_index += 1;
//                }
//
//                for p in mesh.primitives.iter() {
//                    let mut points = Vec::with_capacity(p.vertices.len());
//
//                    for v in p.vertices.iter() {
//                        let t = Vec4::from((v.pos, 1.0));
//
//                        let awt = bone.animated_world_transform(&bones);
//                        let t2 = awt * t;
//                                         
//                        let pos = if v.weights.x == 1.0 { // good
//                            let t = bones[v.bones.x as usize].animated_world_transform(&bones) * t2;
//                            t.xyz()
//                        } else if v.weights != Vec4::ZERO {
//                            let v1 = (bones[v.bones.x as usize].animated_bind_matrix(bones) * v.weights.x) * t;
//                            let v2 = (bones[v.bones.y as usize].animated_bind_matrix(bones) * v.weights.y) * t;
//                            let v3 = (bones[v.bones.z as usize].animated_bind_matrix(bones) * v.weights.z) * t;
//                            let v4 = (bones[v.bones.w as usize].animated_bind_matrix(bones) * v.weights.w) * t;
//                            (v1 + v2 + v3 + v4).xyz()
//                        } else {
//                            t2.xyz()
//                        };
//                        
//                        points.push(pos);
//                    }
//
//                    match p.primitive_type {
//                        PrimitiveType::Triangles => {
//                            for t in points.chunks_exact(3) {
//                                println!("v {} {} {}", t[0].x, t[0].y, t[0].z);
//                                println!("v {} {} {}", t[1].x, t[1].y, t[1].z);
//                                println!("v {} {} {}", t[2].x, t[2].y, t[2].z);
//
//                                println!("f {} {} {}", i, i+1, i+2);
//                                i += 3;
//                            }
//                        }
//                        PrimitiveType::TriangleStrip => {
//                            println!("v {} {} {}", points[0].x, points[0].y, points[0].z);
//                            println!("v {} {} {}", points[1].x, points[1].y, points[1].z);
//
//                            for p in &points[2..] {
//                                println!("v {} {} {}", p.x, p.y, p.z);
//
//                                println!("f {} {} {}", i, i+1, i+2);
//                                i += 1;
//                            }
//                            i += 2;
//                        }
//                        PrimitiveType::Quads => {
//                            for t in points.chunks_exact(4) {
//                                println!("v {} {} {}", t[0].x, t[0].y, t[0].z);
//                                println!("v {} {} {}", t[1].x, t[1].y, t[1].z);
//                                println!("v {} {} {}", t[2].x, t[2].y, t[2].z);
//                                println!("v {} {} {}", t[3].x, t[3].y, t[3].z);
//
//                                println!("f {} {} {} {3}", i, i+1, i+2, i+3);
//                                i += 4;
//                            }
//                        }
//                        p => panic!("{:?}", p)
//                    }
//                }
//            }
//        };
//
//        for root in self.bone_tree_roots.iter() {
//            root.inspect_high_poly_bones(&self.bones, &mut bone_to_obj)
//        }
//    }
//}

//impl Skeleton {
//    pub fn apply_animation(&mut self, frame: f32, animation: &Animation) {
//        for transform in animation.transforms.iter() {
//            let bone = &mut self.bones[transform.bone_index];
//            bone.animated_transform = transform.compute_transform_at(frame, &bone.base_transform);
//        }
//    }
//}

//impl Bone {
//    pub fn animated_bind_matrix(&self, bones: &[Bone]) -> Mat4 {
//        self.animated_world_transform(bones) * self.inv_world_transform(bones)
//    }
//}

impl PrimitiveType {
    pub fn from_u8(n: u8) -> Option<Self> {
        Some(match n {
            //0xB8 => Self::Points       ,
            //0xA8 => Self::Lines        ,
            //0xB0 => Self::LineStrip    ,
            0x90 => Self::Triangles    ,
            0x98 => Self::TriangleStrip,
            //0xA0 => Self::TriangleFan  ,
            0x80 => Self::Quads        ,
            _ => return None
        })
    }
}
