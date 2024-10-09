use crate::dat::*;

use glam::f32::{Quat, Mat4, Vec3};

// TODO merge Phong into TexTransform

pub fn demangle_anim_name(name: &str) -> Option<&str> {
    // PlyFox5K_Share_ACTION_WallDamage_figatree => WallDamage

    const PREFIX: &str = "_ACTION_";
    let start = name.find(PREFIX)?;
    let end = name.rfind("_figatree")?;
    Some(&name[start+PREFIX.len()..end])
}

#[derive(Clone, Debug)]
pub struct AnimationFrame {
    // one for each bone
    pub transforms: Box<[Mat4]>,

    // one for each dobj
    pub phongs: Box<[PhongF32]>,
    pub tex_transforms: Box<[TexTransform]>,
}

impl AnimationFrame {
    pub fn new_default_pose(model: &Model) -> Self {
        let transforms = model.base_transforms.to_vec().into_boxed_slice();
        let phongs = model.phongs.iter().map(|&p| p.into()).collect::<Vec<PhongF32>>();
        let tex_transforms = vec![TexTransform::default(); model.phongs.len()];

        AnimationFrame {
            transforms,
            phongs: phongs.into_boxed_slice(),
            tex_transforms: tex_transforms.into_boxed_slice(),
        }
    }

    pub fn default_pose(&mut self, model: &Model) {
        self.transforms.copy_from_slice(model.base_transforms.as_ref());
        for i in 0..model.phongs.len() {
            self.phongs[i] = model.phongs[i].into();
        }
        self.tex_transforms.fill(TexTransform::default());
    }

    pub fn apply_animation(&mut self, model: &Model, anim: &Animation, frame: f32) {
        for transform in anim.bone_transforms.iter() {
            let bone_index = transform.bone_index;
            if let Some(base_transform) = model.base_transforms.get(bone_index) {
                let f = transform.compute_transform_at(frame, base_transform);
                self.transforms[bone_index] = f.transform;
            }
        }

        for transform in anim.material_transforms.iter() {
            let dobj_index = transform.dobj_index;
            let base_phong = model.phongs[dobj_index];
            let animated_material = transform.compute_transform_at(frame, base_phong);
            self.phongs[dobj_index] = animated_material.phong;
            self.tex_transforms[dobj_index] = animated_material.tex_transform;
        }
    }

    // progress: 0 => self, 1 => other
    pub fn interpolate(&mut self, other: &[Mat4], progress: f32) {
        if progress == 0.0 { return; }

        // don't interpolate first four bones. If we do, then animations that turn around interpolate a turning animation.
        // It looks okay, but I'm not sure if this is the correct way to interpolate.
        for bone in 4..self.transforms.len() {
            let (base_scale, base_rotation, base_translation) = self.transforms[bone].to_scale_rotation_translation();
            let (target_scale, target_rotation, target_translation) = other[bone].to_scale_rotation_translation();

            let scale = base_scale.lerp(target_scale, progress);
            let rotation = base_rotation.slerp(target_rotation, progress);
            let translation = base_translation.lerp(target_translation, progress);
            self.transforms[bone] = Mat4::from_scale_rotation_translation(scale, rotation, translation);
        }
    }

    pub fn remove_root_translation(&mut self) {
        // Remove translation from root jobj. This is given by slippi recording.
        // If this is not done, translation done by rolls, dashes, many actions, is applied twice -
        // once by slp position and once by the animation translation.
        //
        // Not entirely accurate - I think slp is recording some other character "centre" 
        // (a different bone?).
        // not ECB x position - tried that
        self.transforms[1].w_axis.z = 0.0;
        self.transforms[1].w_axis.y = 0.0;
    }

    pub fn remove_root_translation_2(&mut self) {
        // Same as above, but removes from the third bone.
        // Needed for Mewtwo for whatever reason.
        self.transforms[2].w_axis.z = 0.0;
        self.transforms[2].w_axis.y = 0.0;
    }

    //pub fn obj(&self, model: &Model) {
    //    let mut i = 1;
    //    for (bone_idx, bone) in model.bones.iter().enumerate() {
    //        let pg_i = bone.pgroup_start as usize;
    //        let pg_len = bone.pgroup_len as usize;

    //        for pgroup in model.primitive_groups[pg_i..pg_i+pg_len].iter() {
    //            let p_i = pgroup.prim_start as usize;
    //            let p_len = pgroup.prim_len as usize;

