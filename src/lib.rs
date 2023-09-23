pub mod dat;
pub mod isoparser;

use dat::{Image, FighterData};
use isoparser::{ISOParseError, ISODatFiles};
use slp_parser::{Stage, Character, CharacterColour, character_colours::*};

pub fn parse_string(bytes: &[u8]) -> Option<&str> {
    let terminator = bytes.iter().position(|b| *b == 0)?;
    let str_bytes = &bytes[..terminator];
    std::str::from_utf8(str_bytes).ok()
}

pub fn open_iso(path: &str) -> Result<ISODatFiles, ISOParseError> {
    let file = std::fs::File::open(path).map_err(|_| ISOParseError::FileNotFound)?;
    ISODatFiles::new(file)
}

pub fn get_fighter_data(
    files: &mut ISODatFiles, 
    character_colour: CharacterColour,
) -> Result<FighterData, ISOParseError> {
    let character = character_colour.character();
    let data_filename = character_data_filename(character);
    let anim_filename = character_animation_filename(character);
    let model_filename = character_model_filename(character_colour);

    let base_dat = files.read_file(data_filename)?;
    let anim_dat = files.read_file(anim_filename)?;
    let model_dat = files.read_file(model_filename)?;

    dat::parse_fighter_data(&base_dat, &anim_dat, &model_dat)
        .ok_or(ISOParseError::InvalidISO)
}

pub const fn stage_filename(stage: Stage) -> &'static str {
    match stage {
        Stage::FountainOfDreams     => "GrIz.dat",
        Stage::PokemonStadium       => "GrPs.dat",
        Stage::PrincessPeachsCastle => "GrCs.dat",
        Stage::KongoJungle          => "GrKg.dat",
        Stage::Brinstar             => "GrZe.dat",
        Stage::Corneria             => "GrCn.dat",
        Stage::YoshisStory          => "GrOy.dat",
        Stage::Onett                => "GrOt.dat",
        Stage::MuteCity             => "GrMc.dat",
        Stage::RainbowCruise        => "GrRc.dat",
        Stage::JungleJapes          => "GrGd.dat",
        Stage::GreatBay             => "GrGb.dat",
        Stage::HyruleTemple         => "GrSh.dat",
        Stage::BrinstarDepths       => "GrKr.dat",
        Stage::YoshisIsland         => "GrYt.dat",
        Stage::GreenGreens          => "GrGr.dat",
        Stage::Fourside             => "GrFs.dat",
        Stage::MushroomKingdomI     => "GrI1.dat",
        Stage::MushroomKingdomII    => "GrI2.dat",
        Stage::Venom                => "GrVe.dat",
        Stage::PokeFloats           => "GrPu.dat",
        Stage::BigBlue              => "GrBB.dat",
        Stage::IcicleMountain       => "GrIm.dat",
        Stage::FlatZone             => "GrFz.dat",
        Stage::DreamLandN64         => "GrOp.dat",
        Stage::YoshisIslandN64      => "GrYt.dat",
        Stage::KongoJungleN64       => "GrOk.dat",
        Stage::Battlefield          => "GrNBa.dat",
        Stage::FinalDestination     => "GrNLa.dat",
    }
}

pub fn extract_stock_icons(files: &mut ISODatFiles) -> Option<Box<[Image]>> {
    let mut icons = Vec::with_capacity(130);

    let dat = files.read_file("IfAll.dat").ok()?;
    let hsd_if_dat = dat::HSDRawFile::new(&dat);

    // root is HSD_SOBJ
    let root = hsd_if_dat.roots.iter()
        .find(|r| r.root_string == "Stc_scemdls")
        .unwrap().hsd_struct.clone();
    
    // don't question it
    let mut mat_anim_j = root.get_reference(0x00)
        .get_reference(0x08)
        .get_reference(0x00);

    while let Some(sibling_mat_anim_j) = mat_anim_j.try_get_reference(0x00) {
        mat_anim_j = sibling_mat_anim_j;

        let mat_anim = mat_anim_j.get_reference(0x08);
        let tex_anim = mat_anim.get_reference(0x08);
        let im_buffers = tex_anim.get_reference(0x0C);
        let tlut_buffers = tex_anim.get_reference(0x10);

        for i in 0..(im_buffers.len() / 4) {
            let hsd_image = im_buffers.get_reference(i * 4);
            let tlut = tlut_buffers.try_get_reference(i * 4)
                .map(dat::TLUT::new);
            let icon = dat::decode_image(hsd_image, tlut);
            icons.push(icon);
        }
    }

    Some(icons.into_boxed_slice())
}

