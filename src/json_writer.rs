use structs::*;

use json::{codegen::Generator, JsonValue};
use std::collections::HashMap;

fn hash_u32_to_string(value: u32, hash_map: &mut HashMap<u64, String>) -> String {
    let str_value = hash_map.get(&(value as u64));
    if let Some(str_value) = str_value {
        str_value.clone()
    } else {
        format!("0x{:08X}", value)
    }
}

fn hash_u64_to_string(value: u64, hash_map: &mut HashMap<u64, String>) -> String {
    let str_value = hash_map.get(&value);
    if let Some(str_value) = str_value {
        str_value.clone()
    } else {
        format!("0x{:016X}", value)
    }
}

fn serialize_bintype(bintype: &BinType) -> JsonValue {
    JsonValue::String(format!("{:?}", bintype))
}

fn serialize_bindata(bindata: &BinData, hash_map: &mut HashMap<u64, String>) -> JsonValue {
    match bindata {
        BinData::None => JsonValue::Null,
        BinData::Bool(bool) => JsonValue::Boolean(*bool),
        BinData::SInt8(i8) => From::<i8>::from(*i8),
        BinData::UInt8(u8) => From::<u8>::from(*u8),
        BinData::SInt16(i16) => From::<i16>::from(*i16),
        BinData::UInt16(u16) => From::<u16>::from(*u16),
        BinData::SInt32(i32) => From::<i32>::from(*i32),
        BinData::UInt32(u32) => From::<u32>::from(*u32),
        BinData::SInt64(i64) => From::<i64>::from(*i64),
        BinData::UInt64(u64) => From::<u64>::from(*u64),
        BinData::Float32(f32) => from_f32(*f32),
        BinData::Vector2(vec2) => {
            assert_eq!(vec2.len(), 2);
            let mut vec2_array: Vec<JsonValue> = Vec::with_capacity(2);
            for value in vec2 {
                vec2_array.push(from_f32(*value));
            }
            JsonValue::Array(vec2_array)
        }
        BinData::Vector3(vec3) => {
            assert_eq!(vec3.len(), 3);
            let mut vec3_array: Vec<JsonValue> = Vec::with_capacity(3);
            for value in vec3 {
                vec3_array.push(from_f32(*value));
            }
            JsonValue::Array(vec3_array)
        }
        BinData::Vector4(vec4) => {
            assert_eq!(vec4.len(), 4);
            let mut vec4_array: Vec<JsonValue> = Vec::with_capacity(4);
            for value in vec4 {
                vec4_array.push(from_f32(*value));
            }
            JsonValue::Array(vec4_array)
        }
        BinData::Matrix4x4(mtx44) => {
            assert_eq!(mtx44.len(), 16);
            let mut mtx44_array: Vec<JsonValue> = Vec::with_capacity(16);
            for value in mtx44 {
                mtx44_array.push(from_f32(*value));
            }
            JsonValue::Array(mtx44_array)
        }
        BinData::Rgba(rgba) => {
            assert_eq!(rgba.len(), 4);
            let mut rgba_array: Vec<JsonValue> = Vec::with_capacity(4);
            for value in rgba {
                rgba_array.push(From::<u8>::from(*value));
            }
            JsonValue::Array(rgba_array)
        }
        BinData::String(string) => JsonValue::String(string.clone()),
        BinData::Hash(hash) => JsonValue::String(hash_u32_to_string(*hash, hash_map)),
        BinData::WadEntryLink(wadentrylink) => {
            JsonValue::String(hash_u64_to_string(*wadentrylink, hash_map))
        }
        BinData::ContainerOrStruct(cs) => serialize_containerorstruct(cs, hash_map),
        BinData::PointerOrEmbedded(pe) => serialize_pointerorembedded(pe, hash_map),
        BinData::Optional(optional) => serialize_optional(optional, hash_map),
        BinData::Link(link) => JsonValue::String(hash_u32_to_string(*link, hash_map)),
        BinData::Map(map) => serialize_map(map, hash_map),
        BinData::Flag(flag) => JsonValue::Boolean(*flag),
    }
}

fn serialize_containerorstruct(
    cs: &ContainerOrStruct,
    hash_map: &mut HashMap<u64, String>,
) -> JsonValue {
    let mut array = JsonValue::new_array();
    for bindata in &cs.items {
        array.push(serialize_bindata(bindata, hash_map)).unwrap();
    }
    let mut object = JsonValue::new_object();
    object.insert("type", serialize_bintype(&cs.btype)).unwrap();
    object.insert("data", array).unwrap();
    object
}