    //            for p in model.primitives[p_i..p_i+p_len].iter().copied() {
    //                let vertices: Box<dyn Iterator<Item=Vertex>> = match p {
    //                    Primitive::Triangles { vert_start, vert_len } => {
    //                        let v_i = vert_start as usize;
    //                        let v_len = vert_len as usize;
    //                        Box::new(model.vertices[v_i..v_i+v_len].iter().copied())
    //                    }
    //                    Primitive::TriangleStrip { vert_start, vert_len } => {
    //                        let v_i = vert_start as usize;
    //                        let v_len = vert_len as usize;
    //                        Box::new(model.vertices[v_i..v_i+v_len].iter().copied())
    //                    }
    //                };

    //                let mut points = Vec::new();

    //                for v in vertices {
    //                    let t = Vec4::from((v.pos, 1.0));

    //                    let awt = self.animated_world_transforms[bone_idx];
    //                    let t2 = awt * t;
    //                                     
    //                    let pos = if v.weights.x == 1.0 { // good
    //                        let t = self.animated_world_transforms[v.bones.x as usize] * t2;
    //                        t.xyz()
    //                    } else if v.weights != Vec4::ZERO {
    //                        let v1 = (self.animated_bind_transforms[v.bones.x as usize] * v.weights.x) * t;
    //                        let v2 = (self.animated_bind_transforms[v.bones.y as usize] * v.weights.y) * t;
    //                        let v3 = (self.animated_bind_transforms[v.bones.z as usize] * v.weights.z) * t;
    //                        let v4 = (self.animated_bind_transforms[v.bones.w as usize] * v.weights.w) * t;
    //                        (v1 + v2 + v3 + v4).xyz()
    //                    } else {
    //                        t2.xyz()
    //                    };
    //                    
    //                    points.push(pos);
    //                }

    //                match p {
    //                    Primitive::Triangles { .. } => {
    //                        for t in points.chunks_exact(3) {
    //                            println!("v {} {} {}", t[0].x, t[0].y, t[0].z);
    //                            println!("v {} {} {}", t[1].x, t[1].y, t[1].z);
    //                            println!("v {} {} {}", t[2].x, t[2].y, t[2].z);

    //                            println!("f {} {} {}", i, i+1, i+2);
    //                            i += 3;
    //                        }
    //                    }
    //                    Primitive::TriangleStrip { .. } => {
    //                        println!("v {} {} {}", points[0].x, points[0].y, points[0].z);
    //                        println!("v {} {} {}", points[1].x, points[1].y, points[1].z);

    //                        for p in &points[2..] {
    //                            println!("v {} {} {}", p.x, p.y, p.z);

    //                            println!("f {} {} {}", i, i+1, i+2);
    //                            i += 1;
    //                        }
    //                        i += 2;
    //                    }
    //                }
    //            }
    //        }
    //    }
    //}
}

#[derive(Debug, Copy, Clone)]
pub struct AnimDatFile<'a> {
    pub data: &'a [u8],
}

#[derive(Clone, Debug, Default)]
pub struct Animation {
    pub bone_transforms: Vec<AnimTransformBone>,
    pub material_transforms: Vec<AnimTransformMaterial>,
}

pub type AOBJFlags = u32;
pub mod aobj_flags {
    pub const REWOUND      : u32 = 1 << 26;
    pub const FIRST_PLAY   : u32 = 1 << 27;
    pub const NO_UPDATE    : u32 = 1 << 28;
    pub const LOOP         : u32 = 1 << 29;
    pub const NO_ANIM      : u32 = 1 << 30;
}

#[derive(Clone, Debug)]
pub struct AnimTransformBone {
    pub tracks: Box<[AnimTrack<TrackTypeBone>]>,
    pub flags: AOBJFlags,
    pub end_frame: f32,
    pub bone_index: usize,
}

#[derive(Clone, Debug)]
pub struct AnimTransformMaterial {
    pub material_tracks: Box<[AnimTrack<TrackTypeMaterial>]>,
    pub material_flags: AOBJFlags,
    pub material_end_frame: f32,

    pub texture_tracks: Box<[AnimTrack<TrackTypeTexture>]>,
    pub texture_flags: AOBJFlags,
    pub texture_end_frame: f32,
    pub dobj_index: usize,
}

#[derive(Clone, Debug)]
pub struct AnimTransformTexture {
    pub tracks: Box<[AnimTrack<TrackTypeTexture>]>,
    pub flags: AOBJFlags,
    pub end_frame: f32,
    pub texture_index: usize,
}

