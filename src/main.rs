extern crate byteorder;
extern crate clap;
extern crate dtoa;
extern crate glob;
extern crate json;

use std::{
    collections::HashMap,
    fs::File,
    io::{BufRead, BufReader, Read, Write},
    path::Path,
};

mod hashes;
mod json_reader;
mod json_writer;
mod reader;
mod structs;
mod writer;

fn main() {
    let matches = clap::Command::new("BinReader-Rust")
        .version(env!("CARGO_PKG_VERSION"))
        .author("https://github.com/autergame/")
        .about("League Of Legends Bin Reader And Writter")
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommand(
            clap::Command::new("decode")
                .about("Decodes the given file")
                .arg(
                    clap::Arg::new("INPUT")
                        .help("Sets the input file to use")
                        .required(true)
                        .index(1),
                )
                .arg(
                    clap::Arg::new("OUTPUT")
                        .help("Sets the output file to use")
                        .required(false)
                        .index(2),
                ),
        )
        .subcommand(
            clap::Command::new("encode")
                .about("Encodes the given file")
                .arg(
                    clap::Arg::new("INPUT")
                        .help("Sets the input file to use")
                        .required(true)
                        .index(1),
                )
                .arg(
                    clap::Arg::new("OUTPUT")
                        .help("Sets the output file to use")
                        .required(false)
                        .index(2),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("decode", args)) => {
            let input = args.get_one::<String>("INPUT").unwrap();
            let output = args.get_one::<String>("OUTPUT");

            let mut hash_map: HashMap<u64, String> = HashMap::new();
            add_to_hash_map(&["path", "patch", "value"], &mut hash_map);

            println!("Loading hashes");
            let mut lines =
                load_hashes_from_file(Path::new("files/hashes.bintypes.txt"), &mut hash_map);
            lines += load_hashes_from_file(Path::new("files/hashes.binfields.txt"), &mut hash_map);
            lines += load_hashes_from_file(Path::new("files/hashes.binhashes.txt"), &mut hash_map);
            lines += load_hashes_from_file(Path::new("files/hashes.binentries.txt"), &mut hash_map);
            lines += load_hashes_from_file(Path::new("files/hashes.lcu.txt"), &mut hash_map);
            lines += load_hashes_from_file(Path::new("files/hashes.game.txt"), &mut hash_map);
            println!("Loaded total of hashes: {lines}");
            println!("Finished loading hashes.\n");

            match output {
                Some(output) => {
                    let contents = read_to_u8(Path::new(input));
                    let bin_file = reader::read_bin(&contents);
                    let jsonstr = json_writer::convert_bin_to_json(&bin_file, &mut hash_map);
                    write_u8(Path::new(output), jsonstr.as_bytes());
                }
                None => {
                    let input_paths = glob::glob(input)
                        .expect("Failed to read glob pattern")
                        .filter_map(Result::ok);

                    for mut input_path in input_paths {
                        let contents = read_to_u8(&input_path);
                        let bin_file = reader::read_bin(&contents);
                        let jsonstr = json_writer::convert_bin_to_json(&bin_file, &mut hash_map);
                        input_path.set_extension("json");
                        write_u8(&input_path, jsonstr.as_bytes());
                        println!();
                    }
                }
            }
        }
        Some(("encode", args)) => {
            let input = args.get_one::<String>("INPUT").unwrap();
            let output = args.get_one::<String>("OUTPUT");

            match output {
                Some(output) => {
                    let contents = read_string(Path::new(input));
                    let bin_file = json_reader::convert_json_to_bin(&contents);
                    let bin = writer::write_bin(&bin_file);
                    write_u8(Path::new(output), &bin);
                }
                None => {
                    let input_paths = glob::glob(input)
                        .expect("Failed to read glob pattern")
                        .filter_map(Result::ok);

                    for mut input_path in input_paths {
                        let contents = read_string(&input_path);
                        let bin_file = json_reader::convert_json_to_bin(&contents);
                        let bin = writer::write_bin(&bin_file);
                        input_path.set_extension("bin");
                        write_u8(&input_path, &bin);
                        println!();
                    }
                }
            }
        }
        _ => {}
    }
}

fn load_hashes_from_file(path: &Path, hash_map: &mut HashMap<u64, String>) -> u32 {
    let path_str = path.to_str().unwrap();

    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) => {
            println!("Could not open hash file: {} error: {}", path_str, error);
            return 0;
        }
    };

    let mut lines = 0;
    let mut reader = BufReader::new(file);
    let mut line = String::with_capacity(1024);

    let msg = |error| {
        println!(
            "Could not read line hash file: {} error: {}",
            path_str, error
        );
        0
    };

    while reader.read_line(&mut line).unwrap_or_else(msg) != 0 {
        let mut line_split = line.split(' ');

        if line_split.clone().count() == 2 {
            let key_str = line_split.next().unwrap();

            let key = match key_str.len() {
                8 => u32::from_str_radix(key_str, 16).unwrap_or_else(|_| {
                    println!("Invalid hex: {}", key_str);
                    0
                }) as u64,
                16 => u64::from_str_radix(key_str, 16).unwrap_or_else(|_| {
                    println!("Invalid hex: {}", key_str);
                    0
                }),
                _ => {
                    line.clear();
                    continue;
                }
            };

            if key == 0 {
                line.clear();
                continue;
            }

            lines += hash_map
                .insert(
                    key,
                    line_split
                        .next()
                        .unwrap()
                        .trim_end_matches(&['\n', '\r'])
                        .to_string(),
                )
                .is_none() as u32;
        }

        line.clear();
    }

    println!("File: {} loaded: {} lines", path_str, lines);

    lines
}

fn add_to_hash_map(hashes_to_insert: &[&str], hash_map: &mut HashMap<u64, String>) {
    for hash_name in hashes_to_insert {
        hash_map.insert(hashes::fnv1a(hash_name) as u64, hash_name.to_string());
        hash_map.insert(hashes::xxhash(hash_name), hash_name.to_string());
    }
}

fn read_to_u8(path: &Path) -> Vec<u8> {
    let mut file = File::open(path).expect("Could not open file");
    let mut contents: Vec<u8> = Vec::new();
    println!("Reading file: {}", path.to_str().unwrap());
    file.read_to_end(&mut contents)
        .expect("Could not read file");
    println!("Finished reading file");
    contents
}

fn write_u8(path: &Path, v: &[u8]) {
    let mut file = File::create(path).expect("Could not create file");
    println!("Writing to file: {}", path.to_str().unwrap());
    file.write_all(v).expect("Could not write to file");
    println!("Finished writing to file");
}

fn read_string(path: &Path) -> String {
    let mut file = File::open(path).expect("Could not open file");
    let mut contents = String::new();
    println!("Reading file: {}", path.to_str().unwrap());
    file.read_to_string(&mut contents)
        .expect("Could not read file");
    println!("Finished reading file");
    contents
}
