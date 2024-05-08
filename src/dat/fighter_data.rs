use crate::dat::{
    HSDStruct, DatFile, Model, JOBJ, extract_model_from_jobj, parse_joint_anim,
    HSDRawFile, Animation, extract_anim_from_action, extract_character_model,
};
use crate::parse_string;

#[derive(Debug, Clone)]
pub struct FighterData {
    pub character_name: Box<str>,
    pub model: Model,

    pub attributes: FighterAttributes,
    pub articles: Box<[Article]>,
    pub action_table: Box<[FighterAction]>,
}

#[derive(Debug, Clone)]
pub struct FighterAction {
    pub name: Option<Box<str>>,
    pub animation: Option<Animation>,
    pub subactions: Option<Box<[u32]>>,
    pub flags: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct FighterAttributes {
    pub shield_bone: u16,
    pub item_hold_bone: u16,
    pub shield_size: f32,
}

// SBM_FighterData.cs
#[derive(Debug, Clone)]
pub struct FighterDataRoot<'a> {
    pub hsd_struct: HSDStruct<'a>
}

// SBM_ArticlePointer.cs (SBM_Article)
#[derive(Debug, Clone)]
pub struct Article {
    pub model: Option<Model>,
    pub bone: Option<u32>,
    pub scale: f32,
    pub animations: Option<Box<[Option<Animation>]>>, // TODO unused
}

impl<'a> FighterDataRoot<'a> {
    pub fn new(hsd_struct: HSDStruct<'a>) -> Self {
        Self { hsd_struct }
    }

    pub fn attributes(&self) -> FighterAttributes {
        // SBM_CommonFighterAttributes.cs
        let common_attributes = self.hsd_struct.get_reference(0x00);

        // SBM_PlayerModelLookupTables.cs
        let player_model_lookup_table = self.hsd_struct.get_reference(0x08);

        FighterAttributes {
            item_hold_bone: player_model_lookup_table.get_u8(0x10) as u16,
            shield_bone: player_model_lookup_table.get_u8(0x11) as u16,
            shield_size: common_attributes.get_f32(0x090),
        }
    }

    pub fn articles(&self) -> Option<Box<[Article]>> {
        let article_ptrs = match self.hsd_struct.try_get_reference(0x48) {
            Some(ptrs) => ptrs,
            None => return Some(Box::new([])),
        };
        let count = article_ptrs.len() / 4;
        let mut articles = Vec::with_capacity(count);

        let mut unused_articles = 0;

        for i in 0..count {
            // SBM_ArticlePointer.cs (SBM_Article)
            if let Some(article) = article_ptrs.try_get_reference(4 * i) {
                let mut model = None;

                let scale = match article.try_get_reference(0x00) {
                    // occasionally returns a struct half the size?????????
                    Some(item_common_attributes) if item_common_attributes.len() != 132 => 1.0,

                    // SBM_ArticlePointer.cs (SBM_ItemCommonAttr)
                    Some(item_common_attributes) => item_common_attributes.get_f32(0x60),

                    None => 1.0,
                };

                let mut bone = None;

                // SBM_ArticlePointer.cs (SBM_ItemModel)
                if let Some(item_model) = article.try_get_reference(0x10) {
                    if let Some(root_jobj) = item_model.try_get_reference(0x00) {
                        let model_root_jobj = JOBJ::new(root_jobj);
                        model = Some(extract_model_from_jobj(model_root_jobj, None).ok()?);
                    }

                    bone = Some(item_model.get_u32(0x08));

                    // 0x0C usually zero
                }

                let mut animations = None;

                if let Some(item_states) = article.try_get_reference(0x0C) {
                    let count = item_states.len() / 0x10;
                    let mut anim_vec = Vec::with_capacity(count);

                    let mut n = 0;
                    for i in 0..count {
                        // Melee/Pl/SBM_ArticlePointer.cs (SBM_ItemState)
                        let item_state = item_states.get_embedded_struct(i * 0x10, 0x10);

                        if let Some(_) = item_state.try_get_reference(0x04) {
                            println!("unused mat anim");
                        }

                        if let Some(joint_anim_joint) = item_state.try_get_reference(0x00) {
                            let anim = parse_joint_anim(joint_anim_joint).unwrap();
                            anim_vec.push(Some(anim));
                        } else {
                            anim_vec.push(None);
                            n += 1;
                        }
                    }

                    if n != 0 {
                        println!("{} unused article states out of {}", n, count);
                    }

                    animations = Some(anim_vec.into_boxed_slice());
                }

                //let mut images = Vec::new();

                //// SBM_ArticlePointer.cs (SBM_ItemState)
                //if let Some(item_state_array) = article.try_get_reference(0x0C) {
                //    for offset in (0..item_state_array.len()).step_by(0x10) {
                //        //if let Some(anim_joint) = item_state_array.try_get_reference(offset + 0x00) {
                //        //    crate::dat::extract_anim_joint_models(&mut models, anim_joint);
                //        //    println!("good try 1");
                //        //}

                //        if let Some(mat_anim_joint) = item_state_array.try_get_reference(offset + 0x04) {
                //            crate::dat::extract_mat_anim_joint_textures(&mut images, mat_anim_joint);
                //        }
                //    }
                //}

                articles.push(Article { model, scale, animations, bone });
            } else {
                unused_articles += 1
            }
        }

        if unused_articles != 0 {
            println!("{} unused articles", unused_articles);
        }

        Some(articles.into_boxed_slice())
    }
}

