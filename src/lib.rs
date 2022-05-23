use std::path::Path;

use image::GenericImageView;

pub mod parser;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

// Compare two hashes and return a percent similarity
pub fn compare_hashes(hash1: usize, hash2: usize) -> u8 {
    let hash_xor = format!("{:#b}", (hash1 ^ hash2));
    let occurences: f64 = hash_xor.match_indices('1').count() as f64;
    (((64.0 - occurences) * 100.0) / 64.0) as u8
}

// Takes the bytes of an image, then generates a hash from their pixels
pub fn generate_hash<P>(path: P) -> Result<usize>
where
    P: AsRef<Path>,
{
    // Avoid panicking in unrecognized formats, return 0 for easy of ignoring
    let image_original = match image::open(path) {
        Ok(val) => val,
        Err(_err) => return Ok(0),
    };

    // Resize to 8x8px, ignore aspect ratio, convert to greyscale
    let img = image_original
        .resize_exact(8, 8, image::imageops::Lanczos3)
        .to_luma8();

    /*
    This is a lot of chained method calls, so here's a breakdown:
    - Get the image pixels, clone and map a closure on them that gets their value
    - Collect that into a Vec<u8>, then chunk that Vec<u8> into chunks of 8
    - Collect those chunks into a Vec<Vec<u8>>
    */
    let mut pixels: Vec<Vec<u8>> = img
        .pixels()
        .cloned()
        .map(|px| px[0])
        .collect::<Vec<u8>>()
        .chunks(8)
        .map(|chunk| chunk.to_vec())
        .collect();

    // Reverse every other chunk
    for i in (1..8).step_by(2) {
        let mut to_flip = pixels.remove(i);
        to_flip.reverse();
        pixels.insert(i, to_flip);
    }

    let mut prev_px = img.get_pixel(0, 7)[0];
    let mut diff_hash = 0;

    for pixel in pixels.concat() {
        diff_hash <<= 1;
        diff_hash |= (pixel >= prev_px) as usize;
        prev_px = pixel;
    }

    diff_hash |= {
        let (h, w) = image_original.dimensions();
        (h * w) as usize
    };

    Ok(diff_hash)
}
