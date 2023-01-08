use std::io;
use std::sync::Arc;
use std::sync::mpsc;
use opencv::core::{flip, CV_8UC3};
use opencv::{
	prelude::*,
	highgui::*,
};

use app::utils::*;
use app::v4l2;
use moveneter_sdk::recognizer::Recognizer;
use shared::threadpool::ThreadPool;
use std::time::SystemTime;

const THRESHOLD: f32 = 0.25;
const N_WORKERS: usize = 20;
const TIME_INTERVEL: u128 = 150;
const SHOW_PREVIEW: bool = false;

fn main() -> io::Result<()> {
    let recog = Arc::new(Recognizer::try_new().unwrap());
    let (tx, rx) = mpsc::channel();
    let pool = ThreadPool::new(N_WORKERS);

    let mut cam = v4l2::VideoCapture::new().expect("Failed to open camera.");
    let (frame_width, frame_height) = cam.prep_stream(
        Some(1),
        None
    );
    
    let mut frame = unsafe {
        Mat::new_rows_cols(
            frame_height as i32, 
            frame_width as i32, 
            CV_8UC3
        ).unwrap()
    };
    let mut flipped = Mat::default();
    let mut throttle_timer = SystemTime::now();
    let mut rgb_data: Vec<u8>;

    loop {
        let out = cam.read().unwrap();

        if SHOW_PREVIEW {
            let converter = shared::utils::EasyConverter::new();
            rgb_data = converter.rgb(&out);
            
            unsafe {
                frame.set_data(rgb_data.as_mut_ptr());
            }
        }

        if frame_width > 0 {
			flip(&frame, &mut flipped, 1).expect("flip [FAILED]");

            if SystemTime::now().duration_since(throttle_timer).unwrap().as_millis() > TIME_INTERVEL {
                let recog = Arc::clone(&recog);
                let job_tx = tx.clone();
                pool.execute(move || {
                    if let Ok(data) = recog.detect(&out, [frame_width, frame_height]) {
                        job_tx.send(data.clone()).unwrap();
                    }
                });
                throttle_timer = SystemTime::now();
            }

            let mut data_out = Vec::<f32>::default();
            while let Ok(data) = rx.try_recv() {
                data_out = data;
            }

            if !data_out.is_empty() {
                draw_keypoints(
                    &mut flipped, data_out.as_slice(), THRESHOLD
                );
            }

			imshow("MoveNet", &flipped).expect("imshow [ERROR]");
		}

		// keypress check
		let key = wait_key(1).unwrap();
		if key > 0 && key != 255 {
			break;
		}
    }
    println!("Exiting...");
    Ok(())
}
