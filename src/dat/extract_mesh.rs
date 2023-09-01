use crate::dat::{HSDStruct, HSDRawFile, JOBJ, DatExtractError, textures::{try_decode_texture, Texture}};
use glam::f32::{Mat4, Vec3, Vec4, Vec2};
use glam::u32::UVec4;

use std::collections::HashMap;

#[derive(Copy, Clone, Default, Debug)]
pub struct Bone {
    pub parent: Option<u16>,
    pub child_start: u16,
    pub child_len: u16, // zero if none

    pub pgroup_start: u16,
    pub pgroup_len: u16, // zero if none
}

#[derive(Copy, Clone, Default, Debug)]
pub struct PrimitiveGroup {
    pub texture_idx: Option<u16>,

    pub prim_start: u16,
    pub prim_len: u16,
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
pub enum Primitive {
    Triangles {
        vert_start: u32,
        vert_len: u16,
    },
    TriangleStrip {
        vert_start: u32,
        vert_len: u16,
    },
}


#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos: Vec3,
    pub uv: Vec2,
    pub normal: Vec3,
    pub weights: Vec4,
    pub bones: UVec4,
    pub colour: Vec4,
    //pub colour1: Vec4,
}

unsafe impl bytemuck::NoUninit for Vertex {}

#[derive(Debug, Clone)]
pub struct MeshBuilder {
    pub primitives: Vec<Primitive>,
    pub vertices: Vec<Vertex>,
}

#[derive(Debug, Clone)]
pub struct Model {
    pub bones: Box<[Bone]>,
    pub bone_child_idx: Box<[u16]>,
    pub base_transforms: Box<[Mat4]>,
    pub inv_world_transforms: Box<[Mat4]>,

    pub primitive_groups: Box<[PrimitiveGroup]>,
    pub textures: Box<[Texture]>,

    pub primitives: Box<[Primitive]>,
    pub vertices: Box<[Vertex]>,
}

pub fn extract_character_model<'a>(
    parsed_fighter_dat: &HSDRawFile<'a>,
    parsed_model_dat: &HSDRawFile<'a>,
) -> Result<Model, DatExtractError> {
    let root_jobj = parsed_model_dat.roots.iter()
        .find_map(|root| JOBJ::try_from_root_node(root))
        .ok_or(DatExtractError::InvalidDatFile)?;

    let high_poly_bone_indicies = super::get_high_poly_bone_indicies(parsed_fighter_dat);

    extract_model_from_jobj(root_jobj, Some(high_poly_bone_indicies))
}

/// returns the model's jobjs and the extracted model
pub fn extract_model_from_jobj<'a>(
    root_jobj: JOBJ<'a>, 
    high_poly_bone_indicies: Option<&[u8]> // extracts all if None
) -> Result<Model, DatExtractError> {
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

            // are set later
            pgroup_start: 0,
            pgroup_len: 0,
        };
        bone_idx
    }

    for jobj in root_jobj.siblings() {
        set_bone_idx(&mut bone_jobjs, &mut bone_child_idx, &mut bones, None, jobj);
    }

    // get meshes / primitives / vertices ------------------------------------------------------
    let mut builder = MeshBuilder {
        primitives: Vec::with_capacity(256),
        vertices: Vec::with_capacity(8192),
    };

    let mut pgroups = Vec::with_capacity(128);
    let mut textures = Vec::with_capacity(64);

    // cache image data ptrs to prevent decoding textures multiple times
    let mut texture_cache = HashMap::with_capacity(64);

    let mut dobj_idx = 0;
    for (i, jobj) in bone_jobjs.iter().enumerate() {
        let pgroup_start = pgroups.len() as _;
        let mut pgroup_len = 0;

        if let Some(dobj) = jobj.get_dobj() {
            for dobj in dobj.siblings() {
                // hack to skip low poly mesh
                if let Some(indicies) = high_poly_bone_indicies {
                    if !indicies.contains(&dobj_idx) {
                        dobj_idx += 1;
                        continue;
                    }
                }
                dobj_idx += 1;

                pgroup_len += 1;

                let prim_start = builder.primitives.len() as _;
                let mut prim_len = 0;

                for pobj in dobj.get_pobj().siblings() {
                    prim_len += pobj.decode_primitives(&mut builder, &bone_jobjs);
                }

                let texture_idx = try_decode_texture(&mut texture_cache, &mut textures, dobj);

                pgroups.push(PrimitiveGroup {
                    texture_idx,
                    prim_start,
                    prim_len,
                })
            }
        }

        let bone = &mut bones[i];
        bone.pgroup_start = pgroup_start;
        bone.pgroup_len = pgroup_len;
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
        primitive_groups: pgroups.into_boxed_slice(),
        textures: textures.into_boxed_slice(),
        primitives: builder.primitives.into_boxed_slice(),
        vertices: builder.vertices.into_boxed_slice(),
    };

    Ok(model)
}

pub struct MapHead<'a> {
    pub hsd_struct: HSDStruct<'a>
}

impl<'a> MapHead<'a> {
    pub fn new(hsd_struct: HSDStruct<'a>) -> Self {
        Self { hsd_struct }
    }

    pub fn get_model_groups(&self) -> impl Iterator<Item=MapGOBJ<'a>> {
        self.hsd_struct.get_array(0x34, 0x08)
            .map(MapGOBJ::new)
    }
}

pub struct MapGOBJ<'a> {
    pub hsd_struct: HSDStruct<'a>
}

impl<'a> MapGOBJ<'a> {
    pub fn new(hsd_struct: HSDStruct<'a>) -> Self {
        Self { hsd_struct }
    }

    pub fn root_jobj(&self) -> JOBJ<'a> {
        JOBJ::new(self.hsd_struct.get_reference(0x00))
    }
}

pub fn extract_stage<'a>(parsed_stage_dat: &HSDRawFile<'a>) -> Result<Model, DatExtractError> {
    let stage_root = parsed_stage_dat.roots.iter()
        .find(|root| root.root_string == "map_head")
        .ok_or(DatExtractError::InvalidDatFile)?
        .hsd_struct.clone();
    let stage_root = MapHead::new(stage_root);

    //for m in stage_root.get_model_groups() {
    //    extract_model_from_jobj(m.root_jobj(), 0..0).unwrap();
    //}

    let model_group = stage_root.get_model_groups().nth(3).unwrap();
    let root_jobj = model_group.root_jobj();

    extract_model_from_jobj(root_jobj, None)
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