pub const fn character_model_filename(character: CharacterColour) -> &'static str {
    use CharacterColour::*;
    match character {
        CaptainFalcon (CaptainFalconColour ::Neutral  ) => "PlCaNr.dat",
        CaptainFalcon (CaptainFalconColour ::Grey     ) => "PlCaGy.dat",
        CaptainFalcon (CaptainFalconColour ::Red      ) => "PlCaRe.dat",
        CaptainFalcon (CaptainFalconColour ::White    ) => "PlCaWh.dat",
        CaptainFalcon (CaptainFalconColour ::Green    ) => "PlCaGr.dat",
        CaptainFalcon (CaptainFalconColour ::Blue     ) => "PlCaBu.dat",

        DonkeyKong    (DonkeyKongColour    ::Neutral  ) => "PlDkNr.dat",
        DonkeyKong    (DonkeyKongColour    ::Black    ) => "PlDkBk.dat",
        DonkeyKong    (DonkeyKongColour    ::Red      ) => "PlDkRe.dat",
        DonkeyKong    (DonkeyKongColour    ::Blue     ) => "PlDkBu.dat",
        DonkeyKong    (DonkeyKongColour    ::Green    ) => "PlDkGr.dat",

        Fox           (FoxColour           ::Neutral  ) => "PlFxNr.dat",
        Fox           (FoxColour           ::Orange   ) => "PlFxOr.dat",
        Fox           (FoxColour           ::Lavender ) => "PlFxLa.dat",
        Fox           (FoxColour           ::Green    ) => "PlFxGr.dat",

        MrGameAndWatch(_) => "PlGwNr.dat", // all colours use the same model

        Kirby         (KirbyColour         ::Neutral  ) => "PlKbNr.dat",
        Kirby         (KirbyColour         ::Yellow   ) => "PlKbYe.dat",
        Kirby         (KirbyColour         ::Blue     ) => "PlKbBu.dat",
        Kirby         (KirbyColour         ::Red      ) => "PlKbRe.dat",
        Kirby         (KirbyColour         ::Green    ) => "PlKbGr.dat",
        Kirby         (KirbyColour         ::White    ) => "PlKbWh.dat",

        Bowser        (BowserColour        ::Neutral  ) => "PlKpNr.dat",
        Bowser        (BowserColour        ::Red      ) => "PlKpRe.dat",
        Bowser        (BowserColour        ::Blue     ) => "PlKpBu.dat",
        Bowser        (BowserColour        ::Black    ) => "PlKpBk.dat",

        Link          (LinkColour          ::Neutral  ) => "PlLkNr.dat",
        Link          (LinkColour          ::Red      ) => "PlLkRe.dat",
        Link          (LinkColour          ::Blue     ) => "PlLkBu.dat",
        Link          (LinkColour          ::Black    ) => "PlLkBk.dat",
        Link          (LinkColour          ::White    ) => "PlLkWh.dat",

        Luigi         (LuigiColour         ::Neutral  ) => "PlLgNr.dat",
        Luigi         (LuigiColour         ::White    ) => "PlLgWh.dat",
        Luigi         (LuigiColour         ::Aqua     ) => "PlLgAq.dat",
        Luigi         (LuigiColour         ::Pink     ) => "PlLgPi.dat",

        Mario         (MarioColour         ::Neutral  ) => "PlMrNr.dat",
        Mario         (MarioColour         ::Yellow   ) => "PlMrYe.dat",
        Mario         (MarioColour         ::Black    ) => "PlMrBk.dat",
        Mario         (MarioColour         ::Blue     ) => "PlMrBu.dat",
        Mario         (MarioColour         ::Green    ) => "PlMrGr.dat",

        Marth         (MarthColour         ::Neutral  ) => "PlMsNr.dat",
        Marth         (MarthColour         ::Red      ) => "PlMsRe.dat",
        Marth         (MarthColour         ::Green    ) => "PlMsGr.dat",
        Marth         (MarthColour         ::Black    ) => "PlMsBk.dat",
        Marth         (MarthColour         ::White    ) => "PlMsWh.dat",

        Mewtwo        (MewtwoColour        ::Neutral  ) => "PlMtNr.dat",
        Mewtwo        (MewtwoColour        ::Red      ) => "PlMtRe.dat",
        Mewtwo        (MewtwoColour        ::Blue     ) => "PlMtBu.dat",
        Mewtwo        (MewtwoColour        ::Green    ) => "PlMtGr.dat",

        Ness          (NessColour          ::Neutral  ) => "PlNsNr.dat",
        Ness          (NessColour          ::Yellow   ) => "PlNsYe.dat",
        Ness          (NessColour          ::Blue     ) => "PlNsBu.dat",
        Ness          (NessColour          ::Green    ) => "PlNsGr.dat",

        Peach         (PeachColour         ::Neutral  ) => "PlPeNr.dat",
        Peach         (PeachColour         ::Yellow   ) => "PlPeYe.dat",
        Peach         (PeachColour         ::White    ) => "PlPeWh.dat",
        Peach         (PeachColour         ::Blue     ) => "PlPeBu.dat",
        Peach         (PeachColour         ::Green    ) => "PlPeGr.dat",

        Pikachu       (PikachuColour       ::Neutral  ) => "PlPkNr.dat",
        Pikachu       (PikachuColour       ::Red      ) => "PlPkRe.dat",
        Pikachu       (PikachuColour       ::Blue     ) => "PlPkBu.dat",
        Pikachu       (PikachuColour       ::Green    ) => "PlPkGr.dat",

        Popo          (IceClimbersColour   ::Neutral  ) => "PlPpNr.dat",
        Popo          (IceClimbersColour   ::Green    ) => "PlPpGr.dat",
        Popo          (IceClimbersColour   ::Orange   ) => "PlPpOr.dat",
        Popo          (IceClimbersColour   ::Red      ) => "PlPpRe.dat",

        Nana          (IceClimbersColour   ::Neutral  ) => "PlNnNr.dat",
        Nana          (IceClimbersColour   ::Green    ) => "PlNnYe.dat",
        Nana          (IceClimbersColour   ::Orange   ) => "PlNnAq.dat",
        Nana          (IceClimbersColour   ::Red      ) => "PlNnWh.dat",

        Jigglypuff    (JigglypuffColour    ::Neutral  ) => "PlPrNr.dat",
        Jigglypuff    (JigglypuffColour    ::Red      ) => "PlPrRe.dat",
        Jigglypuff    (JigglypuffColour    ::Blue     ) => "PlPrBu.dat",
        Jigglypuff    (JigglypuffColour    ::Green    ) => "PlPrGr.dat",
        Jigglypuff    (JigglypuffColour    ::Yellow   ) => "PlPrYe.dat",

        Samus         (SamusColour         ::Neutral  ) => "PlSsNe.dat",
        Samus         (SamusColour         ::Pink     ) => "PlSsPi.dat",
        Samus         (SamusColour         ::Black    ) => "PlSsBk.dat",
        Samus         (SamusColour         ::Green    ) => "PlSsGr.dat",
        Samus         (SamusColour         ::Lavender ) => "PlSsLa.dat",

        Yoshi         (YoshiColour         ::Neutral  ) => "PlYsNr.dat",
        Yoshi         (YoshiColour         ::Red      ) => "PlYsRe.dat",
        Yoshi         (YoshiColour         ::Blue     ) => "PlYsBu.dat",
        Yoshi         (YoshiColour         ::Yellow   ) => "PlYsYe.dat",
        Yoshi         (YoshiColour         ::Pink     ) => "PlYsPi.dat",
        Yoshi         (YoshiColour         ::Aqua     ) => "PlYsAq.dat",

        Sheik         (ZeldaColour         ::Neutral  ) => "PlSkNr.dat",
        Sheik         (ZeldaColour         ::Red      ) => "PlSkRe.dat",
        Sheik         (ZeldaColour         ::Blue     ) => "PlSkBu.dat",
        Sheik         (ZeldaColour         ::Green    ) => "PlSkGr.dat",
        Sheik         (ZeldaColour         ::White    ) => "PlSkWh.dat",

        Zelda         (ZeldaColour         ::Neutral  ) => "PlZdNr.dat",
        Zelda         (ZeldaColour         ::Red      ) => "PlZdRe.dat",
        Zelda         (ZeldaColour         ::Blue     ) => "PlZdBu.dat",
        Zelda         (ZeldaColour         ::Green    ) => "PlZdGr.dat",
        Zelda         (ZeldaColour         ::White    ) => "PlZdWh.dat",

        Falco         (FalcoColour         ::Neutral  ) => "PlFcNr.dat",
        Falco         (FalcoColour         ::Red      ) => "PlFcRe.dat",
        Falco         (FalcoColour         ::Blue     ) => "PlFcBu.dat",
        Falco         (FalcoColour         ::Green    ) => "PlFcGr.dat",

        YoungLink     (YoungLinkColour     ::Neutral  ) => "PlClNr.dat",
        YoungLink     (YoungLinkColour     ::Red      ) => "PlClRe.dat",
        YoungLink     (YoungLinkColour     ::Blue     ) => "PlClBu.dat",
        YoungLink     (YoungLinkColour     ::White    ) => "PlClWh.dat",
        YoungLink     (YoungLinkColour     ::Black    ) => "PlClBk.dat",

        DrMario       (DrMarioColour       ::Neutral  ) => "PlDrNr.dat",
        DrMario       (DrMarioColour       ::Red      ) => "PlDrRe.dat",
        DrMario       (DrMarioColour       ::Blue     ) => "PlDrBu.dat",
        DrMario       (DrMarioColour       ::Green    ) => "PlDrGr.dat",
        DrMario       (DrMarioColour       ::Black    ) => "PlDrBk.dat",

        Roy           (RoyColour           ::Neutral  ) => "PlFeNr.dat",
        Roy           (RoyColour           ::Red      ) => "PlFeRe.dat",
        Roy           (RoyColour           ::Blue     ) => "PlFeBu.dat",
        Roy           (RoyColour           ::Green    ) => "PlFeGr.dat",
        Roy           (RoyColour           ::Yellow   ) => "PlFeYe.dat",

        Pichu         (PichuColour         ::Neutral  ) => "PlPcNr.dat",
        Pichu         (PichuColour         ::Red      ) => "PlPcRe.dat",
        Pichu         (PichuColour         ::Blue     ) => "PlPcBu.dat",
        Pichu         (PichuColour         ::Green    ) => "PlPcGr.dat",

        Ganondorf     (GanondorfColour     ::Neutral  ) => "PlGnNr.dat",
        Ganondorf     (GanondorfColour     ::Red      ) => "PlGnRe.dat",
        Ganondorf     (GanondorfColour     ::Blue     ) => "PlGnBu.dat",
        Ganondorf     (GanondorfColour     ::Green    ) => "PlGnGr.dat",
        Ganondorf     (GanondorfColour     ::Lavender ) => "PlGnLa.dat",
    }
}

