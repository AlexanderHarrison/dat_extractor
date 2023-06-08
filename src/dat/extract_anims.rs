use crate::dat::{FighterAction, HSDRawFile, Stream, HSDStruct, DatFile};
use glam::f32::{Quat, Mat4, Vec3};

#[derive(Debug, Copy, Clone)]
pub struct AnimDatFile<'a> {
    pub data: &'a [u8],
}

#[derive(Copy, Clone, Debug)]
pub enum AnimParseError {
    InvalidAnimFile,
    CharacterMismatch
}

#[derive(Clone, Debug)]
pub struct Animation {
    pub name: Box<str>,
    pub transforms: Box<[AnimTransform]>,
}

#[derive(Clone, Debug)]
pub struct AnimTransform {
    pub tracks: Box<[AnimTrack]>,
    pub bone_index: usize,
}

#[derive(Clone, Debug)]
pub struct AnimTrack {
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

/// Pass in the raw data of the animations file - Pl*AJ.dat
/// That is necessary (for now).
pub fn extract_anims(
    aj_dat: &DatFile,
    actions: Vec<FighterAction>
) -> Result<Box<[Animation]>, AnimParseError> {
    let mut animations: Vec<Animation> = Vec::with_capacity(actions.len());

    for action in actions.into_iter() {
        let offset = action.animation_offset;
        let size = action.animation_size;
        let data = &aj_dat.data[offset..offset+size];

        // TODO might be discarding some animations??
        if let Some(name) = action.name {
            if animations.iter().find(|a| &*a.name == &*name).is_none() {
                let animation = Animation {
                    name,
                    transforms: extract_anim_transforms(data)
                };

                animations.push(animation);
            }
        }
    }

    Ok(animations.into_boxed_slice())
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
            let val: f32 = track.get_value(frame);

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

fn extract_anim_transforms(anim_data: &[u8]) -> Box<[AnimTransform]> {
    let stream = Stream::new(anim_data);
    let hsd_file = HSDRawFile::open(stream);

    let anim_root = &hsd_file.roots[0];
    assert!(anim_root.root_string.contains("figatree"));

    let figatree = FigaTree::new(anim_root.hsd_struct.clone());

    let mut transforms = Vec::new();

    // I pray that bone_index is correct here...
    // It looks like the skeleton bone array is depth-first (SBSkeleton.cs:48)
    // and the flat list of nodes here corresponds with that access method (IO_HSDAnim.cs:76).
    // Not obvious.
    for (i, node) in figatree.get_nodes().iter().enumerate() {
        let mut tracks = Vec::new();
        for track in node.tracks.iter() {
            tracks.push(decode_track(track))
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

fn decode_track<'a, 'b>(track: &'b Track<'a>) -> AnimTrack {
    let track_type = track.track_type();

    // buffer not at 0x04 as in FOBJ! 
    // FOBJs are constructed from Tracks, and hold the Buffer ptr at 0x08 instead
    // We never bother to convert Tracks to FOBJs
    let mut buffer = Stream::new(track.hsd_struct.get_reference(0x08).data);
    let stream = &mut buffer;
    let mut clock: f32 = 0.0;

    let value_scale = track.value_scale();
    let tan_scale = track.tan_scale();
    let value_format = track.value_format();
    let tan_format = track.tan_format();

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

        op_intrp = interpolation.clone();
    }

    AnimState { t0, t1, p0, p1, d0, d1, _op: op, op_intrp }
}

fn lerp(av: f32, bv: f32, v0: f32, v1: f32, t: f32) -> f32 {
    if v0 == v1 { return av };

    if t == v0 { return av };
    if t == v1 { return bv };

    let mu = (t - v0) / (v1 - v0);
    return (av * (1.0 - mu)) + (bv * mu);
}

fn hermite(frame: f32, frame_left: f32, frame_right: f32, ls: f32, rs: f32, lhs: f32, rhs: f32) -> f32 {
    let frame_diff = frame - frame_left;
    let weight = frame_diff / (frame_right - frame_left);

    let mut result = lhs + (lhs - rhs) * (2.0 * weight - 3.0) * weight * weight;
    result += (frame_diff * (weight - 1.0)) * (ls * (weight - 1.0) + rs * weight);

    return result;
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
                return self.keys[left].value;
            },
            InterpolationType::Linear => {
                let left_value = self.keys[left].value;
                let right_value = self.keys[right].value;
                let left_frame = self.keys[left].frame;
                let right_frame = self.keys[right].frame;

                let value = lerp(left_value, right_value, left_frame, right_frame, frame);

                assert!(!value.is_nan());

                return value;
            },
            InterpolationType::Hermite => {
                let left_value = self.keys[left].value;
                let right_value = self.keys[right].value;
                let left_tan = self.keys[left].out_tan;
                let right_tan = self.keys[right].in_tan;
                let left_frame = self.keys[left].frame;
                let right_frame = self.keys[right].frame;

                let value = hermite(frame, left_frame, right_frame, left_tan, right_tan, left_value, right_value);

                assert!(!value.is_nan());

                return value;
            }
        }
    }

    // SBKeyGroup.cs:80
    pub fn binary_search_keys(&self, frame: f32) -> usize {
        let mut lower = 0;
        let mut upper = self.keys.len() - 1;
        let mut middle;

        while lower <= upper {
            middle = (upper + lower) / 2;
            if frame == self.keys[middle].frame {
                return middle;
            } else if frame < self.keys[middle].frame {
                assert!(middle != 0); // otherwise we need minus checks...
                upper = middle - 1;
            } else {
                lower = middle + 1;
            }
        }
        
        return if lower < upper { lower } else { upper };
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
    pub fn track_type<'b>(&'b self) -> TrackType {
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
    //BRANCH,
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
    //PTCL = 40
}


impl TrackType {
    pub fn from_u8(n: u8) -> Option<Self> {
        use TrackType::*;
        Some(match n {
            //00 => NONE,
            01 => RotateX,
            02 => RotateY,
            03 => RotateZ,
            //04 => PATH,
            05 => TranslateX,
            06 => TranslateY,
            07 => TranslateZ,
            08 => ScaleX,
            09 => ScaleY,
            10 => ScaleZ,
            //11 => NODE,
            
            // IO_HSDAnims.cs:239 (DecodeFOBJ)
            // HACK - reproduces strange (buggy?) behaviour in StudioSB
            // Node case not covered, TranslateX is the default.
            11 => TranslateX, 

            //12 => BRANCH,
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
            //40 => PTCL,
            _ => return None,
        })
    }       
}           
