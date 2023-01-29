use hashes;
use structs::*;

use json::JsonValue;

fn string_to_hash_u32(value: &str) -> u32 {
    if let Some(hex) = hex_or_decimal_from_string_u32(value) {
        hex
    } else {
        hashes::fnv1a(value)
    }
}

fn string_to_hash_u64(value: &str) -> u64 {
    if let Some(hex) = hex_or_decimal_from_string_u64(value) {
        hex
    } else {
        hashes::xxhash(value)
    }
}

fn deserialize_bintype(bin_type: &JsonValue) -> BinType {
    match bin_type.as_str().expect("Expected String") {
        "None" => BinType::None,
        "Bool" => BinType::Bool,
        "SInt8" => BinType::SInt8,
        "UInt8" => BinType::UInt8,
        "SInt16" => BinType::SInt16,
        "UInt16" => BinType::UInt16,
        "SInt32" => BinType::SInt32,
        "UInt32" => BinType::UInt32,
        "SInt64" => BinType::SInt64,
        "UInt64" => BinType::UInt64,
        "Float32" => BinType::Float32,
        "Vector2" => BinType::Vector2,
        "Vector3" => BinType::Vector3,
        "Vector4" => BinType::Vector4,
        "Matrix4x4" => BinType::Matrix4x4,
        "Rgba" => BinType::Rgba,
        "String" => BinType::String,
        "Hash" => BinType::Hash,
        "WadEntryLink" => BinType::WadEntryLink,
        "Container" => BinType::Container,
        "Struct" => BinType::Struct,
        "Pointer" => BinType::Pointer,
        "Embedded" => BinType::Embedded,
        "Optional" => BinType::Optional,
        "Link" => BinType::Link,
        "Map" => BinType::Map,
        "Flag" => BinType::Flag,
        _ => panic!("Unknown bin type: {}", bin_type),
    }
}

fn deserialize_bindata(value: &JsonValue, bin_type: &BinType) -> BinData {
    match *bin_type {
        BinType::None => BinData::None,
        BinType::Bool => BinData::Bool(value.as_bool().expect("Expected Bool")),
        BinType::SInt8 => BinData::SInt8(value.as_i8().expect("Expected SInt8")),
        BinType::UInt8 => BinData::UInt8(value.as_u8().expect("Expected UInt8")),
        BinType::SInt16 => BinData::SInt16(value.as_i16().expect("Expected SInt16")),
        BinType::UInt16 => BinData::UInt16(value.as_u16().expect("Expected UInt16")),
        BinType::SInt32 => BinData::SInt32(value.as_i32().expect("Expected SInt32")),
        BinType::UInt32 => BinData::UInt32(value.as_u32().expect("Expected UInt32")),
        BinType::SInt64 => BinData::SInt64(value.as_i64().expect("Expected SInt64")),
        BinType::UInt64 => BinData::UInt64(value.as_u64().expect("Expected UInt64")),
        BinType::Float32 => BinData::Float32(value.as_f32().expect("Expected Float32")),
        BinType::Vector2 => {
            if let JsonValue::Array(vec2_array) = value {
                assert_eq!(vec2_array.len(), 2);
                let mut vec2: Vec<f32> = Vec::with_capacity(2);
                for value in vec2_array {
                    vec2.push(value.as_f32().expect("Expected Vector2 as Float32"));
                }
                BinData::Vector2(vec2)
            } else {
                panic!("Expected Vector2 as Array");
            }
        }
        BinType::Vector3 => {
            if let JsonValue::Array(vec3_array) = value {
                assert_eq!(vec3_array.len(), 3);
                let mut vec3: Vec<f32> = Vec::with_capacity(3);
                for value in vec3_array {
                    vec3.push(value.as_f32().expect("Expected Vector3 as Float32"));
                }
                BinData::Vector3(vec3)
            } else {
                panic!("Expected Vector3 as Array");
            }
        }
        BinType::Vector4 => {
            if let JsonValue::Array(vec4_array) = value {
                assert_eq!(vec4_array.len(), 4);
                let mut vec4: Vec<f32> = Vec::with_capacity(4);
                for value in vec4_array {
                    vec4.push(value.as_f32().expect("Expected Vector4 as Float32"));
                }
                BinData::Vector4(vec4)
            } else {
                panic!("Expected Vector4 as Array");
            }
        }
        BinType::Matrix4x4 => {
            if let JsonValue::Array(mtx44_array) = value {
                assert_eq!(mtx44_array.len(), 16);
                let mut mtx44: Vec<f32> = Vec::with_capacity(16);
                for value in mtx44_array {
                    mtx44.push(value.as_f32().expect("Expected Matrix4x4 as Float32"));
                }
                BinData::Matrix4x4(mtx44)
            } else {
                panic!("Expected Matrix4x4 as Array");
            }
        }
        BinType::Rgba => {
            if let JsonValue::Array(rgba_array) = value {
                assert_eq!(rgba_array.len(), 4);
                let mut rgba: Vec<u8> = Vec::with_capacity(4);
                for value in rgba_array {
                    rgba.push(value.as_u8().expect("Expected Rgba as UInt8"));
                }
                BinData::Rgba(rgba)
            } else {
                panic!("Expected Rgba as Array");
            }
        }
        BinType::String => BinData::String(value.as_str().expect("Expected String").to_string()),
        BinType::Hash => BinData::Hash(string_to_hash_u32(
            value.as_str().expect("Expected Hash as String"),
        )),
        BinType::WadEntryLink => BinData::WadEntryLink(string_to_hash_u64(
            value.as_str().expect("Expected WadEntryLink as String"),
        )),
        BinType::Container | BinType::Struct => deserialize_containerorstruct(value),
        BinType::Pointer | BinType::Embedded => deserialize_pointerorembedded(value),
        BinType::Optional => deserialize_optional(value),
        BinType::Link => BinData::Link(string_to_hash_u32(
            value.as_str().expect("Expected Link as String"),
        )),
        BinType::Map => deserialize_map(value),
        BinType::Flag => BinData::Flag(value.as_bool().expect("Expected Flag as Bool")),
    }
}

