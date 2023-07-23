mod hsd_struct;
use hsd_struct::HSDStruct;

mod jobj;
pub use jobj::JOBJ;

mod extract_mesh;
pub use extract_mesh::{Model, Bone, extract_model, Primitive, PrimitiveType, Vertex};

mod extract_anims;
pub use extract_anims::{extract_anims, Animation, AnimationFrame};

mod fighter_data;
pub use fighter_data::{FighterData, FighterAction, parse_fighter_data};

mod textures;

use std::collections::{HashMap, HashSet};
use std::rc::Rc;

#[derive(Copy, Clone, Debug)]
pub enum DatExtractError {
    InvalidDatFile,
    CharacterMismatch,
}

#[derive(Clone, Debug)]
pub struct DatFile {
    pub filename: Rc<str>,
    pub data: Rc<[u8]>,
}

impl DatFile {
    pub fn stream(&self) -> Stream {
        Stream::new(&self.data)
    }
}

#[derive(Debug, Clone)]
pub struct HSDRootNode<'a> {
    pub root_string: &'a str,
    pub hsd_struct: HSDStruct<'a>,
}

/// Generic over any specific type of dat file.
#[derive(Debug)]
pub struct HSDRawFile<'a> {
    pub version_chars: &'a str,

    pub roots: Vec<HSDRootNode<'a>>,
    pub references: Vec<HSDRootNode<'a>>,

    pub struct_cache: Vec<HSDStruct<'a>>,
    pub struct_cache_to_offset: HashMap<HSDStruct<'a>, usize>,
}

impl<'a> HSDRawFile<'a> {
    pub fn new(r: &'a DatFile) -> Self {
        Self::open(Stream::new(&r.data))
    }

