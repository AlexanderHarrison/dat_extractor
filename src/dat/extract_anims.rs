use crate::dat::*;

use glam::f32::{Quat, Mat4, Vec3, Vec4};

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
    pub animated_transforms: Box<[Mat4]>,
    pub animated_world_transforms: Box<[Mat4]>,
    pub animated_bind_transforms: Box<[Mat4]>,
    
    pub animated_world_inv_transforms: Box<[Mat4]>,
    pub animated_bind_inv_transforms: Box<[Mat4]>,
}

impl AnimationFrame {
    pub fn new_default_pose(model: &Model) -> Self {
        let bone_len = model.base_transforms.len();
        let animated_transforms = model.base_transforms.to_vec().into_boxed_slice();
        let mut animated_world_transforms = Vec::with_capacity(bone_len);
        let mut animated_bind_transforms = Vec::with_capacity(bone_len);
        let mut animated_world_inv_transforms = Vec::with_capacity(bone_len);
        let mut animated_bind_inv_transforms = Vec::with_capacity(bone_len);

        for (i, base_transform) in model.base_transforms.iter().enumerate() {
            let world_transform = match model.bones[i].parent {
                Some(p_i) => animated_world_transforms[p_i as usize] * *base_transform,
                None => *base_transform
            };

            animated_world_transforms.push(world_transform);
            animated_world_inv_transforms.push(world_transform.inverse());
            let bind_transform = world_transform * model.inv_world_transforms[i];
            animated_bind_transforms.push(bind_transform);
            animated_bind_inv_transforms.push(bind_transform.inverse());
        }

        AnimationFrame {
            animated_transforms,
            animated_world_transforms: animated_world_transforms.into_boxed_slice(),
            animated_bind_transforms: animated_bind_transforms.into_boxed_slice(),
            animated_world_inv_transforms: animated_world_inv_transforms.into_boxed_slice(),
            animated_bind_inv_transforms: animated_bind_inv_transforms.into_boxed_slice(),
        }
    }

    pub fn default_pose(&mut self, model: &Model) {
        self.animated_transforms.copy_from_slice(&model.base_transforms);

        for (i, base_transform) in model.base_transforms.iter().enumerate() {
            let world_transform = match model.bones[i].parent {
                Some(p_i) => self.animated_world_transforms[p_i as usize] * *base_transform,
                None => *base_transform
            };

            self.animated_world_transforms[i] = world_transform;
            self.animated_world_inv_transforms[i] = world_transform.inverse();
            let bind_transform = world_transform * model.inv_world_transforms[i];
            self.animated_bind_transforms[i] = bind_transform;
            self.animated_bind_inv_transforms[i] = bind_transform.inverse();
        }
    }

