#![allow(unused, non_camel_case_types)]

mod enums;
pub use enums::*;

pub type DatFile<'a> = &'a [u8];

pub trait HSDStruct {
    const SIZE: u32;
}

macro_rules! hsd_struct_fn {
    ($inner_offset:expr, $field:ident : string $n:ty) => {
        pub fn $field<'a>(self, dat: DatFile<'a>) -> Option<StringRef> { 
            let start = (self.offset+$inner_offset) as usize;
            let offset = u32::from_be_bytes(dat[start..start+4].try_into().unwrap());
            if offset != 0 {
                Some(StringRef { offset })
            } else {
                None
            }
        }
    };

    ($inner_offset:expr, $field:ident : bool) => {
        pub fn $field<'a>(self, dat: DatFile<'a>) -> bool { 
            let start = (self.offset+$inner_offset) as usize;
            dat[start] != 0
        }
    };

    ($inner_offset:expr, $field:ident : array $n:ty, $count:expr) => {
        pub fn $field<'a>(self, dat: DatFile<'a>) -> [$n; $count] { 
            let mut arr = [Default::default(); $count];

            let start = (self.offset+$inner_offset) as usize;
            for i in 0..$count {
                let ele = start + i*std::mem::size_of::<$n>();
                arr[i] = <$n>::from_be_bytes(dat[ele..ele+std::mem::size_of::<$n>()].try_into().unwrap());
            }

            arr
        }
    };

    ($inner_offset:expr, $field:ident : num $n:ty) => {
        pub fn $field<'a>(self, dat: DatFile<'a>) -> $n { 
            let start = (self.offset+$inner_offset) as usize;
            let size = std::mem::size_of::<$n>();
            <$n>::from_be_bytes(dat[start..start+size].try_into().unwrap())
        }
    };

    ($inner_offset:expr, $field:ident : ref $n:ty) => {
        pub fn $field<'a>(self, dat: DatFile<'a>) -> Option<$n> { 
            let start = (self.offset+$inner_offset) as usize;
            let offset = u32::from_be_bytes(dat[start..start+4].try_into().unwrap());
            if offset != 0 {
                Some( <$n>::from_offset(offset) )
            } else {
                None
            }
        }
    };

    ($inner_offset:expr, $field:ident : flags $n:ty) => {
        pub fn $field<'a>(self, dat: DatFile<'a>) -> $n { 
            let start = (self.offset+$inner_offset) as usize;
            let size = std::mem::size_of::<$n>();
            let flags = <$n>::from_be_bytes(dat[start..start+size].try_into().unwrap());
            flags
        }
    };
}

macro_rules! hsd_struct {
    ($name:ident, $size:expr, $($offset:expr => $field:ident : $mode:ident ($($get:tt)*)),* $(,)?) => {
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        pub struct $name { pub offset: u32 }
        impl HSDStruct for $name {
            const SIZE: u32 = $size;
        }

        impl $name {
            pub fn from_offset(offset: u32) -> Self { $name { offset } }

            $(hsd_struct_fn!($offset, $field : $mode $($get)*);)*
        }
    }
}

// dat file roots -------------------------------------

pub struct DatFileHead<'a> {
    root_offsets: &'a [u32],
    root_string_offsets: &'a [u32],

    reference_offsets: &'a [u32],
    reference_string_offsets: &'a [u32],
}

pub fn parse_dat_file<'a>(dat: DatFile<'_>, bump: &'a Bump) -> Option<DatFileHead<'a>> {
    let fsize        = u32::from_be_bytes(dat[00..04].try_into().unwrap());
    let reloc_offset = u32::from_be_bytes(dat[04..08].try_into().unwrap()) + 0x20;
    let reloc_count  = u32::from_be_bytes(dat[08..12].try_into().unwrap());
    let root_count   = u32::from_be_bytes(dat[12..16].try_into().unwrap());
    let ref_count    = u32::from_be_bytes(dat[16..20].try_into().unwrap());
    //let version_chars = r.read_chars(4);

    let root_start = reloc_offset + reloc_count * 4;
    let string_start = root_start + (ref_count + root_count) * 8;

    let mut root_offsets = bump.alloc_slice_fill_copy(root_count as usize, 0);
    let mut root_string_offsets = bump.alloc_slice_fill_copy(root_count as usize, 0);
    let mut reference_offsets = bump.alloc_slice_fill_copy(root_count as usize, 0);
    let mut reference_string_offsets = bump.alloc_slice_fill_copy(root_count as usize, 0);

    // parse roots -----------------------------

    for i in 0..root_count {
        let root_offset_start = root_start + i*8;
        let root_offset = u32::from_be_bytes(dat[root_offset_start..root_offset_start+4].try_into().unwrap()) + 0x20;
        let root_string_offset_start = root_start + i*8 + 4;
        let root_string_offset = u32::from_be_bytes(dat[root_string_offset_start..root_string_offset_start+4].try_into().unwrap()) + 0x20;

        root_offsets.push(root_offset);
        let j = r.read_i32() as usize;
        let rstring = r.read_string(string_start + j);
        root_strings.push(rstring);
    }
}

// mesh and textures -----------------------------------------

#[derive(Copy, Clone, Debug)]
pub struct StringRef { pub offset: u32 }

hsd_struct!(JOBJ, 0x40,
    0x00 => class_name  : string(Option<StringRef>),
    0x04 => flags       : flags(JOBJFlags),
    0x08 => child       : ref(JOBJ),
    0x0C => sibling     : ref(JOBJ),

    // depends on flags - usually DOBJ
    0x10 => dobj        : ref(DOBJ),
    //0x10 => spline         : ref Spline,
    //0x10 => particle_joint : ref ParticleJoint,

    0x14 => rotation    : array(f32, 3),
    0x20 => scale       : array(f32, 3),
    0x2C => translation : array(f32, 3),
    0x38 => inverse_world_transform: array(f32, 12),

    0x3C => robj: ref(ROBJ),
);