fn deserialize_containerorstruct(object: &JsonValue) -> BinData {
    let mut object = object.entries();
    let btype = deserialize_bintype(object.next().expect("Expected container or struct type").1);
    let data: Vec<BinData> = object
        .next()
        .expect("Expected container or struct data")
        .1
        .members()
        .map(|field| deserialize_bindata(field, &btype))
        .collect();
    BinData::ContainerOrStruct(ContainerOrStruct::new(btype, data))
}

fn deserialize_binfield(object: &JsonValue) -> BinField {
    let mut object = object.entries();
    let name = string_to_hash_u32(
        object
            .next()
            .expect("Expected bin field name")
            .1
            .as_str()
            .expect("Expected bin field name as string"),
    );
    let btype = deserialize_bintype(object.next().expect("Expected bin field type").1);
    let data = deserialize_bindata(object.next().expect("Expected bin field data").1, &btype);
    BinField::new(name, btype, data)
}

fn deserialize_pointerorembedded(object: &JsonValue) -> BinData {
    let mut object = object.entries();
    if object.len() > 0 {
        let namedata = object.next().expect("Expected pointer or embedded object");
        let name = string_to_hash_u32(namedata.0);
        let data: Vec<BinField> = namedata.1.members().map(deserialize_binfield).collect();
        BinData::PointerOrEmbedded(PointerOrEmbedded::new(name, data))
    } else {
        BinData::PointerOrEmbedded(PointerOrEmbedded::new(0, Vec::new()))
    }
}

fn deserialize_optional(object: &JsonValue) -> BinData {
    let mut object = object.entries();
    let btype = deserialize_bintype(object.next().expect("Expected optional type").1);
    let mut data = object.next().expect("Expected optional data").1.members();
    let item = if data.len() > 0 {
        let value = data.next().expect("Expected optional data value");
        Some(deserialize_bindata(value, &btype))
    } else {
        None
    };
    BinData::Optional(Optional::new(btype, item))
}

fn deserialize_mappair(object: &JsonValue, keytype: &BinType, valuetype: &BinType) -> MapPair {
    let mut object = object.entries();
    match keytype {
        BinType::WadEntryLink | BinType::Hash | BinType::Link => {
            let keyvalue = object.next().expect("Expected object in MapPair");
            let keydata = deserialize_bindata(&JsonValue::String(keyvalue.0.to_string()), keytype);
            let valuedata = deserialize_bindata(keyvalue.1, valuetype);
            MapPair::new(keydata, valuedata)
        }
        _ => {
            let key = object.next().expect("Expected keydata object in MapPair");
            let keydata = deserialize_bindata(key.1, keytype);
            let value = object.next().expect("Expected valuedata object in MapPair");
            let valuedata = deserialize_bindata(value.1, valuetype);
            MapPair::new(keydata, valuedata)
        }
    }
}

