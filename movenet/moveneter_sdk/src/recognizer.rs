//! Provides a interface for communicating with server-side application.

use std::net::{TcpStream, SocketAddr, AddrParseError};
use std::io::prelude::*;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;

use crate::error::RecogError;

const ENV_FILE_PATH: &str = "moveneter_sdk/env";

pub struct Recognizer {
    socket_addr: SocketAddr,
}

impl Recognizer {
    pub fn try_new_with(addr: &str) -> Result<Self, AddrParseError> {
        Ok(Recognizer { 
            socket_addr: addr.parse()?,
        })
    }

    pub fn try_new() -> Result<Self, AddrParseError> {
        let socket_addr = fs::read_to_string(ENV_FILE_PATH).unwrap();
        let socket_addr = socket_addr.trim();
        let socket_addr: SocketAddr = socket_addr.parse()?;
        Ok(Recognizer { socket_addr })
    }

    fn prepare_timestamp() -> u128 {
        let time = SystemTime::now();
        let since_the_epoch = time
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards");
        since_the_epoch.as_millis()
    }

    pub fn detect(&self, data: &Vec<u8>, frame_size: [u32; 2]) -> Result<Vec<f32>, RecogError> {
        let mut msg = "";
        
        let time_to_send = Recognizer::prepare_timestamp();

        if let Ok(mut stream) = TcpStream::connect(self.socket_addr) {
            stream.set_nodelay(true).unwrap();

            let timestamp_write = stream.write(&time_to_send.to_be_bytes()).is_ok();

            let frame_width_write = stream.write(
                frame_size[0].to_be_bytes().as_slice()
            ).is_ok();
            let frame_height_write = stream.write(
                frame_size[1].to_be_bytes().as_slice()
            ).is_ok();
            let frame_size_write = frame_width_write && frame_height_write;
            
            let len_write = stream.write(
                (data.len() as u64).to_be_bytes().as_slice()
            ).is_ok();
            let data_write = stream.write_all(
                data.as_slice()
            ).is_ok();

            stream.flush().unwrap();
            
            if timestamp_write && frame_size_write && len_write && data_write {
                let mut len_to_read = [0u8; 8];
                if stream.read_exact(&mut len_to_read).is_ok() {
                    // Number of elements of an array of f32 data.
                    let data_len = u64::from_be_bytes(len_to_read) as usize;
                    let mut data_buf = vec![0u8; data_len * 4];
                    if stream.read_exact(&mut data_buf).is_ok() {
                        let mut data_to_process: Vec<f32> = Vec::new();
                        for i in 0..data_len {
                            let start_idx = i * 4;
                            let end_idx = (i + 1) * 4;
                            data_to_process.push(f32::from_be_bytes(
                                data_buf[start_idx..end_idx]
                                    .try_into()
                                    .unwrap_or_default()
                            ));
                        }
                        return Ok(data_to_process);
                    }
                } else {
                    msg = "Failed to read out output data length.";
                    eprintln!("{}", msg);
                }
            } else {
                msg = "Failed to write data to the server.";
                eprintln!("{}", msg);
            }
        } else {
            msg = "Failed to connect to the server.";
            eprintln!("{}", msg);
        }
        Err(RecogError::new(msg))
    }
}
