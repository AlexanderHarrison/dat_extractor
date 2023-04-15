use crate::dat::{HSDRawFile, JOBJ, extract_anims::Animation};
use glam::f32::Mat4;

#[derive(Debug)]
pub enum ExtractMeshError {
    InvalidDatFile,
}

pub fn extract_scene<'a, 'b>(file: &'a HSDRawFile<'b>) -> Result<Scene<'a, 'b>, ExtractMeshError> {
    let root_jobj = root_jobj(file).ok_or(ExtractMeshError::InvalidDatFile)?;

    let mut bones = Vec::new();
    let mut bone_tree_roots = Vec::new();

    for jobj in root_jobj.siblings() {
        bone_tree_roots.push(BoneTree::new(jobj, &mut bones));
    }

    let skeleton = Skeleton { 
        bone_tree_roots: bone_tree_roots.into_boxed_slice(),
        bones: bones.into_boxed_slice(),
    };

    Ok(Scene {
        file,
        skeleton,
    })
}

fn root_jobj<'a, 'b>(file: &'a HSDRawFile<'b>) -> Option<JOBJ<'b>> {
    for root in &file.roots {
        let j = JOBJ::try_from_root_node(root);
        if j.is_some() { 
            return j 
        }
    }
    None
}

pub struct Scene<'a, 'b> {
    pub file: &'a HSDRawFile<'b>,
    pub skeleton: Skeleton,
}

pub struct Skeleton {
    pub bone_tree_roots: Box<[BoneTree]>,
    pub bones: Box<[Bone]>,
}

pub struct Mesh {
    pub vertices: Box<[[f32; 3]]>,
}

pub struct BoneTree {
    pub index: usize, // index into Skeleton.bones
    pub children: Box<[BoneTree]>,
}

pub struct Bone {
    pub mesh: Option<Mesh>,
    pub base_transform: Mat4,
    pub animated_transform: Mat4,
}

impl Skeleton {
    pub fn apply_animation(&mut self, frame: f32, animation: &Animation) {
        for transform in animation.transforms.iter() {
            let bone = &mut self.bones[transform.bone_index];
            bone.animated_transform = transform.compute_transform_at(frame, &bone.base_transform);
        }
    }
}

impl BoneTree {
    pub fn new(jobj: JOBJ<'_>, bones: &mut Vec<Bone>) -> Self {
        let mesh = jobj.get_dobj().map(
            |dobj| Mesh { vertices: dobj.decode_vertices().into_boxed_slice() }
        );

        let bone = Bone {
            mesh,
            base_transform: Mat4::IDENTITY,
            animated_transform: Mat4::IDENTITY,
        };
        let index = bones.len();
        bones.push(bone);

        let mut children = Vec::new();
        for child_jobj in jobj.children() {
            children.push(BoneTree::new(child_jobj, bones));
        }

        BoneTree {
            index,
            children: children.into_boxed_slice(),
        }
    }

    pub fn inspect_each<'b, F>(&'b self, f: &mut F) where
        F: FnMut(&'b Self)
    {
        f(self);
        for c in self.children.iter() {
            c.inspect_each(f);
        }
    }
}
