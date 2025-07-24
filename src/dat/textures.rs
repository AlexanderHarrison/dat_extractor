#![allow(clippy::upper_case_acronyms)]

use crate::dat::{DOBJ, HSDStruct};

use std::collections::HashMap;

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MOBJ<'a> {
    pub hsd_struct: HSDStruct<'a>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TOBJ<'a> {
    pub hsd_struct: HSDStruct<'a>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TLUT<'a> {
    pub hsd_struct: HSDStruct<'a>,
}

#[derive(Debug, Clone)]
pub struct Texture {
    pub width: usize,
    pub height: usize,
    pub rgba_data: Box<[u32]>,
    pub scale_x: f32,
    pub scale_y: f32,
    pub wrap_u: WrapMode,
    pub wrap_v: WrapMode,
}

// GX/Enums.cs:192 (GXWrapMode)
#[derive(Copy, Clone, Debug)]
pub enum WrapMode {
    Clamp,
    Repeat,
    Mirror,
}

#[derive(Copy, Clone, Debug)]
pub struct Phong {
    pub ambient: [u8; 4],
    pub diffuse: [u8; 4],
    pub specular: [u8; 4],
}

impl Default for Phong {
    fn default() -> Self {
        Phong {
            ambient: [200u8; 4],
            diffuse: [30u8; 4],
            specular: [25u8; 4],
        }
    }
}

impl From<Phong> for PhongF32 {
    fn from(p: Phong) -> Self {
        PhongF32 {
            ambient: p.ambient.map(|a| a as f32 / 255.0),
            diffuse: p.diffuse.map(|a| a as f32 / 255.0),
            specular: p.specular.map(|a| a as f32 / 255.0),
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PhongF32 {
    pub ambient: [f32; 4],
    pub diffuse: [f32; 4],
    pub specular: [f32; 4],
}

impl Default for PhongF32 {
    fn default() -> Self { Phong::default().into() }
}

unsafe impl bytemuck::NoUninit for PhongF32 {}

// GX/Enums.cs:122 (GXTexFmt)
#[derive(Copy, Clone, Debug)]
pub enum InternalTextureFormat {
    I4 = 0,
    I8 = 1,
    IA4 = 2,
    IA8 = 3,
    RGB565 = 4,
    RGB5A3 = 5,
    RGBA8 = 6,
    CI4 = 8,
    CI8 = 9,
    CI14X2 = 10,
    CMP = 14, // only value used for fox
}

impl WrapMode {
    pub fn from_u32(n: u32) -> Self {
        match n {
            0 => WrapMode::Clamp,
            1 => WrapMode::Repeat,
            2 => WrapMode::Mirror,
            _ => panic!("unknown wrap mode"),
        }
    }
}

pub fn try_decode_texture<'a>(
    cache: &mut HashMap<*const u8, u16>,
    textures: &mut Vec<Texture>,
    dobj: DOBJ<'a>
) -> Option<u16> {
    let mobj = dobj.get_mobj()?;
    let tobj = mobj.get_tobj()?;

    // There are other tobjs present that are currently unused.
    // They contain bump maps, lighting maps, etc.

    let render_mode = mobj.flags();
    if render_mode & (1 << 24) != 0 { eprintln!("unused z offset") }
    let data_ptr = tobj.image_buffer()?.as_ptr();

    use std::collections::hash_map::Entry;
    let id = match cache.entry(data_ptr) {
        Entry::Occupied(entry) => *entry.get(),
        Entry::Vacant(entry) => {
            let texture = tobj.texture().unwrap();
            let texture_idx = textures.len() as _;
            textures.push(texture);
            entry.insert(texture_idx);
            texture_idx
        }
    };

    Some(id)
}

impl<'a> TOBJ<'a> {
    pub fn new(hsd_struct: HSDStruct<'a>) -> Self {
        Self {
            hsd_struct
        }
    }

    /// Includes self
    // Fox textures have no siblings
    pub fn siblings(&self) -> impl Iterator<Item=TOBJ<'a>> {
        std::iter::successors(Some(self.clone()), |ch| ch.get_sibling())
    }

    pub fn get_sibling(&self) -> Option<TOBJ<'a>> {
        self.hsd_struct.try_get_reference(0x04).map(TOBJ::new)
    }

    // image buffers are shared between multiple tobjs, need to expose this to deduplicate
    pub fn image_buffer(&self) -> Option<&'a [u8]> {
        self.hsd_struct.try_get_reference(0x4C).map(|t| t.get_buffer(0x00))
    }

    pub fn flags(&self) -> u32 {
        self.hsd_struct.get_u32(0x40)
    }

    pub fn format(&self) -> Option<InternalTextureFormat> {
        self.hsd_struct.try_get_reference(0x4C)
            .map(|hsd_image| InternalTextureFormat::new(hsd_image.get_i32(0x08) as u32).unwrap())
    }

    // HSDScene.cs:194 (RefreshTextures)
    // HSD_TOBJ.cs:226 (GetDecodedImageData)
    // GXImageConverter.cs:77 (DecodeTPL)
    pub fn texture(&self) -> Option<Texture> {
        // TODO get other texture properties?
        let hsd_image = self.hsd_struct.try_get_reference(0x4C)?;

        let wrap_u = WrapMode::from_u32(self.hsd_struct.get_u32(0x34));
        let wrap_v = WrapMode::from_u32(self.hsd_struct.get_u32(0x38));

        let scale_x = self.hsd_struct.get_i8(0x3C) as f32 / self.hsd_struct.get_f32(0x1C);
        let scale_y = self.hsd_struct.get_i8(0x3D) as f32 / self.hsd_struct.get_f32(0x20);
        let tlut_data = self.hsd_struct.try_get_reference(0x50)
            .map(TLUT::new);

        let Image { width, height, rgba_data } = decode_image(hsd_image, tlut_data);

        Some(Texture {
            width,
            height,
            rgba_data,
            scale_x,
            scale_y,
            wrap_u,
            wrap_v,
        })
    }
}

