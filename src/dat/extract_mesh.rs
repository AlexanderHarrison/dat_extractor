use crate::dat::{
    HSDStruct, HSDRawFile, JOBJ, HighPolyBoneIndicies, DatExtractError, 
    textures::{try_decode_texture, Texture},
    Animation, parse_joint_anim,
};
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

    pub indices_start: u16,
    pub indices_len: u16,
    pub model_group_idx: u8,
}

#[derive(Copy, Clone, Default, Debug)]
pub struct ModelGroup {
    pub prim_group_start: u16,
    pub prim_group_len: u16,
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

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
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
    pub indices: Vec<u16>,
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

    pub indices: Box<[u16]>,
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
    extract_model_from_jobj(root_jobj, Some(&high_poly_bone_indicies))
}

pub fn extract_model_from_jobj<'a>(
    root_jobj: JOBJ<'a>, 
    high_poly_bone_indicies: Option<&HighPolyBoneIndicies> // extracts all if None
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
        indices: Vec::with_capacity(8192),
        vertices: Vec::with_capacity(8192),
    };

    let mut pgroups = Vec::with_capacity(128);
    let mut textures = Vec::with_capacity(64);

    // cache image data ptrs to prevent decoding textures multiple times
    let mut texture_cache = HashMap::with_capacity(64);

    let mut dobj_idx = 0;
    let t = std::time::Instant::now();
    for (i, jobj) in bone_jobjs.iter().enumerate() {
        let pgroup_start = pgroups.len() as _;
        let mut pgroup_len = 0;

        if let Some(dobj) = jobj.get_dobj() {
            for dobj in dobj.siblings() {
                // hack to skip low poly mesh
                let model_group_idx = match high_poly_bone_indicies {
                    None => 0,
                    Some(ref high_poly_bone_indicies) => {
                        let dobj_idx_idx = match high_poly_bone_indicies.indicies.iter()
                            .copied()
                            .position(|idx| idx == dobj_idx)
                        {
                            Some(idx) => idx as u16,
                            None => {
                                dobj_idx += 1;
                                continue;
                            }
                        };

                        let mut model_group_idx = None;
                        for (i, group) in high_poly_bone_indicies.groups.iter().enumerate() {
                            let range = (group.0)..(group.0 + group.1);
                            if range.contains(&dobj_idx_idx) {
                                model_group_idx = Some(i);
                            }
                        }

                        model_group_idx.unwrap() as _
                    }
                };

                dobj_idx += 1;
                pgroup_len += 1;

                let indices_start = builder.indices.len() as u16;

                if let Some(pobj) = dobj.get_pobj() {
                    for pobj in pobj.siblings() {
                        pobj.decode_primitives(&mut builder, &bone_jobjs);
                    }
                }

                let texture_idx = try_decode_texture(&mut texture_cache, &mut textures, dobj);

                let indices_len = builder.indices.len() as u16 - indices_start;

                pgroups.push(PrimitiveGroup {
                    model_group_idx,
                    texture_idx,
                    indices_start,
                    indices_len,
                })
            }
        }

        let bone = &mut bones[i];
        bone.pgroup_start = pgroup_start;
        bone.pgroup_len = pgroup_len;
    }
    println!("mesh decode time: {}us\t {} vertices", t.elapsed().as_micros(), builder.vertices.len());

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
        indices: builder.indices.into_boxed_slice(),
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

    pub fn joint_animations(&self) -> Vec<Animation> {
        if let Some(iter) = self.hsd_struct.try_get_null_ptr_array(0x04) {
            iter.filter_map(|s| parse_joint_anim(s))
                .collect()
        } else {
            Vec::new()
        }
    }
}

pub struct StageSection {
    pub model: Model,
    pub joint_animations: Vec<Animation>,
}