pub trait TrackType: Copy + Clone + Sized + std::fmt::Debug {
    fn from_u8(n: u8) -> Option<Self>;
}

#[derive(Clone, Debug)]
pub struct AnimTrack<T: TrackType> {
    pub start_frame: f32,
    pub track_type: T,
    pub keys: Box<[Key]>,
}

#[derive(Clone, Debug)]
pub struct FigaTree<'a> {
    pub hsd_struct: HSDStruct<'a>
}

#[derive(Clone, Debug)]
pub struct FigaTreeNode<'a> {
    pub tracks: Box<[Track<'a>]>,
}

#[derive(Clone, Debug)]
pub struct Track<'a> {
    pub hsd_struct: HSDStruct<'a>,
}

#[derive(Copy, Clone, Debug)]
pub struct Key {
    pub frame: f32,
    pub interpolation: InterpolationType,
    pub value: f32,
    pub in_tan: f32,
    pub out_tan: f32,
}

#[derive(Copy, Clone, Debug)]
pub enum InterpolationType {
    Constant,
    Linear,
    Hermite,
    Step
}

pub fn parse_mat_anim(
    prev: &mut Animation,
    mat_anim_joint: HSDStruct<'_>
) {
    // HSD_MatAnimJoint
    let mut dobj_index = 0;
    for mat_anim_joint in mat_anim_joint.iter_joint_tree(0x00, 0x04) {
        let mat_anim = match mat_anim_joint.try_get_reference(0x08) {
            Some(mat_anim) => mat_anim,
            None => continue,
        };
        
        for mat_anim in mat_anim.iter_joint_list(0x00) {
            let mut material_tracks: Box<[AnimTrack<_>]> = Box::new([]);
            let mut material_end_frame = 0.0;
            let mut material_flags = 0;

            let mut texture_tracks: Box<[AnimTrack<_>]> = Box::new([]);
            let mut texture_end_frame = 0.0;
            let mut texture_flags = 0;

            // mat anim
            if let Some(aobj) = mat_anim.try_get_reference(0x04) {
                (material_tracks, material_flags, material_end_frame) = parse_aobj::<TrackTypeMaterial>(aobj);
            };

            // tex anim
            if let Some(tex_anim) = mat_anim.try_get_reference(0x08) {
                // ignore tex anims for other textures for now
                //
                // for tex_anim in tex_anim.iter_joint_list(0x00) {
                    if let Some(aobj) = tex_anim.try_get_reference(0x08) {
                        (texture_tracks, texture_flags, texture_end_frame) = parse_aobj::<TrackTypeTexture>(aobj);
                    }
                // }
            }

            if material_tracks.len() + texture_tracks.len() != 0 {
                prev.material_transforms.push(AnimTransformMaterial { 
                    material_tracks, 
                    material_flags,
                    material_end_frame,
                    texture_tracks,
                    texture_flags,
                    texture_end_frame,
                    dobj_index,
                });
            }

            dobj_index += 1;
        }
    }
}

/// HSD_AnimJoint -> Animation
pub fn parse_joint_anim(
    prev: &mut Animation,
    joint_anim_joint: HSDStruct<'_>
) {
    // HSD_AnimJoint
    for (i, joint_anim_joint) in joint_anim_joint.iter_joint_tree(0x00, 0x04).enumerate() {
        let aobj = match joint_anim_joint.try_get_reference(0x08) {
            Some(aobj) => aobj,
            None => continue,
        };

        let (tracks, flags, end_frame) = parse_aobj::<TrackTypeBone>(aobj);
        prev.bone_transforms.push(AnimTransformBone {
            tracks,
            flags,
            end_frame,
            bone_index: i,
        })
    }
}

fn parse_aobj<T: TrackType>(aobj: HSDStruct) -> (Box<[AnimTrack<T>]>, AOBJFlags, f32) {
    let flags: AOBJFlags = aobj.get_u32(0x00);
    let end_frame = aobj.get_f32(0x04);
    let fobj_desc = aobj.get_reference(0x08);

    let mut tracks = Vec::new();

    for fobj_desc in fobj_desc.iter_joint_list(0x00) {
        if let Some(fobj_desc_data) = fobj_desc_data::<T>(&fobj_desc) {
            let track = decode_anim_data::<T>(fobj_desc_data);
            tracks.push(track);
        }
    }

    (
        tracks.into_boxed_slice(),
        flags,
        end_frame
    )
}

