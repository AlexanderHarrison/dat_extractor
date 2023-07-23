use dat_tools::isoparser::ISODatFiles;

fn main() {
    let file = std::fs::File::open("/home/alex/melee/melee_vanilla.iso").unwrap();
    let mut files = ISODatFiles::new(file).unwrap();
    let mesh_dat = files.read_file("PlFxGr.dat").unwrap();
    let mesh_dat = dat_tools::dat::HSDRawFile::open(mesh_dat.stream());
    let scene = dat_tools::dat::extract_scene(&mesh_dat).unwrap();
    
    for bone in scene.skeleton.root_bones {
        bone.inspect_each(&mut |bone| 
            if let Some(root_dobj) = bone.jobj.get_dobj() {
                for dobj in root_dobj.siblings() {
                    let vertices = dobj.decode_vertices();
                    for [x,y,z] in vertices {
                        println!("{}, {}, {}", x, y, z);
                    }
                }
            }
        )
    }
}