    /// I have no idea what is happening here.
    /// This is straight up copied from HSDRaw.
    /// It works and I do not want to ever look at this again.
    pub fn open(r: Stream<'a>) -> Self {
        let mut struct_cache: Vec<HSDStruct> = Vec::new();
        let mut struct_cache_to_offset: HashMap<HSDStruct, usize> = HashMap::new();
        let mut roots: Vec<HSDRootNode> = Vec::new();
        let mut references: Vec<HSDRootNode> = Vec::new();

        // Parse Header -----------------------------
        let fsize = r.read_i32() as usize; // dat size
        let reloc_offset = r.read_i32() as usize + 0x20;
        let reloc_count = r.read_i32() as usize;
        let root_count = r.read_i32() as usize;
        let ref_count = r.read_i32() as usize;
        let version_chars = r.read_chars(4);

        // Parse Relocation Table -----------------------------
        let mut offsets: Vec<usize> = Vec::new();
        let mut offset_contain: HashSet<usize> = HashSet::new();
        let mut reloc_offsets: HashMap<usize, usize> = HashMap::new();

        offsets.push(reloc_offset);

        for i in 0..reloc_count {
            r.set_cursor(reloc_offset + 4 * i);
            let offset = r.read_i32() as usize + 0x20;

            r.set_cursor(offset);
            let object_off = r.read_i32();
            if object_off < 0 { continue; }
            let object_off = object_off as usize + 0x20;

            // if we need to read past end of file then we need to include filesize as an offset
            // this fixes files that had previously been manually relocated to end of file
            if object_off > reloc_offset && !offset_contain.contains(&fsize) {
                offsets.push(fsize);
            }

            reloc_offsets.insert(offset, object_off);

            if !offset_contain.contains(&object_off) {
                offset_contain.insert(object_off);

                if object_off % 4 == 0 {
                    offsets.push(object_off);
                } else {
                    //Debug.WriteLine(object_off + " " + (reloc_offset + 4 * i).ToString("X"));
                    dbg!()
                }
            }
        }

        // Parse Roots ---------------------------------
        r.set_cursor(reloc_offset + reloc_count * 4);
        let mut root_offsets: Vec<usize> = Vec::new();
        let mut root_strings: Vec<&'a str> = Vec::new();
        let mut ref_offsets: Vec<usize> = Vec::new();
        let mut ref_strings: Vec<&'a str> = Vec::new();
        let string_start = r.cursor() + (ref_count + root_count) * 8;

        for _ in 0..root_count {
            root_offsets.push(r.read_i32() as usize + 0x20);

            let j = r.read_i32() as usize;
            let rstring = r.read_string(string_start + j);
            root_strings.push(rstring);
        }

        for _ in 0..ref_count {
            let mut refp = r.read_i32() as usize + 0x20;
            ref_offsets.push(refp);

            let j = r.read_i32() as usize;
            ref_strings.push(r.read_string(string_start + j));

            let temp = r.cursor();
            let mut special = refp;

            loop {
                r.seek(special);
                let read = r.read_i32();

                if read == 0 || read == -1 {
                    break;
                }

                special = read as usize;

                special += 0x20;

                reloc_offsets.insert(refp, special);

                refp = special;

                if !offset_contain.contains(&special) {
                    offset_contain.insert(special);
                    offsets.push(special);
                }
            }

            r.seek(temp);
        }

        for v in root_offsets.iter() {
            if !offset_contain.contains(v) {
                offset_contain.insert(*v);
                offsets.push(*v);
            }
        }

        for v in ref_offsets.iter() {
            if !offset_contain.contains(v) {
                offset_contain.insert(*v);
                offsets.push(*v);
            }
        }

        // Split Raw Struct Data --------------------------
        offsets.sort();

        let mut offset_to_struct      : HashMap<usize, HSDStruct> = HashMap::new();
        let mut offset_to_offsets     : HashMap<usize, Vec<usize>>  = HashMap::new();
        let mut offset_to_inner_offsets: HashMap<usize, Vec<usize>>  = HashMap::new();

        let mut relockeys = reloc_offsets.keys().copied().collect::<Vec<usize>>();
        relockeys.sort();

        for i in 0..(offsets.len() - 1) {
            r.set_cursor(offsets[i]);
            let data = r.read_bytes(offsets[i + 1] - offsets[i]);

            if offset_to_offsets.get(&offsets[i]).is_none() {
                let mut reloc_kets = Vec::new();
                let mut list = Vec::new();

                let min = binary_search(offsets[i], &relockeys);
                let max = binary_search(offsets[i + 1], &relockeys).map(|x| x+1);


                if let Some(min) = min {
                    if let Some(max) = max {
                        for v in min..max {
                            if relockeys[v] >= offsets[i] && relockeys[v] < offsets[i + 1] {
                                reloc_kets.push(relockeys[v]);
                                list.push(reloc_offsets[&relockeys[v]]);
                            }
                        }
                    }
                }

                offset_to_offsets.insert(offsets[i], list);
                offset_to_inner_offsets.insert(offsets[i], reloc_kets);
            }

            offset_to_struct
                .entry(offsets[i])
                .or_insert_with(|| HSDStruct::new(data, HashMap::new()));
        }


        let mut orphans: HashSet<HSDStruct> = HashSet::new();

        for s in offset_to_struct.values() {
            orphans.insert(s.clone()); 
        }
        
        // set references -------------------------
        for (_o, _s) in offset_to_struct.iter() { 

            let offsets: &[usize] = &offset_to_offsets[_o];
            let inner_offsets: &[usize] = &offset_to_inner_offsets[_o];

            // set references in struct
            for i in 0..offsets.len() {
                if offset_to_struct.contains_key(&offsets[i]) && _s.len() >= inner_offsets[i] - _o + 4 {
                    let refstruct = &offset_to_struct[&offsets[i]];
                    _s.set_reference_struct(inner_offsets[i] - _o, refstruct.clone());

                    // this not is not an orphan
                    if *refstruct != *_s && orphans.contains(refstruct) {
                        orphans.remove(refstruct);
                    }

                }
            }

            struct_cache.push(_s.clone());
            struct_cache_to_offset.insert(_s.clone(), *_o);
        }

        // set roots
        for i in 0..root_offsets.len() {
            let s = &offset_to_struct[&root_offsets[i]];
            //let a = Self::GuessAccessor(&root_strings[i], s.clone());

            roots.push(HSDRootNode { root_string: root_strings[i], hsd_struct: s.clone() });

            if orphans.contains(s) {
                orphans.remove(s);
            }
        }

        // set references
        for i in 0..ref_offsets.len() {
            let s = &offset_to_struct[&ref_offsets[i]];
            references.push(HSDRootNode { root_string: ref_strings[i], hsd_struct: s.clone() });

            if orphans.contains(s) {
                orphans.remove(s);
            }
        }

        // process special orphans
        for s in &orphans {
            // hack: if this is a subaction append it to previous struct
            if s.reference_count() > 0 {
                let maxkey = s.max_key().unwrap();
                if s.get_reference(maxkey) == *s && maxkey >= 8 && s.get_i32(maxkey - 4) == 0x1C000000 {
                    // get previous struct
                    let prev = Self::get_previous_struct(&struct_cache, s.clone());

                    // add goto pointer to subaction
                    if let Some(_prev) = prev {
                        panic!("Removed this section because I don't want arbitrary SetReferenceStructs.");
                        //let len = prev.len();
                        //prev.Resize(prev.len() + 8);
                        //prev.SetInt32(len, 0x1C000000);
                        //prev.SetReferenceStruct(len + 4, Some(s));
                        //continue;
                    }
                }
            }
        }

        Self {
            version_chars,
            struct_cache,
            struct_cache_to_offset,
            roots,
            references,
        }
    }