#[derive(Clone, Debug)]
pub struct Image {
    pub width: usize,
    pub height: usize,
    pub rgba_data: Box<[u32]>,
}

/// -> width, height, rgba_data
pub fn decode_image(hsd_image: HSDStruct<'_>, tlut_data: Option<TLUT<'_>>) -> Image {
    let data_buffer = hsd_image.get_buffer(0x00);
    let width = hsd_image.get_i16(0x04) as usize;
    let height = hsd_image.get_i16(0x06) as usize;
    let format = InternalTextureFormat::new(hsd_image.get_i32(0x08) as u32).unwrap();

    let mut rgba_data = vec![0u32; width * height].into_boxed_slice();

    let palette = tlut_data.map(|tlut| tlut.palette());
    let pal_ref = palette.as_ref().map(|pal| &**pal);

    decode_data(format, width, height, data_buffer, pal_ref, &mut rgba_data);

    Image { width, height, rgba_data }
}

pub fn decode_image_preallocated(
    hsd_image: HSDStruct<'_>, 
    tlut_data: Option<TLUT<'_>>,
    rgba_data: &mut [u32],
) -> (usize, usize) {
    let data_buffer = hsd_image.get_buffer(0x00);
    let width = hsd_image.get_i16(0x04) as usize;
    let height = hsd_image.get_i16(0x06) as usize;
    let format = InternalTextureFormat::new(hsd_image.get_i32(0x08) as u32).unwrap();

    let palette = tlut_data.map(|tlut| tlut.palette());
    let pal_ref = palette.as_ref().map(|pal| &**pal);

    decode_data(format, width, height, data_buffer, pal_ref, rgba_data);

    (width, height)
}

pub fn decode_data(format: InternalTextureFormat, width: usize, height: usize, data_buffer: &[u8], palette: Option<&[u32]>, rgba_data: &mut [u32]) {
    assert!(rgba_data.len() >= width * height);

    match format {
        InternalTextureFormat::CMP => decode_compressed_image(data_buffer, width, height, rgba_data),
        InternalTextureFormat::I4 => decode_i4_image(data_buffer, width, height, rgba_data),
        InternalTextureFormat::I8 => decode_i8_image(data_buffer, width, height, rgba_data),
        InternalTextureFormat::IA4 => decode_ia4_image(data_buffer, width, height, rgba_data),
        InternalTextureFormat::IA8 => decode_ia8_image(data_buffer, width, height, rgba_data),
        InternalTextureFormat::RGBA8 => decode_rgba8_image(data_buffer, width, height, rgba_data),
        InternalTextureFormat::RGB565 => decode_rgb565_image(data_buffer, width, height, rgba_data),
        InternalTextureFormat::RGB5A3 => decode_rgb5a3_image(data_buffer, width, height, rgba_data),
        InternalTextureFormat::CI4 => {
            let palette = palette.unwrap();
            decode_ci4_image(data_buffer, &palette, width, height, rgba_data)
        }
        InternalTextureFormat::CI8 => {
            let palette = palette.unwrap();
            decode_ci8_image(data_buffer, &palette, width, height, rgba_data)
        }
        t => panic!("texture format {:?} unimplemented", t),
    };
}

