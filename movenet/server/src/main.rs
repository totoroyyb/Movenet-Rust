use std::net::{TcpListener, TcpStream};
use std::sync::Mutex;
use std::{
    io::{self, prelude::*}, 
    env
};
use server::utils;
use tflitec::interpreter::{Interpreter, Options};
use shared::threadpool::ThreadPool;
use std::time::Duration;
use std::thread;
use std::cmp::{min, max};

const N_WORKERS: usize = 10;

// in millseconds
static TIME_INTERVAL: Mutex<u128> = Mutex::new(500);
const INTERVEL_STEP: u128 = 100;
const INTERVEL_UPPER: u128 = 2000;
const INTERVEL_LOWER: u128 = 0;

static LATEST_TIMESTAMP: Mutex<Option<u128>> = Mutex::new(None);

static COUNTER: Mutex<ExecutionCounter> = Mutex::new(ExecutionCounter::new());

struct ExecutionCounter {
    num_running: u64,
}

impl ExecutionCounter {
    const fn new() -> Self {
        Self { num_running: 0 }
    }

    fn mark_received(&mut self) {
        self.num_running += 1;
    }

    fn mark_finished(&mut self) {
        self.num_running -= 1;
    }

    fn get_num_running(&self) -> u64 {
        self.num_running
    }
}

fn should_drop(sender_timestamp: u128) -> bool {
    let mut will_drop = false;

    let mut latest_timestamp = LATEST_TIMESTAMP.lock().unwrap();
    if let Some(timestamp) = *latest_timestamp {
        will_drop = sender_timestamp < (timestamp + *TIME_INTERVAL.lock().unwrap());
    }

    if !will_drop {
        *latest_timestamp = Some(sender_timestamp);
    }

    will_drop
}

fn handle_client(
    mut stream: TcpStream
) -> io::Result<()> {
    // 1. read timestamp: `u128` as `be_bytes`.
    // 2. read upcoming data length: `u64` as `be_bytes`
    // 3. read all data by the data length
    // 4. check the timestamp to decide whether to drop this request.
    // 5. process the data with Tensorflow
    // 6. extract the data part with output_tensor.data::<f32>()
    // 7. write data length back: `u64` as `be_bytes`
    // 8. write data back

    // println!("Started to handle client - {}", stream.peer_addr().unwrap());

    stream.set_nodelay(true).unwrap();

    let mut timestamp = [0u8; 16];
    stream.read_exact(&mut timestamp)?;

    let mut width = [0u8; 4];
    stream.read_exact(&mut width)?;
    let width = u32::from_be_bytes(width) as i32;

    let mut height = [0u8; 4];
    stream.read_exact(&mut height)?;
    let height = u32::from_be_bytes(height) as i32;

    let mut data_len = [0u8; 8];
    stream.read_exact(&mut data_len)?;
    let data_len = u64::from_be_bytes(data_len) as usize;

    let mut data_in = vec![0u8; data_len];
    stream.read_exact(&mut data_in)?;
    
    let converter = shared::utils::EasyConverter::new();
    let mut data_in = converter.rgb(&data_in);
    let data_in = utils::resize_with_padding(
        &mut data_in, height, width, [192, 192]
    );

    if should_drop(u128::from_be_bytes(timestamp)) {
        // println!("Dropped request.");
        stream.write(0u64.to_be_bytes().as_slice())?;
        return Ok(());
    }

    let mut options = Options::default();
    options.thread_count = 5;
	let path = format!("resource/lite-model_movenet_singlepose_lightning_tflite_int8_4.tflite");
    let interpreter = Interpreter::with_model_path(&path, Some(options)).unwrap();
    interpreter.allocate_tensors().expect("Allocate tensors [FAILED]");

    interpreter.copy(&data_in[..], 0).unwrap();
    
    // run interpreter
    interpreter.invoke().expect("Invoke [FAILED]");

    let output_tensor = interpreter.output(0).unwrap();
    let data_out = output_tensor.data::<f32>();

    stream.write(
        (data_out.len() as u64).to_be_bytes().as_slice()
    )?;
    
    let data_out: Vec<u8> = data_out.iter().flat_map(|d| 
        d.to_be_bytes()
    ).collect();
    stream.write_all(&data_out)?;
    stream.flush()?;
    
    // println!("Finished handling");
    Ok(())
}

fn adjust_time_interval() {
    let mut current_interval = TIME_INTERVAL.lock().unwrap();
    // println!("Running: {}", COUNTER.lock().unwrap().get_num_running());
    if COUNTER.lock().unwrap().get_num_running() > 10 {
        // Increase interval to throttle.
        *current_interval = min(*current_interval + INTERVEL_STEP, INTERVEL_UPPER);
    } else {
        // Decrease interval to speed up.
        // Avoid overflow
        if *current_interval < INTERVEL_STEP {
            *current_interval = INTERVEL_LOWER
        } else {
            *current_interval = max(*current_interval - INTERVEL_STEP, INTERVEL_LOWER);
        }
    }
}

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("No Ip address is supplied. Format: IP_ADDR:PORT");
        return Err(
            io::Error::new(
                io::ErrorKind::Other, "Arguments are missing."
            )
        );
    }

    let listener = TcpListener::bind(&args[1])?;
    listener.set_nonblocking(true).expect("Cannot set non-blocking");

    let local_addr = listener.local_addr()?;
    println!("Listening to local address: {}", local_addr);

    let pool = ThreadPool::new(N_WORKERS);

    // accept connections and process them serially
    for stream in listener.incoming() {
        match stream {
            Ok(s) => {
                {
                    COUNTER.lock().unwrap().mark_received();
                }
                
                pool.execute(move || {
                    if let Err(e) = handle_client(s) {
                        println!("Potential error occurs in the server. Message: {}.", e);
                    }
                    {
                        COUNTER.lock().unwrap().mark_finished();
                        adjust_time_interval()
                    }
                });
            }
            Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                // wait until network socket is ready, typically implemented
                // via platform-specific APIs such as epoll or IOCP
                thread::sleep(Duration::from_millis(50));
                continue;
            }
            Err(e) => panic!("encountered IO error: {e}"),
        }
    }
    Ok(())
}
