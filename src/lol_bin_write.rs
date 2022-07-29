use lol_bin_struct::*;

use byteorder::{LittleEndian, WriteBytesExt};
use std::io::Write;

fn write_string(writer: &mut Vec<u8>, string: &str) {
    writer
        .write_u16::<LittleEndian>(string.len() as u16)
        .expect("Could not write string length");
    writer
        .write_all(string.as_bytes())
        .expect("Could not write string");
}

pub fn write_bin(bin_file: &BinFile) -> Vec<u8> {
    println!("Writing bin file");

    let mut writer: Vec<u8> = Vec::new();

    if bin_file.is_patch {
        writer
            .write_all("PTCH".as_bytes())
            .expect("Could not write PTCH");
        if let Some(unknown) = &bin_file.unknown {
            writer
                .write_u64::<LittleEndian>(*unknown)
                .expect("Could not write unknown data");
        }
    }

    writer
        .write_all("PROP".as_bytes())
        .expect("Could not write PROP");

    writer
        .write_u32::<LittleEndian>(bin_file.version)
        .expect("Could not write version");

    if bin_file.version >= 2 {
        writer
            .write_u32::<LittleEndian>(bin_file.linked_list.len() as u32)
            .expect("Could not write linked list count");
        for linked in &bin_file.linked_list {
            write_string(&mut writer, linked);
        }
    }

    writer
        .write_u32::<LittleEndian>(bin_file.entries.items.len() as u32)
        .expect("Could not write entries count");

    for entry in &bin_file.entries.items {
        if let BinData::PointerOrEmbedded(pe) = &*entry.valuedata {
            writer
                .write_u32::<LittleEndian>(pe.name)
                .expect("Could not write entry type");
        } else {
            panic!("Expected Pointer or Embedded in entry valuedata");
        }
    }

    for entry in &bin_file.entries.items {
        let entry_name = if let BinData::Hash(hash) = &*entry.keydata {
            *hash
        } else {
            panic!("Expected Hash in entry keydata");
        };
        if let BinData::PointerOrEmbedded(pe) = &*entry.valuedata {
            let field_count = pe.items.len() as u16;

            let mut entry_length = 4 + 2;
            for field in &pe.items {
                entry_length += get_total_bin_data_size(&field.data) + 4 + 1;
            }

            writer
                .write_u32::<LittleEndian>(entry_length)
                .expect("Could not write entry length");
            writer
                .write_u32::<LittleEndian>(entry_name)
                .expect("Could not write entry name");
            writer
                .write_u16::<LittleEndian>(field_count)
                .expect("Could not write entry field count");

            for field in &pe.items {
                let ftype = type_to_u8(&field.btype);

                writer
                    .write_u32::<LittleEndian>(field.name)
                    .expect("Could not write entry field name");
                writer
                    .write_u8(ftype)
                    .expect("Could not write entry field type");

                write_value_by_bin_data(&mut writer, &field.data, &field.btype);
            }
        } else {
            panic!("Expected Pointer or Embedded in entry valuedata");
        }
    }

    if bin_file.is_patch {
        if let Some(patches) = &bin_file.patches {
            writer
                .write_u32::<LittleEndian>(patches.items.len() as u32)
                .expect("Could not write patches count");

            for patch in &patches.items {
                let patch_name = if let BinData::Hash(hash) = &*patch.keydata {
                    *hash
                } else {
                    panic!("Expected Hash in patch keydata");
                };
                if let BinData::PointerOrEmbedded(pe) = &*patch.valuedata {
                    let first_field = &pe.items[0].data;
                    let second_field = &pe.items[1].data;

                    let mut patch_length = 1;
                    patch_length += get_total_bin_data_size(first_field);
                    patch_length += get_total_bin_data_size(second_field);

                    writer
                        .write_u32::<LittleEndian>(patch_name)
                        .expect("Could not write patch name");
                    writer
                        .write_u32::<LittleEndian>(patch_length)
                        .expect("Could not write patch length");

                    let ftype = type_to_u8(&pe.items[1].btype);
                    writer
                        .write_u8(ftype)
                        .expect("Could not write patch field type");

                    write_value_by_bin_data(&mut writer, first_field, &pe.items[0].btype);
                    write_value_by_bin_data(&mut writer, second_field, &pe.items[1].btype);
                }
            }
        } else {
            panic!("Expected patches in patch file");
        }
    }

    println!("Finished writing bin file");

    writer
}