    pub fn custom(&mut self, model: &Model, updates: &[(u32, Mat4)]) {
        for (bone, mat) in updates.iter().copied() {
            let bone = bone as usize;
            self.animated_transforms[bone] = mat;
        }

        for (i, animated_transform) in self.animated_transforms.iter().copied().enumerate() {

            let animated_world_transform = match model.bones[i].parent {
                Some(p_i) => self.animated_world_transforms[p_i as usize] * animated_transform,
                None => animated_transform
            };

            //let animated_world_transform = world_transform * animated_transform;

            self.animated_world_transforms[i] = animated_world_transform;
            self.animated_world_inv_transforms[i] = animated_world_transform.inverse();
            let animated_bind_transform = animated_world_transform * model.inv_world_transforms[i];
            self.animated_bind_transforms[i] = animated_bind_transform;
            self.animated_bind_inv_transforms[i] = animated_bind_transform.inverse();
        }
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

pub type AnimFlags = u32;
pub mod anim_flags {
    use super::AnimFlags;

    // TODO
    pub const LOOP: AnimFlags = 1 << 29;
}

#[derive(Clone, Debug)]
pub struct Animation {
    pub name: Box<str>,
    pub transforms: Box<[AnimTransform]>,
    pub flags: AnimFlags,
    pub end_frame: f32,
}

#[derive(Clone, Debug)]
pub struct AnimTransform {
    pub tracks: Box<[AnimTrack]>,
    pub bone_index: usize,
}

#[derive(Clone, Debug)]
pub struct AnimTrack {
    pub start_frame: f32,
    pub track_type: TrackType,
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
struct AnimState {
    pub t0: f32,
    pub t1: f32,
    pub p0: f32,
    pub p1: f32,
    pub d0: f32,
    pub d1: f32,
    pub _op: MeleeInterpolationType,
    pub op_intrp: MeleeInterpolationType, // idk
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Copy, Clone, Debug)]
pub enum MeleeInterpolationType {
    //NONE,
    CON  { value: f32, time: u16 },
    LIN  { value: f32, time: u16 },
    SPL0 { value: f32, time: u16 },
    SPL  { value: f32, tan: f32, time: u16 },
    SLP  { tan: f32 },
    KEY  { value: f32 },
}

#[derive(Copy, Clone, Debug)]
pub enum InterpolationType {
    Constant,
    Linear,
    Hermite,
    Step
}

/// HSD_AnimJoint -> Animation
pub fn parse_joint_anim(joint_anim_joint: HSDStruct<'_>) -> Option<Animation> {
    let mut joint_anims: Vec<AnimTransform> = Vec::new();
    
    let mut flags = 0;
    let mut end_frame = 0.0;

    // HSD_AnimJoint
    for (i, joint_anim_joint) in joint_anim_joint.iter_joint_tree(0x00, 0x04).enumerate() {
        let aobj = match joint_anim_joint.try_get_reference(0x08) {
            Some(aobj) => aobj,
            None => continue,
        };

        // anim data are expanded to full animation, so make sure it doesn't have different start/end frames, flags, etc.
        // not sure how to handle if this isn't the case. hopefully it's fine
        let new_flags = aobj.get_u32(0x00);
        assert!(flags == 0 || flags == new_flags);
        flags = new_flags;
        
        let new_end_frame = aobj.get_f32(0x04);
        assert!(end_frame == 0.0 || end_frame == new_end_frame);
        end_frame = new_end_frame;

        let fobj_desc = aobj.get_reference(0x08);

        let mut tracks = Vec::new();

        for fobj_desc in fobj_desc.iter_joint_list(0x00) {
            let track_type = match TrackType::from_u8(fobj_desc.get_u8(0x0C)) {
                Some(t) => t,
                None => {
                    eprintln!("no track type");
                    continue;
                }
            };
            if track_type == TrackType::PTCL || track_type == TrackType::BRANCH {
                continue;
            }

            let track = decode_anim_data(fobj_desc_data(&fobj_desc));
            tracks.push(track);
        }

        joint_anims.push(AnimTransform {
            tracks: tracks.into_boxed_slice(),
            bone_index: i,
        });
    }

    if joint_anims.len() == 0 {
        None
    } else {
        Some(Animation {
            name: String::new().into_boxed_str(),
            transforms: joint_anims.into_boxed_slice(),
            end_frame,
            flags,
        })
    }
}

// TODO
/// HSD_MatAnimJoint -> Animation
//pub fn parse_material_anim(material_anim_joint: HSDStruct<'_>) -> Option<Animation> {
//    let mut joint_anims: Vec<AnimTransform> = Vec::new();
//    
//    let mut flags = 0;
//    let mut end_frame = 0.0;
//
//    // HSD_AnimJoint
//    for (i, joint_anim_joint) in joint_anim_joint.iter_joint_tree(0x00, 0x04).enumerate() {
//        let aobj = match joint_anim_joint.try_get_reference(0x08) {
//            Some(aobj) => aobj,
//            None => continue,
//        };
//
//        // anim data are expanded to full animation, so make sure it doesn't have different start/end frames, flags, etc.
//        // not sure how to handle if this isn't the case. hopefully it's fine
//        let new_flags = aobj.get_u32(0x00);
//        assert!(flags == 0 || flags == new_flags);
//        flags = new_flags;
//        
//        let new_end_frame = aobj.get_f32(0x04);
//        assert!(end_frame == 0.0 || end_frame == new_end_frame);
//        end_frame = new_end_frame;
//
//        let fobj_desc = aobj.get_reference(0x08);
//
//        let mut tracks = Vec::new();
//
//        for fobj_desc in fobj_desc.iter_joint_list(0x00) {
//            let track_type = TrackType::from_u8(fobj_desc.get_u8(0x0C)).unwrap();
//            if track_type == TrackType::PTCL || track_type == TrackType::BRANCH {
//                continue;
//            }
//
//            let track = decode_anim_data(fobj_desc_data(&fobj_desc));
//            tracks.push(track);
//        }
//
//        joint_anims.push(AnimTransform {
//            tracks: tracks.into_boxed_slice(),
//            bone_index: i,
//        });
//    }
//
//    if joint_anims.len() == 0 {
//        None
//    } else {
//        Some(Animation {
//            name: String::new().into_boxed_str(),
//            transforms: joint_anims.into_boxed_slice(),
//            end_frame,
//            flags,
//        })
//    }
//}

/// Pass in the raw data of the animations file - Pl*AJ.dat
/// That is necessary (for now).
pub fn extract_anims(
    aj_dat: &DatFile,
    actions: Box<[FighterAction]>
) -> Result<Box<[Animation]>, DatExtractError> {
    let mut animations: Vec<Animation> = Vec::with_capacity(actions.len());

    for action in actions.to_vec().into_iter() {
        let offset = action.animation_offset;
        let size = action.animation_size;
        let anim_data = &aj_dat.data[offset..offset+size];

        // TODO might be discarding some animations??
        if let Some(name) = action.name {
            if animations.iter().all(|a| *a.name != *name) {
                let stream = Stream::new(anim_data);
                let hsd_file = HSDRawFile::open(stream);

                // likely no other roots
                let anim_root = &hsd_file.roots[0];
                assert!(anim_root.root_string.contains("figatree"));

                let figatree = FigaTree::new(anim_root.hsd_struct.clone());

                let frame_count = figatree.frame_count();

                let animation = Animation {
                    name,
                    transforms: extract_anim_transforms(figatree),
                    end_frame: frame_count,
                    flags: 0,
                };

                animations.push(animation);
            }
        }
    }

    Ok(animations.into_boxed_slice())
}

impl Animation {
    pub fn frame_at(
        &self, 
        mut frame_num: f32, 
        prev_frame: &mut AnimationFrame,
        model: &Model,
        prev_end_frame: Option<&[Mat4]>,
        frames_in_anim: usize,
        remove_animation_translation: bool,
    ) {
        if self.flags & anim_flags::LOOP != 0 {
            while frame_num > self.end_frame {
                frame_num -= self.end_frame
            }
        }

        for transform in self.transforms.iter() {
            let bone_index = transform.bone_index;
            let base = &model.base_transforms[bone_index];
            prev_frame.animated_transforms[bone_index] = transform.compute_transform_at(frame_num, base);
        }

        if let Some(end_frame) = prev_end_frame {
            if frames_in_anim <= 3 {
                for transform in self.transforms.iter() {
                    let bone_index = transform.bone_index;

                    let base = &end_frame[bone_index];
                    let (base_scale, base_qrot, base_translation) = base.to_scale_rotation_translation();

                    let new = &prev_frame.animated_transforms[bone_index];
                    let (new_scale, new_qrot, new_translation) = new.to_scale_rotation_translation();

                    let scale = 0.8 + 0.05*frame_num;
                    let lerp_scale = base_scale.lerp(new_scale, scale);
                    let slerp_qrot = base_qrot.slerp(new_qrot, scale);
                    let lerp_translation = base_translation.lerp(new_translation, scale);

                    prev_frame.animated_transforms[bone_index] = 
                        Mat4::from_scale_rotation_translation(lerp_scale, slerp_qrot, lerp_translation);
                }
            } 
        }

        if remove_animation_translation {
            // Remove translation from root jobj.
            // This is given by slippi recording.
            prev_frame.animated_transforms[1].w_axis = Vec4::new(0.0, 0.0, 0.0, 1.0);
        }

        for (i, animated_transform) in prev_frame.animated_transforms.iter().enumerate() {
            let animated_world_transform = match model.bones[i].parent {
                Some(p_i) => prev_frame.animated_world_transforms[p_i as usize] * *animated_transform,
                None => *animated_transform
            };

            prev_frame.animated_world_transforms[i] = animated_world_transform;
            prev_frame.animated_world_inv_transforms[i] = animated_world_transform.inverse();
            let animated_bind_transform = animated_world_transform * model.inv_world_transforms[i];
            prev_frame.animated_bind_transforms[i] = animated_bind_transform;
            prev_frame.animated_bind_inv_transforms[i] = animated_bind_transform.inverse();
        }
    }
}

impl AnimTransform {
    // SBTransformAnimation.cs:86 (GetTransformAt)
    pub fn compute_transform_at(&self, frame: f32, base_transform: &Mat4) -> Mat4 {
        // TODO RotationOnly ????????? (SBAnimation.cs:80)

        let (mut scale, qrot, mut translation) = base_transform.to_scale_rotation_translation();
        let (rz, ry, rx) = qrot.to_euler(glam::EulerRot::ZYX);
        let mut euler_rotation = Vec3 { x: rx, y: ry, z: rz };

        use TrackType::*;
        for track in self.tracks.iter() {
            let val: f32 = track.get_value(frame + track.start_frame);

            match track.track_type {
                RotateX => euler_rotation.x = val,
                RotateY => euler_rotation.y = val,
                RotateZ => euler_rotation.z = val,
                TranslateX => translation.x = val,
                TranslateY => translation.y = val,
                TranslateZ => translation.z = val,
                ScaleX => scale.x = val,
                ScaleY => scale.y = val,
                ScaleZ => scale.z = val,
                PTCL | BRANCH => todo!(),
            }
        }

        let rotation = Quat::from_euler(
            glam::EulerRot::ZYX,
            euler_rotation.z,
            euler_rotation.y,
            euler_rotation.x,
        );

        Mat4::from_scale_rotation_translation(scale, rotation, translation)
    }
}

fn extract_anim_transforms(figatree: FigaTree) -> Box<[AnimTransform]> {
    let mut transforms = Vec::new();

    // I pray that bone_index is correct here...
    // It looks like the skeleton bone array is depth-first (SBSkeleton.cs:48)
    // and the flat list of nodes here corresponds with that access method (IO_HSDAnim.cs:76).
    // Not obvious.
    for (i, node) in figatree.get_nodes().iter().enumerate() {
        let mut tracks = Vec::new();
        for track in node.tracks.iter() {
            let data = hsd_track_data(track);
            tracks.push(decode_anim_data(data))
        }

        let transform = AnimTransform {
            tracks: tracks.into_boxed_slice(),
            bone_index: i,
        };

        transforms.push(transform);
    }

    transforms.into_boxed_slice()
}

// no clue what this does.
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

pub struct TrackOrFOBJData<'a> {
    pub track_type: TrackType,
    pub data: &'a [u8],
    pub value_scale: f32,
    pub tan_scale: f32,
    pub value_format: AnimDataFormat,
    pub tan_format: AnimDataFormat,
    pub start_frame: f32,
}

pub fn fobj_desc_data<'a>(fobj_desc: &HSDStruct<'a>) -> TrackOrFOBJData<'a> {
    let value_flag = fobj_desc.get_u8(0x0D);
    let tan_flag = fobj_desc.get_u8(0x0E);
    let value_scale = (1 << (value_flag & 0x1F)) as f32;
    let tan_scale = (1 << (tan_flag & 0x1F)) as f32;
    let value_format = AnimDataFormat::from_u8(value_flag & 0xE0);
    let tan_format = AnimDataFormat::from_u8(tan_flag & 0xE0);

    let start_frame = fobj_desc.get_f32(0x08);

    let track_type = TrackType::from_u8(fobj_desc.get_u8(0x0C)).unwrap();

    TrackOrFOBJData {
        track_type,
        data: fobj_desc.get_buffer(0x10),
        value_scale,
        tan_scale,
        value_format,
        tan_format,
        start_frame,
    }
}

