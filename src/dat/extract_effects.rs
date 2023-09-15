use crate::dat::{HSDStruct, Texture, JOBJ, extract_model_from_jobj, Model};

// Melee/Ef/SBM_EffectTable.cs (SBM_EffectTable)
#[derive(Clone, Debug)]
pub struct EffectTable<'a> {
    pub hsd_struct: HSDStruct<'a>
}

// Common/HSD_TEXGraphic.cs (HSD_TEXGraphicBank)
#[derive(Clone, Debug)]
pub struct TextureBank<'a> {
    pub hsd_struct: HSDStruct<'a>
}

impl<'a> EffectTable<'a> {
    pub fn new(hsd_struct: HSDStruct<'a>) -> Self {
        Self {
            hsd_struct
        }
    }

    pub fn texture_bank(&self) -> TextureBank<'a> {
        TextureBank::new(self.hsd_struct.get_reference(0x04))
    }

    pub fn models(&self) -> Box<[Model]> {
        let count = (self.hsd_struct.len() - 0x08) / 0x14;
        let mut models = Vec::with_capacity(count);

        for i in 0..count {
            // Melee/Ef/SBM_EffectTable.cs (SBM_EffectModel)
            let model_struct = self.hsd_struct.get_embedded_struct(0x08 + 0x14 * i, 0x14);
            let jobj = JOBJ::new(model_struct.get_reference(0x04));
            models.push(extract_model_from_jobj(jobj, None).unwrap());
        }

        models.into_boxed_slice()
    }
}

impl<'a> TextureBank<'a> {
    pub fn new(hsd_struct: HSDStruct<'a>) -> Self {
        Self {
            hsd_struct
        }
    }

    pub fn length(&self) -> u32 {
        self.hsd_struct.get_u32(0x00)
    }

    pub fn textures(&self) -> Box<[Texture]> {
        let len = self.length() as usize;

        //let mut textures = Vec::with_capacity(len);

        for i in 0..len {
            let offset = self.hsd_struct.get_u32(0x04 * i) as usize; 
            let texture_len = if i+1 < len {
                self.hsd_struct.get_u32(0x04 * (i+1)) as usize - offset
            } else {
                self.hsd_struct.len() - offset
            };

            let bank_texture = self.hsd_struct.get_embedded_struct(offset, texture_len);
            let image_count = bank_texture.get_u32(0x00);
            println!("{}", image_count);
        }
        
        todo!()
    }
}