pub fn extract_anim_from_action( 
    aj_dat: &DatFile,
    fighter_action_struct: HSDStruct,
) -> Option<Animation> {
    let offset = fighter_action_struct.get_u32(0x04) as usize;
    let size = fighter_action_struct.get_u32(0x08) as usize;
    if offset == 0 && size == 0 { return None; }
    let anim_data = &aj_dat.data[offset..offset+size];

    let stream = Stream::new(anim_data);
    let hsd_file = HSDRawFile::open(stream);

    let anim_root = &hsd_file.roots[0];
    let figatree = FigaTree::new(anim_root.hsd_struct.clone());

    let bone_transforms = extract_figatree_transforms(figatree);

    Some(Animation {
        bone_transforms,
        material_transforms: Vec::new(),
    })
}

pub fn effective_frame(frame_num: f32, flags: AOBJFlags, end_frame: f32) -> f32 {
    if end_frame <= 0.0 { return frame_num; }
    //if end_frame == 0.0 { return frame_num; }

    let mut effective_frame = frame_num;
    let rewound = flags & aobj_flags::REWOUND != 0;
    let loop_anim = flags & aobj_flags::LOOP != 0;

    // hacky guess
    match (loop_anim, rewound) {
        (false, false) => {}
        (true, false) => {
            while effective_frame > end_frame {
                effective_frame -= end_frame;
            }
        }
        (false, true) => {
            if effective_frame > end_frame && effective_frame < end_frame * 2.0 {
                effective_frame = end_frame * 2.0 - effective_frame;
            }
        }
        (true, true) => {
            while effective_frame > end_frame*2.0 {
                effective_frame -= end_frame*2.0;
            }

            if effective_frame > end_frame {
                effective_frame = end_frame * 2.0 - effective_frame;
            }
        }
    }

    if effective_frame < 0.0 { effective_frame = 0.0; } // idk
    effective_frame += 0.001; // avoid issues with exact frames (fox utilt)
    
    effective_frame
}

impl Animation {
    // max end frame of all tracks
    pub fn end_frame(&self) -> f32 {
        let mut end_frame: f32 = 0.0;

        for t in self.bone_transforms.iter() {
            end_frame = end_frame.max(t.end_frame);
        }

        for t in self.material_transforms.iter() {
            end_frame = end_frame.max(t.texture_end_frame);
            end_frame = end_frame.max(t.material_end_frame);
        }

        end_frame
    }
}

#[derive(Copy, Clone, Debug)]
pub struct AnimTransformBoneFrame {
    pub transform: Mat4,
}

