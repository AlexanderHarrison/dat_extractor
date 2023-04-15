use crate::dat::{FighterData, HSDRawFile, Stream, HSDStruct};
use glam::f32::{Quat, Mat4, Vec3};

pub struct AnimDatFile<'a> {
    pub data: &'a [u8],
    pub name: &'a str,
}

#[derive(Debug)]
pub enum AnimParseError {
    InvalidAnimFile,
    CharacterMismatch
}

/// Pass in the raw data of the animations file - Pl*AJ.dat
/// That is necessary (for now).
pub fn extract_anim_dat_files<'a>(fighter_data: &'a FighterData, aj_data: &'a [u8]) -> Result<Vec<AnimDatFile<'a>>, AnimParseError> {
    let aj_hsd = HSDRawFile::open(Stream::new(aj_data));
    if aj_hsd.roots.len() == 0 { return Err(AnimParseError::InvalidAnimFile) }
    let s = &aj_hsd.roots[0].root_string;

    // 'PlyFox5K_Share_ACTION_Wait1_figatree' -> 'Fox'
    let anim_char_name: &str = s.strip_prefix("Ply")
        .and_then(|s| s.find("K_").map(|p| &s[..p-1]))
        .ok_or(AnimParseError::InvalidAnimFile)?;

    if anim_char_name != &*fighter_data.character_name { return Err(AnimParseError::CharacterMismatch); }

    let mut animations: Vec<AnimDatFile> = Vec::with_capacity(fighter_data.fighter_actions.len());

    for action in &fighter_data.fighter_actions {
        let offset = action.animation_offset;
        let size = action.animation_size;
        let data = &aj_data[offset..offset+size];

        if let Some(name) = &action.name {
            if animations.iter().find(|a| a.name == &**name).is_none() {
                animations.push(AnimDatFile {
                    data,
                    name,
                })
            }
        }
    }

    Ok(animations)
}

pub struct Animation {
    pub transforms: Box<[AnimTransform]>,
}

pub struct AnimTransform {
    pub tracks: Box<[AnimTrack]>,
    pub bone_index: usize,
}

pub struct AnimTrack {
    pub track_type: TrackType,
    pub keys: Box<[Key]>,
}

pub struct FigaTree<'a> {
    pub hsd_struct: HSDStruct<'a>
}

pub struct FigaTreeNode<'a> {
    pub tracks: Box<[Track<'a>]>,
}

pub struct Track<'a> {
    pub hsd_struct: HSDStruct<'a>,
}

impl AnimTransform {
    // SBTransformAnimation.cs:86 (GetTransformAt)
    pub fn compute_transform_at(&self, frame: f32, base_transform: &Mat4) -> Mat4 {
        // TODO RotationOnly ????????? (SBAnimation.cs:80)

        let (mut scale, qrot, mut translation) = base_transform.to_scale_rotation_translation();
        let (rx, ry, rz) = qrot.to_euler(glam::EulerRot::XYZ); // TODO Maybe?? ...
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
            glam::EulerRot::XYZ, // TODO not sure which to use... 
            euler_rotation.x,
            euler_rotation.y,
            euler_rotation.z,
        );

        Mat4::from_scale_rotation_translation(scale, rotation, translation)
    }
}

pub fn extract_anim_from_dat_file<'a>(dat_file: AnimDatFile<'a>) -> Animation {
    let stream = Stream::new(dat_file.data);
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

    Animation {
        transforms: transforms.into_boxed_slice()
    }
}

