pub type JOBJFlags = u32;
pub mod jobj_flags {
    pub const SKELETON           : u32 = 1 << 0;
    pub const SKELETON_ROOT      : u32 = 1 << 1;
    pub const ENVELOPE_MODEL     : u32 = 1 << 2;
    pub const CLASSICAL_SCALING  : u32 = 1 << 3;
    pub const HIDDEN             : u32 = 1 << 4;
    pub const PTCL               : u32 = 1 << 5;
    pub const MTX_DIRTY          : u32 = 1 << 6;
    pub const LIGHTING           : u32 = 1 << 7;
    pub const TEXGEN             : u32 = 1 << 8;
    pub const BILLBOARD          : u32 = 1 << 9;
    pub const VBILLBOARD         : u32 = 2 << 9;
    pub const HBILLBOARD         : u32 = 3 << 9;
    pub const RBILLBOARD         : u32 = 4 << 9;
    pub const INSTANCE           : u32 = 1 << 12;
    pub const PBILLBOARD         : u32 = 1 << 13;
    pub const SPLINE             : u32 = 1 << 14;
    pub const FLIP_IK            : u32 = 1 << 15;
    pub const SPECULAR           : u32 = 1 << 16;
    pub const USE_QUATERNION     : u32 = 1 << 17;
    pub const OPA                : u32 = 1 << 18;
    pub const XLU                : u32 = 1 << 19;
    pub const TEXEDGE            : u32 = 1 << 20;
    pub const NULL               : u32 = 0 << 21;
    pub const JOINT1             : u32 = 1 << 21;
    pub const JOINT2             : u32 = 2 << 21;
    pub const EFFECTOR           : u32 = 3 << 21;
    pub const USER_DEFINED_MTX   : u32 = 1 << 23;
    pub const MTX_INDEPEND_PARENT: u32 = 1 << 24;
    pub const MTX_INDEPEND_SRT   : u32 = 1 << 25;
    pub const ROOT_OPA           : u32 = 1 << 28;
    pub const ROOT_XLU           : u32 = 1 << 29;
    pub const ROOT_TEXEDGE       : u32 = 1 << 30;
}

pub type ROBJFlags = u32;
pub mod robj_flags {
    const EXP     : u32 = 0x00000000;
    const JOBJ    : u32 = 0x10000000;
    const LIMIT   : u32 = 0x20000000;
    const BYTECODE: u32 = 0x30000000;
    const IKHINT  : u32 = 0x40000000;

    const MIN_ROTX: u32 = 01;
    const MAX_ROTX: u32 = 02;
    const MIN_ROTY: u32 = 03;
    const MAX_ROTY: u32 = 04;
    const MIN_ROTZ: u32 = 05;
    const MAX_ROTZ: u32 = 06;
    const MIN_TRAX: u32 = 07;
    const MAX_TRAX: u32 = 08;
    const MIN_TRAY: u32 = 09;
    const MAX_TRAY: u32 = 10;
    const MIN_TRAZ: u32 = 11;
    const MAX_TRAZ: u32 = 12;
}

pub type PixelProcessFlags = u8;
pub mod pixel_process_enable_flags {
    pub const COLOR_UPDATE: u8 = 1 << 0;
    pub const ALPHA_UPDATE: u8 = 1 << 1;
    pub const DST_ALPHA   : u8 = 1 << 2;
    pub const BEFORE_TEX  : u8 = 1 << 3;
    pub const COMPARE     : u8 = 1 << 4;
    pub const ZUPDATE     : u8 = 1 << 5;
    pub const DITHER      : u8 = 1 << 6;
}

pub type POBJFlags = u16;
pub mod pobj_flags {
    pub const SHAPESET_AVERAGE : u16 = 1 << 0;
    pub const SHAPESET_ADDITIVE: u16 = 1 << 1;
    pub const UNKNOWN2         : u16 = 1 << 2;
    pub const ANIM             : u16 = 1 << 3;
    pub const SHAPEANIM        : u16 = 1 << 12;
    pub const ENVELOPE         : u16 = 1 << 13;
    pub const CULLBACK         : u16 = 1 << 14;
    pub const CULLFRONT        : u16 = 1 << 15;
}

