use dat_tools::{dat::Article, isoparser::ISODatFiles};
use slp_parser::Character;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();

    let data_filename = dat_tools::character_data_filename(Character::Peach);
    let data_dat = files.read_file(data_filename).unwrap();
    let parsed_data_dat = dat_tools::dat::HSDRawFile::new(&data_dat);
    let data_root = dat_tools::dat::FighterDataRoot::new(parsed_data_dat.roots[0].hsd_struct.clone());

    let article_ptrs = data_root.hsd_struct.try_get_reference(0x48).unwrap(); 
    let count = article_ptrs.len() / 4;

    let article = article_ptrs.try_get_reference(4).unwrap();

    let ext = article.get_reference(0x04);

    println!("{}", ext.len());

    for i in (0..ext.len()).step_by(4) {
        println!("{i}:\t{}", ext.get_u32(i));
    }
    println!("{:?}", ext.get_references());
}
