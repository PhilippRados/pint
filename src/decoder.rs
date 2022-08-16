use inflate::inflate_bytes_zlib;
use std::str;
use std::{fs::File, io::Read};

mod tests;
use crate::types::RGB;

#[derive(Default)]
struct PngChunk {
    chunk_type: String,
    data_len: usize,
    chunk_len: usize,
    data: Vec<u8>,
    crc: Vec<u8>,
}

#[derive(PartialEq, Debug)]
enum ColorType {
    TrueColorRGB,
    Indexed,
}
impl ColorType {
    fn num_channels(&self) -> usize {
        match self {
            ColorType::TrueColorRGB => 3,
            ColorType::Indexed => 1,
        }
    }
}
struct IHDRData {
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: ColorType,
}

impl Default for IHDRData {
    fn default() -> IHDRData {
        IHDRData {
            width: 0,
            height: 0,
            bit_depth: 0,
            color_type: ColorType::TrueColorRGB,
        }
    }
}

fn to_hex_string(bytes: Vec<u8>) -> Vec<String> {
    bytes.iter().map(|b| format!("{:02X}", b)).collect()
}

pub fn check_valid_png(file: &mut File) {
    const VALID_PNG: &str = "89504E470D0A1A0A";

    let first_bytes: &mut Vec<u8> = &mut vec![0; 8];
    let _ = File::read(file, first_bytes);
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
    let chunk_types = ["IHDR", "PLTE", "IDAT", "IEND"];

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
        color_type: match data[9] {
            2 => ColorType::TrueColorRGB,
            3 => ColorType::Indexed,
            _ => panic!("Image Color-type is not Indexed or TrueColorRGB"),
        },
    };
    assert!(result.bit_depth == 8, "images need bit-depth of 8");
    result
}

#[derive(Clone, Copy, Debug)]
enum RGBorU8 {
    Rgb(RGB),
    U8(u8),
}

fn get_current_pixel(
    current_row: &[u8],
    line_pos: usize,
    plte: &Option<Vec<RGB>>,
    color_type: &ColorType,
) -> RGBorU8 {
    match color_type {
        ColorType::TrueColorRGB => RGBorU8::Rgb(RGB(
            current_row[line_pos],
            current_row[line_pos + 1],
            current_row[line_pos + 2],
        )),
        ColorType::Indexed => RGBorU8::U8(current_row[line_pos]),
    }
}

