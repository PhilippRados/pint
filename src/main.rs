#![allow(unused)]

use clap::Parser;
// use compress::zlib;
use inflate::inflate_bytes_zlib;
use std::str;
use std::{fs::File, io::Read};

#[derive(Parser)]
struct Cli {
    /// The path to the file to read
    #[clap(parse(from_os_str))]
    path: std::path::PathBuf,
}

struct PngChunk {
    chunk_type: String,
    data_len: usize,
    chunk_len: usize,
    data: Vec<u8>,
    crc: Vec<u8>,
}

impl Default for PngChunk {
    fn default() -> PngChunk {
        PngChunk {
            chunk_type: String::new(),
            data_len: 0,
            chunk_len: 0,
            data: vec![],
            crc: vec![],
        }
    }
}

struct IHDRData {
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: u8,
}

fn to_hex_string(bytes: Vec<u8>) -> Vec<String> {
    let strs: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
    strs
}

fn parse_png_chunks(buf: &[u8]) -> Option<PngChunk> {
    let mut result: PngChunk = PngChunk::default();
    let chunk_types = ["IHDR", "IDAT", "IEND"];

    result.data_len = buf[0..4].iter().fold(0usize, |acc, x| acc + *x as usize);
    result.chunk_len = result.data_len + 12;
    result.chunk_type = match str::from_utf8(&buf[4..8]) {
        Ok(s) => s.to_string(),
        Err(e) => return None,
    };

    if (!chunk_types.contains(&result.chunk_type.as_str())) {
        return None;
    }

    result.data = buf[8..8 + result.data_len].to_vec();
    result.crc = buf[8 + result.data_len..8 + result.data_len].to_vec();

    Some(result)
}
fn parse_ihdr(data: Vec<u8>) -> IHDRData {
    let result = IHDRData {
        width: data[0..4].iter().fold(0, |acc, x| acc + *x as u32),
        height: data[4..8].iter().fold(0, |acc, x| acc + *x as u32),
        bit_depth: data[8],
        color_type: data[9],
    };
    assert!(result.color_type == 2, "no rgb encoded png");
    assert!(
        result.bit_depth == 8 || result.bit_depth == 16,
        "rgb encoded images need bit-depth of 8/16"
    );
    result
}

fn parse_data(image_data: Vec<Vec<u8>>) {
    let inflated = inflate_bytes_zlib(&image_data[0] as &[u8]).unwrap();
    println!("{:?}", inflated);
}

fn main() {
    const VALID_PNG: &str = "89504E470D0A1A0A";
    let args = Cli::parse();
    let file = &mut File::open(args.path).unwrap();

    let first_bytes: &mut Vec<u8> = &mut vec![0; 8];
    let is_valid_png = File::read(file, first_bytes);
    if (to_hex_string(first_bytes.to_vec()).join("") != VALID_PNG) {
        println!("pint: given file is not a valid png");
        std::process::exit(0);
    }

    let file_byte_size = File::metadata(file).unwrap().len();
    let buf: &mut [u8] = &mut vec![0; (file_byte_size - 8) as usize]; // cut of beginning identifier sequence
    let result = File::read(file, buf);
    let mut i: u32 = 0;
    let mut meta_data: IHDRData;
    let mut data: Vec<Vec<u8>> = Vec::new();

    assert!(file_byte_size - 8 > 0);
    while i < (file_byte_size - 8) as u32 {
        let chunk = match parse_png_chunks(&buf[i as usize..]) {
            Some(c) => c,
            None => {
                i += 1;
                continue;
            }
        };

        match chunk.chunk_type.as_str() {
            "IHDR" => meta_data = parse_ihdr(chunk.data),
            "IDAT" => data.push(chunk.data),
            "IEND" => {
                parse_data(data);
                break;
            }
            _ => {
                i += 1;
                continue;
            }
        }
        i += chunk.chunk_len as u32;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn can_parse_chunks() {
        // ihdr block
        let buf = vec![
            0b0,
            0b0,
            0b0,
            0b0000_1101,
            0b0100_1001,
            0b0100_1000,
            0b0100_0100,
            0b0101_0010,
            0b0,
            0b0,
            0b0,
            0b10010110,
            0b0,
            0b0,
            0b0,
            0b10010001,
            0b0000_1000,
            0b0000_0010,
            0b0,
            0b0,
            0b0,
            0b10101110,
            0b01100110,
            0b11010110,
            0b00001101,
        ];

        let result = parse_png_chunks(&buf);
        assert_eq!(result.as_ref().unwrap().chunk_len, 25);
        assert_eq!(result.as_ref().unwrap().data_len, 13);
        assert_eq!(result.as_ref().unwrap().chunk_type, "IHDR");
        assert_eq!(
            result.as_ref().unwrap().data,
            [
                0b0,
                0b0,
                0b0,
                0b10010110,
                0b0,
                0b0,
                0b0,
                0b10010001,
                0b0000_1000,
                0b0000_0010,
                0b0,
                0b0,
                0b0,
            ]
        );
    }
    #[test]
    fn can_parse_ihdr_data() {
        // ihdr block
        let buf = vec![
            0b0,
            0b0,
            0b0,
            0b10010110,
            0b0,
            0b0,
            0b0,
            0b10010001,
            0b0000_1000,
            0b0000_0010,
            0b0,
            0b0,
            0b0,
        ];

        let result = parse_ihdr(buf);
        assert_eq!(result.width, 150);
        assert_eq!(result.height, 145);
        assert_eq!(result.bit_depth, 8);
        assert_eq!(result.color_type, 2);
    }
}