#[derive(Copy, Clone, Debug)]
pub struct AnimTransformMaterialFrame {
    pub phong: PhongF32,
    pub tex_transform: TexTransform,
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TexTransform {
    pub konst_colour: [f32; 4], // idk
    pub tex_colour: [f32; 4],
    pub uv_scale: [f32; 2],
    pub uv_translation: [f32; 2],
}

impl Default for TexTransform {
    fn default() -> Self {
        TexTransform {
            konst_colour: [0.0; 4],
            tex_colour: [1.0; 4],
            uv_scale: [1.0; 2],
            uv_translation: [0.0; 2],
        }
    }
}

unsafe impl bytemuck::NoUninit for TexTransform {}

impl AnimTransformBone {
    pub fn compute_transform_at(
        &self, 
        frame: f32, 
        base_transform: &Mat4,
    ) -> AnimTransformBoneFrame {
        let effective_frame = effective_frame(frame, self.flags, self.end_frame);

        let (mut scale, qrot, mut translation) = base_transform.to_scale_rotation_translation();
        let (rz, ry, rx) = qrot.to_euler(glam::EulerRot::ZYX);
        let mut euler_rotation = Vec3 { x: rx, y: ry, z: rz };

        use TrackTypeBone::*;
        for track in self.tracks.iter() {
            if effective_frame < track.start_frame { continue; }

            let val: f32 = track.get_value(effective_frame - track.start_frame);

            match track.track_type {
                // joint
                RotateX => euler_rotation.x = val,
                RotateY => euler_rotation.y = val,
                RotateZ => euler_rotation.z = val,
                TranslateX => translation.x = val,
                TranslateY => translation.y = val,
                TranslateZ => translation.z = val,
                ScaleX => scale.x = val,
                ScaleY => scale.y = val,
                ScaleZ => scale.z = val,
            }
        }

        let rotation = Quat::from_euler(
            glam::EulerRot::ZYX,
            euler_rotation.z,
            euler_rotation.y,
            euler_rotation.x,
        );

        let transform = Mat4::from_scale_rotation_translation(scale, rotation, translation);
        AnimTransformBoneFrame { transform }
    }
}

impl AnimTransformMaterial {
    pub fn compute_transform_at(
        &self, 
        frame: f32, 
        base_phong: Phong,
    ) -> AnimTransformMaterialFrame {
        let mut phong: PhongF32 = base_phong.into();

        let effective_frame_material = effective_frame(frame, self.material_flags, self.material_end_frame);

        for track in self.material_tracks.iter() {
            use TrackTypeMaterial::*;

            let val: f32 = track.get_value(effective_frame_material + track.start_frame);

            match track.track_type {
                AmbientR  => phong.ambient[0]  = val,
                AmbientG  => phong.ambient[1]  = val,
                AmbientB  => phong.ambient[2]  = val,
                DiffuseR  => phong.diffuse[0]  = val,
                DiffuseG  => phong.diffuse[1]  = val,
                DiffuseB  => phong.diffuse[2]  = val,
                SpecularR => phong.specular[0] = val,
                SpecularG => phong.specular[1] = val,
                SpecularB => phong.specular[2] = val,
                Alpha => {
                    phong.ambient[3]  = val;
                    phong.diffuse[3]  = val;
                    phong.specular[3] = val;
                }
            }
        }

        let mut tex_t = TexTransform::default();

        let effective_frame_texture = effective_frame(frame, self.texture_flags, self.texture_end_frame);

        for track in self.texture_tracks.iter() {
            use TrackTypeTexture::*;

            let val: f32 = track.get_value(effective_frame_texture + track.start_frame);

            match track.track_type {
                TraU   => tex_t.uv_translation[0] = val,
                TraV   => tex_t.uv_translation[1] = val,
                ScaU   => tex_t.uv_scale[0]       = val,
                ScaV   => tex_t.uv_scale[1]       = val,
                KonstR => tex_t.konst_colour[0]   = val,
                KonstG => tex_t.konst_colour[1]   = val,
                KonstB => tex_t.konst_colour[2]   = val,
                KonstA => tex_t.konst_colour[3]   = val,
                Tev0R  => tex_t.tex_colour[0]     = val,
                Tev0G  => tex_t.tex_colour[1]     = val,
                Tev0B  => tex_t.tex_colour[2]     = val,
                Tev0A  => tex_t.tex_colour[3]     = val,
            }
        }

        AnimTransformMaterialFrame { phong, tex_transform: tex_t }
    }
}

fn extract_figatree_transforms(figatree: FigaTree) -> Vec<AnimTransformBone> {
    let mut transforms = Vec::new();

    let end_frame = figatree.frame_count();

    // The skeleton bone array is depth-first (SBSkeleton.cs:48)
    // and the flat list of bones corresponds with that access method (IO_HSDAnim.cs:76).
    for (bone_index, node) in figatree.get_nodes().iter().enumerate() {
        let mut tracks = Vec::new();
        for track in node.tracks.iter() {
            let data = hsd_track_data(track);
            tracks.push(decode_anim_data(data))
        }

        let transform = AnimTransformBone {
            tracks: tracks.into_boxed_slice(),
            flags: 0,
            end_frame,
            bone_index,
        };

        transforms.push(transform);
    }

    transforms
}

// BinaryReaderExt.cs:249
fn read_packed(stream: &mut Stream<'_>) -> u16 {
    let a = stream.read_byte() as u16;
    if (a & 0x80) != 0 {
        let b = stream.read_byte() as u16;
        (a & 0x7F) | (b << 7)
    } else {
        a
    }
}

fn parse_float(stream: &mut Stream<'_>, format: AnimDataFormat, scale: f32) -> f32 {
    // not big endian for some reason...

    use AnimDataFormat::*;
    match format {
        Float => {
            let bytes = stream.read_const_bytes::<4>();
            f32::from_le_bytes(bytes)
        },
        I16 => {
            let bytes = stream.read_const_bytes::<2>();
            let n = i16::from_le_bytes(bytes);
            n as f32 / scale
        },
        U16 => {
            let bytes = stream.read_const_bytes::<2>();
            let n = u16::from_le_bytes(bytes);
            n as f32 / scale
        },
        I8 => {
            let n = stream.read_byte() as i8;
            n as f32 / scale
        },
        U8 => {
            let n = stream.read_byte();
            n as f32 / scale
        },
    }
}

pub struct TrackOrFOBJData<'a, T: TrackType> {
    pub track_type: T,
    pub data: &'a [u8],
    pub value_scale: f32,
    pub tan_scale: f32,
    pub value_format: AnimDataFormat,
    pub tan_format: AnimDataFormat,
    pub start_frame: f32,
}