pub fn hsd_track_data<'a>(track: &Track<'a>) -> TrackOrFOBJData<'a> {
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

pub fn decode_anim_data(track: TrackOrFOBJData<'_>) -> AnimTrack {
    let track_type = track.track_type;

    if track_type == TrackType::PTCL { 
        return AnimTrack {
            start_frame: track.start_frame,
            track_type,
            keys: Box::new([]),
        }
    }

    let mut buffer = Stream::new(track.data);
    let stream = &mut buffer;
    let mut clock: f32 = 0.0;

    let value_scale = track.value_scale;
    let tan_scale = track.tan_scale;
    let value_format = track.value_format;
    let tan_format = track.tan_format;

    let mut melee_keys = Vec::new();

    // FOBJ_Decoder.cs:55 (GetKeys)
    while !stream.finished() {
        let typ = read_packed(stream);
        
        let interp_type = typ & 0x0F;
        if interp_type == 0x00 { break }
        let num_keys = (typ >> 4) + 1;
        
        for _ in 0..num_keys {
            let mut time = 0;

            let interpolation = match interp_type {
                0x01 => {
                    let value = parse_float(stream, value_format, value_scale);
                    time = read_packed(stream);
                    MeleeInterpolationType::CON { value, time }
                }
                0x02 => {
                    let value = parse_float(stream, value_format, value_scale);
                    time = read_packed(stream);
                    MeleeInterpolationType::LIN { value, time }
                }
                0x03 => {
                    let value = parse_float(stream, value_format, value_scale);
                    time = read_packed(stream);
                    MeleeInterpolationType::SPL0 { value, time }
                }
                0x04 => {
                    let value = parse_float(stream, value_format, value_scale);
                    let tan = parse_float(stream, tan_format, tan_scale);
                    time = read_packed(stream);
                    MeleeInterpolationType::SPL { value, tan, time }
                }
                0x05 => {
                    let tan = parse_float(stream, tan_format, tan_scale);
                    MeleeInterpolationType::SLP { tan }
                }
                0x06 => {
                    let value = parse_float(stream, value_format, value_scale);
                    MeleeInterpolationType::KEY { value }
                }
                _ => panic!(),
            };

            melee_keys.push((clock, interpolation));

            clock += time as f32;
        }
    }
    
    // TODO fix something about animations that don't start on frame 1?
    // FOBJ_Decoder.cs:110

    // I HAVE NO IDEA WHAT THIS DOES
    // IO_HSDAnim.cs:259-293
    let mut keys = Vec::with_capacity(melee_keys.len());

    let mut prev_state: Option<AnimState> = None;
    for i in 0..melee_keys.len() {
        let (frame, interpolation) = melee_keys[i];
        let mut state = get_state(&melee_keys, frame);
        let next_slope = i+1 < melee_keys.len() && matches!(melee_keys[i+1].1, MeleeInterpolationType::SLP { .. } );

        if frame == state.t1 {
            state.t0 = state.t1;
            state.p0 = state.p1;
            state.d0 = state.d1;
        }

        if matches!(interpolation, MeleeInterpolationType::SLP { .. }) {
            continue
        }

        let key = match state.op_intrp {
            //MeleeInterpolationType::NONE => panic!(),
            MeleeInterpolationType::CON { .. } | MeleeInterpolationType::KEY { .. } => Key {
                frame: state.t0,
                value: state.p0,
                interpolation: InterpolationType::Step,
                in_tan: 0.0,
                out_tan: 0.0,
            },

            MeleeInterpolationType::LIN { .. } => Key {
                frame: state.t0,
                value: state.p0,
                interpolation: InterpolationType::Linear,
                in_tan: 0.0,
                out_tan: 0.0,
            },

            MeleeInterpolationType::SPL { .. } 
                | MeleeInterpolationType::SPL0 { .. }
                | MeleeInterpolationType::SLP { .. } => 
            {
                let in_tan = match prev_state {
                    Some(s) if next_slope => s.d1,
                    _ => state.d0
                };

                Key {
                    frame: state.t0,
                    value: state.p0,
                    interpolation: InterpolationType::Hermite,
                    in_tan,
                    out_tan: state.d0,
                }
            }
        };
        
        keys.push(key);
        prev_state = Some(state);
    }

    AnimTrack {
        start_frame: track.start_frame,
        keys: keys.into_boxed_slice(),
        track_type,
    }
}