pub type RenderModeFlags = u32;
pub mod render_mode_flags {
    use super::RenderModeFlags;
    pub const CONSTANT      : RenderModeFlags = 1 << 0;
    pub const VERTEX        : RenderModeFlags = 1 << 1;
    pub const DIFFUSE       : RenderModeFlags = 1 << 2;
    pub const SPECULAR      : RenderModeFlags = 1 << 3;
    pub const TEX0          : RenderModeFlags = 1 << 4;
    pub const TEX1          : RenderModeFlags = 1 << 5;
    pub const TEX2          : RenderModeFlags = 1 << 6;
    pub const TEX3          : RenderModeFlags = 1 << 7;
    pub const TEX4          : RenderModeFlags = 1 << 8;
    pub const TEX5          : RenderModeFlags = 1 << 9;
    pub const TEX6          : RenderModeFlags = 1 << 10;
    pub const TEX7          : RenderModeFlags = 1 << 11;
    pub const TOON          : RenderModeFlags = 1 << 12;
    pub const ALPHA_MAT     : RenderModeFlags = 1 << 13;
    pub const ALPHA_VTX     : RenderModeFlags = 2 << 13;
    pub const ALPHA_BOTH    : RenderModeFlags = 3 << 13;
    pub const ZOFST         : RenderModeFlags = 1 << 24;
    pub const EFFECT        : RenderModeFlags = 1 << 25;
    pub const SHADOW        : RenderModeFlags = 1 << 26;
    pub const ZMODE_ALWAYS  : RenderModeFlags = 1 << 27;
    pub const DF_ALL        : RenderModeFlags = 1 << 28;
    pub const NO_ZUPDATE    : RenderModeFlags = 1 << 29;
    pub const XLU           : RenderModeFlags = 1 << 30;
    pub const USER          : RenderModeFlags = 1 << 31;
}

impl<'a> MOBJ<'a> {
    pub fn new(hsd_struct: HSDStruct<'a>) -> Self {
        Self {
            hsd_struct
        }
    }

    pub fn flags(&self) -> RenderModeFlags {
        self.hsd_struct.get_u32(0x04)
    }

    // might never fail, but return option to be sure
    pub fn get_tobj(&self) -> Option<TOBJ<'a>> {
        self.hsd_struct.try_get_reference(0x08)
            .map(TOBJ::new)
    }

    pub fn get_material(&self) -> Option<HSDStruct<'a>> {
        self.hsd_struct.try_get_reference(0x0C)
    }

    pub fn get_phong(&self) -> Phong {
        match self.get_material() {
            None => Phong::default(),
            Some(mat) => Phong {
                ambient: [
                    mat.get_u8(0),
                    mat.get_u8(1),
                    mat.get_u8(2),
                    mat.get_u8(3),
                ],
                diffuse: [
                    mat.get_u8(4),
                    mat.get_u8(5),
                    mat.get_u8(6),
                    mat.get_u8(7),
                ],
                specular: [
                    mat.get_u8(8),
                    mat.get_u8(9),
                    mat.get_u8(10),
                    mat.get_u8(11),
                ],
            }
        }
    }
}

// GX/Enums.cs:185 (GXTlutFmt)
#[derive(Copy, Clone, Debug)]
pub enum TLUTFormat {
    IA8 = 0,
    RGB565 = 1,
    RGB5A3 = 2
}

// HSD_TOBJ.cs:373 (HSD_Tlut)
impl<'a> TLUT<'a> {
    pub fn new(hsd_struct: HSDStruct<'a>) -> Self {
        Self {
            hsd_struct
        }
    }