pub fn fobj_desc_data<'a, T: TrackType>(fobj_desc: &HSDStruct<'a>) -> Option<TrackOrFOBJData<'a, T>> {
    let value_flag = fobj_desc.get_u8(0x0D);
    let tan_flag = fobj_desc.get_u8(0x0E);
    let value_scale = (1 << (value_flag & 0x1F)) as f32;
    let tan_scale = (1 << (tan_flag & 0x1F)) as f32;
    let value_format = AnimDataFormat::from_u8(value_flag & 0xE0);
    let tan_format = AnimDataFormat::from_u8(tan_flag & 0xE0);

    let start_frame = fobj_desc.get_f32(0x08);

    let track_type = T::from_u8(fobj_desc.get_u8(0x0C))?;

    Some(TrackOrFOBJData {
        track_type,
        data: fobj_desc.get_buffer(0x10),
        value_scale,
        tan_scale,
        value_format,
        tan_format,
        start_frame,
    })
}

pub fn hsd_track_data<'a>(track: &Track<'a>) -> TrackOrFOBJData<'a, TrackTypeBone> {
    TrackOrFOBJData {
        track_type: track.track_type(),
        data: track.hsd_struct.get_reference(0x08).data,
        value_scale: track.value_scale(),
        tan_scale: track.tan_scale(),
        value_format: track.value_format(),
        tan_format: track.tan_format(),
        start_frame: track.start_frame(),
    }
}

pub fn decode_anim_data<T: TrackType>(track: TrackOrFOBJData<'_, T>) -> AnimTrack<T> {
    let mut buffer = Stream::new(track.data);
    let stream = &mut buffer;
    let mut frame: f32 = 0.0;

    let value_scale = track.value_scale;
    let tan_scale = track.tan_scale;
    let value_format = track.value_format;
    let tan_format = track.tan_format;

    let mut keys = Vec::new();

    // Tools/FOBJ_Decoder.cs:55 (GetKeys)
    while !stream.finished() {
        let typ = read_packed(stream);
        
        let interp_type = typ & 0x0F;
        if interp_type == 0x00 { break }
        let num_keys = (typ >> 4) + 1;

        for _ in 0..num_keys {
            match interp_type {
                0x01 => {
                    let value = parse_float(stream, value_format, value_scale);
                    keys.push(Key {
                        frame,
                        value,
                        interpolation: InterpolationType::Step,
                        in_tan: 0.0,
                        out_tan: 0.0,
                    });
                    frame += read_packed(stream) as f32;
                }
                0x02 => {
                    let value = parse_float(stream, value_format, value_scale);
                    keys.push(Key {
                        frame,
                        value,
                        interpolation: InterpolationType::Linear,
                        in_tan: 0.0,
                        out_tan: 0.0,
                    });
                    frame += read_packed(stream) as f32;
                }
                0x03 => { // SPL0
                    let value = parse_float(stream, value_format, value_scale);
                    keys.push(Key {
                        frame,
                        value,
                        interpolation: InterpolationType::Hermite,
                        in_tan: 0.0,
                        out_tan: 0.0,
                    });
                    frame += read_packed(stream) as f32;
                }
                0x04 => { // SPL
                    // NOT SURE ABOUT THIS, BUT I TRIED A LOT AND THIS LOOKED THE BEST
                    // in_tan might not be 'tan'
                    let value = parse_float(stream, value_format, value_scale);
                    let tan = parse_float(stream, tan_format, tan_scale);
                    keys.push(Key {
                        frame,
                        value,
                        interpolation: InterpolationType::Hermite,
                        in_tan: tan,
                        out_tan: tan,
                    });
                    frame += read_packed(stream) as f32;
                }
                0x05 => { // SLP
                    let tan = parse_float(stream, tan_format, tan_scale);
                    keys.last_mut().unwrap().out_tan = tan;
                }
                0x06 => { // not used so far
                    eprintln!("unused key frame!");
                    parse_float(stream, value_format, value_scale);
                    continue;
                }
                _ => panic!(),
            };
        }
    }

    AnimTrack {
        start_frame: track.start_frame,
        keys: keys.into_boxed_slice(),
        track_type: track.track_type,
    }
}

