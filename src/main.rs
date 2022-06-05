#![allow(unused)]

use clap::Parser;
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
    width: i64,
    height: i64,
    bit_depth: i8,
    color_type: i8,
}

fn to_hex_string(bytes: Vec<u8>) -> Vec<String> {
    let strs: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
    strs
}

fn parse_png_chunks(buf: &[u8]) -> PngChunk {
    let mut result: PngChunk = PngChunk::default();

    result.data_len = buf[0..4].iter().fold(0usize, |acc, x| acc + *x as usize);
    result.chunk_len = result.data_len + 12;
    result.chunk_type = match str::from_utf8(&buf[4..8]) {
        Ok(s) => s.to_string(),
        Err(e) => panic!("Invalid byte sequence in png: {}", e),
    };
    result.data = buf[8..8 + result.data_len].to_vec();
    result.crc = buf[8 + result.data_len..8 + result.data_len].to_vec();
    result
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
    let mut chunk_len = 1;
    let meta_data: IHDRData;
    let data: Vec<Vec<u8>>;

    for i in (0..(file_byte_size - 8)).step_by(chunk_len) {
        println!("{}", i);
        let chunk = parse_png_chunks(&buf[i as usize..]);

        // match chunk.chunk_type.as_str() {
        //     "IHDR" => meta_data = parseIhdrData(chunk),
        //     "IDAT" => data.push(chunk.data),
        //     "IEND" => parseData(),
        // }
        chunk_len = chunk.chunk_len;
    }
    println!("{:02X?}", buf);
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
        assert_eq!(result.chunk_len, 25);
        assert_eq!(result.data_len, 13);
        assert_eq!(result.chunk_type, "IHDR");
        assert_eq!(
            result.data,
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
}