/// None if not a fighter dat file.
/// Filename should be "PlFx.dat" or the like.
pub fn parse_fighter_data(fighter_dat: &DatFile, anim_dat: &DatFile, model_dat: &DatFile) -> Option<FighterData> {
    let fighter_hsdfile = HSDRawFile::new(fighter_dat);

    let fighter_root_node = &fighter_hsdfile.roots[0];
    let name = fighter_root_node.root_string;
    if !name.starts_with("ftData") || name.contains("Copy") {
        return None;
    }

    let fighter_data_root = FighterDataRoot::new(fighter_root_node.hsd_struct.clone());
    let attributes = fighter_data_root.attributes();
    let action_table = parse_actions(anim_dat, &fighter_hsdfile)?;
    let parsed_model_dat = HSDRawFile::new(model_dat);
    let model = extract_character_model(&fighter_hsdfile, &parsed_model_dat).ok()?;
    let articles = fighter_data_root.articles()?;

    Some(FighterData {
        character_name: name.strip_prefix("ftData").unwrap().to_string().into_boxed_str(),
        model,
        attributes,
        articles,
        action_table,
    })
}

pub struct HighPolyBoneIndicies {
    pub groups: Box<[(u16, u16)]>, // turned into model groups
    pub indicies: Box<[u8]>,
}

pub fn get_high_poly_bone_indicies<'a>(fighter_hsd: &HSDRawFile<'a>) -> HighPolyBoneIndicies {
    let fighter_root = &fighter_hsd.roots[0];

    // SBM_PlayerModelLookupTables
    let lookup_tables = fighter_root.hsd_struct.get_reference(0x08);

    let costume_table = lookup_tables.get_array(0x10, 0x04).next().unwrap();

    let mut indicies = Vec::with_capacity(64);
    let mut groups = Vec::with_capacity(8);
    for high_poly_table in costume_table.get_array(0x08, 0x00) {
        if let Some(jobj_table_iter) = high_poly_table.try_get_array(0x08, 0x04) {
            for jobj_table in jobj_table_iter {
                let count = jobj_table.get_i32(0x00) as usize;
                let new = &jobj_table.get_buffer(0x04)[..count];
                groups.push((indicies.len() as u16, new.len() as u16));
                indicies.extend_from_slice(new);
            }
        }
    }

    HighPolyBoneIndicies {
        groups: groups.into_boxed_slice(),
        indicies: indicies.into_boxed_slice(),
    }
}