fn serialize_binfield(binfield: &BinField, hash_map: &mut HashMap<u64, String>) -> JsonValue {
    let mut object = JsonValue::new_object();
    object
        .insert(
            "name",
            JsonValue::String(hash_u32_to_string(binfield.name, hash_map)),
        )
        .unwrap();
    object
        .insert("type", serialize_bintype(&binfield.btype))
        .unwrap();
    object
        .insert("data", serialize_bindata(&binfield.data, hash_map))
        .unwrap();
    object
}

fn serialize_pointerorembedded(
    pe: &PointerOrEmbedded,
    hash_map: &mut HashMap<u64, String>,
) -> JsonValue {
    let mut array = JsonValue::new_array();
    for binfield in &pe.items {
        array.push(serialize_binfield(binfield, hash_map)).unwrap();
    }
    let mut object = JsonValue::new_object();
    object
        .insert(hash_u32_to_string(pe.name, hash_map).as_str(), array)
        .unwrap();
    object
}

fn serialize_optional(optional: &Optional, hash_map: &mut HashMap<u64, String>) -> JsonValue {
    let mut object = JsonValue::new_object();
    object
        .insert("type", serialize_bintype(&optional.btype))
        .unwrap();
    if let Some(bindata) = &optional.data {
        let item = serialize_bindata(bindata, hash_map);
        object
            .insert("data", JsonValue::Array([item].to_vec()))
            .unwrap();
    } else {
        object.insert("data", JsonValue::Null).unwrap();
    }
    object
}

fn serialize_mappair(mappair: &MapPair, hash_map: &mut HashMap<u64, String>) -> JsonValue {
    match *mappair.keydata {
        BinData::Hash(key) | BinData::Link(key) => {
            let mut object = JsonValue::new_object();
            object
                .insert(
                    hash_u32_to_string(key, hash_map).as_str(),
                    serialize_bindata(&mappair.valuedata, hash_map),
                )
                .unwrap();
            object
        }
        BinData::WadEntryLink(key) => {
            let mut object = JsonValue::new_object();
            object
                .insert(
                    hash_u64_to_string(key, hash_map).as_str(),
                    serialize_bindata(&mappair.valuedata, hash_map),
                )
                .unwrap();
            object
        }
        _ => {
            let mut object = JsonValue::new_object();
            object
                .insert("keydata", serialize_bindata(&mappair.keydata, hash_map))
                .unwrap();
            object
                .insert("valuedata", serialize_bindata(&mappair.valuedata, hash_map))
                .unwrap();
            object
        }
    }
}

fn serialize_map(map: &Map, hash_map: &mut HashMap<u64, String>) -> JsonValue {
    let mut array = JsonValue::new_array();
    for mappair in &map.items {
        array.push(serialize_mappair(mappair, hash_map)).unwrap();
    }
    let mut object = JsonValue::new_object();
    object
        .insert("keytype", serialize_bintype(&map.keytype))
        .unwrap();
    object
        .insert("valuetype", serialize_bintype(&map.valuetype))
        .unwrap();
    object.insert("data", array).unwrap();
    object
}

pub fn convert_bin_to_json(bin_file: &BinFile, hash_map: &mut HashMap<u64, String>) -> String {
    println!("Converting bin to JSON");

    let mut root = JsonValue::new_object();

    root.insert("IsPatch", JsonValue::Boolean(bin_file.is_patch))
        .unwrap();

    if let Some(unknown) = bin_file.unknown {
        root.insert("Unknown", JsonValue::from(unknown)).unwrap();
    }

    root.insert("Version", JsonValue::from(bin_file.version))
        .unwrap();

    let linked_list: Vec<JsonValue> = bin_file
        .linked_list
        .iter()
        .map(|linked| JsonValue::String(linked.to_string()))
        .collect();
    root.insert("LinkedList", linked_list).unwrap();

    let mut entries = JsonValue::new_array();

    for entry in &bin_file.entries.items {
        entries.push(serialize_mappair(entry, hash_map)).unwrap();
    }

    root.insert("Entries", entries).unwrap();

    if let Some(patches) = &bin_file.patches {
        let mut patches_array = JsonValue::new_array();

        for patch in &patches.items {
            patches_array
                .push(serialize_mappair(patch, hash_map))
                .unwrap();
        }

        root.insert("Patches", patches_array).unwrap();
    }

    let mut gen = MyPrettyGenerator::new();
    gen.write_json(&root).expect("Can't write json");

    println!("Finished converting bin to JSON");

    gen.consume()
}

