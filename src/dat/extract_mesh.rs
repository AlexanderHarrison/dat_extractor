use crate::dat::{
    HSDStruct, HSDRawFile, JOBJ, ModelBoneIndices, DatExtractError, 
    textures::{try_decode_texture, Texture},
    Animation, parse_joint_anim, parse_mat_anim, Phong, RenderModeFlags
};
use glam::f32::{Mat4, Vec3, Vec4, Vec2};

use std::collections::HashMap;

#[derive(Copy, Clone, Default, Debug)]
pub struct Bone {
    pub parent: Option<u16>,
    pub pgroup_start: u16,
    pub pgroup_len: u16, // zero if none
}

#[derive(Copy, Clone, Default, Debug)]
pub struct PrimitiveGroup {
    pub texture_idx: Option<u16>,

    pub indices_start: u16,
    pub indices_len: u16,
    pub model_group_idx: u8,
    pub mobj_render_flags: RenderModeFlags,
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

/// I hate messing with wgsl <-> rust alignment
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Vertex {
    // pos: [f32; 3]
    // uv: [f32; 2]
    // normal: [f32; 3]
    // weights: [f32; 6]
    // bones: [u32; 6]
    // colour: [f32; 4]
    pub raw: [u8; Vertex::NUM_BYTES],
}

impl Vertex {
    // 24 * 4 bytes
    const NUM_BYTES: usize = (3+2+3+6+6+4) * 4;

    pub const ZERO: Vertex = Vertex {
        raw: [0u8; Vertex::NUM_BYTES],
    };

    pub fn from_parts(
        pos: [f32; 3],
        uv: [f32; 2],
        normal: [f32; 3],
        weights: [f32; 6],
        bones: [u32; 6],
        colour: [f32; 4],
    ) -> Self {
        let mut raw = [0u8; Vertex::NUM_BYTES];
        raw[0*4..3*4].copy_from_slice(bytemuck::cast_slice(&pos));
        raw[3*4..5*4].copy_from_slice(bytemuck::cast_slice(&uv));
        raw[5*4..8*4].copy_from_slice(bytemuck::cast_slice(&normal));
        raw[8*4..14*4].copy_from_slice(bytemuck::cast_slice(&weights));
        raw[14*4..20*4].copy_from_slice(bytemuck::cast_slice(&bones));
        raw[20*4..24*4].copy_from_slice(bytemuck::cast_slice(&colour));
        Vertex { raw }
    }

    #[inline(always)]
    fn f32_i(self, i: usize) -> f32 {
        return f32::from_ne_bytes(self.raw[i*4..i*4+4].try_into().unwrap());
    }

    #[inline(always)]
    fn u32_i(self, i: usize) -> u32 {
        return u32::from_ne_bytes(self.raw[i*4..i*4+4].try_into().unwrap());
    }

    pub fn pos(self) -> Vec3 {
        Vec3::new(self.f32_i(0), self.f32_i(1), self.f32_i(2))
    }

    pub fn uv(self) -> Vec2 {
        Vec2::new(self.f32_i(3), self.f32_i(4))
    }

    pub fn normal(self) -> Vec3 {
        Vec3::new(self.f32_i(5), self.f32_i(6), self.f32_i(7))
    }

    pub fn weights(self) -> [f32; 6] {
        [
            self.f32_i(8), self.f32_i(9), self.f32_i(10),
            self.f32_i(11), self.f32_i(12), self.f32_i(13),
        ]
    }

    pub fn bones(self) -> [u32; 6] {
        [
            self.u32_i(14), self.u32_i(15), self.u32_i(16),
            self.u32_i(17), self.u32_i(18), self.u32_i(19),
        ]
    }
    
    pub fn colour(self) -> Vec4 {
        Vec4::new(self.f32_i(20), self.f32_i(21), self.f32_i(22), self.f32_i(24))
    }
}

unsafe impl bytemuck::NoUninit for Vertex {}

#[derive(Debug, Clone)]
pub struct MeshBuilder {
    pub indices: Vec<u16>,
    pub vertices: Vec<Vertex>,
}

#[derive(Debug, Clone)]
pub struct Model {
    // one for each bone
    pub bones: Box<[Bone]>,
    pub base_transforms: Box<[Mat4]>,
    pub inv_world_transforms: Box<[Mat4]>,

    // one for each dobj
    pub phongs: Box<[Phong]>,
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

