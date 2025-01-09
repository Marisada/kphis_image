use image::{imageops::FilterType, ImageReader, ImageFormat, ImageResult};
use std::{
    cmp::Ordering,
    io::Cursor,
};

// const MAX_WIDTH: u32 = 720; // 9x64=576, 9x80=720, 9x96=864
// const MAX_HEIGHT: u32 = 1280; // 16x64=1024, 16x80=1280, 16*96=1536
// const THUMB_WIDTH: u32 = 144; // 9x16
// const THUMB_HEIGHT: u32 = 256; // 16x16

const IMAGE_SIZE: u32 = 1024;
const THUMB_SIZE: u32 = 128;

pub fn image_bytes_parser(raw_data: &[u8]) -> ImageResult<(Vec<u8>, Vec<u8>)> {
    let raw_image = ImageReader::new(Cursor::new(raw_data)).with_guessed_format()?.decode()?;
    let raw_w = raw_image.width();
    let raw_h = raw_image.height();
    let image = if raw_w > IMAGE_SIZE || raw_h > IMAGE_SIZE {
        // see filters detail at https://docs.rs/image/latest/image/imageops/enum.FilterType.html
        raw_image.resize(IMAGE_SIZE, IMAGE_SIZE, FilterType::Triangle)
    } else {
        raw_image
    };
    let mut res_image = Vec::new();
    image.write_to(&mut Cursor::new(&mut res_image), ImageFormat::WebP)?;

    let img_w = image.width();
    let img_h = image.height();
    let cubic = match img_h.cmp(&img_w) {
        Ordering::Equal => image,
        Ordering::Greater => image.crop_imm(0, (img_h - img_w) / 2, img_w, img_w),
        Ordering::Less => image.crop_imm((img_w - img_h) / 2, 0, img_h, img_h),
    };
    let thumb = cubic.thumbnail(THUMB_SIZE, THUMB_SIZE);
    let mut res_thumb = Vec::new();
    thumb.write_to(&mut Cursor::new(&mut res_thumb), ImageFormat::WebP)?;

    Ok((res_image, res_thumb))
}