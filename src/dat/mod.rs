mod hsd_struct;
pub use hsd_struct::*;

mod jobj;
pub use jobj::{DOBJ, JOBJ};

mod extract_mesh;
pub use extract_mesh::*;

mod extract_anims;
pub use extract_anims::*;

mod extract_effects;
pub use extract_effects::*;

mod fighter_data;
pub use fighter_data::*;

mod textures;
pub use textures::*;

use ahash::{HashMap, HashSet, HashMapExt, HashSetExt};
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
        let mut struct_cache: Vec<HSDStruct> = Vec::with_capacity(256);
        let mut struct_cache_to_offset: HashMap<HSDStruct, usize> = HashMap::with_capacity(256);
        let mut roots: Vec<HSDRootNode> = Vec::with_capacity(2);
        let mut references: Vec<HSDRootNode> = Vec::new();

        // Parse Header -----------------------------
        let fsize = r.read_i32() as usize; // dat size
        let reloc_offset = r.read_i32() as usize + 0x20;
        let reloc_count = r.read_i32() as usize;
        let root_count = r.read_i32() as usize;
        let ref_count = r.read_i32() as usize;
        let version_chars = r.read_chars(4);

        // Parse Relocation Table -----------------------------
        let mut offsets: Vec<usize> = Vec::with_capacity(256);
        let mut offset_contain: HashSet<usize> = HashSet::with_capacity(256);
        let mut reloc_offsets: HashMap<usize, usize> = HashMap::with_capacity(256);

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
                    dbg!()
                }
            }
        }

        // Parse Roots ---------------------------------
        r.set_cursor(reloc_offset + reloc_count * 4);
        let mut root_offsets: Vec<usize> = Vec::with_capacity(2);
        let mut root_strings: Vec<&'a str> = Vec::with_capacity(2);
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

        let mut offset_to_struct: HashMap<usize, HSDStruct> = HashMap::with_capacity(256);
        let mut offset_to_offsets: HashMap<usize, Vec<usize>>  = HashMap::with_capacity(256);
        let mut offset_to_inner_offsets: HashMap<usize, Vec<usize>>  = HashMap::with_capacity(256);

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
                        // Removed this section because I don't want arbitrary SetReferenceStructs.
                        eprintln!("todo add goto pointer to subaction");
                        //let len = prev.len();
                        //prev.Resize(prev.len() + 8);
                        //prev.SetInt32(len, 0x1C000000);
                        //prev.SetReferenceStruct(len + 4, Some(s));
                        //continue;
                    }
                }
            }
        }

        // debug information
        // it all matches HSDRaw!
        //println!("struct_cache              {}", struct_cache.len());
        //println!("struct_cache_to_offset    {}", struct_cache_to_offset.len());
        //println!("roots                     {}", roots.len());
        //println!("references                {}", references.len());
        //println!("offsets                   {}", offsets.len());
        //println!("offset_contain            {}", offset_contain.len());
        //println!("reloc_offsets             {}", reloc_offsets.len());
        //println!("offset_to_struct          {}", offset_to_struct.len());
        //println!("offset_to_offsets         {}", offset_to_offsets.len());
        //println!("offset_to_inner_offsets   {}", offset_to_inner_offsets.len());
        //println!("orphans                   {}", orphans.len());
        //println!("root_offsets              {}", root_offsets.len());
        //println!("root_strings              {}", root_strings.len());
        //println!("ref_offsets               {}", ref_offsets.len());
        //println!("ref_strings               {}", ref_strings.len());

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

#[derive(Clone)]
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

    pub fn read_u16(&self) -> u16 {
        let bytes: [u8; 2] = self.data[self.cursor()..self.cursor() + 2].try_into().unwrap();
        self.bump_cursor(2);
        u16::from_be_bytes(bytes)
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
