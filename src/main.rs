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
        Err(_) => return None,
    };

    if !chunk_types.contains(&result.chunk_type.as_str()) {
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

    RGB(
        paeth_filter_single(
            current_row[line_pos] as i16,
            prev_codel.0 as i16,
            prev_row[line_pos / 3].0 as i16,
            diag_codel.0 as i16,
        ),
        paeth_filter_single(
            current_row[line_pos + 1] as i16,
            prev_codel.1 as i16,
            prev_row[line_pos / 3].1 as i16,
            diag_codel.1 as i16,
        ),
        paeth_filter_single(
            current_row[line_pos + 2] as i16,
            prev_codel.2 as i16,
            prev_row[line_pos / 3].2 as i16,
            diag_codel.2 as i16,
        ),
    )
}
fn paeth_filter_single(current_pixel: i16, prev_pixel: i16, up_pixel: i16, diag_pixel: i16) -> u8 {
    let p = prev_pixel + up_pixel - diag_pixel;
    let p_left_codel = (p - prev_pixel).abs();
    let p_up_codel = (p - up_pixel).abs();
    let p_diag_codel = (p - diag_pixel).abs();

    let mut prediction = 0;
    if p_left_codel <= p_up_codel && p_left_codel <= p_diag_codel {
        prediction = prev_pixel;
    } else if p_up_codel <= p_diag_codel {
        prediction = up_pixel;
    } else {
        prediction = diag_pixel;
    }
    ((current_pixel as i16 + prediction as i16) % 256) as u8
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

fn parse_data(image_data: Vec<Vec<u8>>, byte_width: usize) -> Vec<RGB> {
    let mut inflated = Vec::new();
    for i in image_data {
        inflated.append(&mut inflate_bytes_zlib(&i as &[u8]).unwrap());
    }
    let mut rgb_img: Vec<RGB> = Vec::new();
    let mut j = 0;
    let mut i = 0;
    let pixel_width = byte_width / 3; // bc 3 channels in rgb
    let prev_row: &mut [RGB] = &mut vec![RGB(0, 0, 0); pixel_width];

    while j + byte_width < inflated.len() {
        let current_row = &inflated[j..j + byte_width + 1]; // + 1 for the line-filter
        let filter = current_row[0];

        add_to_array(current_row, prev_row, &mut rgb_img, filter);

        prev_row.clone_from_slice(&rgb_img[i..i + pixel_width]);
        j += byte_width + 1;
        i += pixel_width;
    }
    return rgb_img;
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
    use std::io::Write;
    use std::process::Command;
    use tempfile::NamedTempFile;
    #[test]
    fn can_parse_idat_to_rgb() {
        let idat = vec![vec![
            120, 218, 237, 221, 81, 110, 234, 48, 16, 64, 209, 166, 98, 95, 241, 210, 237, 149,
            185, 253, 181, 43, 25, 220, 152, 140, 175, 184, 231, 15, 245, 169, 228, 113, 53, 234,
            40, 16, 114, 212, 47, 128, 210, 62, 60, 191, 110, 58, 234, 210, 62, 115, 250, 125, 230,
            253, 124, 71, 31, 128, 174, 50, 33, 158, 9, 241, 76, 136, 247, 136, 122, 226, 163, 78,
            172, 36, 245, 136, 58, 76, 0, 167, 16, 207, 132, 120, 38, 196, 51, 33, 94, 216, 58, 51,
            230, 254, 242, 58, 167, 16, 207, 132, 120, 38, 196, 51, 33, 222, 113, 219, 155, 77,
            227, 211, 49, 123, 238, 47, 91, 30, 84, 207, 41, 196, 51, 33, 158, 9, 241, 76, 136,
            103, 66, 60, 19, 226, 153, 16, 207, 132, 120, 38, 196, 123, 235, 217, 25, 196, 199,
            140, 71, 60, 59, 163, 59, 152, 16, 207, 132, 120, 38, 196, 187, 244, 217, 153, 238,
            175, 125, 157, 253, 185, 86, 112, 10, 241, 76, 136, 103, 66, 60, 19, 226, 205, 173, 51,
            227, 253, 196, 237, 38, 132, 83, 136, 103, 66, 60, 19, 226, 153, 16, 239, 201, 58, 51,
            181, 129, 76, 110, 55, 90, 195, 41, 196, 51, 33, 158, 9, 241, 76, 136, 55, 247, 217,
            153, 110, 67, 201, 185, 121, 152, 82, 243, 112, 114, 127, 217, 113, 221, 241, 179, 51,
            186, 131, 9, 241, 76, 136, 103, 66, 188, 169, 111, 179, 123, 250, 203, 22, 30, 86, 192,
            107, 241, 244, 127, 148, 243, 22, 135, 213, 173, 141, 78, 33, 158, 9, 241, 76, 136,
            103, 66, 188, 149, 235, 12, 226, 92, 198, 74, 119, 45, 59, 41, 149, 193, 79, 157, 66,
            60, 19, 226, 153, 16, 207, 132, 120, 151, 214, 153, 79, 219, 95, 114, 137, 57, 59, 147,
            134, 175, 180, 83, 136, 103, 66, 60, 19, 226, 153, 16, 111, 242, 202, 166, 63, 247,
            130, 212, 13, 114, 234, 30, 55, 91, 149, 83, 136, 103, 66, 60, 19, 226, 153, 16, 111,
            110, 157, 169, 235, 246, 151, 195, 11, 159, 94, 214, 221, 65, 188, 219, 41, 157, 66,
            60, 19, 226, 153, 16, 207, 132, 120, 75, 63, 10, 60, 163, 180, 111, 220, 164, 238, 143,
            244, 153, 162, 94, 145, 129, 168, 55, 155, 198, 156, 66, 60, 19, 226, 153, 16, 207,
            132, 120, 143, 227, 200, 65, 79, 221, 238, 47, 53, 69, 191, 20, 84, 78, 33, 158, 9,
            241, 76, 136, 103, 66, 188, 75, 55, 57, 248, 52, 41, 53, 239, 144, 109, 114, 221, 182,
            83, 136, 103, 66, 60, 19, 226, 153, 16, 111, 155, 117, 102, 120, 149, 84, 237, 190,
            176, 239, 182, 131, 106, 191, 226, 165, 59, 140, 50, 243, 171, 222, 199, 41, 196, 51,
            33, 158, 9, 241, 76, 136, 183, 203, 58, 211, 159, 231, 8, 218, 95, 250, 163, 218, 227,
            48, 198, 156, 66, 60, 19, 226, 153, 16, 207, 132, 120, 38, 196, 51, 33, 158, 9, 241,
            76, 136, 103, 66, 60, 19, 226, 153, 16, 207, 132, 120, 38, 196, 51, 33, 222, 46, 111,
            54, 17, 157, 231, 255, 191, 22, 185, 172, 187, 230, 219, 41, 196, 51, 33, 158, 9, 241,
            76, 136, 231, 58, 19, 163, 187, 141, 228, 21, 78, 33, 158, 9, 241, 76, 136, 103, 66,
            60, 215, 153, 32, 117, 217, 13, 175, 156, 66, 60, 19, 226, 153, 16, 207, 132, 120, 174,
            51, 11, 229, 215, 255, 233, 194, 111, 172, 113, 10, 241, 76, 136, 103, 66, 60, 19, 226,
            133, 173, 51, 53, 183, 247, 127, 74, 209, 175, 4, 150, 83, 136, 103, 66, 60, 19, 226,
            153, 16, 207, 132, 120, 38, 196, 51, 33, 158, 9, 241, 76, 136, 103, 66, 60, 19, 226,
            153, 16, 207, 132, 120, 38, 196, 251, 1, 201, 164, 87, 175,
        ]];
        let byte_width = 450;
        let result = parse_data(idat, byte_width);

        // result is too big so its stored in temp-file
        let mut tmp_file = NamedTempFile::new().expect("");
        writeln!(tmp_file, "{:?}", result);

        // correct file output is compared to test-file
        let output = Command::new("diff")
            .arg("./tests/fixtures/correct_rgb_idat")
            .arg(tmp_file.path())
            .output()
            .expect("failed to run diff-process");

        assert_eq!(output.stdout, []);
        assert_eq!(output.stderr, []);
    }
}