    let high_poly_bone_indices = super::get_high_poly_bone_indices(parsed_fighter_dat);
    extract_model_from_jobj(root_jobj, Some(&high_poly_bone_indices))
}

pub fn extract_model_from_jobj<'a>(
    root_jobj: JOBJ<'a>, 
    high_poly_bone_indices: Option<&ModelBoneIndices> // extracts all if None
) -> Result<Model, DatExtractError> {
    let mut bones = Vec::with_capacity(128);
    let mut bone_jobjs = Vec::with_capacity(128);

    // set child + parent idx --------------------------------------------------------
    fn set_bone_idx<'a, 'b>(
        bone_jobjs: &'a mut Vec<JOBJ<'b>>, 
        bones: &'a mut Vec<Bone>, 
        parent: Option<u16>,
        jobj: JOBJ<'b>
    ) {
        let bone_idx = bones.len() as _;
        bone_jobjs.push(jobj.clone());
        bones.push(Bone::default());

        for child_jobj in jobj.children() {
            set_bone_idx(bone_jobjs, bones, Some(bone_idx), child_jobj);
        }

        bones[bone_idx as usize] = Bone {
            parent,

            // set later
            pgroup_start: 0,
            pgroup_len: 0,
        };
    }

    for jobj in root_jobj.siblings() {
        set_bone_idx(&mut bone_jobjs, &mut bones, None, jobj);
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

    let mut phongs = Vec::with_capacity(128);

    let mut dobj_idx = 0;
    let t = std::time::Instant::now();
    for (i, jobj) in bone_jobjs.iter().enumerate() {
        let pgroup_start = pgroups.len() as _;
        let mut pgroup_len = 0;

        if let Some(dobj) = jobj.get_dobj() {
            for dobj in dobj.siblings() {
                // hack to skip low poly mesh
                let model_group_idx = match high_poly_bone_indices {
                    None => 0,
                    Some(ref high_poly_bone_indices) => {
                        let dobj_idx_idx = match high_poly_bone_indices.indices.iter()
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
                        for (i, group) in high_poly_bone_indices.groups.iter().enumerate() {
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

                let (phong, mobj_render_flags) = dobj.get_mobj()
                    .map(|m| (m.get_phong(), m.flags()))
                    .unwrap_or((Phong::default(), 0));
                let texture_idx = try_decode_texture(&mut texture_cache, &mut textures, dobj);

                let indices_len = builder.indices.len() as u16 - indices_start;

                pgroups.push(PrimitiveGroup {
                    model_group_idx,
                    texture_idx,
                    indices_start,
                    indices_len,
                    mobj_render_flags,
                });
                phongs.push(phong);
            }
        }

        let bone = &mut bones[i];
        bone.pgroup_start = pgroup_start;
        bone.pgroup_len = pgroup_len;
    }

    println!("mesh decode time: {}us\t {} vertices {} groups", t.elapsed().as_micros(), builder.vertices.len(), dobj_idx);

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
        base_transforms: base_transforms.into_boxed_slice(),
        phongs: phongs.into_boxed_slice(),
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

    pub fn animations(&self) -> Vec<Animation> {
        let mut anims = Vec::new();

        if let Some(iter) = self.hsd_struct.try_get_null_ptr_array(0x04) {
            for (i, joint_anim_joint) in iter.enumerate() {
                while anims.len() <= i {
                    anims.push(Animation::default());
                }

                parse_joint_anim(&mut anims[i], joint_anim_joint);
            }
        }

        if let Some(iter) = self.hsd_struct.try_get_null_ptr_array(0x08) {
            for (i, mat_anim_joint) in iter.enumerate() {
                while anims.len() <= i {
                    anims.push(Animation::default());
                }

                parse_mat_anim(&mut anims[i], mat_anim_joint);
            }
        }

        anims
    }
}

#[derive(Clone, Debug)]
pub struct StageData {
    pub sections: Vec<StageSection>,
    pub scale: f32,
}

#[derive(Clone, Debug)]
pub struct StageSection {
    pub model: Model,
    pub animations: Vec<Animation>,
}

/// returns (scale, models)
pub fn extract_stage<'a>(parsed_stage_dat: &HSDRawFile<'a>) -> Result<StageData, DatExtractError> {
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

    let sections = stage_root.get_model_groups()
        //.take(4)
        .map(move |m| {
            let model = extract_model_from_jobj(m.root_jobj(), None).unwrap();
            let animations = m.animations();

            StageSection { model, animations }
        }).collect();

    Ok(StageData {
        sections,
        scale
    })
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