fn write_value_by_bin_data(writer: &mut Vec<u8>, bin_data: &BinData, bin_type: &BinType) {
    match bin_data {
        BinData::None => {}
        BinData::Bool(bool) => {
            writer.write_u8(*bool as u8).expect("Could not write Bool");
        }
        BinData::SInt8(i8) => {
            writer.write_i8(*i8).expect("Could not write SInt8");
        }
        BinData::UInt8(u8) => {
            writer.write_u8(*u8).expect("Could not write Uint8");
        }
        BinData::SInt16(i16) => {
            writer
                .write_i16::<LittleEndian>(*i16)
                .expect("Could not write SInt16");
        }
        BinData::UInt16(u16) => {
            writer
                .write_u16::<LittleEndian>(*u16)
                .expect("Could not write UInt16");
        }
        BinData::SInt32(i32) => {
            writer
                .write_i32::<LittleEndian>(*i32)
                .expect("Could not write SInt32");
        }
        BinData::UInt32(u32) => {
            writer
                .write_u32::<LittleEndian>(*u32)
                .expect("Could not write UInt32");
        }
        BinData::SInt64(i64) => {
            writer
                .write_i64::<LittleEndian>(*i64)
                .expect("Could not write SInt64");
        }
        BinData::UInt64(u64) => {
            writer
                .write_u64::<LittleEndian>(*u64)
                .expect("Could not write UInt64");
        }
        BinData::Float32(f32) => {
            writer
                .write_f32::<LittleEndian>(*f32)
                .expect("Could not write Float32");
        }
        BinData::Vector2(vec2) => {
            assert_eq!(vec2.len(), 2);
            for value in vec2 {
                writer
                    .write_f32::<LittleEndian>(*value)
                    .expect("Could not write Vector2");
            }
        }
        BinData::Vector3(vec3) => {
            assert_eq!(vec3.len(), 3);
            for value in vec3 {
                writer
                    .write_f32::<LittleEndian>(*value)
                    .expect("Could not write Vector3");
            }
        }
        BinData::Vector4(vec4) => {
            assert_eq!(vec4.len(), 4);
            for value in vec4 {
                writer
                    .write_f32::<LittleEndian>(*value)
                    .expect("Could not write Vector4");
            }
        }
        BinData::Matrix4x4(mtx44) => {
            assert_eq!(mtx44.len(), 16);
            for value in mtx44 {
                writer
                    .write_f32::<LittleEndian>(*value)
                    .expect("Could not write Matrix4x4");
            }
        }
        BinData::Rgba(rgba) => {
            assert_eq!(rgba.len(), 4);
            for value in rgba {
                writer.write_u8(*value).expect("Could not write Rgba");
            }
        }
        BinData::String(string) => {
            write_string(writer, string);
        }
        BinData::Hash(hash) => {
            writer
                .write_u32::<LittleEndian>(*hash)
                .expect("Could not write Hash");
        }
        BinData::WadEntryLink(wadentrylink) => {
            writer
                .write_u64::<LittleEndian>(*wadentrylink)
                .expect("Could not write WadEntryLink");
        }
        BinData::ContainerOrStruct(cs) => {
            let mut length: u32 = 4;
            let field_count = cs.items.len() as u32;
            let ftype = type_to_u8(&cs.btype);

            for field in &cs.items {
                length += get_total_bin_data_size(field);
            }

            writer
                .write_u8(ftype)
                .unwrap_or_else(|_| panic!("Could not write {:?} type", bin_type));
            writer
                .write_u32::<LittleEndian>(length)
                .unwrap_or_else(|_| panic!("Could not write {:?} length", bin_type));
            writer
                .write_u32::<LittleEndian>(field_count)
                .unwrap_or_else(|_| panic!("Could not write {:?} field count", bin_type));

            for field in &cs.items {
                write_value_by_bin_data(writer, field, &cs.btype);
            }
        }
        BinData::PointerOrEmbedded(pe) => {
            writer
                .write_u32::<LittleEndian>(pe.name)
                .unwrap_or_else(|_| panic!("Could not write {:?} name", bin_type));
            if pe.name == 0 {
                return;
            }

            let mut length: u32 = 2;
            let field_count = pe.items.len() as u16;

            for field in &pe.items {
                length += get_total_bin_data_size(&field.data) + 4 + 1;
            }

            writer
                .write_u32::<LittleEndian>(length)
                .unwrap_or_else(|_| panic!("Could not write {:?} length", bin_type));
            writer
                .write_u16::<LittleEndian>(field_count)
                .unwrap_or_else(|_| panic!("Could not write {:?} field count", bin_type));

            for field in &pe.items {
                let ftype = type_to_u8(&field.btype);

                writer
                    .write_u32::<LittleEndian>(field.name)
                    .unwrap_or_else(|_| panic!("Could not write {:?} field name", bin_type));
                writer
                    .write_u8(ftype)
                    .unwrap_or_else(|_| panic!("Could not write {:?} field type", bin_type));

                write_value_by_bin_data(writer, &field.data, &field.btype);
            }
        }
        BinData::Link(link) => {
            writer
                .write_u32::<LittleEndian>(*link)
                .expect("Could not write link");
        }
        BinData::Optional(option) => {
            let ftype = type_to_u8(&option.btype);

            writer
                .write_u8(ftype)
                .expect("Could not write Optional type");

            if let Some(data) = &option.data {
                writer.write_u8(1).expect("Could not write Optional data");

                write_value_by_bin_data(writer, data, &option.btype);
            } else {
                writer.write_u8(0).expect("Could not write Optional data");
            }
        }
        BinData::Map(map) => {
            let mut length: u32 = 4;
            let field_count = map.items.len() as u32;

            for mappair in &map.items {
                length += get_total_bin_data_size(&mappair.keydata)
                    + get_total_bin_data_size(&mappair.valuedata);
            }

            let fkeytype = type_to_u8(&map.keytype);
            let fvaluetype = type_to_u8(&map.valuetype);

            writer
                .write_u8(fkeytype)
                .expect("Could not write Map key type");
            writer
                .write_u8(fvaluetype)
                .expect("Could not write Map value type");

            writer
                .write_u32::<LittleEndian>(length)
                .expect("Could not write Map length");
            writer
                .write_u32::<LittleEndian>(field_count)
                .expect("Could not write Map field count");

            for mappair in &map.items {
                write_value_by_bin_data(writer, &mappair.keydata, &map.keytype);
                write_value_by_bin_data(writer, &mappair.valuedata, &map.valuetype);
            }
        }
        BinData::Flag(flag) => {
            writer.write_u8(*flag as u8).expect("Could not write Flag");
        }
    }
}

