use crate::dat::{InternalTextureFormat, HSDStruct, Image, TLUTFormat, Animation,
    JOBJ, extract_model_from_jobj, decode_palette, Model, decode_data,
    parse_joint_anim};

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

    pub fn texture_bank(&self) -> Option<TextureBank<'a>> {
        self.hsd_struct.try_get_reference(0x04).map(TextureBank::new)
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

    pub fn models_and_animations(&self) -> Box<[(Model, Option<Animation>)]> {
        let count = (self.hsd_struct.len() - 0x08) / 0x14;
        let mut models = Vec::with_capacity(count);

        for i in 0..count {
            // Melee/Ef/SBM_EffectTable.cs (SBM_EffectModel)
            let model_struct = self.hsd_struct.get_embedded_struct(0x08 + 0x14 * i, 0x14);
            let jobj = JOBJ::new(model_struct.get_reference(0x04));
            let model = extract_model_from_jobj(jobj, None).unwrap();

            // joint anim
            let anim = model_struct.try_get_reference(0x08)
                .and_then(parse_joint_anim);
            // TODO other animations???/
            models.push((model, anim));
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

    pub fn joint_anim(&self, model_idx: usize) -> Option<Animation> {
        let count = (self.hsd_struct.len() - 0x08) / 0x14;
        if model_idx >= count { return None }

        let model_struct = self.hsd_struct.get_embedded_struct(0x08 + 0x14 * model_idx, 0x14);
        parse_joint_anim(model_struct.try_get_reference(0x08)?)
    }

    pub fn mat_anim(&self, model_idx: usize) -> Option<Box<[MaterialAnim]>> {
        let count = (self.hsd_struct.len() - 0x08) / 0x14;
        if model_idx >= count { return None }

        let model_struct = self.hsd_struct.get_embedded_struct(0x08 + 0x14 * model_idx, 0x14);
        Some(parse_material_anims(model_struct.try_get_reference(0x0C)?))
    }

    pub fn shape_anim(&self, model_idx: usize) -> Option<Box<[ShapeAnim]>> {
        let count = (self.hsd_struct.len() - 0x08) / 0x14;
        if model_idx >= count { return None }

        let model_struct = self.hsd_struct.get_embedded_struct(0x08 + 0x14 * model_idx, 0x14);
        Some(parse_shape_anims(model_struct.try_get_reference(0x10)?))
    }

    pub fn hidden_mat_animation_textures(&self) -> Box<[Image]> {
        let mut images = Vec::new();

        let count = (self.hsd_struct.len() - 0x08) / 0x14;
        for i in 0..count {
            // Melee/Ef/SBM_EffectTable.cs (SBM_EffectModel)
            let model_struct = self.hsd_struct.get_embedded_struct(0x08 + 0x14 * i, 0x14);

            // HSD_MatAnimJoint
            let mat_anim_joint = match model_struct.try_get_reference(0x0C) {
                Some(a) => a,
                None => continue,
            };

            extract_mat_anim_joint_textures(
                &mut std::collections::HashSet::new(),
                &mut images, 
                mat_anim_joint
            );
        }

        images.into_boxed_slice()
    }

    pub fn hidden_animation_models(&self) -> Box<[Model]> {
        let mut models = Vec::new();

        let count = (self.hsd_struct.len() - 0x08) / 0x14;
        for i in 0..count {
            // Melee/Ef/SBM_EffectTable.cs (SBM_EffectModel)
            let model_struct = self.hsd_struct.get_embedded_struct(0x08 + 0x14 * i, 0x14);

            // HSD_AnimJoint
            match model_struct.try_get_reference(0x08) {
                Some(anim_joint) => extract_anim_joint_models(&mut models, anim_joint),
                None => (),
            };

            // HSD_MatAnimJoint
            match model_struct.try_get_reference(0x0C) {
                Some(mat_anim_joint) => extract_mat_anim_joint_models(&mut models, mat_anim_joint),
                None => (),
            };

            // HSD_ShapeAnimJoint
            match model_struct.try_get_reference(0x10) {
                Some(shape_anim_joint) => extract_mat_anim_joint_models(&mut models, shape_anim_joint),
                None => (),
            };
        }

        models.into_boxed_slice()
    }
}

// TODO
pub struct ShapeAnim {}
fn parse_shape_anims(_shape_anim_joint: HSDStruct<'_>) -> Box<[ShapeAnim]> {
    todo!()
    //let mut shape_anims = Vec::new();

    //for shape_anim_joint in shape_anim_joint.iter_joint_tree(0x00, 0x04) {
    //    // HSD_ShapeAnim
    //    let shape_anim = shape_anim_joint.get_reference(0x08);

    //    for shape_anim in shape_anim.iter_joint_list(0x00) {
    //        let aobj_desc = shape_anim.get_reference(0x04);  

    //        for aobj_desc in aobj_desc.iter_joint_list(0x00) {
    //            todo!();
    //        }
    //    }
    //}

    //shape_anims.into_boxed_slice()
}

// TODO
pub struct MaterialAnim {}
fn parse_material_anims(_mat_anim_joint: HSDStruct<'_>) -> Box<[MaterialAnim]> {
    todo!()
    //let mut material_anims = Vec::new();

    //for mat_anim_joint in mat_anim_joint.iter_joint_tree(0x00, 0x04) {
    //    // HSD_MatAnim
    //    let mat_anim = mat_anim_joint.get_reference(0x08);

    //    for mat_anim in mat_anim.iter_joint_list(0x00) {
    //        println!("mat anim");

    //        let aobj = mat_anim.try_get_reference(0x04);
    //        dbg!(aobj);
    //        let tex_anim = mat_anim.try_get_reference(0x08);  
    //        dbg!(tex_anim);
    //    }
    //}

    //material_anims.into_boxed_slice()
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

pub fn extract_mat_anim_joint_textures(
    cache: &mut std::collections::HashSet<*const u8>, 
    textures: &mut Vec<crate::dat::Image>, 
    mat_anim_joint: HSDStruct
) {
    for mat_anim_joint in mat_anim_joint.iter_joint_tree(0x00, 0x04) {
        if let Some(mat_anim) = mat_anim_joint.try_get_reference(0x08) {
            for mat_anim in mat_anim.iter_joint_list(0x00) {
                let tex_anim = mat_anim.get_reference(0x08);
                for tex_anim in tex_anim.iter_joint_list(0x00) {
                    if let Some(tex_buffers) = tex_anim.try_get_reference(0x0c) {
                        if let Some(tlut_buffers) = tex_anim.try_get_reference(0x10) {
                            for offset in (0..tex_buffers.len()).step_by(4) {
                                let image = tex_buffers.get_reference(offset);
                                if cache.insert(image.get_buffer(0x00).as_ptr()) {
                                    let tlut = tlut_buffers.try_get_reference(offset)
                                        .map(crate::dat::TLUT::new);
                                    textures.push(crate::dat::decode_image(image, tlut));
                                }
                            }
                        } else {
                            for offset in (0..tex_buffers.len()).step_by(4) {
                                let image = tex_buffers.get_reference(offset);
                                if cache.insert(image.get_buffer(0x00).as_ptr()) {
                                    textures.push(crate::dat::decode_image(image, None));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<'a> TextureBank<'a> {
    pub fn new(hsd_struct: HSDStruct<'a>) -> Self {
        Self {
            hsd_struct
        }
    }

    pub fn texture_count(&self) -> u32 {
        self.hsd_struct.get_u32(0x00)
    }

    pub fn textures(&self) -> Box<[Image]> {
        let texture_count = self.texture_count() as usize;

        let mut textures = Vec::with_capacity(texture_count);

        for i in 1..=texture_count {
            let start = self.hsd_struct.get_u32(0x04 * i) as usize; 
            let end = if i < texture_count {
                self.hsd_struct.get_u32(0x04 * (i+1)) as usize
            } else {
                self.hsd_struct.len()
            };

            let texture_len = end - start;

            // HSD_TexGraphic
            let bank_texture = self.hsd_struct.get_embedded_struct(start, texture_len);
            let image_count = bank_texture.get_u32(0x00) as usize;

            let width = bank_texture.get_u32(0x0C) as usize;
            let height = bank_texture.get_u32(0x10) as usize;

            let f = bank_texture.get_u32(0x04);
            let tex_format = InternalTextureFormat::new(f).unwrap();
            let tlut_format = TLUTFormat::new(bank_texture.get_u32(0x08)).unwrap();

            for j in 0..image_count {
                let mut rgba_data = vec![0u32; width * height].into_boxed_slice();

                let image_offset = bank_texture.get_u32(j * 4 + 0x18) as usize - start;
                let size = tex_format.data_size(width, height);
                let image_data = bank_texture.get_bytes(image_offset, size);

                if tex_format.is_paletted() {
                    let pal_offset = bank_texture.get_u32((j + image_count) * 4 + 0x18) as usize - start;
                    let pal_data = bank_texture.get_bytes(pal_offset, 0x200); // hardcoded for some reason
                    let palette = decode_palette(0x100, tlut_format, pal_data);
                    
                    decode_data(tex_format, width, height, image_data, Some(&palette), &mut rgba_data);
                } else {
                    decode_data(tex_format, width, height, image_data, None, &mut rgba_data);
                }

                textures.push(Image { width, height, rgba_data });
            }
        }

        textures.into_boxed_slice()
    }
}
