use inflate::inflate_bytes_zlib;
use std::str;
use std::{fs::File, io::Read};

mod tests;

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
    bytes.iter().map(|b| format!("{:02X}", b)).collect()
}

pub fn is_valid_png(file: &mut File) {
    const VALID_PNG: &str = "89504E470D0A1A0A";

    let first_bytes: &mut Vec<u8> = &mut vec![0; 8];
    let is_valid_png = File::read(file, first_bytes);
    if to_hex_string(first_bytes.to_vec()).join("") != VALID_PNG {
        eprintln!("pint: given file is not a valid png");
        std::process::exit(0);
    }
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

fn sub_filter(current_row: &[u8], line_pos: usize, prev_pixel: RGB) -> RGB {
    RGB(
        ((current_row[line_pos] as i16 + prev_pixel.0 as i16) % 256) as u8,
        ((current_row[line_pos + 1] as i16 + prev_pixel.1 as i16) % 256) as u8,
        ((current_row[line_pos + 2] as i16 + prev_pixel.2 as i16) % 256) as u8,
    )
}

fn up_filter(current_row: &[u8], line_pos: usize, prev_row: &[RGB]) -> RGB {
    RGB(
        ((current_row[line_pos] as i16 + prev_row[line_pos / 3].0 as i16) % 256) as u8,
        ((current_row[line_pos + 1] as i16 + prev_row[line_pos / 3].1 as i16) % 256) as u8,
        ((current_row[line_pos + 2] as i16 + prev_row[line_pos / 3].2 as i16) % 256) as u8,
    )
}
fn avg_filter(current_row: &[u8], line_pos: usize, prev_pixel: RGB, prev_row: &[RGB]) -> RGB {
    RGB(
        ((current_row[line_pos] as i16 + (prev_row[line_pos / 3].0 + prev_pixel.0) as i16 / 2)
            % 256) as u8,
        ((current_row[line_pos + 1] as i16 + (prev_row[line_pos / 3].1 + prev_pixel.1) as i16 / 2)
            % 256) as u8,
        ((current_row[line_pos + 2] as i16 + (prev_row[line_pos / 3].2 + prev_pixel.2) as i16 / 2)
            % 256) as u8,
    )
}

fn paeth_filter(current_row: &[u8], line_pos: usize, prev_pixel: RGB, prev_row: &[RGB]) -> RGB {
    let diag_pixel = if (line_pos / 3) > 0 {
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
            prev_pixel.0 as i16,
            prev_row[line_pos / 3].0 as i16,
            diag_pixel.0 as i16,
        ),
        paeth_filter_single(
            current_row[line_pos + 1] as i16,
            prev_pixel.1 as i16,
            prev_row[line_pos / 3].1 as i16,
            diag_pixel.1 as i16,
        ),
        paeth_filter_single(
            current_row[line_pos + 2] as i16,
            prev_pixel.2 as i16,
            prev_row[line_pos / 3].2 as i16,
            diag_pixel.2 as i16,
        ),
    )
}
fn paeth_filter_single(current_pixel: i16, prev_pixel: i16, up_pixel: i16, diag_pixel: i16) -> u8 {
    let p = prev_pixel + up_pixel - diag_pixel;
    let p_left_pixel = (p - prev_pixel).abs();
    let p_up_pixel = (p - up_pixel).abs();
    let p_diag_pixel = (p - diag_pixel).abs();

    let mut prediction = 0;
    if p_left_pixel <= p_up_pixel && p_left_pixel <= p_diag_pixel {
        prediction = prev_pixel;
    } else if p_up_pixel <= p_diag_pixel {
        prediction = up_pixel;
    } else {
        prediction = diag_pixel;
    }
    ((current_pixel as i16 + prediction as i16) % 256) as u8
}

fn add_to_array(current_row: &[u8], prev_row: &[RGB], rgb_img: &mut Vec<RGB>, filter: u8) {
    let mut line_pos = 1;
    let mut prev_pixel = RGB(0, 0, 0);

    while line_pos < current_row.len() - 2 {
        match filter {
            0 => rgb_img.push(RGB(
                current_row[line_pos],
                current_row[line_pos + 1],
                current_row[line_pos + 2],
            )),
            1 => {
                let pixel = sub_filter(current_row, line_pos, prev_pixel);
                rgb_img.push(pixel);
                prev_pixel = pixel;
            }
            2 => rgb_img.push(up_filter(current_row, line_pos, prev_row)),
            3 => {
                let pixel = avg_filter(current_row, line_pos, prev_pixel, prev_row);
                rgb_img.push(pixel);
                prev_pixel = pixel;
            }
            4 => {
                let pixel = paeth_filter(current_row, line_pos, prev_pixel, prev_row);
                rgb_img.push(pixel);
                prev_pixel = pixel;
            }
            _ => println!("this filter doesn't exist"),
        }

        line_pos += 3;
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RGB(u8, u8, u8);

fn parse_data(image_data: Vec<Vec<u8>>, pixel_width: usize) -> Vec<RGB> {
    let mut inflated = Vec::new();
    for i in image_data {
        inflated.append(&mut inflate_bytes_zlib(&i as &[u8]).unwrap());
    }
    let mut rgb_img: Vec<RGB> = Vec::new();
    let mut j = 0;
    let mut i = 0;
    let byte_width = pixel_width * 3; // bc 3 channels in rgb
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

pub fn decode_png(mut file: File) -> Vec<RGB> {
    let file_byte_size = File::metadata(&mut file).unwrap().len();
    let buf: &mut [u8] = &mut vec![0; (file_byte_size - 8) as usize]; // cut of beginning identifier sequence
    let result = File::read(&mut file, buf);
    let mut i: u32 = 0;
    let mut meta_data: IHDRData = IHDRData::default();
    let mut data: Vec<Vec<u8>> = Vec::new();
    let mut rgb_img: Vec<RGB> = vec![];

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
                rgb_img = parse_data(data, meta_data.width as usize);
                break;
            }
            _ => {
                i += 1;
                continue;
            }
        }
        i += chunk.chunk_len as u32;
    }
    return rgb_img;
}
