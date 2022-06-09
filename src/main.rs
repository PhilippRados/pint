#![allow(unused)]
use clap::Parser;
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

impl Default for IHDRData {
    fn default() -> IHDRData {
        IHDRData {
            width: 0,
            height: 0,
            bit_depth: 0,
            color_type: 0,
        }
    }
}

fn to_hex_string(bytes: Vec<u8>) -> Vec<String> {
    let strs: Vec<String> = bytes.iter().map(|b| format!("{:02X}", b)).collect();
    strs
}

fn bytes_to_int(bytes_arr: &[u8]) -> u32 {
    let mut dst = [0u8; 4];
    dst.clone_from_slice(bytes_arr);
    u32::from_be_bytes(dst)
}

fn parse_png_chunks(buf: &[u8]) -> Option<PngChunk> {
    let mut result: PngChunk = PngChunk::default();
    let chunk_types = ["IHDR", "IDAT", "IEND"];

    result.data_len = bytes_to_int(&buf[0..4]) as usize;
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
        width: bytes_to_int(&data[0..4]),
        height: bytes_to_int(&data[4..8]),
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

fn sub_filter(current_row: &[u8], line_pos: usize, prev_codel: RGB) -> RGB {
    RGB(
        ((current_row[line_pos] as i16 + prev_codel.0 as i16) % 256) as u8,
        ((current_row[line_pos + 1] as i16 + prev_codel.1 as i16) % 256) as u8,
        ((current_row[line_pos + 2] as i16 + prev_codel.2 as i16) % 256) as u8,
    )
}

fn up_filter(current_row: &[u8], line_pos: usize, prev_row: &[RGB]) -> RGB {
    RGB(
        ((current_row[line_pos] as i16 + prev_row[line_pos / 3].0 as i16) % 256) as u8,
        ((current_row[line_pos + 1] as i16 + prev_row[line_pos / 3].1 as i16) % 256) as u8,
        ((current_row[line_pos + 2] as i16 + prev_row[line_pos / 3].2 as i16) % 256) as u8,
    )
}
fn avg_filter(current_row: &[u8], line_pos: usize, prev_codel: RGB, prev_row: &[RGB]) -> RGB {
    RGB(
        ((current_row[line_pos] as i16 + (prev_row[line_pos / 3].0 + prev_codel.0) as i16 / 2)
            % 256) as u8,
        ((current_row[line_pos + 1] as i16 + (prev_row[line_pos / 3].1 + prev_codel.1) as i16 / 2)
            % 256) as u8,
        ((current_row[line_pos + 2] as i16 + (prev_row[line_pos / 3].2 + prev_codel.2) as i16 / 2)
            % 256) as u8,
    )
}

fn rgb_sum(rgb: RGB) -> i16 {
    (rgb.0 as i32 + rgb.1 as i32 + rgb.2 as i32) as i16
}
fn paeth_filter(current_row: &[u8], line_pos: usize, prev_codel: RGB, prev_row: &[RGB]) -> RGB {
    let diag_codel = if (line_pos / 3) > 0 {
        RGB(
            prev_row[(line_pos / 3) - 1].0,
            prev_row[(line_pos / 3) - 1].1,
            prev_row[(line_pos / 3) - 1].2,
        )
    } else {
        RGB(0, 0, 0)
    };

    let p = rgb_sum(prev_codel) + rgb_sum(prev_row[line_pos / 3]) - rgb_sum(diag_codel);
    let p_left_codel = (p - rgb_sum(prev_codel)).abs();
    let p_up_codel = (p - rgb_sum(prev_row[line_pos / 3])).abs();
    let p_diag_codel = (p - rgb_sum(diag_codel)).abs();

    if p_left_codel <= p_up_codel && p_left_codel <= p_diag_codel {
        return prev_codel;
    } else if p_up_codel <= p_diag_codel {
        return prev_row[line_pos / 3];
    } else {
        return diag_codel;
    }
}

fn add_to_array(current_row: &[u8], prev_row: &[RGB], rgb_img: &mut Vec<RGB>, filter: u8) {
    let mut line_pos = 1;
    let mut prev_codel = RGB(0, 0, 0);

    while line_pos < current_row.len() - 2 {
        match filter {
            0 => rgb_img.push(RGB(
                current_row[line_pos],
                current_row[line_pos + 1],
                current_row[line_pos + 2],
            )),
            1 => {
                let codel = sub_filter(current_row, line_pos, prev_codel);
                rgb_img.push(codel);
                prev_codel = codel;
            }
            2 => rgb_img.push(up_filter(current_row, line_pos, prev_row)),
            3 => {
                let codel = avg_filter(current_row, line_pos, prev_codel, prev_row);
                rgb_img.push(codel);
                prev_codel = codel;
            }
            4 => {
                let codel = paeth_filter(current_row, line_pos, prev_codel, prev_row);
                rgb_img.push(codel);
                prev_codel = codel;
            }
            _ => println!("this filter doesn't exist"),
        }

        line_pos += 3;
    }
}

#[derive(Copy, Clone, Debug)]
struct RGB(u8, u8, u8);

fn parse_data(image_data: Vec<Vec<u8>>, byte_width: usize) {
    let mut inflated = Vec::new();
    for i in image_data {
        inflated.append(&mut inflate_bytes_zlib(&i as &[u8]).unwrap());
    }
    let mut rgb_img: Vec<RGB> = Vec::new();
    let mut j = 0;
    let mut i = 0;
    let pixel_width = byte_width / 3; // bc 3 channels in rgb
    let prev_row: &mut [RGB] = &mut vec![RGB(0, 0, 0); pixel_width];
    let mut k = 0;

    while j + byte_width < inflated.len() {
        let current_row = &inflated[j..j + byte_width + 1]; // + 1 for the line-filter
        let filter = current_row[0];

        add_to_array(current_row, prev_row, &mut rgb_img, filter);

        prev_row.clone_from_slice(&rgb_img[i..i + pixel_width]);
        // if k == 5 {
        //     dbg!(prev_row);
        //     break;
        // }
        // k += 1;
        j += byte_width + 1;
        i += pixel_width;
    }
    println!("{:?}", rgb_img);
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
    let mut meta_data: IHDRData = IHDRData::default();
    let mut data: Vec<Vec<u8>> = Vec::new();
    const NUM_OF_CHANNELS: u32 = 3; // piet only handles truecolor-rgb

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
            "IDAT" => {
                println!("idat");
                data.push(chunk.data);
            }
            "IEND" => {
                parse_data(data, (meta_data.width * NUM_OF_CHANNELS) as usize);
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
    #[test]
    fn can_convert_multibyte_arr_to_int() {
        let buf = vec![0b0, 0b0, 0b0000_0010, 0b1001_1011];
        let result = bytes_to_int(&buf[..]);
        assert_eq!(result, 667);
    }
    #[test]
    fn can_convert_single_byte_arr_to_int() {
        let buf = vec![0b0, 0b0, 0b0, 0b1001_1011];
        let result = bytes_to_int(&buf[..]);
        assert_eq!(result, 155);
    }
}