pub type MOBJRenderModeFlags = u32;
pub mod mobj_render_mode_flags {
    pub const CONSTANT    : u32 = 1 << 0;
    pub const VERTEX      : u32 = 1 << 1;
    pub const DIFFUSE     : u32 = 1 << 2;
    pub const SPECULAR    : u32 = 1 << 3;
    pub const TEX0        : u32 = 1 << 4;
    pub const TEX1        : u32 = 1 << 5;
    pub const TEX2        : u32 = 1 << 6;
    pub const TEX3        : u32 = 1 << 7;
    pub const TEX4        : u32 = 1 << 8;
    pub const TEX5        : u32 = 1 << 9;
    pub const TEX6        : u32 = 1 << 10;
    pub const TEX7        : u32 = 1 << 11;
    pub const TOON        : u32 = 1 << 12;
    pub const ALPHA_MAT   : u32 = 1 << 13;
    pub const ALPHA_VTX   : u32 = 2 << 13;
    pub const ALPHA_BOTH  : u32 = 3 << 13;
    pub const ZOFST       : u32 = 1 << 24;
    pub const EFFECT      : u32 = 1 << 25;
    pub const SHADOW      : u32 = 1 << 26;
    pub const ZMODE_ALWAYS: u32 = 1 << 27;
    pub const DF_ALL      : u32 = 1 << 28;
    pub const NO_ZUPDATE  : u32 = 1 << 29;
    pub const XLU         : u32 = 1 << 30;
    pub const USER        : u32 = 1 << 31;
}

pub type BlendMode = u8;
pub mod blend_mode {
    pub const NONE    : u8 = 0;
    pub const BLEND   : u8 = 1;
    pub const LOGIC   : u8 = 2;
    pub const SUBTRACT: u8 = 3;
}

pub type LogicOp = u8;
pub mod logic_op {
    pub const CLEAR  : u8 = 0;
    pub const AND    : u8 = 1;
    pub const REVAND : u8 = 2;
    pub const COPY   : u8 = 3;
    pub const INVAND : u8 = 4;
    pub const NOOP   : u8 = 5;
    pub const XOR    : u8 = 6;
    pub const OR     : u8 = 7;
    pub const NOR    : u8 = 8;
    pub const EQUIV  : u8 = 9;
    pub const INV    : u8 = 10;
    pub const REVOR  : u8 = 11;
    pub const INVCOPY: u8 = 12;
    pub const INVOR  : u8 = 13;
    pub const NAND   : u8 = 14;
    pub const SET    : u8 = 15;
}

pub type CompareType = u8;
pub mod compare_type {
    pub const NEVER           : u8 = 0;
    pub const LESS            : u8 = 1;
    pub const EQUAL           : u8 = 2;
    pub const LESS_OR_EQUAL   : u8 = 3;
    pub const GREATER         : u8 = 4;
    pub const NOT_EQUAL       : u8 = 5;
    pub const GREATER_OR_EQUAL: u8 = 6;
    pub const ALWAYS          : u8 = 7;
}

pub type AlphaOp = u8;
pub mod alpha_op {
    pub const AND : u8 = 0;
    pub const OR  : u8 = 1;
    pub const XOR : u8 = 2;
    pub const XNOR: u8 = 3;
}

pub type BlendFactor = u8;
pub mod blend_factor {
    pub const ZERO       : u8 = 0;
    pub const ONE        : u8 = 1;
    pub const SRCCLR     : u8 = 2;
    pub const INVSRCCLR  : u8 = 3;
    pub const SRCALPHA   : u8 = 4;
    pub const INVSRCALPHA: u8 = 5;
    pub const DSTALPHA   : u8 = 6;
    pub const INVDSTALPHA: u8 = 7;

    pub const DSTCLR     : u8 = SRCCLR;
    pub const INVDSTCLR  : u8 = INVSRCCLR;
}