pub fn parse_actions(anim_dat: &DatFile, fighter_hsd: &HSDRawFile) -> Option<Box<[FighterAction]>> {
    let mut actions = Vec::new();

    let fighter_root = &fighter_hsd.roots[0];
    let hsd_struct = &fighter_root.hsd_struct;

    let action_table_struct = hsd_struct.get_reference(0x0C);

    for i in 0..(action_table_struct.len() / 0x18) {
        let s = action_table_struct.get_embedded_struct(i * 0x18, 0x18);
        let action = parse_fighter_action(anim_dat, s);
        actions.push(action);
    }

    Some(actions.into_boxed_slice())
}

fn parse_fighter_action(anim_dat: &DatFile, hsd_struct: HSDStruct) -> FighterAction {
    let name = hsd_struct.try_get_buffer(0x00)
        .and_then(|s| Some(parse_string(s)?.to_string().into_boxed_str()));

    let animation = extract_anim_from_action(anim_dat, hsd_struct.clone());
    let subactions = hsd_struct
        .try_get_reference(0x0C)
        .map(|sub| {
            sub.data
                .chunks_exact(4)
                .map(|b| u32::from_be_bytes(b.try_into().unwrap()))
                .collect::<Vec<_>>()
                .into_boxed_slice()
        });
    let flags = hsd_struct.get_u32(0x10);

    FighterAction {
        name,
        animation,
        subactions,
        flags
    }
}

