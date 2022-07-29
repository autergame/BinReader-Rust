use lol_bin_struct::*;

use byteorder::{LittleEndian, ReadBytesExt};
use std::io::{Cursor, Read};

fn read_string(reader: &mut Cursor<&Vec<u8>>) -> String {
    let string_length = reader
        .read_u16::<LittleEndian>()
        .expect("Could not read string length");
    let mut string = vec![0u8; string_length as usize];
    reader
        .read_exact(&mut string)
        .expect("Could not read string");
    String::from_utf8(string).expect("Invalid UTF-8 sequence")
}

pub fn read_bin(contents: &Vec<u8>) -> BinFile {
    println!("Reading bin file");
    let mut reader = Cursor::new(contents);

    let mut is_patch = false;
    let mut unknown_data: Option<u64> = None;

    let mut signature: Vec<u8> = vec![0u8; 4];
    reader
        .read_exact(&mut signature)
        .expect("Could not read signature");
    if signature == b"PTCH" {
        unknown_data = Some(
            reader
                .read_u64::<LittleEndian>()
                .expect("Could not read unknown data"),
        );
        reader
            .read_exact(&mut signature)
            .expect("Could not read second signature");
        is_patch = true;
    }
    if signature != b"PROP" {
        panic!("Bin has no valid signature");
    }

    let version = reader
        .read_u32::<LittleEndian>()
        .expect("Could not read version");

    let mut linked_list: Vec<String> = Vec::new();

    if version >= 2 {
        let linked_list_count = reader
            .read_u32::<LittleEndian>()
            .expect("Could not read linked list count");
        for _ in 0..linked_list_count {
            linked_list.push(read_string(&mut reader));
        }
    }

    let entries_count = reader
        .read_u32::<LittleEndian>()
        .expect("Could not read entries count");

    let mut entry_types: Vec<u32> = Vec::with_capacity(entries_count as usize);
    for _ in 0..entries_count {
        entry_types.push(
            reader
                .read_u32::<LittleEndian>()
                .expect("Could not read entry type"),
        );
    }

    let mut entries_map = Map::new(
        BinType::Hash,
        BinType::Embedded,
        Vec::with_capacity(entries_count as usize),
    );

    for entry_type in entry_types {
        let entry_length = reader
            .read_u32::<LittleEndian>()
            .expect("Could not read entry length");

        let old_offset = reader.position();

        let entry_name = reader
            .read_u32::<LittleEndian>()
            .expect("Could not read entry name");
        let field_count = reader
            .read_u16::<LittleEndian>()
            .expect("Could not read entry field count");

        let mut embedded =
            PointerOrEmbedded::new(entry_type, Vec::with_capacity(field_count as usize));

        for _ in 0..field_count as usize {
            let name = reader
                .read_u32::<LittleEndian>()
                .expect("Could not read entry field name");
            let ftype = reader.read_u8().expect("Could not read entry field type");

            let field = BinField::new(
                name,
                u8_to_type(ftype).unwrap_or_else(|| panic!("Unknown entry field type {}", ftype)),
                read_value_by_type(&mut reader, ftype),
            );

            embedded.items.push(field);
        }

        entries_map.items.push(MapPair::new(
            BinData::Hash(entry_name),
            BinData::PointerOrEmbedded(embedded),
        ));

        let new_offset = reader.position();
        if old_offset + entry_length as u64 != new_offset {
            let diff_offset = new_offset - old_offset;
            panic!(
                "Wrong entry {} size from {} to {} in entries. Its {}, but should have been {}",
                entry_name, old_offset, new_offset, diff_offset, entry_length
            );
        }
    }

    let mut patches_map: Option<Map> = None;

    if is_patch {
        let patches_count = reader
            .read_u32::<LittleEndian>()
            .expect("Could not read patches count");

        patches_map = Some(Map::new(
            BinType::Hash,
            BinType::Embedded,
            Vec::with_capacity(patches_count as usize),
        ));

        for _ in 0..patches_count {
            let patch_name = reader
                .read_u32::<LittleEndian>()
                .expect("Could not read patch name");
            let patch_length = reader
                .read_u32::<LittleEndian>()
                .expect("Could not read patch length");

            let old_offset = reader.position();

            let ftype = reader.read_u8().expect("Could not read patch type");

            let string = BinData::String(read_string(&mut reader));

            let mut embedded = PointerOrEmbedded::new(
                0xF9100AA9, // patch FNV1a
                Vec::with_capacity(2),
            );

            let first_field = BinField::new(
                0x84874D36, // path FNV1a
                BinType::String,
                string,
            );
            embedded.items.push(first_field);

            let second_field = BinField::new(
                0x425ED3CA, // value FNV1a
                u8_to_type(ftype).unwrap_or_else(|| panic!("Unknown patch field type {}", ftype)),
                read_value_by_type(&mut reader, ftype),
            );
            embedded.items.push(second_field);

            patches_map.as_mut().unwrap().items.push(MapPair::new(
                BinData::Hash(patch_name),
                BinData::PointerOrEmbedded(embedded),
            ));

            let new_offset = reader.position();
            if old_offset + patch_length as u64 != new_offset {
                let diff_offset = new_offset - old_offset;
                panic!(
                    "Wrong patch {} size from {} to {} in patches. Its {}, but should have been {}",
                    patch_name, old_offset, new_offset, diff_offset, patch_length
                );
            }
        }
    }

    println!("Finished reading bin file");

    BinFile::new(
        is_patch,
        unknown_data,
        version,
        linked_list,
        entries_map,
        patches_map,
    )
}

