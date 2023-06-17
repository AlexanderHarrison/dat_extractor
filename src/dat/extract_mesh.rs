use crate::dat::{HSDRawFile, jobj::DOBJ, JOBJ, extract_anims::Animation, DatExtractError};
use glam::f32::{Mat4, Vec3, Vec4, Vec2};
use glam::u32::UVec4;
use glam::Vec4Swizzles;

#[derive(Clone, Debug)]
pub struct Skeleton {
    pub bone_tree_roots: Box<[BoneTree]>,
    pub bones: Box<[Bone]>,
}

#[derive(Copy, Clone, Debug)]
pub struct Vertex {
    pub pos: Vec3,
    pub tex0: Vec2, // uv ?
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

pub fn extract_bones<'a>(parsed_model_dat: &HSDRawFile<'a>) -> Result<(Vec<JOBJ<'a>>, Vec<BoneTree>), DatExtractError> {
    let root_jobj = parsed_model_dat.roots.iter()
        .find_map(|root| JOBJ::try_from_root_node(root))
        .ok_or(DatExtractError::InvalidDatFile)?;

    let mut bone_jobjs = Vec::new();
    let mut bone_tree_roots = Vec::new();
    for jobj in root_jobj.siblings() {
        bone_tree_roots.push(BoneTree::new(jobj, &mut bone_jobjs));
    }

    Ok((bone_jobjs, bone_tree_roots))
}

pub fn extract_skeleton(bone_jobjs: &[JOBJ], bone_tree_roots: Vec<BoneTree>) -> Skeleton {
    let mut bones = Vec::new();

    for jobj in bone_jobjs {
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

    Skeleton { 
        bone_tree_roots: bone_tree_roots.into_boxed_slice(),
        bones: bones.into_boxed_slice(),
    }
}

impl Skeleton {
    pub fn apply_animation(&mut self, frame: f32, animation: &Animation) {
        for transform in animation.transforms.iter() {
            let bone = &mut self.bones[transform.bone_index];
            bone.animated_transform = transform.compute_transform_at(frame, &bone.base_transform);
        }
    }

    //pub fn iter_high_poly_meshes(&self) -> impl Iterator<Item=Mesh> {
    //    self.bone_tree_roots.iter()
    //        .flat_map(|root| root.iter_bones())
    //}

    pub fn obj(&self) {
        let bones = &*self.bones;

        let mut i = 1;
        let mut mesh_index = 0;
        let mut bone_to_obj = move |bone: &Bone| {
            for mesh in bone.meshes.iter() {
                // skip low poly mesh
                if 36 <= mesh_index && mesh_index <= 67 { 
                    mesh_index += 1;
                    continue;
                } else {
                    mesh_index += 1;
                }

                for p in mesh.primitives.iter() {
                    let mut points = Vec::with_capacity(p.vertices.len());

                    for v in p.vertices.iter() {
                        let t = Vec4::from((v.pos, 1.0));

                        let awt = bone.animated_world_transform(&bones);
                        let t2 = awt * t;
                                         
                        let pos = if v.weights.x == 1.0 { // good
                            let t = bones[v.bones.x as usize].animated_world_transform(&bones) * t2;
                            t.xyz()
                        } else if v.weights != Vec4::ZERO {
                            let v1 = (bones[v.bones.x as usize].animated_bind_matrix(bones) * v.weights.x) * t;
                            let v2 = (bones[v.bones.y as usize].animated_bind_matrix(bones) * v.weights.y) * t;
                            let v3 = (bones[v.bones.z as usize].animated_bind_matrix(bones) * v.weights.z) * t;
                            let v4 = (bones[v.bones.w as usize].animated_bind_matrix(bones) * v.weights.w) * t;
                            (v1 + v2 + v3 + v4).xyz()
                        } else {
                            t2.xyz()
                        };
                        
                        points.push(pos);
                    }

                    match p.primitive_type {
                        PrimitiveType::Triangles => {
                            for t in points.chunks_exact(3) {
                                println!("v {} {} {}", t[0].x, t[0].y, t[0].z);
                                println!("v {} {} {}", t[1].x, t[1].y, t[1].z);
                                println!("v {} {} {}", t[2].x, t[2].y, t[2].z);

                                println!("f {} {} {}", i, i+1, i+2);
                                i += 3;
                            }
                        }
                        PrimitiveType::TriangleStrip => {
                            println!("v {} {} {}", points[0].x, points[0].y, points[0].z);
                            println!("v {} {} {}", points[1].x, points[1].y, points[1].z);

                            for p in &points[2..] {
                                println!("v {} {} {}", p.x, p.y, p.z);

                                println!("f {} {} {}", i, i+1, i+2);
                                i += 1;
                            }
                            i += 2;
                        }
                        PrimitiveType::Quads => {
                            for t in points.chunks_exact(4) {
                                println!("v {} {} {}", t[0].x, t[0].y, t[0].z);
                                println!("v {} {} {}", t[1].x, t[1].y, t[1].z);
                                println!("v {} {} {}", t[2].x, t[2].y, t[2].z);
                                println!("v {} {} {}", t[3].x, t[3].y, t[3].z);

                                println!("f {} {} {} {3}", i, i+1, i+2, i+3);
                                i += 4;
                            }
                        }
                        p => panic!("{:?}", p)
                    }
                }
            }
        };

        for root in self.bone_tree_roots.iter() {
            root.inspect_high_poly_bones(&self.bones, &mut bone_to_obj)
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
