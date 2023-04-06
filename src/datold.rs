#![allow(non_upper_case_globals, non_snake_case, non_camel_case_types, dead_code, unused_doc_comments)]
use std::collections::HashMap;
use std::any::Any;

use utils::Stream;

pub struct Shader;
pub struct Bitmap;
pub struct JOBJ;
pub struct VBN;
pub struct TreeNode;
pub struct Point;
pub struct COLL_DATA;
pub struct Bounds;

const MaxWeightCount: i32 = 4;
const headerSize: i32 = 0x20;

pub struct DAT {
    header: Header,
    pub filename: String,

    pub tree: Vec<TreeNode>,
    pub displayList: Vec<TreeNode>,
    pub collisions: COLL_DATA,
    pub spawns: Vec<Point>,
    pub spawnOffs: Vec<i32>,
    pub respawns: Vec<Point>,
    pub respawnOffs: Vec<i32>,
    pub itemSpawns: Vec<Point>,
    pub itemSpawnOffs: Vec<i32>,
    pub targets: Vec<Point>,
    pub targetOffs: Vec<i32>,
    pub cameraBounds: Bounds,
    pub cameraBoundOffs: Vec<i32>,
    pub blastzones: Bounds,
    pub blastzoneOffs: Vec<i32>,

    //pub headNode: Map_Head.Head_Node,

    pub stageScale: f32,

    pub bones: VBN,

    /// unneeded, gl not used
    // gl buffer objects
    //ubo_bones: i32,
    //vbo_position: i32,
    //vbo_color:    i32,
    //vbo_nrm:      i32,
    //vbo_uv:       i32,
    //vbo_weight:   i32,
    //vbo_bone:     i32,
    //ibo_elements: i32,

    pub testtex: i32,

    pub shader: &'static Shader,

    jobjOffsetLinker: HashMap<i32, JOBJ>,
    pub texturesLinker: HashMap<i32, Bitmap>,
    pub tobjLinker: HashMap<i32, Vec<Box<dyn Any>>>,
}

impl DAT {
    pub fn Read(&mut self, d: &mut FileData) {
        let header = Header::Read(d);
        let dataBlockOffset = d.Pos();
        d.Skip(header.dataBlockSize); // skip to relocation table

        let relocationTableOffset = d.Pos();

        // update relocation table and data offset
        for i in 0..header.relocationTableCount {
           let relocationOffset = relocationTableOffset + i * 4;

           d.Seek(relocationOffset);

           let dataOffset = d.ReadInt() + headerSize;

           d.WriteInt(relocationOffset, dataOffset);

           d.Seek(dataOffset);

           d.WriteInt(dataOffset, d.ReadInt() + headerSize);
        }

        d.Seek(relocationTableOffset + header.relocationTableCount * 4); // skip relocation table

        let strOffset = d.Pos() + header.rootCount * 8 + header.referenceNodeCount * 8;
        let sectionOffset = Vec::with_capacity(header.rootCount as usize);
        let sectionNames = Vec::with_capacity(header.rootCount as usize);
        

        //Console.WriteLine(d.Pos().ToString("x") + " " + strOffset.ToString("x"));
    
        for i in 0..header.rootCount {
            // data then string
            let data = d.ReadInt() + headerSize;
            let s = d.ReadString(d.ReadInt() + strOffset, -1);

            sectionOffset.push(data);
            sectionNames.push(s);
            //Console.WriteLine(s + " " + data.ToString("x"));

            TreeNode node = new TreeNode();
            node.Text = s;
            node.Tag = data;
            tree.Add(node);
        }
        Console.WriteLine(d.Pos().ToString("x") + " " + strOffset.ToString("x"));

        //foreach (TreeNode node in tree)
        //{
        //    // then a file system is read... it works like a tree?
        //    d.Seek((int)node.Tag);
        //    // now, the name determines what happens here
        //    // for now, it just assumes the _joint
        //    if (node.Text.EndsWith("_joint") && !node.Text.Contains("matanim") && !node.Text.Contains("anim_joint"))
        //    {
        //        JOBJ j = new JOBJ();
        //        j.Read(d, this, node);
        //        //break;
        //    }
        //    else if (node.Text.EndsWith("grGroundParam"))
        //    {
        //        stageScale = d.ReadFloat();
        //        Console.WriteLine($"Stage scale - {stageScale}");
        //    }
        //    else if (node.Text.EndsWith("map_head"))
        //    {
        //        Map_Head head = new Map_Head();
        //        head.Read(d, this, node);
        //    }
        //    else if (node.Text.EndsWith("coll_data"))
        //    {
        //        collisions = new COLL_DATA();
        //        collisions.Read(d);
        //    }
        //}

        //Console.WriteLine("Done");
        ////ExportTextures("",0);

        //// now to fix single binds
        //List<JOBJ> boneTrack = GetBoneOrder();
        //Matrix4 mt = new Matrix4();
        //int w = 0;
        //foreach (Vertex v in vertBank)
        //{
        //    w = 0;
        //    v.bones.Clear();
        //    mt = new Matrix4();

        //    foreach (object o in v.Tags)
        //    {
        //        if (o is JOBJ)
        //        {
        //            v.bones.Add(boneTrack.IndexOf((JOBJ)o));
        //            mt = Matrix4.CreateScale(1, 1, 1);
        //            v.nrm = TransformNormal(((JOBJ)o).transform, v.nrm);
        //        }
        //        else
        //        if (o is int)
        //        {
        //            v.bones.Add(boneTrack.IndexOf(jobjOffsetLinker[(int)o]));
        //            mt += jobjOffsetLinker[(int)o].transform * v.weights[w++];
        //        }
        //    }

        //    if (v.bones.Count == 1)
        //    {
        //        v.pos = Vector3.TransformPosition(v.pos, mt);
        //        v.nrm = Vector3.TransformNormal(v.nrm, mt);
        //    }
        //    // scale it
        //    v.pos = Vector3.Multiply(v.pos, stageScale);
        //}

        //PreRender();
    }
}