pub type TexMapID = u32;
pub mod tex_map_id {
    pub const TEXMAP0       : u32 = 0;
    pub const TEXMAP1       : u32 = 1;
    pub const TEXMAP2       : u32 = 2;
    pub const TEXMAP3       : u32 = 3;
    pub const TEXMAP4       : u32 = 4;
    pub const TEXMAP5       : u32 = 5;
    pub const TEXMAP6       : u32 = 6;
    pub const TEXMAP7       : u32 = 7;
    pub const MAX_TEXMAP    : u32 = 8;
    pub const TEXMAP_NULL   : u32 = 9;
    pub const TEXMAP_DISABLE: u32 = 10;
}

pub type TexGenSrc = u32;
pub mod tex_gen_src {
    pub const POS      : u32 = 0;
    pub const NRM      : u32 = 1;
    pub const BINRM    : u32 = 2;
    pub const TANGENT  : u32 = 3;
    pub const TEX0     : u32 = 4;
    pub const TEX1     : u32 = 5;
    pub const TEX2     : u32 = 6;
    pub const TEX3     : u32 = 7;
    pub const TEX4     : u32 = 8;
    pub const TEX5     : u32 = 9;
    pub const TEX6     : u32 = 10;
    pub const TEX7     : u32 = 11;
    pub const TEXCOORD0: u32 = 12;
    pub const TEXCOORD1: u32 = 13;
    pub const TEXCOORD2: u32 = 14;
    pub const TEXCOORD3: u32 = 15;
    pub const TEXCOORD4: u32 = 16;
    pub const TEXCOORD5: u32 = 17;
    pub const TEXCOORD6: u32 = 18;
    pub const COLOR0   : u32 = 19;
    pub const COLOR1   : u32 = 20;
}

pub type TOBJFlags = u32;
pub mod tobj_flags {
    pub const COORD_UV           : u32 = 0 << 0;
    pub const COORD_REFLECTION   : u32 = 1 << 0;
    pub const COORD_HILIGHT      : u32 = 2 << 0;
    pub const COORD_SHADOW       : u32 = 3 << 0;
    pub const COORD_TOON         : u32 = 4 << 0;
    pub const COORD_GRADATION    : u32 = 5 << 0;
    pub const LIGHTMAP_DIFFUSE   : u32 = 1 << 4;
    pub const LIGHTMAP_SPECULAR  : u32 = 1 << 5;
    pub const LIGHTMAP_AMBIENT   : u32 = 1 << 6;
    pub const LIGHTMAP_EXT       : u32 = 1 << 7;
    pub const LIGHTMAP_SHADOW    : u32 = 1 << 8;
    pub const COLORMAP_ALPHA_MASK: u32 = 1 << 16;
    pub const COLORMAP_RGB_MASK  : u32 = 2 << 16;
    pub const COLORMAP_BLEND     : u32 = 3 << 16;
    pub const COLORMAP_MODULATE  : u32 = 4 << 16;
    pub const COLORMAP_REPLACE   : u32 = 5 << 16;
    pub const COLORMAP_PASS      : u32 = 6 << 16;
    pub const COLORMAP_ADD       : u32 = 7 << 16;
    pub const COLORMAP_SUB       : u32 = 8 << 16;
    pub const ALPHAMAP_ALPHA_MASK: u32 = 1 << 20;
    pub const ALPHAMAP_BLEND     : u32 = 2 << 20;
    pub const ALPHAMAP_MODULATE  : u32 = 3 << 20;
    pub const ALPHAMAP_REPLACE   : u32 = 4 << 20;
    pub const ALPHAMAP_PASS      : u32 = 5 << 20;
    pub const ALPHAMAP_ADD       : u32 = 6 << 20;
    pub const ALPHAMAP_SUB       : u32 = 7 << 20;
    pub const BUMP               : u32 = 1 << 24;
    pub const MTX_DIRTY          : u32 = 1 << 31;
}

pub type WrapMode = u32;
pub mod wrap_mode {
    pub const WRAP  : u32 = 0;
    pub const REPEAT: u32 = 1;
    pub const MIRROR: u32 = 2;
}

pub type TexFilter = u32;
pub mod tex_filter {
    pub const NEAR         : u32 = 0;
    pub const LINEAR       : u32 = 1;
    pub const NEAR_MIP_NEAR: u32 = 2;
    pub const LIN_MIP_NEAR : u32 = 3;
    pub const NEAR_MIP_LIN : u32 = 4;
    pub const LIN_MIP_LIN  : u32 = 5;
}