/// returns (scale, models)
pub fn extract_stage<'a>(parsed_stage_dat: &HSDRawFile<'a>) -> Result<(f32, impl Iterator<Item=StageSection> + 'a), DatExtractError> {
    let stage_root = parsed_stage_dat.roots.iter()
        .find(|root| root.root_string == "map_head")
        .ok_or(DatExtractError::InvalidDatFile)?
        .hsd_struct.clone();
    let stage_root = MapHead::new(stage_root);

    let ground_params = parsed_stage_dat.roots.iter()
        .find(|root| root.root_string.starts_with("grGroundParam"))
        .ok_or(DatExtractError::InvalidDatFile)?
        .hsd_struct.clone();

    let scale = ground_params.get_f32(0x00);

    Ok((
        scale, 
        stage_root.get_model_groups()
            .map(|m| {
                let model = extract_model_from_jobj(m.root_jobj(), None).unwrap();
                let mut joint_animations = m.joint_animations();

                // HACK
                for anim in joint_animations.iter_mut() {
                    anim.flags |= super::anim_flags::LOOP
                }
                StageSection { model, joint_animations }
            })
    ))
}

//impl Model {
//    pub fn cube() -> Self {
//        const V: Vertex = Vertex { pos: Vec3::ZERO, uv: Vec2::ZERO, normal: Vec3::ZERO,
//            weights: Vec4::ZERO, bones: UVec4::ZERO, colour: Vec4::ZERO };
//
//        Self {
//            bones: vec![Bone { parent: None, child_start: 0, child_len: 0, pgroup_start: 0, pgroup_len: 1 }].into(),
//            bone_child_idx: vec![].into(),
//            base_transforms: vec![Mat4::IDENTITY].into(),
//            inv_world_transforms: vec![Mat4::IDENTITY].into(),
//
//            primitive_groups: vec![PrimitiveGroup { texture_idx: None, prim_start: 0, prim_len: 1, model_group_idx: 0 }].into(),
//            textures: vec![].into(),
//
//            primitives: vec![Primitive::TriangleStrip { vert_start: 0, vert_len: 14 }].into(),
//            vertices: vec![
//                Vertex { pos: Vec3::new(-1., -1.,  1.), colour: Vec4::new(0., 0., 1., 1. ) , ..V },
//                Vertex { pos: Vec3::new(-1.,  1.,  1.), colour: Vec4::new(0., 1., 1., 1. ) , ..V },
//                Vertex { pos: Vec3::new( 1., -1.,  1.), colour: Vec4::new(1., 0., 1., 1. ) , ..V },
//                Vertex { pos: Vec3::new( 1.,  1.,  1.), colour: Vec4::new(1., 1., 1., 1. ) , ..V },
//                Vertex { pos: Vec3::new( 1.,  1., -1.), colour: Vec4::new(1., 1., 0., 1. ) , ..V },
//                Vertex { pos: Vec3::new(-1.,  1.,  1.), colour: Vec4::new(0., 1., 1., 1. ) , ..V },
//                Vertex { pos: Vec3::new(-1.,  1., -1.), colour: Vec4::new(0., 1., 0., 1. ) , ..V },
//                Vertex { pos: Vec3::new(-1., -1., -1.), colour: Vec4::new(0., 0., 0., 1. ) , ..V },
//                Vertex { pos: Vec3::new( 1.,  1., -1.), colour: Vec4::new(1., 1., 0., 1. ) , ..V },
//                Vertex { pos: Vec3::new( 1., -1., -1.), colour: Vec4::new(1., 0., 0., 1. ) , ..V },
//                Vertex { pos: Vec3::new( 1., -1.,  1.), colour: Vec4::new(1., 0., 1., 1. ) , ..V },
//                Vertex { pos: Vec3::new(-1., -1., -1.), colour: Vec4::new(0., 0., 0., 1. ) , ..V },
//                Vertex { pos: Vec3::new(-1., -1.,  1.), colour: Vec4::new(0., 0., 1., 1. ) , ..V },
//                Vertex { pos: Vec3::new(-1.,  1.,  1.), colour: Vec4::new(0., 1., 1., 1. ) , ..V },
//            ].into(),
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