    // GXImageConverter.cs:356 (PaletteToRGBA)
    // TODO remove allocation
    pub fn palette(&self) -> Box<[u32]> {
        let count = self.colour_count() as usize;
        let format = self.format();
        let data = self.data();

        decode_palette(count, format, data)
    }

    pub fn data(&self) -> &'a [u8] {
        self.hsd_struct.get_buffer(0x00)
    }

    pub fn format(&self) -> TLUTFormat {
        TLUTFormat::new(self.hsd_struct.get_u32(0x04)).unwrap()
    }

    pub fn colour_count(&self) -> u16 {
        self.hsd_struct.get_u16(0x0c)
    }
}

pub fn decode_palette(count: usize, format: TLUTFormat, data: &[u8]) -> Box<[u32]> {
    let mut palette = Vec::with_capacity(count);

    for i in 0..count {
        let bytes: [u8; 2] = [data[i*2], data[i*2+1]];
        let pixel = u16::from_be_bytes(bytes) as u32;

        let r; let g; let b; let a;
        match format {
            TLUTFormat::IA8 => {
                a = pixel >> 8;
                r = pixel & 0xff;
                b = r;
                g = r;
            }
            TLUTFormat::RGB565 => {
                a = 255;
                b = (((pixel >> 11) & 0x1F) << 3) & 0xff;
                g = (((pixel >> 5) & 0x3F) << 2) & 0xff;
                r = (((pixel >> 0) & 0x1F) << 3) & 0xff;
            }
            TLUTFormat::RGB5A3 => {
                // GXImageConverter.cs:601 (DecodeRGBA3)
                if (pixel & (1 << 15)) != 0 { //RGB555
                    a = 255;
                    b = (((pixel >> 10) & 0x1F) * 255) / 31;
                    g = (((pixel >> 5) & 0x1F) * 255) / 31;
                    r = (((pixel >> 0) & 0x1F) * 255) / 31;
                } else { //RGB4A3
                    a = (((pixel >> 12) & 0x07) * 255) / 7;
                    b = (((pixel >> 8) & 0x0F) * 255) / 15;
                    g = (((pixel >> 4) & 0x0F) * 255) / 15;
                    r = (((pixel >> 0) & 0x0F) * 255) / 15;
                }
            }
        }

        palette.push(convert((r << 0) | (g << 8) | (b << 16) | (a << 24)));
    }

    palette.into_boxed_slice()
}

impl TLUTFormat {
    pub fn new(n: u32) -> Option<Self> {
        Some(match n {
            0 => TLUTFormat::IA8,
            1 => TLUTFormat::RGB565,
            2 => TLUTFormat::RGB5A3,
            _ => return None
        })
    }
}

impl InternalTextureFormat {
    pub fn new(n: u32) -> Option<Self> {
        Some(match n {
            0  => InternalTextureFormat::I4    ,
            1  => InternalTextureFormat::I8    ,
            2  => InternalTextureFormat::IA4   ,
            3  => InternalTextureFormat::IA8   ,
            4  => InternalTextureFormat::RGB565,
            5  => InternalTextureFormat::RGB5A3,
            6  => InternalTextureFormat::RGBA8 ,
            8  => InternalTextureFormat::CI4   ,
            9  => InternalTextureFormat::CI8   ,
            10 => InternalTextureFormat::CI14X2,
            14 => InternalTextureFormat::CMP   ,
            _ => return None,
        })
    }

    // Tools/Textures/GXImageConverter.cs (GetImageSize)
    pub fn data_size(self, width: usize, height: usize) -> usize {
        let width = if width % 4 != 0 { width + 4 - (width % 4) } else { width };
        let size = height * width;

        use InternalTextureFormat::*;
        match self {
            CI4 | I4 | CMP => size / 2,
            IA4 | I8 | CI14X2 | CI8 => size,
            IA8 | RGB565 | RGB5A3 => size * 2,
            RGBA8 => size * 4,
        }
    }

    pub fn is_paletted(self) -> bool {
        match self {
            InternalTextureFormat::I4     => false,
            InternalTextureFormat::I8     => false,
            InternalTextureFormat::IA4    => false,
            InternalTextureFormat::IA8    => false,
            InternalTextureFormat::RGB565 => false,
            InternalTextureFormat::RGB5A3 => false,
            InternalTextureFormat::RGBA8  => false,
            InternalTextureFormat::CI4    => true,
            InternalTextureFormat::CI8    => true,
            InternalTextureFormat::CI14X2 => true,
            InternalTextureFormat::CMP    => false,
        }
    }

