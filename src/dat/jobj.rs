#![allow(clippy::upper_case_acronyms)]

use crate::dat::{HSDStruct, HSDRootNode, Mesh, Vertex, Primitive, PrimitiveType, textures::MOBJ};
use glam::f32::{Vec3, Quat, Mat4};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct JOBJ<'a> {
    pub hsd_struct: HSDStruct<'a>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct DOBJ<'a> {
    pub hsd_struct: HSDStruct<'a>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct POBJ<'a> {
    pub hsd_struct: HSDStruct<'a>,
}

#[derive(Clone, Debug)]
pub struct Attribute<'a> {
    pub name: AttributeName,
    pub typ: AttributeType,
    pub hsd_struct: HSDStruct<'a>,
}

impl<'a> DOBJ<'a> {
    pub fn new(hsd_struct: HSDStruct<'a>) -> Self {
        Self {
            hsd_struct
        }
    }

    pub fn get_pobj(&self) -> POBJ<'a> {
        POBJ::new(self.hsd_struct.get_reference(0x0C))
    }

    pub fn get_mobj(&self) -> Option<MOBJ<'a>> {
        self.hsd_struct.try_get_reference(0x08)
            .map(MOBJ::new)
    }

    /// Includes self
    pub fn siblings(&self) -> impl Iterator<Item=DOBJ<'a>> {
        std::iter::successors(Some(self.clone()), |ch| ch.get_sibling())
    }

    pub fn get_sibling(&self) -> Option<DOBJ<'a>> {
        self.hsd_struct.try_get_reference(0x04).map(DOBJ::new)
    }

    /// includes siblings
    pub fn create_meshes<'b>(&'b self, bones: &'b [JOBJ<'a>]) -> Vec<Mesh> {
        let mut meshes = Vec::new();

        for dobj in self.siblings() {
            let primitives = dobj.get_pobj()
                .siblings()
                .flat_map(|p| p.decode_primitives(bones))
                .collect::<Vec<Primitive>>();

            meshes.push(Mesh {
                primitives: primitives.into_boxed_slice(),
            })
        }

        meshes
    }
}

impl<'a> Attribute<'a> {
    pub fn new(hsd_struct: HSDStruct<'a>) -> Self {
        Self { 
            name: AttributeName::from_u8(hsd_struct.get_i32(0x00) as u8),
            typ: AttributeType::from_u8(hsd_struct.get_i32(0x04) as u8),
            hsd_struct,
        }
    }
    
    pub fn get_decoded_data_at(&self, loc: usize) -> Vec<f32> {
        if self.name == AttributeName::GX_VA_CLR0 || self.name == AttributeName::GX_VA_CLR1 {
            todo!();
            //return GetColorAt(index);
        }

        let comp_type = self.hsd_struct.get_i32(0x0C);
        let stride = self.hsd_struct.get_i16(0x12) as usize;
        let offset = stride * loc;

        let buffer = self.hsd_struct.get_reference(0x14); // buffer same

        let mut data = Vec::new();

        match comp_type {
            0 => { // UInt8
                for i in 0..stride {
                    let f = buffer.get_i8(offset + i) as u8 as f32; 
                    data.push(f);
                }
            } 
            1 => { // Int8
                for i in 0..stride {
                    let f = buffer.get_i8(offset + i) as f32; 
                    data.push(f);
                }
            }
            2 => { // UInt16,
                for i in 0..(stride / 2) {
                    let f = buffer.get_i16(offset + i*2) as u16 as f32; 
                    data.push(f);
                }
            }
            3 => { // Int16 ,
                for i in 0..(stride / 2) {
                    let f = buffer.get_i16(offset + i*2) as f32; 
                    data.push(f);
                }
            }
            4 => { // Float ,
                for i in 0..(stride / 4) {
                    let f = buffer.get_f32(offset + i*4);
                    data.push(f);
                }
            }
                    
            //5 => {} // Unused,
            _ => panic!("invalid comp type"),
        }


        let scale = self.hsd_struct.get_i8(0x10) as usize;
        for f in data.iter_mut() {
            *f /= (1 << scale) as f32
        }

        //println!("loc {}", data[0]);

        data
    }
}