pub const fn character_data_filename(character: Character) -> &'static str {
    match character {
        Character::Mario          => "PlMr.dat",
        Character::Fox            => "PlFx.dat",
        Character::CaptainFalcon  => "PlCa.dat",
        Character::DonkeyKong     => "PlDk.dat",
        Character::Kirby          => "PlKb.dat",
        Character::Bowser         => "PlKp.dat",
        Character::Link           => "PlLk.dat",
        Character::Sheik          => "PlSk.dat",
        Character::Ness           => "PlNs.dat",
        Character::Peach          => "PlPe.dat",
        Character::Popo           => "PlPp.dat",
        Character::Nana           => "PlNn.dat",
        Character::Pikachu        => "PlPk.dat",
        Character::Samus          => "PlSs.dat",
        Character::Yoshi          => "PlYs.dat",
        Character::Jigglypuff     => "PlPr.dat",
        Character::Mewtwo         => "PlMt.dat",
        Character::Luigi          => "PlLg.dat",
        Character::Marth          => "PlMs.dat",
        Character::Zelda          => "PlZd.dat",
        Character::YoungLink      => "PlCl.dat",
        Character::DrMario        => "PlDr.dat",
        Character::Falco          => "PlFc.dat",
        Character::Pichu          => "PlPc.dat",
        Character::MrGameAndWatch => "PlGw.dat",
        Character::Ganondorf      => "PlGn.dat",
        Character::Roy            => "PlFe.dat",
    }
}

