use std::ffi::c_void;
use opencv::core::{flip, Vec3b, Mat_AUTO_STEP};
use opencv::{
	prelude::*,
	imgproc::*,
	core::*,
};

pub fn resize_with_padding(
	data: &mut Vec<u8>, rows: i32, cols: i32, new_shape: [i32; 2]
) -> Vec<u8> {
	let frame = unsafe {
		Mat::new_rows_cols_with_data(
			rows, 
			cols, 
			CV_8UC3, 
			data.as_mut_ptr() as *mut c_void,
			Mat_AUTO_STEP
		).unwrap()
	};

	let mut img = Mat::default();

	flip(&frame, &mut img, 1).expect("flip [FAILED]");

	let img_shape = [img.cols(), img.rows()];

	let width: i32;
	let height: i32;
	if img_shape[0] as f64 / img_shape[1] as f64 > new_shape[0] as f64 / new_shape[1] as f64 {
		width = new_shape[0];
		height = (new_shape[0] as f64 / img_shape[0] as f64 * img_shape[1] as f64) as i32;
	} else {
		width = (new_shape[1] as f64 / img_shape[1] as f64 * img_shape[0] as f64) as i32;
		height = new_shape[1];
	}

	let mut resized = Mat::default();
	resize(
		&img,
		&mut resized,
		Size { width, height },
		0.0, 0.0,
		INTER_LINEAR
	)
	.expect("resize_with_padding: resize [FAILED]");

	let delta_w = new_shape[0] - width;
	let delta_h = new_shape[1] - height;
	let (top, bottom) = (delta_h / 2, delta_h - delta_h / 2);
	let (left, right) = (delta_w / 2, delta_w - delta_w / 2);
		
	let mut rslt = Mat::default();
	copy_make_border(
		&resized,
		&mut rslt,
		top, bottom, left, right,
		BORDER_CONSTANT,
		Scalar::new(0.0, 0.0, 0.0, 0.0))
		.expect("resize_with_padding: copy_make_border [FAILED]");
	
	let vec_2d: Vec<Vec<Vec3b>> = rslt.to_vec_2d().unwrap();
	let vec_1d: Vec<u8> = vec_2d.iter().flat_map(|v| 
		v.iter().flat_map(|w| 
			w.as_slice()
		)
	).cloned().collect();

	vec_1d
}
