use dat_tools::isoparser::ISODatFiles;

fn main() {
    //let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    files.extract_file("GrZe.dat", std::path::Path::new("GrZe.dat")).unwrap();
}