pub struct Envelope<'a> {
    pub hsd_struct: HSDStruct<'a>,
}

impl<'a> Envelope<'a> {
    pub fn new(hsd_struct: HSDStruct<'a>) -> Self {
        Self { hsd_struct }
    }   

    // SBHsdMesh.cs:286 (GXVertexToHsdVertex)
    // HSD_Envelope.cs:13,56 (Weights, GetWeightAt)
    pub fn weights(&self) -> [f32; 4] {
        let mut weights = [0.0f32; 4];
        let len = self.hsd_struct.reference_count().min(4);

        #[allow(clippy::needless_range_loop)]
        for i in 0..len {
            weights[i] = self.hsd_struct.get_f32(i*8 + 4);
        }

        weights
    }

    pub fn jobjs<'b>(&'b self) -> impl Iterator<Item=JOBJ<'a>> + 'b {
        let len = self.hsd_struct.reference_count().min(4);
        (0..len).map(|i| JOBJ::new(self.hsd_struct.get_reference(i*8)))
    }
}

pub enum POBJFlag {
    ShapeSetAverage = (1 << 0),
    ShapeSetAdditive = (1 << 1),
    Unknown2 = (1 << 2),
    Anim = (1 << 3),
    ShapeAnim = (1 << 12),
    Envelope = (1 << 13),
    CullBack = (1 << 14),
    CullFront = (1 << 15)
}

impl<'a> POBJ<'a> {
    pub fn new(hsd_struct: HSDStruct<'a>) -> Self {
        Self {
            hsd_struct
        }
    }