pub type SubactionCmd = u8;
pub mod subaction {
    use super::SubactionCmd;
    pub const END_OF_SCRIPT             : SubactionCmd = 0x00;
    pub const SYNCHRONOUS_TIMER         : SubactionCmd = 0x01;
    pub const ASYNCHRONOUS_TIMER        : SubactionCmd = 0x02;
    pub const SET_LOOP                  : SubactionCmd = 0x03;
    pub const EXECUTE_LOOP              : SubactionCmd = 0x04;
    pub const SUBROUTINE                : SubactionCmd = 0x05;
    pub const RETURN                    : SubactionCmd = 0x06;
    pub const GOTO                      : SubactionCmd = 0x07;
    pub const SET_LOOP_ANIMATION_TIMER  : SubactionCmd = 0x08;
    pub const UNKNOWN_0X09              : SubactionCmd = 0x09;
    pub const GRAPHIC_EFFECT            : SubactionCmd = 0x0A;
    pub const CREATE_HITBOX             : SubactionCmd = 0x0B;
    pub const ADJUST_HITBOX_DAMAGE      : SubactionCmd = 0x0C;
    pub const ADJUST_HITBOX_SIZE        : SubactionCmd = 0x0D;
    pub const SET_HITBOX_INTERACTION    : SubactionCmd = 0x0E;
    pub const REMOVE_HITBOX             : SubactionCmd = 0x0F;
    pub const CLEAR_HITBOXES            : SubactionCmd = 0x10;
    pub const SOUND_EFFECT              : SubactionCmd = 0x11;
    pub const RANDOM_SMASH_SFX          : SubactionCmd = 0x12;
    pub const AUTO_CANCEL               : SubactionCmd = 0x13;
    pub const REVERSE_DIRECTION         : SubactionCmd = 0x14;
    pub const UNKNOWN_0X15              : SubactionCmd = 0x15;
    pub const UNKNOWN_0X16              : SubactionCmd = 0x16;
    pub const ALLOW_INTERRUPT           : SubactionCmd = 0x17;
    pub const PROJECTILE_FLAG           : SubactionCmd = 0x18;
    pub const SET_JUMP_STATE            : SubactionCmd = 0x19;
    pub const SET_BODY_COLLISION_STATE  : SubactionCmd = 0x1A;
    pub const BODY_COLLISION_STATUS     : SubactionCmd = 0x1B;
    pub const SET_BONE_COLLISION_STATE  : SubactionCmd = 0x1C;
    pub const ENABLE_JAB_FOLLOW_UP      : SubactionCmd = 0x1D;
    pub const TOGGLE_JAB_FOLLOW_UP      : SubactionCmd = 0x1E;
    pub const CHANGE_MODEL_STATE        : SubactionCmd = 0x1F;
    pub const REVERT_MODELS             : SubactionCmd = 0x20;
    pub const REMOVE_MODELS             : SubactionCmd = 0x21;
    pub const THROW                     : SubactionCmd = 0x22;
    pub const HELD_ITEM_INVISIBILITY    : SubactionCmd = 0x23;
    pub const BODY_ARTICLE_INVISIBILITY : SubactionCmd = 0x24;
    pub const CHARACTER_INVISIBILITY    : SubactionCmd = 0x25;
    pub const PSEUDO_RANDOM_SOUND_EFFECT: SubactionCmd = 0x26;
    pub const UNKNOWN_0X27              : SubactionCmd = 0x27;
    pub const ANIMATE_TEXTURE           : SubactionCmd = 0x28;
    pub const ANIMATE_MODEL             : SubactionCmd = 0x29;
    pub const UNKNOWN_0X2A              : SubactionCmd = 0x2A;
    pub const RUMBLE                    : SubactionCmd = 0x2B;
    pub const UNKNOWN_0X2C              : SubactionCmd = 0x2C;
    pub const UNKNOWN_0X2D              : SubactionCmd = 0x2D;
    pub const BODY_AURA                 : SubactionCmd = 0x2E;
    pub const REMOVE_COLOR_OVERLAY      : SubactionCmd = 0x2F;
    pub const UNKNOWN_0X30              : SubactionCmd = 0x30;
    pub const SWORD_TRAIL               : SubactionCmd = 0x31;
    pub const ENABLE_RAGDOLL_PHYSICS    : SubactionCmd = 0x32;
    pub const SELF_DAMAGE               : SubactionCmd = 0x33;
    pub const CONTINUATION_CONTROL      : SubactionCmd = 0x34;
    pub const FOOTSNAP_BEHAVIOR         : SubactionCmd = 0x35;
    pub const FOOTSTEP_EFFECT           : SubactionCmd = 0x36;
    pub const LANDING_EFFECT            : SubactionCmd = 0x37;
    pub const START_SMASH_CHARGE        : SubactionCmd = 0x38;
    pub const UNKNOWN_0X39              : SubactionCmd = 0x39;
    pub const AESTHETIC_WIND_EFFECT     : SubactionCmd = 0x3A;
    pub const UNKNOWN_0X3B              : SubactionCmd = 0x3B;
}

