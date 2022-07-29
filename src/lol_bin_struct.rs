#[derive(Debug)]
pub struct BinFile {
    pub is_patch: bool,
    pub unknown: Option<u64>,
    pub version: u32,
    pub linked_list: Vec<String>,
    pub entries: Map,
    pub patches: Option<Map>,
}

#[derive(Debug)]
#[repr(u8)]
pub enum BinType {
    None = 0,
    Bool = 1,
    SInt8 = 2,
    UInt8 = 3,
    SInt16 = 4,
    UInt16 = 5,
    SInt32 = 6,
    UInt32 = 7,
    SInt64 = 8,
    UInt64 = 9,
    Float32 = 10,
    Vector2 = 11,
    Vector3 = 12,
    Vector4 = 13,
    Matrix4x4 = 14,
    Rgba = 15,
    String = 16,
    Hash = 17,
    WadEntryLink = 18,
    Container = 19, // 0x80
    Struct = 20,    // 0x81
    Pointer = 21,   // 0x82
    Embedded = 22,  // 0x83
    Link = 23,      // 0x84
    Optional = 24,  // 0x85
    Map = 25,       // 0x86
    Flag = 26,      // 0x87
}

#[derive(Debug)]
pub enum BinData {
    None,
    Bool(bool),
    SInt8(i8),
    UInt8(u8),
    SInt16(i16),
    UInt16(u16),
    SInt32(i32),
    UInt32(u32),
    SInt64(i64),
    UInt64(u64),
    Float32(f32),
    Vector2(Vec<f32>),
    Vector3(Vec<f32>),
    Vector4(Vec<f32>),
    Matrix4x4(Vec<f32>),
    Rgba(Vec<u8>),
    String(String),
    Hash(u32),
    WadEntryLink(u64),
    ContainerOrStruct(ContainerOrStruct),
    PointerOrEmbedded(PointerOrEmbedded),
    Optional(Optional),
    Link(u32),
    Map(Map),
    Flag(bool),
}

#[derive(Debug)]
pub struct BinField {
    pub name: u32,
    pub btype: BinType,
    pub data: Box<BinData>,
}

#[derive(Debug)]
pub struct ContainerOrStruct {
    pub btype: BinType,
    pub items: Vec<BinData>,
}

#[derive(Debug)]
pub struct PointerOrEmbedded {
    pub name: u32,
    pub items: Vec<BinField>,
}

#[derive(Debug)]
pub struct Optional {
    pub btype: BinType,
    pub data: Option<Box<BinData>>,
}

#[derive(Debug)]
pub struct MapPair {
    pub keydata: Box<BinData>,
    pub valuedata: Box<BinData>,
}

#[derive(Debug)]
pub struct Map {
    pub keytype: BinType,
    pub valuetype: BinType,
    pub items: Vec<MapPair>,
}

impl BinFile {
    pub fn new(
        is_patch: bool,
        unknown: Option<u64>,
        version: u32,
        linked_list: Vec<String>,
        entries: Map,
        patches: Option<Map>,
    ) -> BinFile {
        BinFile {
            is_patch,
            unknown,
            version,
            linked_list,
            entries,
            patches,
        }
    }
}

impl BinField {
    pub fn new(name: u32, btype: BinType, data: BinData) -> BinField {
        BinField {
            name,
            btype,
            data: Box::new(data),
        }
    }
}

impl ContainerOrStruct {
    pub fn new(btype: BinType, items: Vec<BinData>) -> ContainerOrStruct {
        ContainerOrStruct { btype, items }
    }
}

impl PointerOrEmbedded {
    pub fn new(name: u32, items: Vec<BinField>) -> PointerOrEmbedded {
        PointerOrEmbedded { name, items }
    }
}

impl Optional {
    pub fn new(btype: BinType, data: Option<BinData>) -> Optional {
        Optional {
            btype,
            data: data.map(Box::new),
        }
    }
}

impl MapPair {
    pub fn new(keydata: BinData, valuedata: BinData) -> MapPair {
        MapPair {
            keydata: Box::new(keydata),
            valuedata: Box::new(valuedata),
        }
    }
}

impl Map {
    pub fn new(keytype: BinType, valuetype: BinType, items: Vec<MapPair>) -> Map {
        Map {
            keytype,
            valuetype,
            items,
        }
    }
}