    /// Includes self
    pub fn siblings(&self) -> impl Iterator<Item=POBJ<'a>> {
        std::iter::successors(Some(self.clone()), |ch| ch.get_sibling())
    }

    pub fn get_sibling(&self) -> Option<POBJ<'a>> {
        self.hsd_struct.try_get_reference(0x04).map(POBJ::new)
    }

    pub fn get_attributes(&self) -> Vec<Attribute<'a>> {
        let attr_buf = self.hsd_struct.get_reference(0x08);
        let shape_set = self.check_flag(POBJFlag::ShapeAnim);
        assert!(!shape_set); // just a hopeful guess. check ToGXAttributes in HSD_POBJ
    
        let count = attr_buf.len() / 0x18;
        let mut attributes = Vec::with_capacity(count);
        for i in 0..count {
            let attr = Attribute::new(attr_buf.get_embedded_struct(i * 0x18, 0x18));
            let name = attr.name;
            attributes.push(attr);
            if name == AttributeName::GX_VA_NULL {
                break;
            }
        }

        attributes
    }

    pub fn check_flag(&self, flag: POBJFlag) -> bool {
        let flags = self.hsd_struct.get_i16(0x0C) as u32;
        (flags & flag as u32) != 0
    }

    pub fn envelope_weights<'b>(&'b self) -> Option<Box<[Envelope<'a>]>> {
        if !self.check_flag(POBJFlag::Envelope) { return None }

        let envelope_ptrs = self.hsd_struct.get_reference(0x14);
        let length = (envelope_ptrs.len() / 4).max(1) - 1;

        let mut envelopes = Vec::with_capacity(length);

        for i in 0..length {
            match envelope_ptrs.try_get_reference(i * 4) {
                Some(e) => envelopes.push(Envelope::new(e)),
                None => break
            }
        }

        Some(envelopes.into_boxed_slice())
    }

    /// does not decode siblings
    pub fn decode_primitives<'b>(&'b self, bone_jobjs: &[JOBJ<'a>]) -> Vec<Primitive> {
        //println!("decode");
        let attributes = self.get_attributes();
        //println!("attrlen {}", attributes.len());

        let buffer = self.hsd_struct.get_buffer(0x10);
        let envelope_weights = self.envelope_weights();

        let reader = crate::dat::Stream::new(buffer);
        let mut primitives = Vec::new();

        while !reader.finished() {
            let b = reader.read_byte();
            if b == 0 { break }
            //println!("pgroup");

            let primitive_type = PrimitiveType::from_u8(b).unwrap();
            let count = reader.read_i16() as usize;

            let mut vertices = Vec::with_capacity(count);

            for _ in 0..count {
                let mut pos = [0f32; 3];
                let mut bones = [0u32; 4];
                let mut weights = [0f32; 4];
                let mut tex0 = [0f32; 2];
                let mut normal = [0f32; 3];

                for attr in attributes.iter() {
                    if attr.name == AttributeName::GX_VA_NULL {
                        continue;
                    }

                    let index = match attr.typ {
                        // check GX_PrimitiveGroup.Read
                        AttributeType::GX_DIRECT => {
                            if attr.name == AttributeName::GX_VA_CLR0 {
                                todo!();
                            } else if attr.name == AttributeName::GX_VA_CLR1 {
                                todo!();
                            } else { 
                                reader.read_byte() as usize
                            }
                        }

                        AttributeType::GX_INDEX8 => reader.read_byte() as usize,
                        AttributeType::GX_INDEX16 => reader.read_i16() as usize,
                        AttributeType::GX_NONE => todo!(), // unmatched - see GX_PrimitiveGroup:45
                    };

                    if attr.typ != AttributeType::GX_DIRECT {
                        let data = attr.get_decoded_data_at(index);

                        #[allow(clippy::single_match)]
                        match attr.name {
                            AttributeName::GX_VA_POS => {
                                // shapeset?? check GX_VertexAccessor:111

                                pos[0] = data[0];
                                pos[1] = data[1];
                                pos[2] = data[2];
                            },
                            AttributeName::GX_VA_TEX0 => {
                                tex0[0] = data[0];
                                tex0[1] = data[1];
                            },
                            AttributeName::GX_VA_NRM => {
                                normal[0] = data[0];
                                normal[1] = data[1];
                                normal[2] = data[2];
                            },
                            AttributeName::GX_VA_NBT => {
                                normal[0] = data[0];
                                normal[1] = data[1];
                                normal[2] = data[2];
                                // bitan + tan as well
                            }
                            _ => (), // TODO
                        }
                    } else {
                        #[allow(clippy::single_match)]
                        match attr.name {
                            // SBHsdMesh.cs:277 (GXVertexToHsdVertex)
                            AttributeName::GX_VA_PNMTXIDX => if let Some(ref env) = envelope_weights {
                                let jobjweight = &env[index / 3];
                                weights = jobjweight.weights();

                                for (i, jobj) in jobjweight.jobjs().enumerate() {
                                    let jobj_data_ptr = jobj.hsd_struct.data.as_ptr();
                                    for (j, bone_jobj) in bone_jobjs.iter().enumerate() {
                                        if bone_jobj.hsd_struct.data.as_ptr() == jobj_data_ptr {
                                            bones[i] = j as u32;
                                            break;
                                        }
                                    }
                                }
                            }
                            _ => (), // TODO
                        }
                        // TODO
                    }
                }

                let vertex = Vertex {
                    pos: pos.into(),
                    uv: tex0.into(),
                    normal: normal.into(),
                    bones: bones.into(),
                    weights: weights.into(),
                };
                vertices.push(vertex);
            }

            let primitive = Primitive {
                vertices: vertices.into_boxed_slice(),
                primitive_type,
            };

            primitives.push(primitive);
        }

        primitives
    }
}

impl<'a> JOBJ<'a> {
    pub fn try_from_root_node<'b>(s: &'b HSDRootNode<'a>) -> Option<Self> {
        if !s.root_string.ends_with("_joint") {
            return None
        }