pub type TexFormat = u32;
pub mod tex_format {
    pub const I4    : u32 = 0;
    pub const I8    : u32 = 1;
    pub const IA4   : u32 = 2;
    pub const IA8   : u32 = 3;
    pub const RGB565: u32 = 4;
    pub const RGB5A3: u32 = 5;
    pub const RGBA8 : u32 = 6;
    pub const CI4   : u32 = 8;
    pub const CI8   : u32 = 9;
    pub const CI14X2: u32 = 10;
    pub const CMP   : u32 = 14;
}

pub type TlutFormat = u32;
pub mod tlut_format {
    pub const IA8   : u32 = 0;
    pub const RGB565: u32 = 1;
    pub const RGB5A3: u32 = 2;
}

pub type Anisotropy = u32;
pub mod anisotropy {
    pub const ANISO_1: u32 = 0;
    pub const ANISO_2: u32 = 1;
    pub const ANISO_4: u32 = 2;
    pub const MAX_ANISOTROPY: u32 = 3;
}

pub type TevColourOp = u8;
pub mod tev_colour_op {
    pub const ADD          : u8 = 0;
    pub const SUB          : u8 = 1;
    pub const COMP_R8_GT   : u8 = 8;
    pub const COMP_R8_EQ   : u8 = 9;
    pub const COMP_GR16_GT : u8 = 10;
    pub const COMP_GR16_EQ : u8 = 11;
    pub const COMP_BGR24_GT: u8 = 12;
    pub const COMP_BGR24_EQ: u8 = 13;
    pub const COMP_RGB8_GT : u8 = 14;
    pub const COMP_RGB8_EQ : u8 = 15;
}

pub type TevAlphaOp = u8;
pub mod tev_alpha_op {
    pub const ADD          : u8 = 0;
    pub const SUB          : u8 = 1;
    pub const COMP_R8_GT   : u8 = 8;
    pub const COMP_R8_EQ   : u8 = 9;
    pub const COMP_GR16_GT : u8 = 10;
    pub const COMP_GR16_EQ : u8 = 11;
    pub const COMP_BGR24_GT: u8 = 12;
    pub const COMP_BGR24_EQ: u8 = 13;
    pub const COMP_A8_GT   : u8 = 14;
    pub const COMP_A8_EQ   : u8 = 15;
}

pub type TevBias = u8;
pub mod tev_bias {
    pub const ZERO: u8 = 0;
    pub const ADDHALF: u8 = 1;
    pub const SUBHALF: u8 = 2;
}

pub type TevScale = u8;
pub mod tev_scale {
    pub const SCALE_1: u8 = 0;
    pub const SCALE_2: u8 = 1;
    pub const SCALE_4: u8 = 2;
    pub const DIVIDE_2: u8 = 3;
}

pub type TEVActive = u32;
pub mod tev_active {
    pub const KONST_R  : u32 = 0x01 << 0;
    pub const KONST_G  : u32 = 0x01 << 1;
    pub const KONST_B  : u32 = 0x01 << 2;
    pub const KONST_A  : u32 = 0x01 << 3;
    pub const KONST    : u32 = KONST_R | KONST_G | KONST_B | KONST_A;
    pub const TEV0_R   : u32 = 0x01 << 4;
    pub const TEV0_G   : u32 = 0x01 << 5;
    pub const TEV0_B   : u32 = 0x01 << 6;
    pub const TEV0_A   : u32 = 0x01 << 7;
    pub const TEV0     : u32 = TEV0_R | TEV0_G | TEV0_B | TEV0_A;
    pub const TEV1_R   : u32 = 0x01 << 8;
    pub const TEV1_G   : u32 = 0x01 << 9;
    pub const TEV1_B   : u32 = 0x01 << 10;
    pub const TEV1_A   : u32 = 0x01 << 11;
    pub const TEV1     : u32 = TEV1_R | TEV1_G | TEV1_B | TEV1_A;
    pub const COLOR_TEV: u32 = 0x01 << 30;
    pub const ALPHA_TEV: u32 = 0x01 << 31;
}
