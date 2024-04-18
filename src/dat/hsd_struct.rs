//use std::collections::HashMap;
use ahash::{HashMap, HashMapExt};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Eq, Clone, Debug)]
pub struct HSDStruct<'a> {
    pub data: &'a [u8],
    references: Rc<RefCell<HashMap<usize, HSDStruct<'a>>>>,
}

impl<'a> std::hash::Hash for HSDStruct<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.data.as_ptr().hash(state);
        self.data.len().hash(state);
        self.references.as_ptr().hash(state);
    }
}

impl std::cmp::PartialEq for HSDStruct<'_> {
    fn eq(&self, other: &Self) -> bool {
        let a = self.references.as_ptr() == other.references.as_ptr();
        let b = self.data.as_ptr() == other.data.as_ptr();
        let c = self.data.len() == other.data.len();
        a & b & c
    }
}

impl<'a> HSDStruct<'a> {
    pub fn new(data: &'a [u8], references: HashMap<usize, HSDStruct<'a>>) -> Self {
        Self {
            data,
            references: Rc::new(RefCell::new(references)),
        }
    }

    // returns in dfs order
    pub fn iter_joint_tree(self, child_offset: usize, sibling_offset: usize) -> impl Iterator<Item=HSDStruct<'a>> {
        let mut path = vec![self];
        std::iter::from_fn(move || {
            let next = path.last()?.clone();

            if let Some(child) = next.try_get_reference(child_offset) {
                path.push(child);
            } else {
                loop {
                    let parent = match path.pop() {
                        Some(parent) => parent,
                        None => break,
                    };
                    if let Some(sibling) = parent.try_get_reference(sibling_offset) {
                        path.push(sibling);
                        break;
                    }
                }
            }

            Some(next)
        })
    }

    pub fn iter_joint_list(self, sibling_offset: usize) -> impl Iterator<Item=HSDStruct<'a>> {
        std::iter::successors(Some(self), move |prev| prev.try_get_reference(sibling_offset))
    }


    /// stride is length of HSDStruct given by hsdraw
    // HSD_Accessor.cs:467 (HSDArrayAccessor<T>)
    pub fn get_array(&self, stride: usize, loc: usize) -> impl Iterator<Item=HSDStruct<'a>> {
        let data = self.get_reference(loc);
        let len = data.len() / stride;

        (0..len).map(move |i| data.get_embedded_struct(stride * i, stride))
    }

    /// stride is length of HSDStruct given by hsdraw
    // HSD_Accessor.cs:467 (HSDArrayAccessor<T>)
    pub fn try_get_array(&self, stride: usize, loc: usize) -> Option<impl Iterator<Item=HSDStruct<'a>>> {
        let data = match self.try_get_reference(loc) {
            Some(data) => data,
            None => return None,
        };
        let len = data.len() / stride;

        Some((0..len).map(move |i| data.get_embedded_struct(stride * i, stride)))
    }

    pub fn try_get_null_ptr_array(&self, loc: usize) -> Option<impl Iterator<Item=HSDStruct<'a>>> {
        let ptr_array = self.try_get_reference(loc)?;
        let count = (ptr_array.len() / 4) - 1;

        Some((0..count).map(move |i| ptr_array.get_reference(i * 4)))
    }

    pub fn get_embedded_struct<'b>(&'b self, loc: usize, len: usize) -> HSDStruct<'a> {
        let data = self.get_bytes(loc, len);

        let mut references = HashMap::new();

        for (&ref_loc, ref_struct) in self.references.borrow().iter() {
            if ref_loc >= loc && ref_loc < loc + len {
                references.insert(ref_loc - loc, ref_struct.clone());
            }
        }

        HSDStruct::new(data, references)
    }

    pub fn get_buffer<'b>(&'b self, loc: usize) -> &'a [u8] {
        self.get_reference(loc).data
    }

    pub fn try_get_buffer<'b>(&'b self, loc: usize) -> Option<&'a [u8]> {
        self.try_get_reference(loc).map(|s| s.data)
    }

    pub fn set_reference_struct<'b>(&'b self, loc: usize, s: HSDStruct<'a>) {
        let mut refs = self.references.borrow_mut();

        if let Some(r) = refs.get_mut(&loc) {
            *r = s;
        } else {
            refs.insert(loc, s);
        }
        //self.set_i32(loc, 0); //----------------------------------------------- MAYBE
    }

    pub fn reference_count(&self) -> usize {
        self.references.borrow().len()
    }

    pub fn print_reference_locations(&self) {
        for r in self.references.borrow().keys() {
            println!("loc {}", r);
        }
    }

    pub fn get_references(&self) -> Rc<RefCell<HashMap<usize, HSDStruct<'a>>>> {
        self.references.clone()
    }

    pub fn get_bytes<'b>(&'b self, location: usize, len: usize) -> &'a [u8] {
        &self.data[location..location+len]
    }

    pub fn len(&self) -> usize {
        self.data.len()
    }

    pub fn get_reference<'b>(&'b self, offset: usize) -> HSDStruct<'a> {
        self.references.borrow()[&offset].clone()
    }

    pub fn try_get_reference<'b>(&'b self, offset: usize) -> Option<HSDStruct<'a>> {
        self.references.borrow().get(&offset).cloned()
    }

    pub fn max_key(&self) -> Option<usize> {
        self.references.borrow().keys().copied().max()
    }

    pub fn get_i8(&self, loc: usize) -> i8 {
        self.data[loc] as i8
    }

    pub fn get_u8(&self, loc: usize) -> u8 {
        self.data[loc]
    }

    pub fn get_i16(&self, loc: usize) -> i16 {
        let bytes: [u8; 2] = self.data[loc..loc+2].try_into().unwrap();
        i16::from_be_bytes(bytes)
    }

    pub fn get_u16(&self, loc: usize) -> u16 {
        let bytes: [u8; 2] = self.data[loc..loc+2].try_into().unwrap();
        u16::from_be_bytes(bytes)
    }

    pub fn get_i32(&self, loc: usize) -> i32 {
        let bytes: [u8; 4] = self.data[loc..loc+4].try_into().unwrap();
        i32::from_be_bytes(bytes)
    }

    pub fn get_u32(&self, loc: usize) -> u32 {
        let bytes: [u8; 4] = self.data[loc..loc+4].try_into().unwrap();
        u32::from_be_bytes(bytes)
    }

    pub fn get_f32(&self, loc: usize) -> f32 {
        let bytes: [u8; 4] = self.data[loc..loc+4].try_into().unwrap();
        f32::from_be_bytes(bytes)
    }

    pub fn get_string(&self, loc: usize) -> &std::ffi::CStr {
        std::ffi::CStr::from_bytes_until_nul(self.get_buffer(loc)).unwrap()
    }
}