#[derive(Debug, Clone)]
pub enum Subaction {
    EndOfScript,
    SynchronousTimer {
        frame: u32,
    },
    AsynchronousTimer {
        frame: u32,
    },
    SetLoop {
        loop_count: u32,
    },
    ExecuteLoop,
    Subroutine {
        pointer: u32,
    },
    Return,
    GoTo {
        pointer: u32,
    },
    SetLoopAnimationTimer,
    Unknown0x09 {
        unknown: u32,
    },
    GraphicEffect {
        bone_id: u8,
        use_common_bone_id: bool,
        destroy_on_state_change: bool,
        use_unknown_bone_id: bool,
        unknown: u16,
        graphic_id: u16,
        unknown_bone_id: u16,
        z_offset: i16,
        y_offset: i16,
        x_offset: i16,
        z_range: u16,
        y_range: u16,
        x_range: u16,
    },
    CreateHitbox {
        hitbox_id: u8,
        bone_attachment: u8,
        damage: u16,
        size: u16,
        z_offset: i16,
        y_offset: i16,
        x_offset: i16,
        knockback_angle: u16,
        knockback_growth: u16,
        weight_dependent_set_knockback: u16,
        hitbox_interaction: u8,
        base_knockback: u16,
        element: u8,
        unknown: bool,
        shield_damage: u8,
        sound_effect: u8,
        hit_grounded_opponents: bool,
        hit_airborne_opponents: bool,
    },
    AdjustHitboxDamage {
        hitbox_id: u8,
        damage: u32,
    },
    AdjustHitboxSize {
        hitbox_id: u8,
        newsize: u32,
    },
    SetHitboxInteraction {
        hitbox_id: u32,
        interact_type: bool,
        can_interact: bool,
    },
    RemoveHitbox,
    ClearHitboxes,
    SoundEffect {
        unknown_1: u32,
        unknown_2: u8,
        sound_effect_id: u32,
        offset: u32,
    },
    RandomSmashSFX {
        unknown: u32,
    },
    AutoCancel {
        flags: u8,
    },
    ReverseDirection,
    Unknown0x15 {
        unknown: u32,
    },
    Unknown0x16 {
        unknown: u32,
    },
    AllowInterrupt {
        unknown: u32,
    },
    ProjectileFlag {
        unknown: u32,
    },
    SetJumpState {
        value: u32,
    },
    SetBodyCollisionState {
        body_state: u8,
    },
    BodyCollisionStatus,
    SetBoneCollisionState {
        bone_id: u8,
        collision_state: u32,
    },
    EnableJabFollowUp {
        unknown: u32,
    },
    ToggleJabFollowUp,
    ChangeModelState {
        struct_id: u8,
        object_id: u8,
    },
    RevertModels,
    RemoveModels,
    Throw {
        throw_type: u8,
        damage: u16,
        angle: u16,
        knock_back_growth: u16,
        weight_dependent_set_knockback: u16,
        base_knockback: u16,
        element: u8,
        sfx_severity: u8,
        sfx_kind: u8,
    },
    HeldItemInvisibility {
        flag: bool,
    },
    BodyArticleInvisibility {
        flag: bool,
    },
    CharacterInvisibility {
        flag: bool,
    },
    PseudoRandomSoundEffect {
        unknown:  [u8; 0x1b],
    },
    Unknown0x27 {
        unknown: [u8; 0x0a],
    },
    AnimateTexture {
        material_flag: bool,
        material_index: u8,
        frame_flags: u8,
        frame: u16,
    },
    AnimateModel {
        body_part: u16,
        state: u8,
        unknown: u16,
    },
    Unknown0x2A {
        unknown: u32,
    },
    Rumble {
        unknown_flag: bool,
        unknown_value_1: u16,
        unknown_value_2: u16,
    },
    Unknown0x2C {
        flag: bool,
    },
    Unknown0x2D {
        unknown: [u8; 0x0b],
    },
    BodyAura {
        aura_id: u8,
        duration: u32,
    },
    RemoveColorOverlay {
        unknown: u32,
    },
    Unknown0x30 {
        unknown: u32,
    },
    SwordTrail {
        use_beamsword_trail: bool,
        render_status: u8,
    },
    EnableRagdollPhysics {
        bone_id: u32,
    },
    SelfDamage {
        damage: u16,
    },
    ContinuationControl {
        unknown: u32,
    },
    FootsnapBehavior {
        flags: u32,
    },
    FootstepEffect {
        unknown: [u8; 0x0b],
    },
    LandingEffect {
        unknown: [u8; 0x0b],
    },
    StartSmashCharge {
        charge_frames: u8,
        charge_rate: u16,
        visual_effect: u8,
    },
    Unknown0x39 {
        unknown: u32,
    },
    AestheticWindEffect {
        unknown: [u8; 0x0a],
    },
    Unknown0x3b {
        unknown: u32,
    },
}

pub fn parse_subactions(data: &[u32]) -> Vec<Subaction> {
    let mut i = 0;
    let mut subactions = Vec::new();

    while i < data.len() {
        subactions.push(parse_next_subaction(&data[i..]));
        i += subaction_size(subaction_cmd(data[i]));
    }

    subactions
}

