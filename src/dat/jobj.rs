use crate::dat::{HSDStruct, HSDRootNode};

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

impl<'a> DOBJ<'a> {
    pub fn new(hsd_struct: HSDStruct<'a>) -> Self {
        Self {
            hsd_struct
        }
    }

    pub fn get_pobj<'b>(&'b self) -> POBJ<'a> {
        POBJ::new(self.hsd_struct.get_reference(0x0C))
    }

    /// Includes self
    pub fn siblings<'b>(&'b self) -> impl Iterator<Item=DOBJ<'a>> {
        std::iter::successors(Some(self.clone()), |ch| ch.get_sibling())
    }

    pub fn get_sibling<'b>(&'b self) -> Option<DOBJ<'a>> {
        self.hsd_struct.try_get_reference(0x04).map(|s| DOBJ::new(s))
    }

    pub fn decode_vertices<'b>(&'b self) -> Vec<[f32; 3]> {
        let pobj = self.get_pobj();
        let attributes = pobj.get_attributes();

        let buffer = self.hsd_struct.get_buffer(0x10);

        let reader = crate::dat::Stream::new(buffer);
        let mut vertices = Vec::new();

        while !reader.finished() {
            let b = reader.read_byte();
            assert!(b != 0); // check GX_PrimitiveGroup.Read()
            let primitive = PrimitiveType::from_u8(b).unwrap();
            let count = reader.read_i16() as u16;

            for _ in 0..count {
                let mut vertex = [0f32; 3];

                for attr in attributes.iter() {
                    if attr.name == AttributeName::GX_VA_NULL {
                        continue;
                    }

                    let index = match attr.typ {
                        // check GX_PrimitiveGroup.Read
                        AttributeType::GX_DIRECT => todo!(),

                        AttributeType::GX_INDEX8 => reader.read_byte() as usize,
                        AttributeType::GX_INDEX16 => reader.read_i16() as usize,
                        AttributeType::GX_NONE => continue, // maybe?????
                    };

                    if attr.typ != AttributeType::GX_DIRECT {
                        let data = attr.get_decoded_data_at(index);

                        match attr.name {
                            AttributeName::GX_VA_POS => {
                                // shapeset?? check GX_VertexAccessor:111

                                match data.len() {
                                    0 => vertex[0] = data[0],
                                    1 => {
                                        vertex[0] = data[0];
                                        vertex[1] = data[1];
                                    }
                                    2 => {
                                        vertex[0] = data[0];
                                        vertex[1] = data[1];
                                        vertex[2] = data[2];
                                    }
                                    _ => {}
                                }
                            },
                            _ => (), // TODO
                        }
                    } else {
                        // TODO
                    }
                }

                vertices.push(vertex);
            }
        }


        vertices
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Attribute {
    pub name: AttributeName,
    pub typ: AttributeType,
}

impl Attribute {
    pub fn new(hsd_struct: HSDStruct) -> Self {
        Self { 
            name: AttributeName::from_u8(hsd_struct.get_i32(0x00) as u8),
            typ: AttributeType::from_u8(hsd_struct.get_i32(0x04) as u8),
        }
    }
    
    pub fn get_decoded_data_at(&self, loc: usize) -> [f32; 4] {
        todo!()
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
    pub fn siblings<'b>(&'b self) -> impl Iterator<Item=POBJ<'a>> {
        std::iter::successors(Some(self.clone()), |ch| ch.get_sibling())
    }

    pub fn get_sibling<'b>(&'b self) -> Option<POBJ<'a>> {
        self.hsd_struct.try_get_reference(0x04).map(|s| POBJ::new(s))
    }

    pub fn get_attributes<'b>(&'b self) -> Vec<Attribute> {
        let attr_buf = self.hsd_struct.get_reference(0x08);
        let shape_set = self.check_flag(POBJFlag::ShapeAnim);
        assert!(!shape_set); // just a hopeful guess. check ToGXAttributes in HSD_POBJ
    
        let count = attr_buf.len() / 0x18;
        let mut attributes = Vec::new();
        for i in 0..count {
            let attr = Attribute::new(attr_buf.get_embedded_struct(i * 0x18, 0x18));
            attributes.push(attr);
        }

        attributes
    }

    pub fn check_flag<'b>(&'b self, flag: POBJFlag) -> bool {
        let flags = self.hsd_struct.get_i32(0x0C) as u32;
        (flags & flag as u32) != 0
    }

}

impl<'a> JOBJ<'a> {
    pub fn try_from_root_node<'b>(s: &'b HSDRootNode<'a>) -> Option<Self> {
        if !s.root_string.ends_with("_joint") {
            return None
        }

        Some(JOBJ::new(s.hsd_struct.clone()))
    }

    pub fn new(hsd_struct: HSDStruct<'a>) -> Self {
        JOBJ {
            hsd_struct
        }
    }

    pub fn check_flag<'b>(&'b self, flag: JOBJFlag) -> bool {
        let flags = self.hsd_struct.get_i32(0x04) as u32;
        (flags & flag as u32) != 0
    }

    pub fn get_dobj<'b>(&'b self) -> Option<DOBJ<'a>> {
        if self.check_flag(JOBJFlag::Spline) || self.check_flag(JOBJFlag::PTCL) {
            None
        } else {
            Some(DOBJ::new(self.hsd_struct.get_reference(0x10)))
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
        self.hsd_struct.try_get_reference(0x0C).map(|s| JOBJ::new(s))
    }

    pub fn get_child<'b>(&'b self) -> Option<JOBJ<'a>> {
        self.hsd_struct.try_get_reference(0x08).map(|s| JOBJ::new(s))
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

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum AttributeType {
    GX_NONE = 0,
    GX_DIRECT = 1,
    GX_INDEX8 = 2,
    GX_INDEX16 = 3,
}

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
            00 => Self::GX_VA_PNMTXIDX,    
            01 => Self::GX_VA_TEX0MTXIDX,  
            02 => Self::GX_VA_TEX1MTXIDX,  
            03 => Self::GX_VA_TEX2MTXIDX,  
            04 => Self::GX_VA_TEX3MTXIDX,  
            05 => Self::GX_VA_TEX4MTXIDX,  
            06 => Self::GX_VA_TEX5MTXIDX,  
            07 => Self::GX_VA_TEX6MTXIDX,  
            08 => Self::GX_VA_TEX7MTXIDX,  
            09 => Self::GX_VA_POS,
            11 => Self::GX_VA_NRM,         
            12 => Self::GX_VA_CLR0,        
            13 => Self::GX_VA_CLR1,        
            14 => Self::GX_VA_TEX0,        
            15 => Self::GX_VA_TEX1,        
            16 => Self::GX_VA_TEX2,        
            17 => Self::GX_VA_TEX3,        
            18 => Self::GX_VA_TEX4,        
            19 => Self::GX_VA_TEX5,        
            20 => Self::GX_VA_TEX6,        
            21 => Self::GX_VA_TEX7,        
            22 => Self::GX_POS_MTX_ARRAY,  
            23 => Self::GX_NRM_MTX_ARRAY,  
            24 => Self::GX_TEX_MTX_ARRAY,  
            25 => Self::GX_LIGHT_ARRAY,    
            26 => Self::GX_VA_NBT,         
            27 => Self::GX_VA_MAX_ATTR,    
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