    pub fn has_colour(self) -> bool {
        match self {
            InternalTextureFormat::I4     => false,
            InternalTextureFormat::I8     => false,
            InternalTextureFormat::IA4    => false,
            InternalTextureFormat::IA8    => false,
            InternalTextureFormat::RGB565 => true,
            InternalTextureFormat::RGB5A3 => true,
            InternalTextureFormat::RGBA8  => true,
            InternalTextureFormat::CI4    => true,
            InternalTextureFormat::CI8    => true,
            InternalTextureFormat::CI14X2 => true,
            InternalTextureFormat::CMP    => true,
        }
    }
}

// GXImageConverter.cs:396 (fromRGBA8)
pub fn decode_rgba8_image(data: &[u8], width: usize, height: usize, rgba_buffer: &mut [u32]) {
    let mut inp = 0;

    for y in (0..height).step_by(4) {
        for x in (0..width).step_by(4) {
            for k in 0..2 {
                for y1 in y..(y + 4) {
                    for x1 in x..(x + 4) {
                        // TODO check endianness here
                        let pixel = u16::from_be_bytes([data[inp], data[inp+1]]) as u32;
                        inp += 2;

                        if x >= width || y >= height { continue }

                        let a = (pixel >> 8) & 0xff;
                        let b = (pixel >> 0) & 0xff;
                        let (s1, s2) = ([(16, 24), (8, 0)])[k];
                        rgba_buffer[x1 + (y1 * width)] |= convert((a << s1) | (b << s2));
                    }
                }
            }
        }
    }
}

// GXImageConverter.cs:622 (fromRGB565)
pub fn decode_rgb565_image(data: &[u8], width: usize, height: usize, rgba_buffer: &mut [u32]) {
    let mut inp = 0;

    for y in (0..height).step_by(4) {
        for x in (0..width).step_by(4) {
            for y1 in y..(y + 4) {
                for x1 in x..(x + 4) {
                    let pixel = u16::from_be_bytes([data[inp], data[inp+1]]) as u32;
                    inp += 2;

                    if x >= width || y >= height { continue }

                    let b = (((pixel >> 11) & 0x1f) << 3) & 0xff;
                    let g = (((pixel >> 5) & 0x3f) << 2) & 0xff;
                    let r = (((pixel >> 0) & 0x1f) << 3) & 0xff;
                    rgba_buffer[y1 * width + x1] = convert((r << 0) | (g << 8) | (b << 16) | (255 << 24));
                }
            }
        }
    }
}

// GXImageConverter.cs:622 (fromRGB5A3)
pub fn decode_rgb5a3_image(data: &[u8], width: usize, height: usize, rgba_buffer: &mut [u32]) {
    let mut inp = 0;

    for y in (0..height).step_by(4) {
        for x in (0..width).step_by(4) {
            for y1 in y..(y + 4) {
                for x1 in x..(x + 4) {
                    let pixel = u16::from_be_bytes([data[inp], data[inp+1]]) as u32;
                    inp += 2;

                    if x >= width || y >= height { continue }

                    let a; let r; let b; let g;
                    // GXImageConverter.cs:601 (DecodeRGBA3)
                    if (pixel & (1 << 15)) != 0 { //RGB555
                        a = 255;
                        b = (((pixel >> 10) & 0x1F) * 255) / 31;
                        g = (((pixel >> 5) & 0x1F) * 255) / 31;
                        r = (((pixel >> 0) & 0x1F) * 255) / 31;
                    } else { //RGB4A3
                        a = (((pixel >> 12) & 0x07) * 255) / 7;
                        b = (((pixel >> 8) & 0x0F) * 255) / 15;
                        g = (((pixel >> 4) & 0x0F) * 255) / 15;
                        r = (((pixel >> 0) & 0x0F) * 255) / 15;
                    }

                    rgba_buffer[y1 * width + x1] = convert((r << 0) | (g << 8) | (b << 16) | (a << 24));
                }
            }
        }
    }
}