    fn get_previous_struct(struct_cache: &[HSDStruct<'a>], s: HSDStruct<'a>) -> Option<HSDStruct<'a>> {
        for i in 0..struct_cache.len()-1 {
            if struct_cache[i + 1] == s {
                return Some(struct_cache[i].clone());
            }
        }

        None
    }
}

pub struct Stream<'a> {
    pub data: &'a [u8],
    
    // must be mutated while references live to data
    // damn aliasing!
    cursor: std::cell::Cell<usize>, 
}

impl<'a> Stream<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        Self { data, cursor: std::cell::Cell::new(0) }
    }

    pub fn set_cursor(&self, cursor: usize) {
        self.cursor.set(cursor);
    }

    pub fn bump_cursor(&self, bump: usize) {
        let new_cursor = self.cursor.get() + bump;
        self.cursor.set(new_cursor);
    }

    pub fn finished(&self) -> bool {
        self.cursor() == self.data.len()
    }

    pub fn cursor(&self) -> usize {
        self.cursor.get()
    }

    pub fn read_i32(&self) -> i32 {
        let bytes: [u8; 4] = self.data[self.cursor()..self.cursor() + 4].try_into().unwrap();
        self.bump_cursor(4);
        i32::from_be_bytes(bytes)
    }

    pub fn read_i16(&self) -> i16 {
        let bytes: [u8; 2] = self.data[self.cursor()..self.cursor() + 2].try_into().unwrap();
        self.bump_cursor(2);
        i16::from_be_bytes(bytes)
    }

    pub fn read_chars(&self, n: usize) -> &'a str {
        let range = self.cursor()..self.cursor()+n;
        let s = std::str::from_utf8(&self.data[range]).unwrap();
        self.bump_cursor(n);
        s
    }

    pub fn pos(&self) -> usize {
        self.cursor()
    }

    pub fn skip(&self, n: isize) {
        self.set_cursor(self.cursor().wrapping_add_signed(n));
    }

    pub fn seek(&self, pos: usize) {
        self.set_cursor(pos);
    }

    pub fn read_byte(&self) -> u8 {
        let b = self.data[self.cursor()];
        self.bump_cursor(1);
        b
    }

    pub fn read_string(&self, offset: usize) -> &'a str {
        assert!(offset <= self.data.len());

        crate::parse_string(&self.data[offset..]).unwrap()
    }

    pub fn read_bytes(&self, n: usize) -> &'a [u8] {
        let v = &self.data[self.cursor()..self.cursor() + n];
        self.bump_cursor(n);
        v
    }

    #[inline(always)]
    pub fn read_const_bytes<const N: usize>(&self) -> [u8; N] {
        let bytes: [u8; N] = self.data[self.cursor()..self.cursor() + N].try_into().unwrap();
        self.bump_cursor(N);
        bytes
    }
}

/// Copied from HSDRaw.
/// The semantics in this impl are different than std's binary search.
fn binary_search<T: PartialEq + PartialOrd>(item: T, a: &[T]) -> Option<usize> {
    if a.is_empty() {
        return None;
    }

    let mut first: isize = 0;
    let mut last: isize = a.len() as isize - 1;
    let mut mid: isize = first + (last - first) / 2;

    if item > a[mid as usize] {
        first = mid + 1;
    } else {
        last = mid - 1;
    } 

    if a[mid as usize] == item {
        return Some(mid as usize);
    }

    while first <= last {
        mid = first + (last - first) / 2;
        if item > a[mid as usize] {
            first = mid + 1;
        } else {
            last = mid - 1;
        }

        if a[mid as usize] == item {
            return Some(mid as usize);
        }
    }

    Some(mid as usize)
}

