use crate::dat::{HSDRawFile, JOBJ};

#[derive(Debug)]
pub enum ExtractMeshError {
    InvalidDatFile,
}

pub fn extract_scene<'a, 'b>(file: &'a HSDRawFile<'b>) -> Result<Scene<'a, 'b>, ExtractMeshError> {
    //let mut child_to_parent = HashMap::new();
    //let mut jobj_to_bone = HashMap::new();

    let root_jobj = root_jobj(file).ok_or(ExtractMeshError::InvalidDatFile)?;
    //for jobj in root_jobj.get_all_jobjs() {

    let root_bones = root_jobj
        .siblings()
        .map(|jobj| Bone::new(jobj))
        .collect::<Vec<_>>();

    let skeleton = Skeleton { 
        root_bones
    };

    //    let parent = jobj_to_bone.get(jobj).cloned();

    //    let bone = Bone::new(jobj.clone());
    //    jobj_to_bone.insert(jobj.clone(), bone);
    //}
    
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
    pub skeleton: Skeleton<'b>,
}

pub struct Skeleton<'a> {
    pub root_bones: Vec<Bone<'a>>,
}

pub struct Bone<'a> {
    pub jobj: JOBJ<'a>,
    pub children: Vec<Bone<'a>>,
}

impl<'a> Bone<'a> {
    pub fn new(jobj: JOBJ<'a>) -> Self {
        let mut children = Vec::new();
        for child_jobj in jobj.children() {
            children.push(Bone::new(child_jobj));
        }

        Self {
            jobj,
            children,
        }
        // TODO add transform things
    }

    pub fn inspect_each<'b, F>(&'b self, f: &mut F) where
        F: FnMut(&'b Self)
    {
        f(self);
        for c in &self.children {
            c.inspect_each(f);
        }
    }
}