// GXImageConverter.cs:706 (fromI4)
pub fn decode_i4_image(data: &[u8], width: usize, height: usize, rgba_buffer: &mut [u32]) {
    let mut inp = 0;

    //if width < 8 || height < 8 {
    //    panic!("Invalid buffer size");
    //}

    for y in (0..height).step_by(8) {
        for x in (0..width).step_by(8) {
            for y1 in y..y+8 {
                for x1 in (x..x+8).step_by(2) {
                    let pixel = data[inp] as u32;
                    inp += 1;

                    if y1 >= height || x1 >= width { continue; }

                    let i = (pixel >> 4) * 255 / 15;
                    let idx = y1 * width + x1;
                    rgba_buffer[idx] = convert(i | (i << 8) | (i << 16) | (i << 24));

                    let i = (pixel & 0x0F) * 255 / 15;
                    if idx + 1 < rgba_buffer.len() {
                        rgba_buffer[idx + 1] = convert(i | (i << 8) | (i << 16) | (i << 24));
                    } 
                }
            }
        }
    }
}

// GXImageConverter.cs:790 (fromI8)
pub fn decode_i8_image(data: &[u8], width: usize, height: usize, rgba_buffer: &mut [u32]) {
    let mut inp = 0;

    for y in (0..height).step_by(4) {
        for x in (0..width).step_by(8) {
            for y1 in y..y+4 {
                for x1 in x..x+8 {
                    let pixel = data[inp] as u32;
                    inp += 1;

                    if y1 >= height || x1 >= width {
                        continue;
                    }

                    rgba_buffer[y1 * width + x1] = convert((pixel << 0) | (pixel << 8) | (pixel << 16) | (pixel << 24));
                }
            }
        }
    }
}

// GXImageConverter.cs:868 (fromIA4)
pub fn decode_ia4_image(data: &[u8], width: usize, height: usize, rgba_buffer: &mut [u32]) {
    let mut inp = 0;

    for y in (0..height).step_by(4) {
        for x in (0..width).step_by(8) {
            for y1 in y..y+4 {
                for x1 in x..x+8 {
                    let pixel = data[inp] as u32;
                    inp += 1;

                    if y1 >= height || x1 >= width {
                        continue;
                    }

                    let i = ((pixel & 0x0F) * 255 / 15) & 0xff;
                    let a = (((pixel >> 4) * 255) / 15) & 0xff;

                    rgba_buffer[y1 * width + x1] = convert((i << 0) | (i << 8) | (i << 16) | (a << 24));
                }
            }
        }
    }
}

// GXImageConverter.cs:940 (fromIA8)
pub fn decode_ia8_image(data: &[u8], width: usize, height: usize, rgba_buffer: &mut [u32]) {
    let mut inp = 0;

    for y in (0..height).step_by(4) {
        for x in (0..width).step_by(4) {
            for y1 in y..y+4 {
                for x1 in x..x+4 {
                    let pixel = u16::from_be_bytes([data[inp], data[inp+1]]) as u32;
                    inp += 2;

                    if y1 >= height || x1 >= width {
                        continue;
                    }

                    let a = pixel >> 8;
                    let i = pixel & 0xff;

                    rgba_buffer[y1 * width + x1] = convert((i << 0) | (i << 8) | (i << 16) | (a << 24));
                }
            }
        }
    }
}

// GXImageConverter.cs:1103 (fromCI4)
pub fn decode_ci4_image(data: &[u8], palette: &[u32], width: usize, height: usize, rgba_buffer: &mut [u32]) {
    let mut i = 0;

    for y in (0..height).step_by(8) {
        for x in (0..width).step_by(8) {
            for y1 in y..(y + 8) {
                for x1 in (x..(x + 8)).step_by(2) {
                    let pixel = data[i] as usize;
                    i += 1;

                    if y1 >= height || x1+1 >= width {
                        continue
                    }

                    rgba_buffer[y1 * width + x1] = palette[pixel >> 4];
                    rgba_buffer[y1 * width + x1 + 1] = palette[pixel & 0x0F];
                }
            }
        }
    }
}