fn none(current_pixel: u8, prev_pixel: u8, up_pixel: u8, diag_pixel: u8) -> u8 {
    current_pixel
}
fn sub_filter(current_pixel: u8, prev_pixel: u8, up_pixel: u8, diag_pixel: u8) -> u8 {
    // can safely cast back to u8 because % 256 <= 255
    ((current_pixel as u16 + prev_pixel as u16) % 256_u16) as u8
}
fn up_filter(current_pixel: u8, prev_pixel: u8, up_pixel: u8, diag_pixel: u8) -> u8 {
    ((current_pixel as u16 + up_pixel as u16) % 256_u16) as u8
}
fn avg_filter(current_pixel: u8, prev_pixel: u8, up_pixel: u8, diag_pixel: u8) -> u8 {
    ((current_pixel as i16 + (up_pixel + prev_pixel) as i16 / 2) % 256) as u8
}
fn paeth_filter(current_pixel: u8, prev_pixel: u8, up_pixel: u8, diag_pixel: u8) -> u8 {
    let p = prev_pixel as i16 + up_pixel as i16 - diag_pixel as i16;
    let p_left_pixel = (p - prev_pixel as i16).abs();
    let p_up_pixel = (p - up_pixel as i16).abs();
    let p_diag_pixel = (p - diag_pixel as i16).abs();

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

fn do_filter(
    i: usize,
    current: RGBorU8,
    prev: &mut RGBorU8,
    up: RGBorU8,
    diag: RGBorU8,
    plte: &Option<Vec<RGB>>,
) -> RGB {
    let filter = [none, sub_filter, up_filter, avg_filter, paeth_filter];
    assert!(i < filter.len(), "No such filter exists");

    match current {
        RGBorU8::Rgb(c) => {
            let p = match prev {
                RGBorU8::Rgb(p) => p,
                _ => panic!("RGB values dont store u8"),
            };

            let u = match up {
                RGBorU8::Rgb(u) => u,
                _ => panic!("RGB values dont store u8"),
            };
            let d = match diag {
                RGBorU8::Rgb(d) => d,
                _ => panic!("RGB values dont store u8"),
            };
            let applied = RGB(
                filter[i](c.0, p.0, u.0, d.0),
                filter[i](c.1, p.1, u.1, d.1),
                filter[i](c.2, p.2, u.2, d.2),
            );
            *prev = RGBorU8::Rgb(applied);
            applied
        }
        RGBorU8::U8(c) => {
            let p = match prev {
                RGBorU8::U8(p) => *p,
                _ => panic!("Indexed values dont store RGB"),
            };
            let u = match up {
                RGBorU8::U8(u) => u,
                _ => panic!("U8 values dont store RGB"),
            };
            let d = match diag {
                RGBorU8::U8(d) => d,
                _ => panic!("U8 values dont store RGB"),
            };

            let applied = filter[i](c, p, u, d);
            *prev = RGBorU8::U8(applied);
            (*plte).as_ref().expect("plte exists on indexed color-type")[applied as usize]
        }
    }
}

fn get_diag_pixel(
    prev_row: &[RGBorU8],
    line_pos: usize,
    default_val: RGBorU8,
    color_type: &ColorType,
) -> RGBorU8 {
    if (line_pos as i16 / color_type.num_channels() as i16) > 0 {
        prev_row[(line_pos / color_type.num_channels()) - 1]
    } else {
        default_val
    }
}

fn apply_filter(
    current_row: &[u8],
    prev_row: &mut Vec<RGBorU8>,
    filter_index: usize,
    color_type: &ColorType,
    plte: &Option<Vec<RGB>>,
    default_val: RGBorU8,
) -> Vec<RGB> {
    let mut line_pos = 0;
    let mut prev_pixel = default_val;
    let mut result: Vec<RGB> = Vec::new();
    let mut current_row_applied = Vec::new();

    while line_pos < current_row.len() - (color_type.num_channels() - 1) {
        let pixel = get_current_pixel(current_row, line_pos, plte, color_type);
        let up = prev_row[line_pos / color_type.num_channels()];
        let diag = get_diag_pixel(prev_row, line_pos, default_val, color_type);

        result.push(do_filter(
            filter_index,
            pixel,
            &mut prev_pixel,
            up,
            diag,
            plte,
        ));
        current_row_applied.push(prev_pixel);

        line_pos += color_type.num_channels();
    }
    *prev_row = current_row_applied;

    result
}

fn parse_data(
    image_data: Vec<Vec<u8>>,
    meta_data: IHDRData,
    plte: Option<Vec<RGB>>,
    default_val: RGBorU8,
) -> Vec<Vec<RGB>> {
    let mut inflated = Vec::new();
    for i in image_data {
        inflated.append(&mut inflate_bytes_zlib(&i as &[u8]).unwrap());
    }
    let mut rgb_img: Vec<Vec<RGB>> = Vec::new();
    let mut j = 0;
    let byte_width = meta_data.width as usize * meta_data.color_type.num_channels();
    let mut prev_row: Vec<RGBorU8> = vec![default_val; meta_data.width as usize];

    let mut i = 0;
    while j + byte_width < inflated.len() {
        i += 1;
        let filter = inflated[j] as usize;
        let current_row = &inflated[j + 1..j + byte_width + 1]; // + 1 for the line-filter

        rgb_img.push(apply_filter(
            current_row,
            &mut prev_row,
            filter,
            &meta_data.color_type,
            &plte,
            default_val,
        ));

        j += byte_width + 1;
    }

    rgb_img
}

fn parse_plte(data: Vec<u8>) -> Vec<RGB> {
    assert!(data.len() % 3 == 0, "data should be splittable in triplets");

    let mut result: Vec<RGB> = Vec::new();
    for i in (0..data.len()).step_by(3) {
        result.push(RGB(data[i], data[i + 1], data[i + 2]))
    }
    result
}

pub fn decode_png(mut file: File) -> Vec<Vec<RGB>> {
    let file_byte_size = File::metadata(&file).unwrap().len();
    let buf: &mut [u8] = &mut vec![0; (file_byte_size - 8) as usize]; // cut of beginning identifier sequence
    let result = File::read(&mut file, buf);
    let mut i: u32 = 0;
    let mut meta_data: IHDRData = IHDRData::default();
    let mut plte: Option<Vec<RGB>> = None;
    let mut data: Vec<Vec<u8>> = Vec::new();
    let mut rgb_img: Vec<Vec<RGB>> = Vec::new();

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
            "PLTE" => plte = Some(parse_plte(chunk.data)),
            "IDAT" => {
                if meta_data.color_type == ColorType::Indexed && plte == None {
                    // if color is indexed then it needs a palette
                    panic!("pint: couldn't find PLTE chunk in image.");
                }
                data.push(chunk.data);
            }
            "IEND" => {
                let default_val = match meta_data.color_type {
                    ColorType::TrueColorRGB => RGBorU8::Rgb(RGB(0, 0, 0)),
                    ColorType::Indexed => RGBorU8::U8(0),
                };
                rgb_img = parse_data(data, meta_data, plte, default_val);

                break;
            }
            _ => {
                i += 1;
                continue;
            }
        }
        i += chunk.chunk_len as u32;
    }
    rgb_img
}
pub fn infer_codel_size(rgb_img: &Vec<Vec<RGB>>) -> i32 {
    let mut min_size = i32::MAX;
    let mut current_size = 1i32;
    let mut current_color: RGB;

    // go from left to right through img
    for y in 0..rgb_img.len() {
        let mut prev_color: RGB = rgb_img[y][0];
        for x in 1..rgb_img[0].len() {
            current_color = rgb_img[y][x];
            if prev_color == current_color {
                current_size += 1;
            } else if current_size < min_size {
                min_size = current_size;
                current_size = 1;
            }
            prev_color = current_color;
        }
    }
    // go from top to bottom through img
    for x in 0..rgb_img[0].len() {
        let mut prev_color: RGB = rgb_img[0][x];
        for y in 1..rgb_img.len() {
            current_color = rgb_img[y][x];
            if prev_color == current_color {
                current_size += 1;
            } else if current_size < min_size {
                min_size = current_size;
                current_size = 1;
            }
            prev_color = current_color;
        }
    }
    if min_size < 1 {
        eprintln!("warning: inferred codel-size less than 1 => defaults to 1");
        1
    } else if rgb_img[0].len() as i32 % min_size != 0 || rgb_img.len() as i32 % min_size != 0 {
        eprintln!("warning: inferred codel-size doesnt fit image-dimensions => defaults to 1");
        1
    } else {
        min_size
    }
}