hsd_struct!(DOBJ, 0x10,
    0x00 => class_name  : string(Option<StringRef>),
    0x04 => next: ref(DOBJ),
    0x08 => mobj: ref(MOBJ),
    0x0C => pobj: ref(POBJ),
);

hsd_struct!(ROBJ, 0x0C,
    0x00 => next: ref(ROBJ),
    0x04 => flags: flags(ROBJFlags),
    //0x08 => reference: ref(HsdStruct), TODO
);

hsd_struct!(POBJ, 0x18,
    0x04 => next: ref(POBJ),
    // 0x08 => attributes: ???
    0x0C => flags: flags(POBJFlags),
    0x0E => display_list_size_div32: num(u16),
    0x10 => display_list_buffer_offset: num(u32),
);

impl POBJ {
    pub fn display_list_buffer_bytes<'a>(self, dat: DatFile<'a>) -> Option<&[u8]> {
        let size = self.display_list_size_div32(dat) as usize * 32;
        let offset = self.display_list_buffer_offset(dat) as usize;
        if size != 0 && offset != 0 {
            Some(&dat[offset..offset+size])
        } else {
            None
        }
    }
}

hsd_struct!(MOBJ, 0x18,
    0x04 => render_mode_flags: flags(MOBJRenderModeFlags),
    0x08 => textures: ref(TOBJ),
    0x0C => materials: ref(Material),
    0x14 => pe_desc: ref(PEDesc),
);

hsd_struct!(Material, 0x14,
    0x00 => ambient_rgba: array(u8, 4),
    0x04 => diffuse_rgba: array(u8, 4),
    0x08 => specular_rgba: array(u8, 4),
    0x0C => alpha: num(f32),
    0x10 => shininess: num(f32),
);

hsd_struct!(PEDesc, 0x0C,
    0x00 => pixel_process_flags: flags(PixelProcessFlags),
    0x01 => alpha_ref_0: num(u8),
    0x02 => alpha_ref_2: num(u8),
    0x03 => destination_alpha: num(u8),
    0x04 => blend_mode: flags(BlendMode),
    0x05 => blend_factor_src: flags(BlendFactor),
    0x06 => blend_factor_dst: flags(BlendFactor),
    0x07 => blend_operation: flags(LogicOp),
    0x08 => depth_function: flags(CompareType),
    0x09 => alpha_compare_0: flags(CompareType),
    0x0A => alpha_operation: flags(AlphaOp),
    0x0B => alpha_compare_1: flags(CompareType),
);

hsd_struct!(TOBJ, 0x5C,
    0x00 => class_name: string(Option<StringRef>),
    0x04 => next: ref(TOBJ),
    0x08 => tex_map_id: flags(TexMapID),
    0x0C => tex_gen_str: flags(TexGenSrc),
    0x10 => rotation: array(f32, 3),
    0x1C => scale: array(f32, 3),
    0x28 => translation: array(f32, 3),
    0x34 => wrap_mode_s: flags(WrapMode),
    0x38 => wrap_mode_t: flags(WrapMode),
    0x3C => repeat_u: num(u8),
    0x3D => repeat_v: num(u8),
    0x40 => flags: flags(TOBJFlags),
    0x44 => blending: num(f32),
    0x48 => mag_filter: flags(TexFilter),

    0x4C => image: ref(Image),
    0x50 => tlut: ref(Tlut),
    0x54 => lod: ref(TOBJ_LOD),
    0x58 => tev: ref(TOBJ_TEV),
);

hsd_struct!(Image, 0x18,
    0x00 => data_buffer_offset: num(u32),
    0x04 => width: num(u16),
    0x06 => height: num(u16),
    0x08 => format: flags(TexFormat),
    0x0C => mipmap: num(u16),
    0x10 => min_lod: num(f32),
    0x14 => max_lod: num(f32),
);

hsd_struct!(Tlut, 0x20,
    0x00 => data_buffer_offset: num(u32),
    0x04 => format: flags(TlutFormat),
    0x08 => gx_tlut: num(u32), // doesn't seem to be used
    0x0C => colour_count: num(u16),
);

hsd_struct!(TOBJ_LOD, 0x10,
    0x00 => min_filter: flags(TexFilter),
    0x04 => bias: num(f32),
    0x08 => bias_clamp: bool(),
    0x09 => edge_lod: bool(),
    0x0A => anisotropy: flags(Anisotropy),
);

hsd_struct!(TOBJ_TEV, 0x20,
    0x00 => colour_op: flags(TevColourOp),
    0x01 => alpha_op: flags(TevAlphaOp),
    0x02 => colour_bias: flags(TevBias),
    0x03 => alpha_bias: flags(TevBias),
    0x04 => colour_scale: flags(TevScale),
    0x05 => alpha_scale: flags(TevScale),
    0x06 => colour_clamp: bool(),
    0x07 => alpha_clamp: bool(),
    0x08 => colour_a: num(u8),
    0x09 => colour_b: num(u8),
    0x0A => colour_c: num(u8),
    0x0B => colour_d: num(u8),
    0x0C => alpha_a: num(u8),
    0x0D => alpha_b: num(u8),
    0x0E => alpha_c: num(u8),
    0x0F => alpha_d: num(u8),

    0x10 => const_colour: array(u8, 4),
    0x14 => tev_0: array(u8, 4),
    0x18 => tev_1: array(u8, 4),
    0x1C => active: flags(TEVActive),
);
