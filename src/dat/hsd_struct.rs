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

    /// stride is length of HSDStruct I think
    // HSD_Accessor.cs:467 (HSDArrayAccessor<T>)
    pub fn get_array(&self, stride: usize, loc: usize) -> impl Iterator<Item=HSDStruct<'a>> {
        let data = self.get_reference(loc);
        let len = data.len() / stride;

        (0..len).map(move |i| data.get_embedded_struct(stride * i, stride))
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