// no clue what this does.
// BinaryReaderExt.cs:249
fn read_packed(stream: &mut Stream<'_>) -> u8 {
    let a = stream.read_byte();
    if (a & 0x80) != 0 {
        let b = stream.read_byte();
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

#[derive(Clone, Debug)]
pub enum InterpolationType {
    NONE,
    CON  { value: f32, time: u8 },
    LIN  { value: f32, time: u8 },
    SPL0 { value: f32, time: u8 },
    SPL  { value: f32, tan: f32, time: u8 },
    SLP  { tan: f32 },
    KEY  { value: f32 },
}

pub struct Key {
    pub frame: f32,
    pub interpolation: InterpolationType,
}

fn decode_track<'a, 'b>(track: &'b Track<'a>) -> AnimTrack {
    let track_type = track.track_type();

    let mut buffer = Stream::new(track.hsd_struct.get_reference(0x04).data);
    let stream = &mut buffer;
    let mut clock: f32 = 0.0;

    let value_scale = track.value_scale();
    let tan_scale = track.tan_scale();
    let value_format = track.value_format();
    let tan_format = track.tan_format();

    let mut keys = Vec::new();

    while !stream.finished() {
        let typ = read_packed(stream);
        let interp_type = typ & 0x0F;
        let num_keys = (typ >> 4) + 1;
        
        for _ in 0..num_keys {
            let mut time = 0;

            let interpolation = match interp_type {
                0x00 => {     // NONE
                    InterpolationType::NONE
                }
                0x01 => {
                    let value = parse_float(stream, value_format, value_scale);
                    time = read_packed(stream);
                    InterpolationType::CON { value, time }
                }
                0x02 => {
                    let value = parse_float(stream, value_format, value_scale);
                    time = read_packed(stream);
                    InterpolationType::LIN { value, time }
                }
                0x03 => {
                    let value = parse_float(stream, value_format, value_scale);
                    time = read_packed(stream);
                    InterpolationType::SPL0 { value, time }
                }
                0x04 => {
                    let value = parse_float(stream, value_format, value_scale);
                    let tan = parse_float(stream, tan_format, tan_scale);
                    time = read_packed(stream);
                    InterpolationType::SPL { value, tan, time }
                }
                0x05 => {
                    let tan = parse_float(stream, tan_format, tan_scale);
                    InterpolationType::SLP { tan }
                }
                0x06 => {
                    let value = parse_float(stream, value_format, value_scale);
                    InterpolationType::KEY { value }
                }
                _ => panic!(),
            };

            keys.push(Key { 
                frame: clock,
                interpolation,
            });

            clock += time as f32;
        }
    }

    AnimTrack {
        keys: keys.into_boxed_slice(),
        track_type,
    }
}

struct AnimState {
    pub t0: f32,
    pub t1: f32,
    pub p0: f32,
    pub p1: f32,
    pub d0: f32,
    pub d1: f32,
    pub op: InterpolationType,
    pub op_intrp: InterpolationType, // idk
}

impl AnimTrack {
    pub fn get_value(&self, frame: f32) -> f32 {
        use InterpolationType::*;
        let state = self.get_state(frame);

        if frame == state.t0 {
            return state.p0;
        } 
        
        if frame == state.t1 {
            return state.p1
        } 

        if state.t0 == state.t1 || matches!(state.op_intrp, CON {..} | KEY {..}) {
            return state.p0
        }

        let time = frame - state.t0;
        let fterm = state.t1 - state.t0;

        if matches!(state.op_intrp, LIN {..}) {
            let d0 = (state.p1 - state.p0) / fterm;
            return d0 * time + state.p0; 
        }

        if matches!(state.op_intrp, SPL {..} | SPL0 {..} | SLP {..}) {
            // Hermite curve, apparently 
            let ftermr = fterm.recip();

            let fVar1 = time * time;
            let fVar2 = ftermr * ftermr * fVar1 * time;
            let fVar3 = 3.0 * fVar1 * ftermr * ftermr;
            let fVar4 = fVar2 - fVar1 * ftermr;
            let fVar2 = 2.0 * fVar2 * ftermr;
            return state.d1 * fVar4 
                + state.d0 * (time + (fVar4 - fVar1 * ftermr)) 
                + state.p0 * (1.0 + (fVar2 - fVar3)) 
                + state.p1 * (-fVar2 + fVar3);
        }

        state.p0
    }

    fn get_state(&self, frame: f32) -> AnimState {
        let mut t0 = 0.0;
        let mut t1 = 0.0;
        let mut p0 = 0.0;
        let mut p1 = 0.0;
        let mut d0 = 0.0;
        let mut d1 = 0.0;

        let mut op = InterpolationType::CON { time: 0, value: 0.0 };
        let mut op_intrp = InterpolationType::CON { time: 0, value: 0.0 };

        for key in self.keys.iter() {
            op_intrp = op;
            op = key.interpolation.clone();

            match op {
                InterpolationType::NONE => (),
                InterpolationType::CON { value, .. } | InterpolationType::LIN { value, ..} => {
                    p0 = p1;
                    p1 = value;
                    if !matches!(op_intrp, InterpolationType::SLP { .. }) {
                        d0 = d1;
                        d1 = 0.0;
                    }
                    t0 = t1;
                    t1 = key.frame
                }
                InterpolationType::SPL0 { value, .. } => {
                    p0 = p1;
                    d0 = d1;
                    p1 = value;
                    d1 = 0.0;
                    t0 = t1;
                    t1 = key.frame;
                }
                InterpolationType::SPL { value, tan, .. } => {
                    p0 = p1;
                    p1 = value;
                    d0 = d1;
                    d1 = tan;
                    t0 = t1;
                    t1 = key.frame;
                }
                InterpolationType::SLP { tan, .. } => {
                    d0 = d1;
                    d1 = tan;
                }
                InterpolationType::KEY { value, .. } => {
                    p0 = value;
                    p1 = value;
                }
            }

            if t1 > frame && !matches!(key.interpolation, InterpolationType::SLP {..}) {
                break
            }

            op_intrp = key.interpolation.clone();
        }

        AnimState { t0, t1, p0, p1, d0, d1, op, op_intrp }
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
        self.hsd_struct.get_i8(0x01) as u8
    }

    pub fn tan_flag(&self) -> u8 {
        self.hsd_struct.get_i8(0x02) as u8
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

#[derive(Copy, Clone, PartialEq, Eq)]
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
