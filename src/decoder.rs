use inflate::inflate_bytes_zlib;
use std::str;
use std::{fs::File, io::Read};

mod tests;
use crate::types::RGB;

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
    // assert!(result.bit_depth == 8, "images need bit-depth of 8");
    result
}

fn sub_filter(current_pixel: RGB, prev_pixel: RGB) -> RGB {
    RGB(
        ((current_pixel.0 as i16 + prev_pixel.0 as i16) % 256) as u8,
        ((current_pixel.1 as i16 + prev_pixel.1 as i16) % 256) as u8,
        ((current_pixel.2 as i16 + prev_pixel.2 as i16) % 256) as u8,
    )
}

fn up_filter(current_pixel: RGB, prev_row_pixel: RGB) -> RGB {
    RGB(
        ((current_pixel.0 as i16 + prev_row_pixel.0 as i16) % 256) as u8,
        ((current_pixel.1 as i16 + prev_row_pixel.1 as i16) % 256) as u8,
        ((current_pixel.2 as i16 + prev_row_pixel.2 as i16) % 256) as u8,
    )
}

fn avg_filter(current_pixel: RGB, prev_row_pixel: RGB, prev_pixel: RGB) -> RGB {
    RGB(
        ((current_pixel.0 as i16 + (prev_row_pixel.0 + prev_pixel.0) as i16 / 2) % 256) as u8,
        ((current_pixel.1 as i16 + (prev_row_pixel.1 + prev_pixel.1) as i16 / 2) % 256) as u8,
        ((current_pixel.2 as i16 + (prev_row_pixel.2 + prev_pixel.2) as i16 / 2) % 256) as u8,
    )
}

fn paeth_filter(current_pixel: RGB, prev_row_pixel: RGB, prev_pixel: RGB, diag_pixel: RGB) -> RGB {
    RGB(
        paeth_filter_single(
            current_pixel.0 as i16,
            prev_pixel.0 as i16,
            prev_row_pixel.0 as i16,
            diag_pixel.0 as i16,
        ),
        paeth_filter_single(
            current_pixel.1 as i16,
            prev_pixel.1 as i16,
            prev_row_pixel.1 as i16,
            diag_pixel.1 as i16,
        ),
        paeth_filter_single(
            current_pixel.2 as i16,
            prev_pixel.2 as i16,
            prev_row_pixel.2 as i16,
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

fn get_current_pixel(
    current_row: &[u8],
    line_pos: usize,
    plte: &Option<Vec<RGB>>,
    color_type: &ColorType,
) -> RGB {
    match color_type {
        ColorType::TrueColorRGB => RGB(
            current_row[line_pos],
            current_row[line_pos + 1],
            current_row[line_pos + 2],
        ),
        ColorType::Indexed => (*plte).as_ref().expect("plte exists on indexed color-type")
            [current_row[line_pos] as usize],
    }
}

fn get_prev_row_pixel(prev_row: &[RGB], line_pos: usize, num_channels: usize) -> RGB {
    RGB(
        prev_row[(line_pos - 1) / num_channels].0,
        prev_row[(line_pos - 1) / num_channels].1,
        prev_row[(line_pos - 1) / num_channels].2,
    )
}

fn get_diag_pixel(prev_row: &[RGB], line_pos: usize, num_channels: usize) -> RGB {
    if ((line_pos - 1) / num_channels) > 0 {
        RGB(
            prev_row[((line_pos - 1) / num_channels) - 1].0,
            prev_row[((line_pos - 1) / num_channels) - 1].1,
            prev_row[((line_pos - 1) / num_channels) - 1].2,
        )
    } else {
        RGB(0, 0, 0)
    }
}
fn apply_filter(
    current_row: &[u8],
    prev_row: &[RGB],
    filter: u8,
    color_type: &ColorType,
    plte: &Option<Vec<RGB>>,
) -> Vec<RGB> {
    let mut line_pos = 1;
    let mut prev_pixel = RGB(0, 0, 0);
    let mut result: Vec<RGB> = Vec::new();

    while line_pos < current_row.len() - (color_type.num_channels() - 1) {
        match filter {
            0 => result.push(get_current_pixel(current_row, line_pos, plte, color_type)),
            1 => {
                let pixel = sub_filter(
                    get_current_pixel(current_row, line_pos, plte, color_type),
                    prev_pixel,
                );
                result.push(pixel);
                prev_pixel = pixel;
            }
            2 => result.push(up_filter(
                get_current_pixel(current_row, line_pos, plte, color_type),
                get_prev_row_pixel(prev_row, line_pos, color_type.num_channels()),
            )),
            3 => {
                let pixel = avg_filter(
                    get_current_pixel(current_row, line_pos, plte, color_type),
                    get_prev_row_pixel(prev_row, line_pos, color_type.num_channels()),
                    prev_pixel,
                );
                result.push(pixel);
                prev_pixel = pixel;
            }
            4 => {
                let pixel = paeth_filter(
                    get_current_pixel(current_row, line_pos, plte, color_type),
                    get_prev_row_pixel(prev_row, line_pos, color_type.num_channels()),
                    prev_pixel,
                    get_diag_pixel(prev_row, line_pos, color_type.num_channels()),
                );
                result.push(pixel);
                prev_pixel = pixel;
            }
            _ => eprintln!("this filter doesn't exist"),
        }

        line_pos += color_type.num_channels();
    }
    result
}

fn parse_data(
    image_data: Vec<Vec<u8>>,
    meta_data: IHDRData,
    plte: Option<Vec<RGB>>,
) -> Vec<Vec<RGB>> {
    let mut inflated = Vec::new();
    for i in image_data {
        inflated.append(&mut inflate_bytes_zlib(&i as &[u8]).unwrap());
    }
    let mut rgb_img: Vec<Vec<RGB>> = Vec::new();
    let mut j = 0;
    let mut i = 0;
    let byte_width = meta_data.width as usize * meta_data.color_type.num_channels();
    let prev_row: &mut [RGB] = &mut vec![RGB(0, 0, 0); meta_data.width as usize];

    while j + byte_width < inflated.len() {
        let current_row = &inflated[j..j + byte_width + 1]; // + 1 for the line-filter
        let filter = current_row[0];

        rgb_img.push(apply_filter(
            current_row,
            prev_row,
            filter,
            &meta_data.color_type,
            &plte,
        ));

        prev_row.clone_from_slice(&rgb_img[i]);
        j += byte_width + 1;
        i += 1;
    }
    return rgb_img;
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
    let file_byte_size = File::metadata(&mut file).unwrap().len();
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
                rgb_img = parse_data(data, meta_data, plte);
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