        Some(JOBJ::new(s.hsd_struct.clone()))
    }

    pub fn transform(&self) -> Mat4 {
        // values match StudioSB
        let rx = self.hsd_struct.get_f32(0x14);
        let ry = self.hsd_struct.get_f32(0x18);
        let rz = self.hsd_struct.get_f32(0x1C);
        let sx = self.hsd_struct.get_f32(0x20);
        let sy = self.hsd_struct.get_f32(0x24);
        let sz = self.hsd_struct.get_f32(0x28);
        let tx = self.hsd_struct.get_f32(0x2C);
        let ty = self.hsd_struct.get_f32(0x30);
        let tz = self.hsd_struct.get_f32(0x34);

        let trans = Vec3::new(tx, ty, tz); // matches
        let scale = Vec3::new(sx, sy, sz);
        let mut qrot = Quat::from_euler(glam::EulerRot::ZYX, rz, ry, rx);

        // Tools/CrossMath.cs:36
        // I have no idea why this is necessary
        // matches StudioSB though!
        if qrot.w < 0.0 {
            qrot.x *= -1.0;
            qrot.y *= -1.0;
            qrot.z *= -1.0;
            qrot.w *= -1.0;
        }

        // matches!
        Mat4::from_scale_rotation_translation(scale, qrot, trans)
    }

    pub fn new(hsd_struct: HSDStruct<'a>) -> Self {
        JOBJ {
            hsd_struct
        }
    }

    pub fn check_flag(&self, flag: JOBJFlag) -> bool {
        let flags = self.hsd_struct.get_i32(0x04) as u32;
        (flags & flag as u32) != 0
    }

    pub fn get_dobj<'b>(&'b self) -> Option<DOBJ<'a>> {
        if self.check_flag(JOBJFlag::Spline) || self.check_flag(JOBJFlag::PTCL) {
            None
        } else {
            let r = self.hsd_struct.try_get_reference(0x10);
            r.map(DOBJ::new)
        }
    }

    /// Includes self
    pub fn siblings<'b>(&'b self) -> impl Iterator<Item=JOBJ<'a>> {
        std::iter::successors(Some(self.clone()), |ch| ch.get_sibling())
    }

    /// Does not include self
    pub fn children<'b>(&'b self) -> impl Iterator<Item=JOBJ<'a>> {
        let child = self.get_child();
        std::iter::successors(child, |ch| ch.get_sibling())
    }

    pub fn get_sibling<'b>(&'b self) -> Option<JOBJ<'a>> {
        self.hsd_struct.try_get_reference(0x0C).map(JOBJ::new)
    }

    pub fn get_child<'b>(&'b self) -> Option<JOBJ<'a>> {
        self.hsd_struct.try_get_reference(0x08).map(JOBJ::new)
    }

    pub fn get_all_jobjs<'b>(&'b self) -> Vec<JOBJ<'a>> {
        let mut jobjs = Vec::new();
        self.add_jobjs(&mut jobjs);
        jobjs
    }

    fn add_jobjs<'b>(&'b self, jobjs: &mut Vec<JOBJ<'a>>) {
        jobjs.push(self.clone());

        if let Some(sibling) = self.get_sibling() {
            sibling.add_jobjs(jobjs);
        }

        if let Some(child) = self.get_child() {
            child.add_jobjs(jobjs);
        }
    }
}

#[derive(Copy, Clone)]
pub enum JOBJFlag {
    Skeleton = (1 << 0),
    SkeletonRoot = (1 << 1),
    EnvelopeModel = (1 << 2),
    ClassicalScaling = (1 << 3),
    Hidden = (1 << 4),
    PTCL = (1 << 5),
    MTXDirty = (1 << 6),
    Lighting = (1 << 7),
    TexGen = (1 << 8),
    Billboard = (1 << 9),
    VBillboard = (2 << 9),
    HBillboard = (3 << 9),
    RBillboard = (4 << 9),
    Instanca = (1 << 12),
    PBillboard = (1 << 13),
    Spline = (1 << 14),
    FlipIK = (1 << 15),
    Specular = (1 << 16),
    UseQuaternion = (1 << 17),
    OPA = (1 << 18),
    XLU = (1 << 19),
    TexEdge = (1 << 20),
    Null = (0 << 21),
    Joint1 = (1 << 21),
    Joint2 = (2 << 21),
    Effector = (3 << 21),
    UserDefinedMTX = (1 << 23),
    MTXIndependParent = (1 << 24),
    MTXIndependSRT = (1 << 25),
    RootOPA = (1 << 28),
    RootXLU = (1 << 29),
    RootTexEdge = (1 << 30),

