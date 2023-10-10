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

    pub fn model(&self, model_idx: usize) -> Option<Model> {
        let count = (self.hsd_struct.len() - 0x08) / 0x14;
        if model_idx >= count { return None }

        let model_struct = self.hsd_struct.get_embedded_struct(0x08 + 0x14 * model_idx, 0x14);
        let jobj = JOBJ::new(model_struct.get_reference(0x04));
        extract_model_from_jobj(jobj, None).ok()
    }

    pub fn hidden_animation_models(&self) -> Box<[Model]> {
        let mut models = Vec::new();

        let count = (self.hsd_struct.len() - 0x08) / 0x14;
        for i in 0..count {
            // Melee/Ef/SBM_EffectTable.cs (SBM_EffectModel)
            let model_struct = self.hsd_struct.get_embedded_struct(0x08 + 0x14 * i, 0x14);

            // HSD_AnimJoint
            let anim_joint = match model_struct.try_get_reference(0x08) {
                Some(a) => a,
                None => continue,
            };

            extract_anim_joint_models(&mut models, anim_joint);

            // HSD_MatAnimJoint
            let mat_anim_joint = match model_struct.try_get_reference(0x0C) {
                Some(a) => a,
                None => continue,
            };

            extract_mat_anim_joint_models(&mut models, mat_anim_joint);

            // HSD_ShapeAnimJoint
            let shape_anim_joint = match model_struct.try_get_reference(0x10) {
                Some(a) => a,
                None => continue,
            };

            extract_shape_anim_joint_models(&mut models, shape_anim_joint);
        }

        models.into_boxed_slice()
    }
}

pub fn extract_anim_joint_models(models: &mut Vec<Model>, anim_joint: HSDStruct) {
    if let Some(aobj) = anim_joint.try_get_reference(0x08) {
        if let Some(object_reference) = aobj.try_get_reference(0x0C) {
            extract_model_from_jobj(JOBJ::new(object_reference), None).unwrap();
        }
    }

    if let Some(child) = anim_joint.try_get_reference(0x00) {
        extract_anim_joint_models(models, child);
    }

    if let Some(sibling) = anim_joint.try_get_reference(0x04) {
        extract_anim_joint_models(models, sibling);
    }
}

pub fn extract_shape_anim_joint_models(models: &mut Vec<Model>, shape_anim_joint: HSDStruct) {
    if let Some(mut shape_anim) = shape_anim_joint.try_get_reference(0x08) {
        loop {
            if let Some(mut aobj_desc) = shape_anim.try_get_reference(0x04) {
                loop {
                    if let Some(aobj) = aobj_desc.try_get_reference(0x04) {
                        if let Some(object_reference) = aobj.try_get_reference(0x0C) {
                            extract_model_from_jobj(JOBJ::new(object_reference), None).unwrap();
                        }
                    }

                    match aobj_desc.try_get_reference(0x00) {
                        Some(new_aobj_desc) => aobj_desc = new_aobj_desc,
                        None => break,
                    }
                }
            }

            match shape_anim.try_get_reference(0x00) {
                Some(new_shape_anim) => shape_anim = new_shape_anim,
                None => break,
            }
        }
    }

    if let Some(child) = shape_anim_joint.try_get_reference(0x00) {
        extract_shape_anim_joint_models(models, child);
    }

    if let Some(sibling) = shape_anim_joint.try_get_reference(0x04) {
        extract_shape_anim_joint_models(models, sibling);
    }
}

pub fn extract_mat_anim_joint_models(models: &mut Vec<Model>, mat_anim_joint: HSDStruct) {
    if let Some(mut mat_anim) = mat_anim_joint.try_get_reference(0x08) {
        loop {
            if let Some(aobj) = mat_anim.try_get_reference(0x04) {
                if let Some(object_reference) = aobj.try_get_reference(0x0C) {
                    extract_model_from_jobj(JOBJ::new(object_reference), None).unwrap();
                }
            }

            match mat_anim.try_get_reference(0x00) {
                Some(new_mat_anim) => mat_anim = new_mat_anim,
                None => break,
            }
        }
    }

    if let Some(child) = mat_anim_joint.try_get_reference(0x00) {
        extract_mat_anim_joint_models(models, child);
    }

    if let Some(sibling) = mat_anim_joint.try_get_reference(0x04) {
        extract_mat_anim_joint_models(models, sibling);
    }
}

pub fn extract_mat_anim_joint_textures(textures: &mut Vec<crate::dat::Image>, mat_anim_joint: HSDStruct) {
    if let Some(mut mat_anim) = mat_anim_joint.try_get_reference(0x08) {
        loop {
            if let Some(mut tex_anim) = mat_anim.try_get_reference(0x08) {
                loop {
                    if let Some(tex_buffers) = tex_anim.try_get_reference(0x0c) {
                        if let Some(tlut_buffers) = tex_anim.try_get_reference(0x10) {
                            for offset in (0..tex_buffers.len()).step_by(4) {
                                let tlut = tlut_buffers.try_get_reference(offset)
                                    .map(crate::dat::TLUT::new);
                                let image = tex_buffers.get_reference(offset);
                                textures.push(crate::dat::decode_image(image, tlut));
                            }
                        } else {
                            for offset in (0..tex_buffers.len()).step_by(4) {
                                let image = tex_buffers.get_reference(offset);
                                textures.push(crate::dat::decode_image(image, None));
                            }
                        }
                    }

                    match tex_anim.try_get_reference(0x00) {
                        Some(new_tex_anim) => tex_anim = new_tex_anim,
                        None => break,
                    }
                }
            }

            match mat_anim.try_get_reference(0x00) {
                Some(new_mat_anim) => mat_anim = new_mat_anim,
                None => break,
            }
        }
    }

    if let Some(child) = mat_anim_joint.try_get_reference(0x00) {
        extract_mat_anim_joint_textures(textures, child);
    }

    if let Some(sibling) = mat_anim_joint.try_get_reference(0x04) {
        extract_mat_anim_joint_textures(textures, sibling);
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