fn get_total_bin_data_size(bin_data: &BinData) -> u32 {
    match bin_data {
        BinData::None => 0,
        BinData::Bool(_) => 1,
        BinData::SInt8(_) => 1,
        BinData::UInt8(_) => 1,
        BinData::SInt16(_) => 2,
        BinData::UInt16(_) => 2,
        BinData::SInt32(_) => 4,
        BinData::UInt32(_) => 4,
        BinData::SInt64(_) => 8,
        BinData::UInt64(_) => 8,
        BinData::Float32(_) => 4,
        BinData::Vector2(_) => 8,
        BinData::Vector3(_) => 12,
        BinData::Vector4(_) => 16,
        BinData::Matrix4x4(_) => 64,
        BinData::Rgba(_) => 4,
        BinData::String(string) => 2 + string.len() as u32,
        BinData::Hash(_) => 4,
        BinData::WadEntryLink(_) => 8,
        BinData::ContainerOrStruct(cs) => {
            let mut size: u32 = 1 + 4 + 4;
            for field in &cs.items {
                size += get_total_bin_data_size(field);
            }
            size
        }
        BinData::PointerOrEmbedded(pe) => {
            let mut size: u32 = 4;
            if pe.name != 0 {
                size += 4 + 2;
                for field in &pe.items {
                    size += get_total_bin_data_size(&field.data) + 4 + 1;
                }
            }
            size
        }
        BinData::Link(_) => 4,
        BinData::Optional(option) => {
            let mut size: u32 = 2;
            if let Some(data) = &option.data {
                size += get_total_bin_data_size(data);
            }
            size
        }
        BinData::Map(map) => {
            let mut size: u32 = 1 + 1 + 4 + 4;
            for mappair in &map.items {
                size += get_total_bin_data_size(&mappair.keydata)
                    + get_total_bin_data_size(&mappair.valuedata);
            }
            size
        }
        BinData::Flag(_) => 1,
    }
}

fn type_to_u8(ftype: &BinType) -> u8 {
    let mut unpacked_type: u8 = match ftype {
        BinType::None => 0,
        BinType::Bool => 1,
        BinType::SInt8 => 2,
        BinType::UInt8 => 3,
        BinType::SInt16 => 4,
        BinType::UInt16 => 5,
        BinType::SInt32 => 6,
        BinType::UInt32 => 7,
        BinType::SInt64 => 8,
        BinType::UInt64 => 9,
        BinType::Float32 => 10,
        BinType::Vector2 => 11,
        BinType::Vector3 => 12,
        BinType::Vector4 => 13,
        BinType::Matrix4x4 => 14,
        BinType::Rgba => 15,
        BinType::String => 16,
        BinType::Hash => 17,
        BinType::WadEntryLink => 18,
        BinType::Container => 19,
        BinType::Struct => 20,
        BinType::Pointer => 21,
        BinType::Embedded => 22,
        BinType::Link => 23,
        BinType::Optional => 24,
        BinType::Map => 25,
        BinType::Flag => 26,
    };
    if unpacked_type >= BinType::Container as u8 {
        unpacked_type = (unpacked_type - BinType::Container as u8) + 0x80;
    }
    unpacked_type
}