fn read_value_by_type(reader: &mut Cursor<&Vec<u8>>, ftype: u8) -> BinData {
    let old_offset = reader.position();

    let bin_type = u8_to_type(ftype);
    if bin_type.is_none() {
        panic!("Unknown bin type {} starting at: {}.", ftype, old_offset);
    }
    let bin_type = bin_type.unwrap();

    match bin_type {
        BinType::None => BinData::None,
        BinType::Bool => BinData::Bool(reader.read_u8().expect("Could not read Bool") != 0),
        BinType::SInt8 => BinData::SInt8(reader.read_i8().expect("Could not read SInt8")),
        BinType::UInt8 => BinData::UInt8(reader.read_u8().expect("Could not read UInt8")),
        BinType::SInt16 => BinData::SInt16(
            reader
                .read_i16::<LittleEndian>()
                .expect("Could not read SInt16"),
        ),
        BinType::UInt16 => BinData::UInt16(
            reader
                .read_u16::<LittleEndian>()
                .expect("Could not read UInt16"),
        ),
        BinType::SInt32 => BinData::SInt32(
            reader
                .read_i32::<LittleEndian>()
                .expect("Could not read SInt32"),
        ),
        BinType::UInt32 => BinData::UInt32(
            reader
                .read_u32::<LittleEndian>()
                .expect("Could not read UInt32"),
        ),
        BinType::SInt64 => BinData::SInt64(
            reader
                .read_i64::<LittleEndian>()
                .expect("Could not read SInt64"),
        ),
        BinType::UInt64 => BinData::UInt64(
            reader
                .read_u64::<LittleEndian>()
                .expect("Could not read UInt64"),
        ),
        BinType::Float32 => BinData::Float32(
            reader
                .read_f32::<LittleEndian>()
                .expect("Could not read Float32"),
        ),
        BinType::Vector2 => {
            let mut vec2: Vec<f32> = Vec::with_capacity(2);
            for _ in 0..2 {
                vec2.push(
                    reader
                        .read_f32::<LittleEndian>()
                        .expect("Could not read Vector2"),
                );
            }
            BinData::Vector2(vec2)
        }
        BinType::Vector3 => {
            let mut vec3: Vec<f32> = Vec::with_capacity(3);
            for _ in 0..3 {
                vec3.push(
                    reader
                        .read_f32::<LittleEndian>()
                        .expect("Could not read Vector3"),
                );
            }
            BinData::Vector3(vec3)
        }
        BinType::Vector4 => {
            let mut vec4: Vec<f32> = Vec::with_capacity(4);
            for _ in 0..4 {
                vec4.push(
                    reader
                        .read_f32::<LittleEndian>()
                        .expect("Could not read Vector4"),
                );
            }
            BinData::Vector4(vec4)
        }
        BinType::Matrix4x4 => {
            let mut mtx44: Vec<f32> = Vec::with_capacity(16);
            for _ in 0..16 {
                mtx44.push(
                    reader
                        .read_f32::<LittleEndian>()
                        .expect("Could not read Matrix4x4"),
                );
            }
            BinData::Matrix4x4(mtx44)
        }
        BinType::Rgba => {
            let mut rgba: Vec<u8> = Vec::with_capacity(4);
            for _ in 0..4 {
                rgba.push(reader.read_u8().expect("Could not read Rgba"));
            }
            BinData::Rgba(rgba)
        }
        BinType::String => BinData::String(read_string(reader)),
        BinType::Hash => BinData::Hash(
            reader
                .read_u32::<LittleEndian>()
                .expect("Could not read Hash"),
        ),
        BinType::WadEntryLink => BinData::WadEntryLink(
            reader
                .read_u64::<LittleEndian>()
                .expect("Could not read WadEntryLink"),
        ),
        BinType::Container | BinType::Struct => {
            let cs_type = reader
                .read_u8()
                .unwrap_or_else(|_| panic!("Could not read {:?} type", bin_type));
            let cs_length = reader
                .read_u32::<LittleEndian>()
                .unwrap_or_else(|_| panic!("Could not read {:?} length", bin_type));

            let old_offset = reader.position();

            let field_count = reader
                .read_u32::<LittleEndian>()
                .unwrap_or_else(|_| panic!("Could not read {:?} field count", bin_type));

            let mut cs = ContainerOrStruct::new(
                u8_to_type(cs_type)
                    .unwrap_or_else(|| panic!("Unknown {:?} field type {}", bin_type, cs_type)),
                Vec::with_capacity(field_count as usize),
            );

            for _ in 0..field_count {
                cs.items.push(read_value_by_type(reader, cs_type));
            }

            let new_offset = reader.position();
            if old_offset + cs_length as u64 != new_offset {
                let diff_offset = new_offset - old_offset;
                panic!(
                    "Wrong {:?} size from {} to {}. Its {}, but should have been {}",
                    bin_type, old_offset, new_offset, diff_offset, cs_length
                );
            }

            BinData::ContainerOrStruct(cs)
        }
        BinType::Pointer | BinType::Embedded => {
            let name = reader
                .read_u32::<LittleEndian>()
                .unwrap_or_else(|_| panic!("Could not read {:?} name", bin_type));
            if name == 0 {
                return BinData::PointerOrEmbedded(PointerOrEmbedded::new(0, Vec::new()));
            }

            let pe_length = reader
                .read_u32::<LittleEndian>()
                .unwrap_or_else(|_| panic!("Could not read {:?} length", bin_type));

            let old_offset = reader.position();

            let field_count = reader
                .read_u16::<LittleEndian>()
                .unwrap_or_else(|_| panic!("Could not read {:?} field count", bin_type));

            let mut pe = PointerOrEmbedded::new(name, Vec::with_capacity(field_count as usize));

            for _ in 0..field_count {
                let fname = reader
                    .read_u32::<LittleEndian>()
                    .unwrap_or_else(|_| panic!("Could not read {:?} field name", bin_type));
                let ftype = reader
                    .read_u8()
                    .unwrap_or_else(|_| panic!("Could not read {:?} field type", bin_type));

                let field = BinField::new(
                    fname,
                    u8_to_type(ftype)
                        .unwrap_or_else(|| panic!("Unknown {:?} field type {}", bin_type, ftype)),
                    read_value_by_type(reader, ftype),
                );

                pe.items.push(field);
            }

            let new_offset = reader.position();
            if old_offset + pe_length as u64 != new_offset {
                let diff_offset = new_offset - old_offset;
                panic!(
                    "Wrong {:?} size from {} to {}. Its {}, but should have been {}",
                    bin_type, old_offset, new_offset, diff_offset, pe_length
                );
            }

            BinData::PointerOrEmbedded(pe)
        }
        BinType::Link => BinData::Link(
            reader
                .read_u32::<LittleEndian>()
                .expect("Could not read Link"),
        ),
        BinType::Optional => {
            let op_type = reader.read_u8().expect("Could not read Optional type");
            let is_some = reader.read_u8().expect("Could not read Optional some");

            let data = if is_some != 0 {
                Some(read_value_by_type(reader, op_type))
            } else {
                None
            };

            let option = Optional::new(
                u8_to_type(op_type).unwrap_or_else(|| panic!("Unknown Option type {}", op_type)),
                data,
            );

            BinData::Optional(option)
        }
        BinType::Map => {
            let key_type = reader.read_u8().expect("Could not read Map key type");
            let value_type = reader.read_u8().expect("Could not read Map value type");
            let map_length = reader
                .read_u32::<LittleEndian>()
                .expect("Could not read Map length");

            let old_offset = reader.position();

            let field_count = reader
                .read_u32::<LittleEndian>()
                .expect("Could not read Map field count");

            let mut map = Map::new(
                u8_to_type(key_type).unwrap_or_else(|| panic!("Unknown Map key type {}", key_type)),
                u8_to_type(value_type)
                    .unwrap_or_else(|| panic!("Unknown Map value type {}", value_type)),
                Vec::with_capacity(field_count as usize),
            );

            for _ in 0..field_count {
                map.items.push(MapPair::new(
                    read_value_by_type(reader, key_type),
                    read_value_by_type(reader, value_type),
                ));
            }

            let new_offset = reader.position();
            if old_offset + map_length as u64 != new_offset {
                let diff_offset = new_offset - old_offset;
                panic!(
                    "Wrong Map size from {} to {}. Its {}, but should have been {}",
                    old_offset, new_offset, diff_offset, map_length
                );
            }

            BinData::Map(map)
        }
        BinType::Flag => BinData::Flag(reader.read_u8().expect("Could not read flag") != 0),
    }
}

