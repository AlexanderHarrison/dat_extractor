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
        let mut vertices = Vec::new();

        vertices
    }
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

