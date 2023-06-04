use crate::dat::{HSDRawFile, jobj::DOBJ, JOBJ, extract_anims::Animation};
use glam::f32::{Mat4, Vec3, Vec4};
use glam::u32::UVec4;
use glam::swizzles::Vec4Swizzles;

#[derive(Debug)]
pub enum ExtractMeshError {
    InvalidDatFile,
}

pub fn extract_scene<'a, 'b>(file: &'a HSDRawFile<'b>) -> Result<Scene<'a, 'b>, ExtractMeshError> {
    let root_jobj = root_jobj(file).ok_or(ExtractMeshError::InvalidDatFile)?;

    let mut bone_jobjs = Vec::new();
    let mut bone_tree_roots = Vec::new();
    for jobj in root_jobj.siblings() {
        bone_tree_roots.push(BoneTree::new(jobj, &mut bone_jobjs));
    }
    
    let mut bones = Vec::new();
    for jobj in &bone_jobjs {
        let mesh = jobj.get_dobj().map(
            |dobj| Mesh::new(&dobj, &bone_jobjs)
        );

        let transform = jobj.transform();

        let bone = Bone {
            parent_index: None,
            mesh,
            base_transform: transform,
            animated_transform: transform,
        };

        bones.push(bone);
    }

    fn set_parent_indicies(bones: &mut [Bone], tree: &BoneTree) {
        for child in tree.children.iter() {
            bones[child.index].parent_index = Some(tree.index as _);

            set_parent_indicies(bones, child);
        }
    }

    for root in bone_tree_roots.iter() {
        set_parent_indicies(&mut bones, root);
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

#[derive(Copy, Clone)]
pub struct Vertex {
    pub pos: Vec3,
    pub weights: Vec4,
    pub bones: UVec4,
}

pub struct Mesh {
    pub vertices: Box<[Vertex]>,
}

pub struct BoneTree {
    pub index: usize, // index into Skeleton.bones
    pub children: Box<[BoneTree]>,
}

pub struct Bone {
    pub parent_index: Option<u32>, // I dislike this
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

impl Bone {
    pub fn world_transform(&self, bones: &[Bone]) -> Mat4 {
        match self.parent_index {
            Some(i) => self.base_transform * bones[i as usize].world_transform(bones),
            None => self.base_transform,
        }
    }

    pub fn animated_world_transform(&self, bones: &[Bone]) -> Mat4 {
        match self.parent_index {
            // TODO reverse?
            Some(i) => bones[i as usize].animated_world_transform(bones) * self.animated_transform,
            None => self.animated_transform,
        }
    }

    pub fn inv_world_transform(&self, bones: &[Bone]) -> Mat4 {
        self.world_transform(bones).inverse()
    }

    pub fn animated_bind_matrix(&self, bones: &[Bone]) -> Mat4 {
        self.inv_world_transform(bones) * self.animated_world_transform(bones)
    }

    pub fn animated_vertices<'a>(&'a self) -> Option<impl Iterator<Item=Vertex> + 'a> {
        if let Some(ref m) = self.mesh {
            let iter = m.vertices.iter()
                .map(|&v| {
                    let pos = <Vec4 as From<(Vec3, f32)>>::from((v.pos, 0.0));
                    let newpos = self.animated_transform * pos;
                    Vertex { pos: newpos.xyz(), ..v }
                });
            Some(iter)
        } else {
            None
        }
    }
}

impl Mesh {
    pub fn new<'a, 'b>(dobj: &'b DOBJ<'a>, bones: &'b [JOBJ<'a>]) -> Self {
        dobj.create_mesh(bones)
    }
}

impl BoneTree {
    pub fn new<'a>(jobj: JOBJ<'a>, jobjs: &mut Vec<JOBJ<'a>>) -> Self {
        let index = jobjs.len();
        jobjs.push(jobj.clone());
        let mut children = Vec::new();
        for child_jobj in jobj.children() {
            children.push(BoneTree::new(child_jobj, jobjs));
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
