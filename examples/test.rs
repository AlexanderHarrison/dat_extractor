use dat_tools::isoparser::ISODatFiles;
use dat_tools::dat::{Stream, HSDRawFile};

const F: &'static [u8] = include_bytes!("../PlFxAJ.dat");

fn main() {
    //let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    //let mut files = ISODatFiles::new(file).unwrap();
    //let base_dat = files.read_file("PlFx.dat").unwrap();
    //let hsd_raw = HSDRawFile::open(Stream::new(&base_dat.data));
    let hsd_raw = HSDRawFile::open(Stream::new(F));
    for root in hsd_raw.roots {
        println!("{}", root.root_string);
    }
}