/*
// might use later
fn symbol_switch(x: &str) -> Box<dyn HSDAccessor> {
    //use hsd_accessors::*;
    match x {
        //x if x.ends_with("matanim_joint")                    => Box::new(HSD_MatAnimJoint                                       ::new()) ,
        //x if x.ends_with("shapeanim_joint")                  => Box::new(HSD_ShapeAnimJoint                                     ::new()) ,
        //x if x.ends_with("_animjoint")                       => Box::new(HSD_AnimJoint                                          ::new()) ,
        //x if x.ends_with("_joint")                           => Box::new(HSD_JOBJ                                               ::new()) ,
        //x if x.ends_with("_texanim")                         => Box::new(HSD_TexAnim                                            ::new()) ,
        //x if x.ends_with("_figatree")                        => Box::new(HSD_FigaTree                                           ::new()) ,
        //x if x.ends_with("_camera")                          => Box::new(HSD_Camera                                             ::new()) ,
        //x if x.ends_with("_scene_lights")                    => Box::new(HSDNullPointerArrayAccessor<HSD_Light>                 ::new()) ,
        //x if x.ends_with("_scene_models") ||
        //    x.eq("Stc_rarwmdls") ||
        //    x.eq("Stc_scemdls") ||
        //    x.eq("lupe") ||
        //    x.eq("tdsce")                                    => Box::new(HSDNullPointerArrayAccessor<HSD_JOBJDesc>              ::new()) ,
        //x if x.ends_with("_model_set")                       => Box::new(HSD_JOBJDesc                                           ::new()) ,
        //x if x.eq("ftDataMario")                             => Box::new(SBM_ftDataMario                                        ::new()) ,
        //x if x.eq("ftDataMars")                              => Box::new(SBM_ftDataMars                                         ::new()) ,
        //x if x.eq("ftDataEmblem")                            => Box::new(SBM_ftDataMars                                         ::new()) ,
        //x if x.starts_with("ftData") && !x.Contains("Copy")  => Box::new(SBM_FighterData                                        ::new()) ,
        //x if x.ends_with("MnSelectChrDataTable")             => Box::new(SBM_SelectChrDataTable                                 ::new()) ,
        //x if x.ends_with("MnSelectStageDataTable")           => Box::new(SBM_MnSelectStageDataTable                             ::new()) ,
        //x if x.ends_with("coll_data")                        => Box::new(SBM_Coll_Data                                          ::new()) ,
        //x if x.ends_with("_fog")                             => Box::new(HSD_FogDesc                                            ::new()) ,
        //x if x.ends_with("scene_data") ||
        //    x.eq("pnlsce") ||
        //    x.eq("flmsce") ||
        //    x.starts_with("Sc")                              => Box::new(HSD_SOBJ                                               ::new()) ,
        //x if x.starts_with("map_plit")                       => Box::new(HSDNullPointerArrayAccessor<HSD_Light>                 ::new()) ,
        //x if x.starts_with("map_head")                       => Box::new(SBM_Map_Head                                           ::new()) ,
        //x if x.starts_with("grGroundParam")                  => Box::new(SBM_GroundParam                                        ::new()) ,
        //x if x.starts_with("vcDataStar")                     => Box::new(KAR_vcDataStar                                         ::new()) ,
        //x if x.starts_with("vcDataWheel")                    => Box::new(KAR_vcDataWheel                                        ::new()) ,
        //x if x.starts_with("grModelMotion")                  => Box::new(KAR_grModelMotion                                      ::new()) ,
        //x if x.starts_with("grModel")                        => Box::new(KAR_grModel                                            ::new()) ,
        //x if x.starts_with("grData")                         => Box::new(KAR_grData                                             ::new()) ,
        //x if x.ends_with("_texg")                            => Box::new(HSD_TEXGraphicBank                                     ::new()) ,
        //x if x.ends_with("_ptcl")                            => Box::new(HSD_ParticleGroup                                      ::new()) ,
        //x if x.starts_with("effBehaviorTable")               => Box::new(MEX_EffectTypeLookup                                   ::new()) ,
        //x if x.starts_with("eff")                            => Box::new(SBM_EffectTable                                        ::new()) ,
        //x if x.starts_with("itPublicData")                   => Box::new(itPublicData                                           ::new()) ,
        //x if x.starts_with("itemdata")                       => Box::new(HSDNullPointerArrayAccessor<SBM_MapItem>               ::new()) ,
        //x if x.starts_with("smSoundTestLoadData")            => Box::new(smSoundTestLoadData                                    ::new()) ,
        //x if x.starts_with("ftLoadCommonData")               => Box::new(SBM_ftLoadCommonData                                   ::new()) ,
        //x if x.starts_with("quake_model_set")                => Box::new(SBM_Quake_Model_Set                                    ::new()) ,
        //x if x.starts_with("mexData")                        => Box::new(MEX_Data                                               ::new()) ,
        //x if x.starts_with("mexMapData")                     => Box::new(MEX_mexMapData                                         ::new()) ,
        //x if x.starts_with("mexSelectChr")                   => Box::new(MEX_mexSelectChr                                       ::new()) ,
        //x if x.starts_with("mobj")                           => Box::new(HSD_MOBJ                                               ::new()) ,
        //x if x.starts_with("SIS_")                           => Box::new(SBM_SISData                                            ::new()) ,
        //x if x.eq("evMenu")                                  => Box::new(SBM_EventMenu                                          ::new()) ,
        //x if x.ends_with("ColAnimData") ||
        //    x.eq("lbBgFlashColAnimData")                     => Box::new(HSDArrayAccessor<ftCommonColorEffect>                  ::new()) ,
        //x if x.eq("ftcmd")                                   => Box::new(SBM_FighterActionTable                                 ::new()) ,
        //x if x.eq("Stc_icns")                                => Box::new(MEX_Stock                                              ::new()) ,
        //x if x.eq("mexMenu")                                 => Box::new(MEX_Menu                                               ::new()) ,
        //x if x.eq("bgm")                                     => Box::new(MEX_BGMModel                                           ::new()) ,
        //x if x.eq("mexCostume")                              => Box::new(MEX_CostumeSymbol                                      ::new()) ,
        //x if x.starts_with("mnName")                         => Box::new(HSDFixedLengthPointerArrayAccessor<HSD_ShiftJIS_String>::new()) ,
        //x if x.ends_with("move_logic")                       => Box::new(HSDArrayAccessor<MEX_MoveLogic>                        ::new()) ,
        //x if x.starts_with("em") && x.ends_with("DataGroup") => Box::new(KAR_emData                                             ::new()) ,
        //x if x.eq("stData")                                  => Box::new(KAR_stData                                             ::new()) ,
        //x if x.starts_with("rdMotion")                       => Box::new(HSDArrayAccessor<KAR_RdMotion>                         ::new()) ,
        //x if x.starts_with("vcDataCommon")                   => Box::new(KAR_vcDataCommon                                       ::new()) ,
        //x if x.starts_with("rdDataCommon")                   => Box::new(HSDAccessor                                            ::new()) , // TODO:
        //x if x.starts_with("rdData")                         => Box::new(KAR_RdData                                             ::new()) ,
        //x if x.starts_with("rdExt")                          => Box::new(KEX_RdExt                                              ::new()) ,
        //x if x.starts_with("kexData")                        => Box::new(kexData                                                ::new()) ,
        //x if x.eq("gmIntroEasyTable")                        => Box::new(SBM_gmIntroEasyTable                                   ::new()) ,
        //x if x.starts_with("tyDisplayModel")                 => Box::new(HSDArrayAccessor<SBM_tyDisplayModelEntry>              ::new()) ,
        //x if x.starts_with("tyModelFile")                    => Box::new(HSDArrayAccessor<SBM_TyModelFileEntry>                 ::new()) ,
        //x if x.starts_with("tyInitModel")                    => Box::new(HSDArrayAccessor<SBM_tyInitModelEntry>                 ::new()) ,
        //x if x.starts_with("tyModelSort")                    => Box::new(HSDArrayAccessor<SBM_tyModelSortEntry>                 ::new()) ,
        //x if x.starts_with("tyExpDifferent")                 => Box::new(HSDShortArray                                          ::new()) ,
        //x if x.starts_with("tyNoGetUsTbl")                   => Box::new(HSDShortArray                                          ::new()) ,
        //x if x.starts_with("grMurabito")                     => Box::new(HSDNullPointerArrayAccessor<SBM_GrMurabito>            ::new()) ,
        _ => todo!() // return generic
    }
}
*/

