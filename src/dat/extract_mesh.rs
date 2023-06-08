use crate::dat::{HSDRawFile, jobj::DOBJ, JOBJ, extract_anims::Animation, DatFile};
use glam::f32::{Mat4, Vec3, Vec4};
use glam::u32::UVec4;

#[derive(Copy, Clone, Debug)]
pub enum ExtractMeshError {
    InvalidDatFile,
}

#[derive(Clone, Debug)]
pub struct Skeleton {
    pub bone_tree_roots: Box<[BoneTree]>,
    pub bones: Box<[Bone]>,
}

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos: Vec3,
    pub weights: Vec4,
    pub bones: UVec4,
}

#[derive(Clone, Debug)]
pub struct Primitive {
    pub vertices: Box<[Vertex]>,
    pub primitive_type: PrimitiveType,
}

#[derive(Copy, Clone, Debug)]
pub enum PrimitiveType {
    Points = 0xB8,
    Lines = 0xA8,
    LineStrip = 0xB0,
    Triangles = 0x90,
    TriangleStrip = 0x98,
    TriangleFan = 0xA0,
    Quads = 0x80
}

#[derive(Clone, Debug)]
pub struct Mesh {
    pub primitives: Box<[Primitive]>,
}

#[derive(Clone, Debug)]
pub struct BoneTree {
    pub index: usize, // index into Skeleton.bones
    pub children: Box<[BoneTree]>,
}

#[derive(Clone, Debug)]
pub struct Bone {
    pub parent_index: Option<u32>, // I dislike this
    pub meshes: Vec<Mesh>,
    pub base_transform: Mat4,
    pub animated_transform: Mat4,
}

pub fn extract_skeleton(model_dat: &DatFile) -> Result<Skeleton, ExtractMeshError> {
    let hsd_file = HSDRawFile::new(model_dat);

    let root_jobj = root_jobj(&hsd_file).ok_or(ExtractMeshError::InvalidDatFile)?;

    let mut bone_jobjs = Vec::new();
    let mut bone_tree_roots = Vec::new();
    for jobj in root_jobj.siblings() {
        bone_tree_roots.push(BoneTree::new(jobj, &mut bone_jobjs));
    }

    let mut bones = Vec::new();

    for jobj in &bone_jobjs {
        let meshes = jobj.get_dobj().map(
            |dobj| Mesh::new(&dobj, &bone_jobjs)
        );
        let meshes = meshes.unwrap_or(Vec::new());

        let transform = jobj.transform();

        bones.push(Bone {
            parent_index: None,
            meshes,
            base_transform: transform,
            animated_transform: transform,
        })
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

    Ok(Skeleton { 
        bone_tree_roots: bone_tree_roots.into_boxed_slice(),
        bones: bones.into_boxed_slice(),
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

impl PrimitiveType {
    pub fn from_u8(n: u8) -> Option<Self> {
        Some(match n {
            0xB8 => Self::Points       ,
            0xA8 => Self::Lines        ,
            0xB0 => Self::LineStrip    ,
            0x90 => Self::Triangles    ,
            0x98 => Self::TriangleStrip,
            0xA0 => Self::TriangleFan  ,
            0x80 => Self::Quads        ,
            _ => return None
        })
    }
}

impl Skeleton {
    pub fn apply_animation(&mut self, frame: f32, animation: &Animation) {
        for transform in animation.transforms.iter() {
            let bone = &mut self.bones[transform.bone_index];
            bone.animated_transform = transform.compute_transform_at(frame, &bone.base_transform);
        }
    }

    pub fn inspect_high_poly_bones<'b, F>(&'b self, f: &mut F) where
        F: FnMut(&'b Bone)
    {
        for root in self.bone_tree_roots.iter() {
            root.inspect_high_poly_bones(&self.bones, f)
        }
    }
}

impl Bone {
    pub fn world_transform(&self, bones: &[Bone]) -> Mat4 {
        match self.parent_index {
            Some(i) => bones[i as usize].world_transform(bones) * self.base_transform,
            None => self.base_transform,
        }
    }

    pub fn animated_world_transform(&self, bones: &[Bone]) -> Mat4 {
        match self.parent_index {
            Some(i) => bones[i as usize].animated_world_transform(bones) * self.animated_transform,
            None => self.animated_transform,
        }
    }

    pub fn inv_world_transform(&self, bones: &[Bone]) -> Mat4 {
        self.world_transform(bones).inverse()
    }

    pub fn animated_bind_matrix(&self, bones: &[Bone]) -> Mat4 {
        self.animated_world_transform(bones) * self.inv_world_transform(bones)
    }
}

impl Mesh {
    pub fn new<'a, 'b>(dobj: &'b DOBJ<'a>, bones: &'b [JOBJ<'a>]) -> Vec<Self> {
        dobj.create_meshes(bones)
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

    pub fn inspect_high_poly_bones<'b, F>(&'b self, bones: &'b [Bone], f: &mut F) where
        F: FnMut(&'b Bone)
    {
        f(&bones[self.index]);
        for c in self.children.iter() {
            c.inspect_high_poly_bones(bones, f);
        }
    }
}