    // custom
    MTXScaleCompensate = (1 << 26),
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AttributeType {
    GX_NONE = 0,
    GX_DIRECT = 1,
    GX_INDEX8 = 2,
    GX_INDEX16 = 3,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AttributeName {
    GX_VA_PNMTXIDX = 0,    // position/normal matrix index
    GX_VA_TEX0MTXIDX,      // texture 0 matrix index
    GX_VA_TEX1MTXIDX,      // texture 1 matrix index
    GX_VA_TEX2MTXIDX,      // texture 2 matrix index
    GX_VA_TEX3MTXIDX,      // texture 3 matrix index
    GX_VA_TEX4MTXIDX,      // texture 4 matrix index
    GX_VA_TEX5MTXIDX,      // texture 5 matrix index
    GX_VA_TEX6MTXIDX,      // texture 6 matrix index
    GX_VA_TEX7MTXIDX,      // texture 7 matrix index
    GX_VA_POS = 9,    // position
    GX_VA_NRM,             // normal
    GX_VA_CLR0,            // color 0
    GX_VA_CLR1,            // color 1
    GX_VA_TEX0,            // input texture coordinate 0
    GX_VA_TEX1,            // input texture coordinate 1
    GX_VA_TEX2,            // input texture coordinate 2
    GX_VA_TEX3,            // input texture coordinate 3
    GX_VA_TEX4,            // input texture coordinate 4
    GX_VA_TEX5,            // input texture coordinate 5
    GX_VA_TEX6,            // input texture coordinate 6
    GX_VA_TEX7,            // input texture coordinate 7

    GX_POS_MTX_ARRAY,      // position matrix array pointer
    GX_NRM_MTX_ARRAY,      // normal matrix array pointer
    GX_TEX_MTX_ARRAY,      // texture matrix array pointer
    GX_LIGHT_ARRAY,        // light parameter array pointer
    GX_VA_NBT,             // normal, bi-normal, tangent 
    GX_VA_MAX_ATTR,        // maximum number of vertex attributes

    GX_VA_NULL = 0xff  // NULL attribute (to mark end of lists)
}

impl AttributeName {
    pub fn from_u8(n: u8) -> Self {
        match n {
            0 => Self::GX_VA_PNMTXIDX,    
            1 => Self::GX_VA_TEX0MTXIDX,  
            2 => Self::GX_VA_TEX1MTXIDX,  
            3 => Self::GX_VA_TEX2MTXIDX,  
            4 => Self::GX_VA_TEX3MTXIDX,  
            5 => Self::GX_VA_TEX4MTXIDX,  
            6 => Self::GX_VA_TEX5MTXIDX,  
            7 => Self::GX_VA_TEX6MTXIDX,  
            8 => Self::GX_VA_TEX7MTXIDX,  
            9 => Self::GX_VA_POS,
            10 => Self::GX_VA_NRM,         
            11 => Self::GX_VA_CLR0,        
            12 => Self::GX_VA_CLR1,        
            13 => Self::GX_VA_TEX0,        
            14 => Self::GX_VA_TEX1,        
            15 => Self::GX_VA_TEX2,        
            16 => Self::GX_VA_TEX3,        
            17 => Self::GX_VA_TEX4,        
            18 => Self::GX_VA_TEX5,        
            19 => Self::GX_VA_TEX6,        
            20 => Self::GX_VA_TEX7,        
            21 => Self::GX_POS_MTX_ARRAY,  
            22 => Self::GX_NRM_MTX_ARRAY,  
            23 => Self::GX_TEX_MTX_ARRAY,  
            24 => Self::GX_LIGHT_ARRAY,    
            25 => Self::GX_VA_NBT,         
            26 => Self::GX_VA_MAX_ATTR,    
            0xff => Self::GX_VA_NULL,
            _ => panic!("Unknown attribute")
        }
    }
}

impl AttributeType {
    pub fn from_u8(n: u8) -> Self {
        match n {
            0 => Self::GX_NONE,
            1 => Self::GX_DIRECT,
            2 => Self::GX_INDEX8,
            3 => Self::GX_INDEX16,
            _ => panic!("Unknown attribute")
        }
    }
}

