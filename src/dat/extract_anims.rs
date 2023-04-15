use crate::dat::{FighterData, HSDRawFile, Stream, HSDStruct, extract_mesh::Bone};
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
        // TODO base_transform is used strangely
        // TODO Key.GetValue

        let mut translation = Vec3::ZERO;
        let mut euler_rotation = Vec3::ZERO;
        let mut scale = Vec3::ONE;

        use TrackType::*;
        for track in self.tracks.iter() {
            match track.track_type {
                RotateX => euler_rotation.x = 0.0,
                RotateY => euler_rotation.y = 0.0,
                RotateZ => euler_rotation.z = 0.0,
                TranslateX => translation.x = 0.0,
                TranslateY => translation.y = 0.0,
                TranslateZ => translation.z = 0.0,
                ScaleX => scale.x = 1.0,
                ScaleY => scale.y = 1.0,
                ScaleZ => scale.z = 1.0,
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

fn decode_track<'a, 'b>(track: &'b Track<'a>) -> AnimTrack {
    let track_type = track.track_type();

    todo!()
}

impl AnimTrack {

}

impl<'a> Track<'a> {
    pub fn track_type<'b>(&'b self) -> TrackType {
        TrackType::from_u8(self.hsd_struct.get_i8(0x04) as u8).unwrap()
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