fn deserialize_map(object: &JsonValue) -> BinData {
    let mut object = object.entries();
    let keytype = deserialize_bintype(object.next().expect("Expected map keytype").1);
    let valuetype = deserialize_bintype(object.next().expect("Expected map valuetype").1);
    let data: Vec<MapPair> = object
        .next()
        .expect("Expected map data")
        .1
        .members()
        .map(|field| deserialize_mappair(field, &keytype, &valuetype))
        .collect();
    BinData::Map(Map::new(keytype, valuetype, data))
}

pub fn convert_json_to_bin(contents: &str) -> BinFile {
    println!("Converting JSON to bin");

    let root = json::parse(contents).expect("Could not parse json");

    let is_patch = root["IsPatch"].as_bool().expect("Expected bool in IsPatch");

    let unknown = if is_patch {
        Some(root["Unknown"].as_u64().expect("Expected u64 in Unknown"))
    } else {
        None
    };

    let version = root["Version"].as_u32().expect("Expected u32 in Version");
    let linked_list: Vec<String> = root["LinkedList"]
        .members()
        .map(|linked| {
            linked
                .as_str()
                .expect("Expected string in LinkedList")
                .to_string()
        })
        .collect();

    let entries = Map::new(
        BinType::Hash,
        BinType::Embedded,
        root["Entries"]
            .members()
            .map(|entry| deserialize_mappair(entry, &BinType::Hash, &BinType::Embedded))
            .collect(),
    );

    let patches: Option<Map> = if is_patch {
        Some(Map::new(
            BinType::Hash,
            BinType::Embedded,
            root["Patches"]
                .members()
                .map(|entry| deserialize_mappair(entry, &BinType::Hash, &BinType::Embedded))
                .collect(),
        ))
    } else {
        None
    };

    println!("Finished converting JSON to bin");

    BinFile::new(is_patch, unknown, version, linked_list, entries, patches)
}

fn hex_or_decimal_from_string_u32(string: &str) -> Option<u32> {
    if !check_valid_hex_or_decimal(string) {
        return None;
    }

    let mut str_iter = string.chars().peekable();

    let mut negative = false;

    let first = str_iter.peek();
    if first == Some(&'+') {
        str_iter.next();
    } else if first == Some(&'-') {
        str_iter.next();
        negative = true;
    }

    let mut result: u32;

    let str_iter_str = str_iter.clone().collect::<String>();

    if str_iter_str.starts_with("0x") | str_iter_str.starts_with("0X") {
        result = u32::from_str_radix(&str_iter_str[2..], 16).unwrap();
    } else {
        result = str_iter_str.parse::<u32>().unwrap();
    }

    if negative {
        result = (-(result as i32)) as u32;
    }

    Some(result)
}

fn hex_or_decimal_from_string_u64(string: &str) -> Option<u64> {
    if !check_valid_hex_or_decimal(string) {
        return None;
    }

    let mut str_iter = string.chars().peekable();

    let mut negative = false;

    let first = str_iter.peek();
    if first == Some(&'+') {
        str_iter.next();
    } else if first == Some(&'-') {
        str_iter.next();
        negative = true;
    }

    let mut result: u64;

    let str_iter_str = str_iter.clone().collect::<String>();

    if str_iter_str.starts_with("0x") | str_iter_str.starts_with("0X") {
        result = u64::from_str_radix(&str_iter_str[2..], 16).unwrap();
    } else {
        result = str_iter_str.parse::<u64>().unwrap();
    }

    if negative {
        result = (-(result as i64)) as u64;
    }

    Some(result)
}

fn check_valid_hex_or_decimal(string: &str) -> bool {
    if string.is_empty() {
        return false;
    }

    let mut str_iter = string.chars().peekable();

    let first = str_iter.peek();
    if first == Some(&'+') || first == Some(&'-') {
        str_iter.next();
    }

    let str_iter_str = str_iter.clone().collect::<String>();

    if str_iter_str.starts_with("0x") | str_iter_str.starts_with("0X") {
        str_iter.skip(2).all(|c| c.is_ascii_hexdigit())
    } else {
        str_iter.all(|c| c.is_ascii_digit())
    }
}