// top 6 bits are always taken by command byte.
// https://github.com/DRGN-DRC/Melee-Modding-Wizard/blob/acfac9408b71b0575131d7ac7c8e284f849243dd/FileSystem/charFiles.py
pub fn parse_next_subaction(data: &[u32]) -> Subaction {
    let cmd = data[0] >> 26;
    use Subaction::*;

    match cmd {
        0x00 => EndOfScript,
        0x01 => SynchronousTimer {
            frame                          : data[0] & 0x03_FF_FF_FF,
        },
        0x02 => AsynchronousTimer {
            frame                          : data[0] & 0x03_FF_FF_FF,
        },
        0x03 => SetLoop {
            loop_count                     : data[0] & 0x03_FF_FF_FF,
        },
        0x04 => ExecuteLoop,
        0x05 => Subroutine {
            pointer                        : data[1],
        },
        0x06 => Return,
        0x07 => GoTo {
            pointer                        : data[1],
        },
        0x08 => SetLoopAnimationTimer,
        0x09 => Unknown0x09 {
            unknown                        : data[0],
        },
        0x0A => GraphicEffect {
            bone_id                        : ((data[0] >> 18) & 0xFF) as u8,
            use_common_bone_id             : (data[0] >> 17) & 0b1 == 1,
            destroy_on_state_change        : (data[0] >> 16) & 0b1 == 1,
            use_unknown_bone_id            : (data[0] >> 15) & 0b1 == 1,
            unknown                        : (data[0] & 0x7F_FF) as u16,
            graphic_id                     : (data[1] >> 16    ) as u16,
            unknown_bone_id                : (data[1] & 0xFF_FF) as u16,
            z_offset                       : (data[2] >> 16    ) as i16,
            y_offset                       : (data[2] & 0xFF_FF) as i16,
            x_offset                       : (data[3] >> 16    ) as i16,
            z_range                        : (data[3] & 0xFF_FF) as u16,
            y_range                        : (data[4] >> 16    ) as u16,
            x_range                        : (data[4] & 0xFF_FF) as u16,
        },
        0x0B => CreateHitbox {
            hitbox_id                      : ((data[0] >> 23) & 0x07) as u8,
            bone_attachment                : ((data[0] >> 11) & 0x7F) as u8,
            damage                         : (data[0] & 0x1_FF) as u16,
            size                           : (data[1] >> 16    ) as u16,
            z_offset                       : (data[1] & 0xFF_FF) as i16,
            y_offset                       : (data[2] >> 16    ) as i16,
            x_offset                       : (data[2] & 0xFF_FF) as i16,
            knockback_angle                : ((data[3] >> 23) & 0x1_FF) as u16,
            knockback_growth               : ((data[3] >> 14) & 0x1_FF) as u16,
            weight_dependent_set_knockback : ((data[3] >> 5) & 0x1_FF) as u16,
            hitbox_interaction             : (data[3] & 0x03) as u8,
            base_knockback                 : ((data[4] >> 23) & 0x1_FF) as u16,
            element                        : ((data[4] >> 18) & 0x1F) as u8,
            unknown                        : (data[4] >> 17) & 0b1 == 1,
            shield_damage                  : ((data[4] >> 10) & 0x7F_FF) as u8,
            sound_effect                   : ((data[4] >> 2) & 0xFF) as u8,
            hit_grounded_opponents         : (data[4] >> 1) & 0x01 == 1,
            hit_airborne_opponents         : (data[4] >> 0) & 0x01 == 1,
        },
        0x0C => AdjustHitboxDamage {
            hitbox_id                      : ((data[0] >> 23) & 0x07) as u8,
            damage                         : ((data[0] >> 0) & 0x7F_FF_FF) as u32,
        },
        0x0D => AdjustHitboxSize {
            hitbox_id                      : ((data[0] >> 23) & 0x07) as u8,
            newsize                        : ((data[0] >> 0) & 0x7F_FF_FF) as u32,
        },
        0x0E => SetHitboxInteraction {
            hitbox_id                      : ((data[0] >> 2) & 0xFF_FF_FF) as u32,
            interact_type                  : (data[0] >> 1) & 0x01 == 1,
            can_interact                   : (data[0] >> 0) & 0x01 == 1,
        },
        0x0F => RemoveHitbox,
        0x10 => ClearHitboxes,
        0x11 => SoundEffect {
            // weirdness... unknown bytes wrap? dunno what's happening
            // https://github.com/DRGN-DRC/Melee-Modding-Wizard/blob/acfac9408b71b0575131d7ac7c8e284f849243dd/FileSystem/charFiles.py
            unknown_1                      : 0,
            unknown_2                      : 0,
            sound_effect_id                : ((data[1] >> 0) & 0x0F_FF_FF) as u32,
            offset                         : data[2],
        },
        0x12 => RandomSmashSFX {
            unknown                        : data[0] & 0x03_FF_FF_FF,
        },
        0x13 => AutoCancel {
            flags                          : ((data[0] >> 24) & 0x03) as u8,
        },
        0x14 => ReverseDirection,
        0x15 => Unknown0x15 {
            unknown                        : data[0] & 0x03_FF_FF_FF,
        },
        0x16 => Unknown0x16 {
            unknown                        : data[0] & 0x03_FF_FF_FF,
        },
        0x17 => AllowInterrupt {
            unknown                        : data[0] & 0x03_FF_FF_FF,
        },
        0x18 => ProjectileFlag {
            unknown                        : data[0] & 0x03_FF_FF_FF,
        },
        0x19 => SetJumpState {
            value                          : data[0] & 0x03_FF_FF_FF,
        },
        0x1A => SetBodyCollisionState {
            body_state                     : (data[0] & 0x03) as u8,
        },
        0x1B => BodyCollisionStatus,
        0x1C => SetBoneCollisionState {
            bone_id                        : ((data[0] >> 18) & 0xFF) as u8,
            collision_state                : data[0] & 0x03_FF_FF,
        },
        0x1D => EnableJabFollowUp {
            unknown                        : data[0] & 0x03_FF_FF_FF,
        },
        0x1E => ToggleJabFollowUp,
        0x1F => ChangeModelState {
            struct_id                      : ((data[0] >> 20) & 0x3F) as u8,
            object_id                      : (data[0] & 0xFF) as u8,
        },
        0x20 => RevertModels,
        0x21 => RemoveModels,
        0x22 => Throw {
            throw_type                     : ((data[0] >> 23) & 0x03) as u8,
            damage                         : (data[0] & 0x01_FF) as u16,
            angle                          : ((data[1] >> 23) & 0x1_FF) as u16,
            knock_back_growth              : ((data[1] >> 14) & 0x1_FF) as u16,
            weight_dependent_set_knockback : ((data[1] >> 5) & 0x1_FF) as u16,
            base_knockback                 : ((data[2] >> 23) & 0x1_FF) as u16,
            element                        : ((data[2] >> 19) & 0x0F) as u8,
            sfx_severity                   : ((data[2] >> 16) & 0x07) as u8,
            sfx_kind                       : ((data[2] >> 12) & 0x0F) as u8,
        },
        0x23 => HeldItemInvisibility {
            flag                           : data[0] & 0x01 == 1,
        },
        0x24 => BodyArticleInvisibility {
            flag                           : data[0] & 0x01 == 1,
        },
        0x25 => CharacterInvisibility {
            flag                           : data[0] & 0x01 == 1,
        },
        0x26 => PseudoRandomSoundEffect {
            // I don't feel like it
            unknown                        : [0u8; 0x1B],
        },
        0x27 => Unknown0x27 {
            unknown                        : [0u8; 0x0A],
        },
        0x28 => AnimateTexture {
            material_flag                  : (data[0] >> 25) & 0x01 == 1,
            material_index                 : ((data[0] >> 18) & 0x7F) as u8,
            frame_flags                    : ((data[0] >> 11) & 0x7F) as u8,
            frame                          : ((data[0] >> 0) & 0x07_FF) as u16,
        },
        0x29 => AnimateModel {
            body_part                      :((data[0] >> 16) & 0x03_FF) as u16, 
            state                          : ((data[0] >> 12) & 0x0F) as u8,
            unknown                        : ((data[0] >> 0) & 0x0F_FF) as u16,
        },
        0x2A => Unknown0x2A {
            unknown                        : data[0] & 0x03_FF_FF_FF,
        },
        0x2B => Rumble {
            unknown_flag                   : (data[0] >> 25) & 0x01 == 1,
            unknown_value_1                : ((data[0] >> 13) & 0x0F_FF) as u16,
            unknown_value_2                : ((data[0] >> 0) & 0x1F_FF) as u16,
        },
        0x2C => Unknown0x2C {
            flag                           : data[0] & 0x01 == 1,
        },
        0x2D => Unknown0x2D {
            // I don't feel like it
            unknown                        : [0u8; 0x0b],
        },
        0x2E => BodyAura {
            aura_id                        : ((data[0] >> 18) & 0xFF) as u8,
            duration                       : ((data[0] >> 0) & 0x03_FF_FF) as u32,
        },
        0x2F => RemoveColorOverlay {
            unknown                        : data[0] & 0x03_FF_FF_FF,
        },
        0x30 => Unknown0x30 {
            unknown                        : data[0] & 0x03_FF_FF_FF,
        },
        0x31 => SwordTrail {
            use_beamsword_trail            : (data[0] >> 25) & 0x01 == 1,
            render_status                  : (data[0] & 0xFF) as u8,
        },
        0x32 => EnableRagdollPhysics {
            bone_id                        : data[0] & 0x03_FF_FF_FF,
        },
        0x33 => SelfDamage {
            damage                         : (data[0] & 0xFF_FF) as u16,
        },
        0x34 => ContinuationControl {
            unknown                        : data[0] & 0x03_FF_FF_FF,
        },
        0x35 => FootsnapBehavior {
            flags                          : data[0] & 0x03_FF_FF_FF,
        },
        0x36 => FootstepEffect {
            // I don't feel like it
            unknown                        : [0u8; 0x0B],
        },
        0x37 => LandingEffect {
            unknown                        : [0u8; 0x0B],
        },
        0x38 => StartSmashCharge {
            charge_frames                  : ((data[0] >> 16) & 0xFF) as u8,
            charge_rate                    : (data[0] & 0xFF_FF) as u16,
            visual_effect                  : (data[1] >> 24) as u8,
        },
        0x39 => Unknown0x39 {
            unknown                        : data[0] & 0x03_FF_FF_FF,
        },
        0x3A => AestheticWindEffect {
            // I don't feel like it
            unknown                        : [0u8; 0x0A],
        },
        0x3B => Unknown0x3b {
            unknown                        : data[0] & 0x03_FF_FF_FF,
        },
        _ => panic!("invalid subaction"),
    }
}

pub fn subaction_cmd(subaction_word: u32) -> u8 {
    (subaction_word >> 26) as u8
}

// number of u32s to pass (includes the command byte)
pub fn subaction_size(subaction_cmd: u8) -> usize {
    let packed_len = SUBACTION_SIZE[subaction_cmd as usize / 2] as usize;
    let shift = (subaction_cmd as usize % 2) * 4;
    (packed_len >> shift) & 0b1111
}

/// 1 nibble per subaction.
/// gives number of 32 bit segments in subaction.
pub static SUBACTION_SIZE: &'static [u8] = &[
    0x11,
    0x11,
    0x21,
    0x21,
    0x11,
    0x55,
    0x11,
    0x11,
    0x31,
    0x11,
    0x11,
    0x11,
    0x11,
    0x11,
    0x11,
    0x11,
    0x11,
    0x13,
    0x11,
    0x47,
    0x11,
    0x11,
    0x31,
    0x11,
    0x11,
    0x11,
    0x11,
    0x33,
    0x12,
    0x14,
];
