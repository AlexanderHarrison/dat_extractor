use dat_tools::isoparser::ISODatFiles;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    let mesh_dat = files.read_file("PlFxGr.dat").unwrap();
    let mesh_dat = dat_tools::dat::HSDRawFile::open(mesh_dat.stream());
    let scene = dat_tools::dat::extract_model(&mesh_dat).unwrap();
}
