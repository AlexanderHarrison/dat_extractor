#![allow(clippy::upper_case_acronyms)]

use crate::dat::{HSDStruct, JOBJ};

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct MOBJ<'a> {
    pub hsd_struct: HSDStruct<'a>,
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct TOBJ<'a> {
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

pub fn extract_textures(jobjs: &[JOBJ]) -> Box<[Texture]> {
    // HSDScene.cs:98 (GetTOBJS)

    // holy mother of iterators!
    // hopefully the optimizer is ok with this...
    jobjs
        .iter()
        .filter_map(|j| j.get_dobj())
        .flat_map(|d| d.siblings())
        .filter_map(|d| d.get_mobj())
        .filter_map(|m| m.get_tobj())
        .flat_map(|t| t.siblings())
        .filter_map(|t| t.texture())
        .collect::<Vec<Texture>>()
        .into_boxed_slice()
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

    // HSDScene.cs:194 (RefreshTextures)
    // HSD_TOBJ.cs:226 (GetDecodedImageData)
    // GXImageConverter.cs:77 (DecodeTPL)
    pub fn texture(&self) -> Option<Texture> {
        // TODO get other texture properties?
        
        let tlut_data = self.hsd_struct.try_get_reference(0x50);
        assert_eq!(tlut_data, None); // don't want to deal with this right now
        let hsd_image = self.hsd_struct.try_get_reference(0x4C)?;
                                     
        let data_buffer = hsd_image.get_buffer(0x00);

        let width = hsd_image.get_i16(0x04) as usize;
        let height = hsd_image.get_i16(0x06) as usize;
        let format = InternalTextureFormat::new(hsd_image.get_i32(0x08) as u32).unwrap();

        let scale_x = self.hsd_struct.get_i8(0x3C) as f32; // TODO NOT SURE ABOUT THIS CONVERSION ---------------------------------
        let scale_y = self.hsd_struct.get_i8(0x3D) as f32;

        let rgba_data = match format {
            InternalTextureFormat::CMP => decode_compressed_image(data_buffer, width, height),
            _ => todo!()
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
            // matches

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