fn u8_to_type(ftype: u8) -> Option<BinType> {
    let mut unpacked_type = ftype;
    if ftype >= 0x80 {
        unpacked_type = (ftype - 0x80) + BinType::Container as u8;
    }
    match unpacked_type {
        0 => Some(BinType::None),
        1 => Some(BinType::Bool),
        2 => Some(BinType::SInt8),
        3 => Some(BinType::UInt8),
        4 => Some(BinType::SInt16),
        5 => Some(BinType::UInt16),
        6 => Some(BinType::SInt32),
        7 => Some(BinType::UInt32),
        8 => Some(BinType::SInt64),
        9 => Some(BinType::UInt64),
        10 => Some(BinType::Float32),
        11 => Some(BinType::Vector2),
        12 => Some(BinType::Vector3),
        13 => Some(BinType::Vector4),
        14 => Some(BinType::Matrix4x4),
        15 => Some(BinType::Rgba),
        16 => Some(BinType::String),
        17 => Some(BinType::Hash),
        18 => Some(BinType::WadEntryLink),
        19 => Some(BinType::Container),
        20 => Some(BinType::Struct),
        21 => Some(BinType::Pointer),
        22 => Some(BinType::Embedded),
        23 => Some(BinType::Link),
        24 => Some(BinType::Optional),
        25 => Some(BinType::Map),
        26 => Some(BinType::Flag),
        _ => None,
    }
}