pub struct Header {
    pub fileSize: i32,
    pub dataBlockSize: i32,
    pub relocationTableCount: i32,
    pub rootCount: i32,
    pub referenceNodeCount: i32,
    pub unk1: i32,
    pub unk2: i32,
    pub unk3: i32,
}

impl Header {
    pub fn Read(d: &mut FileData) -> Header {
        let fileSize             = d.ReadInt();
        let dataBlockSize        = d.ReadInt();
        let relocationTableCount = d.ReadInt();
        let rootCount            = d.ReadInt();
        let referenceNodeCount   = d.ReadInt();
        let unk1                 = d.ReadInt();
        let unk2                 = d.ReadInt();
        let unk3                 = d.ReadInt();

        Header {
            fileSize,            
            dataBlockSize,       
            relocationTableCount,
            rootCount,           
            referenceNodeCount,  
            unk1,                
            unk2,                
            unk3,       
        }
    }
}

pub struct FileData {
    pub data: Vec<u8>,
    pub cursor: usize,
}

impl FileData {
    pub fn ReadInt(&mut self) -> i32 {
        let ret = Stream::new(&self.data[self.cursor..]).take_u32().unwrap() as i32;
        self.cursor += 4;
        ret
    }

    pub fn Pos(&self) -> i32 {
        self.cursor as _
    }

    pub fn Skip(&self, n: i32) {
        self.cursor += n as usize;
    }

    pub fn Seek(&mut self, pos: i32) {
        self.cursor = pos as usize;
    }

    pub fn WriteInt(&mut self, pos: i32, int: i32) {
        let bytes = int.to_be_bytes();
        let pos = pos as usize;
        self.data[pos+0] = bytes[0];
        self.data[pos+1] = bytes[1];
        self.data[pos+2] = bytes[2];
        self.data[pos+3] = bytes[3];
    }

    pub fn ReadString(&mut self, pos: i32, size: i32) -> String {
        let mut s = String::new();
        let mut pos = pos as usize;

        if size == -1 {
            while (pos as usize) < self.data.len() {
                let c = self.data[pos];
                if c != 0x00 {
                    s.push(c as char);
                } else {
                    break;
                }
                pos += 1;
            }
        } else {
            for p in 0..(size as usize) {
                s.push(self.data[pos+p] as char)
            }
        }

        s
    }
}
