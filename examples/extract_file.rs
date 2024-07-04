use dat_tools::isoparser::ISODatFiles;

fn main() {
    //let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let file = std::fs::File::open("C:\\Users\\Alex\\Documents\\Melee\\melee vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    files.extract_file("PlFxAJ.dat", std::path::Path::new("PlFxAJ.dat")).unwrap();
}
