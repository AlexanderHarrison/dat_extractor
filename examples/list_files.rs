use dat_tools::isoparser::ISODatFiles;

fn main() {
    let iso = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let files = ISODatFiles::new(iso).unwrap();

    for f in files.files.keys() {
        println!("{}", f);
    }
}
