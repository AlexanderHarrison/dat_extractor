use dat_tools::dat::*;
use dat_tools::isoparser::*;

fn main() {
    let file = std::fs::File::open("C:\\Users\\Alex\\Documents\\Melee\\melee vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();

    let anim_idx: usize = 38;
    //let anim_idx: usize = 311;
    let anim_name = "PlFxGuard.dat";

    let anim_dat = files.read_file("PlFxAJ.dat").unwrap();
    let fighter_dat = files.read_file("PlFx.dat").unwrap();
    let fighter_hsd = HSDRawFile::new(&fighter_dat);

    let fighter_root = &fighter_hsd.roots[0];
    let action_table_struct = fighter_root.hsd_struct.get_reference(0x0C);
    let s = action_table_struct.get_embedded_struct(anim_idx * 0x18, 0x18);

    let offset = s.get_u32(0x04) as usize;
    let size = s.get_u32(0x08) as usize;
    let anim_data = &anim_dat.data[offset..offset+size];

    std::fs::write(
        anim_name,
        anim_data,
    ).unwrap();
}
