use std::fs::File;
use std::io::prelude::*;
use yuv::YUV;
use yuv::convert::{RGBConvert, ToRGB};
use yuv::color::{Range, MatrixCoefficients};

pub fn save_bmp(filename: &str, buffer: &[u8], w: usize, h: usize) {
	let mut file = File::options()
							.write(true)
							.read(true)
							.create(true)
							.open(filename)
							.unwrap();

	let filesize = 54 + 3 * w * h;
	let mut img = vec![0u8; 3 * w * h];

	for i in 0..w {
		for j in 0..h {
			let x = i;
			let y = j;

			if (x + y * w) * 3 + 2 >= img.len() || (j * w + i) * 3 + 0 >= buffer.len() {
				continue;
			}

			img[(x + y * w) * 3 + 2] = buffer[(j * w + i) * 3 + 0];
            img[(x + y * w) * 3 + 1] = buffer[(j * w + i) * 3 + 1];
            img[(x + y * w) * 3 + 0] = buffer[(j * w + i) * 3 + 2];
		}
	}

	let mut bmpfileheader: [u8; 14] = [ 'B' as u8, 'M' as u8, 0, 0, 0, 0, 0, 0, 0, 0, 54, 0, 0, 0 ];
	let mut bmpinfoheader: [u8; 40] = [0u8; 40];
	let val_to_change: [u8; 16] = [ 40, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 24, 0 ];
	for (i, val) in val_to_change.iter().enumerate() {
		bmpinfoheader[i] = *val;
	}
	let bmppad: [u8; 3] = [ 0, 0, 0 ];

	bmpfileheader[2] = (filesize) as u8;
    bmpfileheader[3] = (filesize >> 8) as u8;
    bmpfileheader[4] = (filesize >> 16) as u8;
    bmpfileheader[5] = (filesize >> 24) as u8;

    bmpinfoheader[4] = (w) as u8;
    bmpinfoheader[5] = (w >> 8) as u8;
    bmpinfoheader[6] = (w >> 16) as u8;
    bmpinfoheader[7] = (w >> 24) as u8;
    bmpinfoheader[8] = (h) as u8;
    bmpinfoheader[9] = (h >> 8) as u8;
    bmpinfoheader[10] = (h >> 16) as u8;
    bmpinfoheader[11] = (h >> 24) as u8;

	file.write_all(&bmpfileheader).unwrap();
	file.write_all(&bmpinfoheader).unwrap();

	for i in 0..h {
		let start_index = w * (h - i - 1) * 3;
		let end_index = start_index + 3 * w;
		file.write_all(&img[start_index..end_index]).unwrap();
		file.write_all(&bmppad[0..((4 - (w * 3) % 4) % 4)]).unwrap();
	}
}

pub fn convert_and_push(converter: &RGBConvert, yuv_pix: YUV<u8>, result_arr: &mut Vec<u8>) {
    let rgb_pix = converter.to_rgb(yuv_pix);
    result_arr.push(rgb_pix.r);
    result_arr.push(rgb_pix.g);
    result_arr.push(rgb_pix.b);
}

pub fn convert_write_bmp(result: Vec<u8>) {
	let converter = RGBConvert::<u8>::new(
        Range::Full, 
        MatrixCoefficients::Identity
    ).unwrap();

    let mut rgb_result = Vec::<u8>::new();

    let arr_len = result.len();
    let step: usize = 4;
    for i in (0..arr_len).step_by(step) {
        let y1 = result[i];

        if i + 1 >= arr_len { break; }
        let u = result[i + 1];
        if i + 2 >= arr_len { break; }
        let y2 = result[i + 2];
        if i + 3 >= arr_len { break; }
        let v = result[i + 3];

        let yuv1 = YUV { y: y1, u, v };
        let yuv2 = YUV { y: y2, u, v };

        convert_and_push(&converter, yuv1, &mut rgb_result);
        convert_and_push(&converter, yuv2, &mut rgb_result);
    }
	save_bmp("test_frame.bmp", &rgb_result, 1280, 720);
}