fn lerp(av: f32, bv: f32, v0: f32, v1: f32, t: f32) -> f32 {
    if v0 == v1 { return av };

    if t == v0 { return av };
    if t == v1 { return bv };

    let mu = (t - v0) / (v1 - v0);
    (av * (1.0 - mu)) + (bv * mu)
}

fn hermite(frame: f32, frame_left: f32, frame_right: f32, ls: f32, rs: f32, lhs: f32, rhs: f32) -> f32 {
    let frame_diff = frame - frame_left;
    let weight = frame_diff / (frame_right - frame_left);

    let mut result = lhs + (lhs - rhs) * (2.0 * weight - 3.0) * weight * weight;
    result += (frame_diff * (weight - 1.0)) * (ls * (weight - 1.0) + rs * weight);

    result
}

impl<T: TrackType> AnimTrack<T> {
    // SBKeyGroup.cs:107
    pub fn get_value(&self, frame: f32) -> f32 {
        if self.keys.len() == 1 {
            return self.keys[0].value;
        }

        let left = self.binary_search_keys(frame);
        let right = left + 1;

        if right >= self.keys.len() {
            return self.keys[left].value
        }

        match self.keys[left].interpolation {
            InterpolationType::Step | InterpolationType::Constant => {
                self.keys[left].value
            },
            InterpolationType::Linear => {
                let left_value = self.keys[left].value;
                let right_value = self.keys[right].value;
                let left_frame = self.keys[left].frame;
                let right_frame = self.keys[right].frame;

                let mut value = lerp(left_value, right_value, left_frame, right_frame, frame);

                if value.is_nan() {
                    value = 0.0 // Occurs occasionally, what StudioSB does
                }

                value
            },
            InterpolationType::Hermite => {
                let left_value = self.keys[left].value;
                let right_value = self.keys[right].value;
                let left_tan = self.keys[left].out_tan;
                let right_tan = self.keys[right].in_tan;
                let left_frame = self.keys[left].frame;
                let right_frame = self.keys[right].frame;

                let mut value = hermite(frame, left_frame, right_frame, left_tan, right_tan, left_value, right_value);

                if value.is_nan() {
                    value = 0.0 // Occurs occasionally, what StudioSB does
                }

                value
            }
        }
    }

    // SBKeyGroup.cs:80
    pub fn binary_search_keys(&self, frame: f32) -> usize {
        let mut lower: isize = 0;
        let mut upper: isize = self.keys.len() as isize - 1;

        while lower <= upper {
            let middle = (upper + lower) / 2;
            let mid_usize = middle as usize;
            if frame == self.keys[mid_usize].frame {
                return mid_usize;
            } else if frame < self.keys[mid_usize].frame {
                upper = middle - 1;
            } else {
                lower = middle + 1;
            }
        }
        
        lower.min(upper).max(0) as usize
    }
}

#[derive(Copy, Clone, Debug)]
pub enum AnimDataFormat {
    Float,
    I16,
    U16,
    I8,
    U8,
}

impl AnimDataFormat {
    pub fn from_u8(n: u8) -> Self {
        match n {
            0x00 => AnimDataFormat::Float,
            0x20 => AnimDataFormat::I16,
            0x40 => AnimDataFormat::U16,
            0x60 => AnimDataFormat::I8,
            0x80 => AnimDataFormat::U8,
            _ => panic!()
        }
    }
}

impl<'a> Track<'a> {
    pub fn track_type(&self) -> TrackTypeBone {
        TrackTypeBone::from_u8(self.hsd_struct.get_u8(0x04)).unwrap()
    }

    pub fn value_flag(&self) -> u8 {
        // Look in HSD_Track, not FOBJ!
        self.hsd_struct.get_i8(0x05) as u8
    }

    pub fn tan_flag(&self) -> u8 {
        // Look in HSD_Track, not FOBJ!
        self.hsd_struct.get_i8(0x06) as u8
    }

    pub fn value_scale(&self) -> f32 {
        (1 << (self.value_flag() & 0x1F)) as f32
    }

    pub fn tan_scale(&self) -> f32 {
        (1 << (self.tan_flag() & 0x1F)) as f32
    }

    pub fn start_frame(&self) -> f32 {
        self.hsd_struct.get_u16(0x02) as f32
    }

    pub fn value_format(&self) -> AnimDataFormat {
        AnimDataFormat::from_u8(self.value_flag() & 0xE0)
    }

    pub fn tan_format(&self) -> AnimDataFormat {
        AnimDataFormat::from_u8(self.tan_flag() & 0xE0)
    }
}

