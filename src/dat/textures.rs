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
}

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

pub fn try_decode_texture<'a>(
    cache: &mut HashMap<*const u8, u16>,
    textures: &mut Vec<Texture>,
    dobj: DOBJ<'a>
) -> Option<u16> {
    let mobj = dobj.get_mobj()?;
    let tobj = mobj.get_tobj()?;
    if tobj.get_sibling().is_some() { todo!(); }
    let data_ptr = tobj.image_buffer()?.as_ptr();

    use std::collections::hash_map::Entry;
    match cache.entry(data_ptr) {
        Entry::Occupied(entry) => Some(*entry.get()),
        Entry::Vacant(entry) => {
            let texture = tobj.texture().unwrap();
            let texture_idx = textures.len() as _;
            textures.push(texture);
            entry.insert(texture_idx);
            Some(texture_idx)
        }
    }
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

    // HSDScene.cs:194 (RefreshTextures)
    // HSD_TOBJ.cs:226 (GetDecodedImageData)
    // GXImageConverter.cs:77 (DecodeTPL)
    pub fn texture(&self) -> Option<Texture> {
        // TODO get other texture properties?
        let hsd_image = self.hsd_struct.try_get_reference(0x4C)?;
                                     
        let data_buffer = hsd_image.get_buffer(0x00);

        let width = hsd_image.get_i16(0x04) as usize;
        let height = hsd_image.get_i16(0x06) as usize;
        let format = InternalTextureFormat::new(hsd_image.get_i32(0x08) as u32).unwrap();
        println!("{:?}", format);

        let scale_x = self.hsd_struct.get_i8(0x3C) as f32;
        let scale_y = self.hsd_struct.get_i8(0x3D) as f32;

        let rgba_data = match format {
            InternalTextureFormat::CMP => decode_compressed_image(data_buffer, width, height),
            InternalTextureFormat::I4 => decode_i4_image(data_buffer, width, height),
            InternalTextureFormat::I8 => decode_i8_image(data_buffer, width, height),
            InternalTextureFormat::IA4 => decode_ia4_image(data_buffer, width, height),
            InternalTextureFormat::IA8 => decode_ia8_image(data_buffer, width, height),
            InternalTextureFormat::RGBA8 => decode_rgba8_image(data_buffer, width, height),
            InternalTextureFormat::RGB565 => decode_rgb565_image(data_buffer, width, height),
            InternalTextureFormat::CI8 => {
                let tlut_data = TLUT::new(self.hsd_struct.get_reference(0x50));
                let palette = tlut_data.palette();
                decode_ci8_image(data_buffer, &palette, width, height)
            }
            t => panic!("texture format {:?} unimplemented", t),
        };

        Some(Texture {
            width,
            height,
            rgba_data,
            scale_x,
            scale_y,
        })
    }
}

impl<'a> MOBJ<'a> {
    pub fn new(hsd_struct: HSDStruct<'a>) -> Self {
        Self {
            hsd_struct
        }
    }

    // might never fail, but return option to be sure
    pub fn get_tobj(&self) -> Option<TOBJ<'a>> {
        self.hsd_struct.try_get_reference(0x08)
            .map(TOBJ::new)
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

            palette.push((r << 0) | (g << 8) | (b << 16) | (a << 24));
        }

        palette.into_boxed_slice()
    }

    pub fn data(&self) -> &'a [u8] {
        self.hsd_struct.get_buffer(0x00)
    }

    pub fn format(&self) -> TLUTFormat {
        match self.hsd_struct.get_u32(0x04) {
            0 => TLUTFormat::IA8,
            1 => TLUTFormat::RGB565,
            2 => TLUTFormat::RGB5A3,
            _ => panic!("invalid TLUT format")
        }
    }

    pub fn colour_count(&self) -> u16 {
        self.hsd_struct.get_u16(0x0c)
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
}

// GXImageConverter.cs:396 (fromRGBA8)
fn decode_rgba8_image(data: &[u8], width: usize, height: usize) -> Box<[u32]> {
    let mut rgba_buffer = vec![0u32; width * height].into_boxed_slice();
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
                        rgba_buffer[x1 + (y1 * width)] |= (a << s1) | (b << s2);
                    }
                }
            }
        }
    }

    rgba_buffer
}

// GXImageConverter.cs:622 (fromRGB565)
fn decode_rgb565_image(data: &[u8], width: usize, height: usize) -> Box<[u32]> {
    let mut rgba_buffer = vec![0u32; width * height].into_boxed_slice();
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
                    rgba_buffer[y1 * width + x1] = (r << 0) | (g << 8) | (b << 16) | (255 << 24);
                }
            }
        }
    }

    rgba_buffer
}

// GXImageConverter.cs:706 (fromI4)
fn decode_i4_image(data: &[u8], width: usize, height: usize) -> Box<[u32]> {
    let mut rgba_buffer = vec![0u32; width * height].into_boxed_slice();
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
                    rgba_buffer[idx] = i | (i << 8) | (i << 16) | (i << 24);

                    let i = (pixel & 0x0F) * 255 / 15;
                    if idx + 1 < rgba_buffer.len() {
                        rgba_buffer[idx + 1] = i | (i << 8) | (i << 16) | (i << 24);
                    } 
                }
            }
        }
    }

    rgba_buffer
}

// GXImageConverter.cs:790 (fromI8)
fn decode_i8_image(data: &[u8], width: usize, height: usize) -> Box<[u32]> {
    let mut rgba_buffer = vec![0u32; width * height].into_boxed_slice();
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

                    rgba_buffer[y1 * width + x1] = (pixel << 0) | (pixel << 8) | (pixel << 16) | (pixel << 24);
                }
            }
        }
    }

    rgba_buffer
}

// GXImageConverter.cs:868 (fromIA4)
fn decode_ia4_image(data: &[u8], width: usize, height: usize) -> Box<[u32]> {
    let mut rgba_buffer = vec![0u32; width * height].into_boxed_slice();
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

                    //rgba_buffer[y1 * width + x1] = (i << 0) | (i << 8) | (i << 16) | (a << 24);
                    rgba_buffer[y1 * width + x1] = (i << 0) | (i << 8) | (i << 16) | (a << 24);
                }
            }
        }
    }

    rgba_buffer
}

// GXImageConverter.cs:940 (fromIA8)
fn decode_ia8_image(data: &[u8], width: usize, height: usize) -> Box<[u32]> {
    let mut rgba_buffer = vec![0u32; width * height].into_boxed_slice();
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

                    rgba_buffer[y1 * width + x1] = (i << 0) | (i << 8) | (i << 16) | (a << 24);
                }
            }
        }
    }

    rgba_buffer
}

// GXImageConverter.cs:1103 (fromCI8)
fn decode_ci8_image(data: &[u8], palette: &[u32], width: usize, height: usize) -> Box<[u32]> {
    let mut rgba_buffer = vec![0u32; width * height].into_boxed_slice();
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

    rgba_buffer
}

// GXImageConverter.cs:1245 (fromCMP)
//
// only decodes the first mipmap.
// RGBA8 format
fn decode_compressed_image(data: &[u8], width: usize, height: usize) -> Box<[u32]> {
    let mut rgba_buffer = vec![0u32; width * height].into_boxed_slice();

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

            rgba_buffer[i] = (raw & 0x00FFFFFF) | (alpha << 24);
            i += 1;
        }
    }

    rgba_buffer
}

