use dat_tools::isoparser::ISODatFiles;

fn main() {
    let iso = match std::env::args().nth(1) {
        Some(path) => path,
        None => {
            eprintln!("no iso path passed");
            return;
        }
    };
    let dst = match std::env::args().nth(2) {
        Some(path) => path,
        None => {
            eprintln!("no destination dat path passed");
            return;
        }
    };
    let src = match std::env::args().nth(3) {
        Some(path) => path,
        None => {
            eprintln!("no source dat path passed");
            return;
        }
    };

    let file = std::fs::File::options().read(true).write(true).open(iso).unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    let bytes = std::fs::read(src).unwrap().into();
    files.write_file(&dst, bytes).unwrap();
}