fn get_state(keys: &[(f32, MeleeInterpolationType)], frame: f32) -> AnimState {
    let mut t0 = 0.0;
    let mut t1 = 0.0;
    let mut p0 = 0.0;
    let mut p1 = 0.0;
    let mut d0 = 0.0;
    let mut d1 = 0.0;

    let mut op = MeleeInterpolationType::CON { time: 0, value: 0.0 };
    let mut op_intrp = MeleeInterpolationType::CON { time: 0, value: 0.0 };

    for (kframe, interpolation) in keys.iter().copied() {
        op_intrp = op;
        op = interpolation;

        match op {
            //MeleeInterpolationType::NONE => (),
            MeleeInterpolationType::CON { value, .. } | MeleeInterpolationType::LIN { value, ..} => {
                p0 = p1;
                p1 = value;
                if !matches!(op_intrp, MeleeInterpolationType::SLP { .. }) {
                    d0 = d1;
                    d1 = 0.0;
                }
                t0 = t1;
                t1 = kframe
            }
            MeleeInterpolationType::SPL0 { value, .. } => {
                p0 = p1;
                d0 = d1;
                p1 = value;
                d1 = 0.0;
                t0 = t1;
                t1 = kframe;
            }
            MeleeInterpolationType::SPL { value, tan, .. } => {
                p0 = p1;
                p1 = value;
                d0 = d1;
                d1 = tan;
                t0 = t1;
                t1 = kframe;
            }
            MeleeInterpolationType::SLP { tan, .. } => {
                d0 = d1;
                d1 = tan;
            }
            MeleeInterpolationType::KEY { value, .. } => {
                p0 = value;
                p1 = value;
            }
        }

        if t1 > frame && !matches!(interpolation, MeleeInterpolationType::SLP {..}) {
            break
        }

        op_intrp = interpolation;
    }

    AnimState { t0, t1, p0, p1, d0, d1, _op: op, op_intrp }
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

impl AnimTrack {
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
    pub fn track_type(&self) -> TrackType {
        TrackType::from_u8(self.hsd_struct.get_i8(0x04) as u8).unwrap()
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
                tracks.push(Track { hsd_struct: track });
            }

            nodes.push(FigaTreeNode { tracks: tracks.into_boxed_slice() });

            offset += track_count as usize;
        }

        nodes.into_boxed_slice()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum TrackType {
    //NONE = 0,
    RotateX = 1,
    RotateY,
    RotateZ,
    //PATH,
    TranslateX = 5,
    TranslateY,
    TranslateZ,
    ScaleX,
    ScaleY,
    ScaleZ,
    //NODE,
    BRANCH = 12,
    //SETBYTE0,
    //SETBYTE1,
    //SETBYTE2,
    //SETBYTE3,
    //SETBYTE4,
    //SETBYTE5,
    //SETBYTE6,
    //SETBYTE7,
    //SETBYTE8,
    //SETBYTE9,
    //SETFLOAT0,
    //SETFLOAT1,
    //SETFLOAT2,
    //SETFLOAT3,
    //SETFLOAT4,
    //SETFLOAT5,
    //SETFLOAT6,
    //SETFLOAT7,
    //SETFLOAT8,
    //SETFLOAT9,
    PTCL = 40 // Particle
}


impl TrackType {
    pub fn from_u8(n: u8) -> Option<Self> {
        use TrackType::*;
        Some(match n {
            //0 => NONE,
            1 => RotateX,
            2 => RotateY,
            3 => RotateZ,
            //4 => PATH,
            5 => TranslateX,
            6 => TranslateY,
            7 => TranslateZ,
            8 => ScaleX,
            9 => ScaleY,
            10 => ScaleZ,
            //11 => NODE,
            
            // IO_HSDAnims.cs:239 (DecodeFOBJ)
            // HACK - reproduces strange (buggy?) behaviour in StudioSB
            // Node case not covered, TranslateX is the default.
            11 => TranslateX, 

            12 => BRANCH,
            //13 => SETBYTE0,
            //14 => SETBYTE1,
            //15 => SETBYTE2,
            //16 => SETBYTE3,
            //17 => SETBYTE4,
            //18 => SETBYTE5,
            //19 => SETBYTE6,
            //20 => SETBYTE7,
            //21 => SETBYTE8,
            //22 => SETBYTE9,
            //23 => SETFLOAT0,
            //24 => SETFLOAT1,
            //25 => SETFLOAT2,
            //26 => SETFLOAT3,
            //27 => SETFLOAT4,
            //28 => SETFLOAT5,
            //29 => SETFLOAT6,
            //30 => SETFLOAT7,
            //31 => SETFLOAT8,
            //32 => SETFLOAT9,
            40 => PTCL,
            _ => return None,
        })
    }       
}           
