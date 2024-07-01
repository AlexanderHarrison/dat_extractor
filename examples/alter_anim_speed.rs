use dat_tools::isoparser::*;
use dat_tools::dat::*;
use dat_tools::repr;
use dat_tools::*;

use std::rc::Rc;

use slp_parser::*;

fn main() {
    let character = Character::YoungLink;

    let changes: &[(usize, f32)] = &[(310, 0.625), (311, 0.625)];

    let iso = std::fs::OpenOptions::new()
        .read(true)
        .open("/home/alex/melee/melee_vanilla.iso")
        .unwrap();
    let mut files = ISODatFiles::new(iso).unwrap();

    let data_dat_raw = files.read_file(character_data_filename(character)).unwrap();
    let mut anim_dat_raw = files.read_file(character_animation_filename(character)).unwrap().data.to_vec();

    let data_dat = HSDRawFile::new(&data_dat_raw);
    let fighter_root = &data_dat.roots[0];
    let hsd_struct = &fighter_root.hsd_struct;

    let action_table_struct = hsd_struct.get_reference(0x0C);

    for (action_idx, frame_count_mul) in changes.iter().copied() {
        let action = action_table_struct.get_embedded_struct(action_idx * 0x18, 0x18);
        let offset = action.get_u32(0x04) as usize;
        let size = action.get_u32(0x08) as usize;

        let bump = bumpalo::Bump::new();
        let data = &mut anim_dat_raw[offset..offset+size];
        let anim_dat = repr::parse_dat_file(&data, &bump).unwrap();

        let frame_count_i = anim_dat.root_offsets[0] as usize + 0x08;
        let figa = repr::FigaTree { offset: anim_dat.root_offsets[0] };

        let new_f = (figa.frame_count(&data) * frame_count_mul).round();
        data[frame_count_i..frame_count_i+4].copy_from_slice(&new_f.to_be_bytes());
    }

    //let start = figa.track_count_buffer_offset(&data) as usize + 0x20;
    ////let track_count: usize = data[start..].iter().position(|&b| b == 0xFF).unwrap();
    //let track_count: usize = data[start..].iter()
    //    .map(|i| *i as usize)
    //    .take_while(|&b| b == 0xFF)
    //    .sum();

    //let track_buffer_offset = figa.track_buffer_offset(&data) as usize;
    //for i in 0..track_count {
    //    let track_start = track_buffer_offset + 0x0C * i;
    //    let track = &mut data[track_start..track_start+0x0C];

    //    let start_f = u16::from_be_bytes(track[2..4].try_into().unwrap());
    //    let new_start_f = (start_f as f32 * frame_count_mul).round() as u16;
    //    //track[2..4].copy_from_slice(&u16::to_be_bytes(new_start_f));
    //    track[2..4].copy_from_slice(&u16::to_be_bytes(0));
    //}

    let new_data: Rc<[u8]> = anim_dat_raw.into_boxed_slice().into();

    let iso = std::fs::OpenOptions::new()
        .read(true).write(true)
        .open("/Windows/Users/Alex/My Documents/Melee/balance/balance.iso")
        //.open("/home/alex/melee/tm_balance.iso")
        .unwrap();
    let mut files = ISODatFiles::new(iso).unwrap();
    files.write_file(character_animation_filename(character), new_data).unwrap();
}