// GXImageConverter.cs:1103 (fromCI8)
pub fn decode_ci8_image(data: &[u8], palette: &[u32], width: usize, height: usize, rgba_buffer: &mut [u32]) {
    let mut i = 0;

    for y in (0..height).step_by(4) {
        for x in (0..width).step_by(8) {
            for y1 in y..(y + 4) {
                for x1 in x..(x + 8) {
                    let pixel = data[i];
                    i += 1;

                    if y1 >= height || x1 >= width {
                        continue
                    }

                    rgba_buffer[y1 * width + x1] = palette[pixel as usize];
                }
            }
        }
    }
}

// GXImageConverter.cs:1245 (fromCMP)
//
// only decodes the first mipmap.
// RGBA8 format
pub fn decode_compressed_image(data: &[u8], width: usize, height: usize, rgba_buffer: &mut [u32]) {
    let mut c = [0u32; 4];

    fn get_r(n: u32) -> u32 { (n & 0x00FF0000) >> 16 }
    fn get_g(n: u32) -> u32 { (n & 0x0000FF00) >> 8 }
    fn get_b(n: u32) -> u32 { n & 0x000000FF }

    fn make_color_565(b1: u32, b2: u32) -> u32 {
        let bt = (b2 << 8) | b1;

        let a = 0xFF;
        let mut r = (bt >> 11) & 0x1F;
        let mut g = (bt >> 5) & 0x3F;
        let mut b = (bt) & 0x1F;

        r = (r << 3) | (r >> 2);
        g = (g << 2) | (g >> 4);
        b = (b << 3) | (b >> 2);

        (a << 24) | (r << 16) | (g << 8) | b
    }
    
    let mut i = 0;

    for y in 0..height {
        for x in 0..width {
            // int ww = Shared.AddPadding(width, 8);
            // bug? they might have meant x instead of width but idk
            let ww = if width % 8 != 0 {
                width + (8 - (width % 8))
            } else {
                width
            };

            let x0 = x & 0x03;
            let x1 = (x >> 2) & 0x01;
            let x2 = x >> 3;

            let y0 = y & 0x03;
            let y1 = (y >> 2) & 0x01;
            let y2 = y >> 3;


            let off = (8 * x1) + (16 * y1) + (32 * x2) + (4 * ww * y2);

            c[0] = make_color_565(data[off + 1] as u32 & 0xFF, data[off + 0] as u32 & 0xFF);
            c[1] = make_color_565(data[off + 3] as u32 & 0xFF, data[off + 2] as u32 & 0xFF);


            let mode = ((data[off] as u32 & 0xFF) << 8) | (data[off + 1] as u32 & 0xFF) 
                > ((data[off + 2] as u32 & 0xFF) << 8) | (data[off + 3] as u32 & 0xFF);

            if mode {
                let mut r = (2 * get_r(c[0]) + get_r(c[1])) / 3;
                let mut g = (2 * get_g(c[0]) + get_g(c[1])) / 3;
                let mut b = (2 * get_b(c[0]) + get_b(c[1])) / 3;
                
                c[2] = (0xFF << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32;

                r = (2 * get_r(c[1]) + get_r(c[0])) / 3;
                g = (2 * get_g(c[1]) + get_g(c[0])) / 3;
                b = (2 * get_b(c[1]) + get_b(c[0])) / 3;

                c[3] = (0xFF << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32;

            } else {
                let r = (get_r(c[0]) + get_r(c[1])) / 2;
                let g = (get_g(c[0]) + get_g(c[1])) / 2;
                let b = (get_b(c[0]) + get_b(c[1])) / 2;

                c[2] = (0xFF << 24) | ((r as u32) << 16) | ((g as u32) << 8) | b as u32;
                c[3] = 0;
            }

            let pixel = u32::from_be_bytes(data[off+4..off+8].try_into().unwrap());

            let ix = x0 + (4 * y0);
            let raw = c[((pixel >> (30 - (2 * ix))) & 0x03) as usize];
            
            let alpha = if ((pixel >> (30 - (2 * ix))) & 0x03) == 3 && !mode {
                0x00
            } else {
                0xFF
            };

            rgba_buffer[i] = convert((raw & 0x00FFFFFF) | (alpha << 24));

            i += 1;
        }
    }
}


fn convert(n: u32) -> u32 {
      (n & 0xff00ff00)
    | ((n & 0x00ff0000) >> 16)
    | ((n & 0x000000ff) << 16)
}