pub const fn character_effect_filename(character: Character) -> Option<&'static str> {
    match character {
        Character::Fox   => Some("EfFxData.dat"),
        Character::Falco => Some("EfFxData.dat"), // maybe??
        Character::Marth => Some("PlMsData.dat"),
        Character::Peach => Some("PlMsData.dat"),
        _ => todo!(),
    }
}

pub const fn character_animation_filename(character: Character) -> &'static str {
    match character {
        Character::Mario          => "PlMrAJ.dat",
        Character::Fox            => "PlFxAJ.dat",
        Character::CaptainFalcon  => "PlCaAJ.dat",
        Character::DonkeyKong     => "PlDkAJ.dat",
        Character::Kirby          => "PlKbAJ.dat",
        Character::Bowser         => "PlKpAJ.dat",
        Character::Link           => "PlLkAJ.dat",
        Character::Sheik          => "PlSkAJ.dat",
        Character::Ness           => "PlNsAJ.dat",
        Character::Peach          => "PlPeAJ.dat",
        Character::Popo           => "PlPpAJ.dat",
        Character::Nana           => "PlNnAJ.dat",
        Character::Pikachu        => "PlPkAJ.dat",
        Character::Samus          => "PlSsAJ.dat",
        Character::Yoshi          => "PlYsAJ.dat",
        Character::Jigglypuff     => "PlPrAJ.dat",
        Character::Mewtwo         => "PlMtAJ.dat",
        Character::Luigi          => "PlLgAJ.dat",
        Character::Marth          => "PlMsAJ.dat",
        Character::Zelda          => "PlZdAJ.dat",
        Character::YoungLink      => "PlClAJ.dat",
        Character::DrMario        => "PlDrAJ.dat",
        Character::Falco          => "PlFcAJ.dat",
        Character::Pichu          => "PlPcAJ.dat",
        Character::MrGameAndWatch => "PlGwAJ.dat",
        Character::Ganondorf      => "PlGnAJ.dat",
        Character::Roy            => "PlFeAJ.dat",
    }
}

pub const fn inner_character_prefix(character: Character) -> &'static str {
    match character {
        Character::Mario          => "Mr",
        Character::Fox            => "Fx",
        Character::CaptainFalcon  => "Ca",
        Character::DonkeyKong     => "Dk",
        Character::Kirby          => "Kb",
        Character::Bowser         => "Kp",
        Character::Link           => "Lk",
        Character::Sheik          => "Sk",
        Character::Ness           => "Ns",
        Character::Peach          => "Pe",
        Character::Popo           => "Pp",
        Character::Nana           => "Nn",
        Character::Pikachu        => "Pk",
        Character::Samus          => "Ss",
        Character::Yoshi          => "Ys",
        Character::Jigglypuff     => "Pr",
        Character::Mewtwo         => "Mt",
        Character::Luigi          => "Lg",
        Character::Marth          => "Ms",
        Character::Zelda          => "Zd",
        Character::YoungLink      => "Cl",
        Character::DrMario        => "Dr",
        Character::Falco          => "Fc",
        Character::Pichu          => "Pc",
        Character::MrGameAndWatch => "Gw",
        Character::Ganondorf      => "Gn",
        Character::Roy            => "Fe",
    }
}