impl<'a> FigaTree<'a> {
    pub fn new(hsd_struct: HSDStruct<'a>) -> Self {
        Self {
            hsd_struct
        }
    }

    pub fn frame_count(&self) -> f32 {
        self.hsd_struct.get_f32(0x08)
    }

    pub fn get_nodes<'b>(&'b self) -> Box<[FigaTreeNode<'a>]> {
        let track_info = self.hsd_struct.get_reference(0x0C);
        let track_data = self.hsd_struct.get_reference(0x10);

        let mut offset = 0;

        let node_count = track_info.data.iter().take_while(|&&b| b != 0xFF).count();
        let mut nodes = Vec::with_capacity(node_count);

        for track_count in track_info.data.iter().copied() {
            if track_count == 0xFF { break }

            let mut tracks = Vec::with_capacity(track_count as usize);
            for j in 0..track_count as usize {
                let track_index = offset + j;
                let track = track_data.get_embedded_struct(track_index * 0x0C, 0x0C);
                let track_type = track.get_u8(0x04);
                if TrackTypeBone::from_u8(track_type) == None {
                    println!("unused figatree track {}", track_type);
                    continue;
                }
                tracks.push(Track { hsd_struct: track });
            }

            nodes.push(FigaTreeNode { tracks: tracks.into_boxed_slice() });

            offset += track_count as usize;
        }

        nodes.into_boxed_slice()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TrackTypeBone {
    RotateX,
    RotateY,
    RotateZ,
    TranslateX,
    TranslateY,
    TranslateZ,
    ScaleX,
    ScaleY,
    ScaleZ,
    //BRANCH,
    //PTCL,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TrackTypeMaterial {
    AmbientR,
    AmbientG,
    AmbientB,
    DiffuseR,
    DiffuseG,
    DiffuseB,
    SpecularR,
    SpecularG,
    SpecularB,
    Alpha,
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TrackTypeTexture {
    //TImg, //
    TraU,
    TraV,
    ScaU, //
    ScaV,
    //RotX,
    //RotY,
    //RotZ,
    //Blend, //
    //TCLT = 10,
    //LOD_BIAS = 11,
    KonstR, //
    KonstG,
    KonstB,
    KonstA, //
    Tev0R,
    Tev0G,
    Tev0B,
    Tev0A,
    //TEV1_R = 20,
    //TEV1_G = 21,
    //TEV1_B = 22,
    //TEV1_A = 23,
    //TS_BLEND = 24
}

impl TrackType for TrackTypeBone {
    fn from_u8(n: u8) -> Option<Self> {
        use TrackTypeBone::*;
        
        // HSD_FOBJ.cs:90 (JointTrackType)
        Some(match n {
            1 => RotateX,
            2 => RotateY,
            3 => RotateZ,
            5 => TranslateX,
            6 => TranslateY,
            7 => TranslateZ,
            8 => ScaleX,
            9 => ScaleY,
            10 => ScaleZ,
            
            // IO_HSDAnims.cs:239 (DecodeFOBJ)
            // HACK - reproduces strange (buggy?) behaviour in StudioSB
            // Node case not covered, TranslateX is the default.
            11 => TranslateX, 

            //12 => BRANCH,
            //40 => PTCL,
            _ => return None,
        })
    }
}

impl TrackType for TrackTypeMaterial {
    fn from_u8(n: u8) -> Option<Self> {
        use TrackTypeMaterial::*;
        
        // HSD_FOBJ.cs:16 (MatTrackType)
        Some(match n {
            1 => AmbientR,
            2 => AmbientG,
            3 => AmbientB,
            4 => DiffuseR,
            5 => DiffuseG,
            6 => DiffuseB,
            7 => SpecularR,
            8 => SpecularG,
            9 => SpecularB,
            10 => Alpha,
            _ => {
                println!("skipping material track {}", n);
                return None;
            }
        })
    }       
}           

impl TrackType for TrackTypeTexture {
    fn from_u8(n: u8) -> Option<Self> {
        use TrackTypeTexture::*;
        
        // HSD_FOBJ.cs:34 (TexTrackType)
        Some(match n {
            2 => TraU,
            3 => TraV,
            4 => ScaU, //
            5 => ScaV,
            12 => KonstR,
            13 => KonstG,
            14 => KonstB,
            15 => KonstA,
            16 => Tev0R,
            17 => Tev0G,
            18 => Tev0B,
            19 => Tev0A,
            _ => {
                //println!("skipping texture track {}", n);
                return None;
            }
        })
    }       
}           