pub struct MyPrettyGenerator {
    buf: Vec<u8>,
    dent: u16,
}

impl MyPrettyGenerator {
    pub fn new() -> Self {
        MyPrettyGenerator {
            buf: Vec::with_capacity(1024),
            dent: 0,
        }
    }

    pub fn consume(self) -> String {
        String::from_utf8(self.buf).expect("JSON have invalid UTF-8")
    }
}

impl json::codegen::Generator for MyPrettyGenerator {
    type T = Vec<u8>;

    #[inline(always)]
    fn get_writer(&mut self) -> &mut Vec<u8> {
        &mut self.buf
    }

    #[inline(always)]
    fn write(&mut self, slice: &[u8]) -> std::io::Result<()> {
        std::io::Write::write_all(&mut self.get_writer(), slice)
    }

    #[inline(always)]
    fn write_char(&mut self, ch: u8) -> std::io::Result<()> {
        self.write(&[ch])
    }

    #[inline(always)]
    fn write_min(&mut self, slice: &[u8], _: u8) -> std::io::Result<()> {
        self.write(slice)
    }

    #[inline(always)]
    fn new_line(&mut self) -> std::io::Result<()> {
        self.write_char(b'\n')?;
        for _ in 0..self.dent {
            self.write_char(b'\t')?;
        }
        Ok(())
    }

    #[inline(always)]
    fn indent(&mut self) {
        self.dent += 1;
    }

    #[inline(always)]
    fn dedent(&mut self) {
        self.dent -= 1;
    }

    #[inline(always)]
    fn write_number(&mut self, num: &json::number::Number) -> std::io::Result<()> {
        if num.is_nan() {
            return self.write(b"null");
        }
        let (positive, mantissa, exponent) = num.as_parts();
        if exponent >= 0 {
            if positive {
                self.write(format!("{}", mantissa).as_bytes())
            } else {
                self.write(format!("{}", -(mantissa as i64)).as_bytes())
            }
        } else {
            let float = f32::from_bits(mantissa as u32);
            // let float_str = format!("{}", float);
            let mut buffer = dtoa::Buffer::new();
            let float_str = buffer.format_finite(float);
            self.write(float_str.as_bytes())
        }
    }

    fn write_json(&mut self, json: &JsonValue) -> std::io::Result<()> {
        match *json {
            JsonValue::Null => self.write(b"null"),
            JsonValue::Short(ref short) => self.write_string(short.as_str()),
            JsonValue::String(ref string) => self.write_string(string),
            JsonValue::Number(ref number) => self.write_number(number),
            JsonValue::Boolean(true) => self.write(b"true"),
            JsonValue::Boolean(false) => self.write(b"false"),
            JsonValue::Array(ref array) => {
                self.write_char(b'[')?;
                let mut iter = array.iter();

                if let Some(item) = iter.next() {
                    self.indent();
                    self.new_line()?;
                    self.write_json(item)?;
                } else {
                    self.write_char(b']')?;
                    return Ok(());
                }

                for item in iter {
                    if let JsonValue::Number(number) = item {
                        self.write(b", ")?;
                        self.write_number(number)?;
                    } else {
                        self.write_char(b',')?;
                        self.new_line()?;
                        self.write_json(item)?;
                    }
                }

                self.dedent();
                self.new_line()?;
                self.write_char(b']')
            }
            JsonValue::Object(ref object) => {
                self.write_char(b'{')?;
                let mut iter = object.iter();

                if let Some((key, value)) = iter.next() {
                    self.indent();
                    self.new_line()?;
                    self.write_string(key)?;
                    self.write(b": ")?;
                    self.write_json(value)?;
                } else {
                    self.write_char(b'}')?;
                    return Ok(());
                }

                for (key, value) in iter {
                    self.write_char(b',')?;
                    self.new_line()?;
                    self.write_string(key)?;
                    self.write(b": ")?;
                    self.write_json(value)?;
                }

                self.dedent();
                self.new_line()?;
                self.write_char(b'}')
            }
        }
    }
}

fn from_f32(float: f32) -> JsonValue {
    JsonValue::Number(unsafe {
        json::number::Number::from_parts_unchecked(false, float.to_bits() as u64, -1)
    })
}
