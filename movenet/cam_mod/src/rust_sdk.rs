use kernel::prelude::*;
use kernel::sync::smutex::Mutex;
use core::result::Result::Ok;
use kernel::net::{
    self, 
    SocketAddr,
    SocketAddrV4, 
    Ipv4Addr, 
    TcpListener
};

use kernel::bindings::{
    timespec64,
    ktime_get_real_ts64
};

module! {
    type: RustSdk,
    name: "rust_sdk",
    author: "Yibo Yan",
    description: "A simple module that runs the server-facing logics",
    license: "GPL",
}

#[allow(dead_code)]
pub static SDK_ACCESSOR: Mutex<RustSdk> = Mutex::new(RustSdk {
    socket_addr: SocketAddr::V4(
        SocketAddrV4::new(
            Ipv4Addr::LOOPBACK,
            11111 as u16
        )
    )
});

pub struct RustSdk {
    socket_addr: SocketAddr,
}

impl RustSdk {
    pub fn set_server_addr(&mut self, addr: Ipv4Addr, port: u16) {
        self.socket_addr = SocketAddr::V4( 
            SocketAddrV4::new(
                addr, port
            )
        );
    }

    fn prepare_timestamp() -> u128 {
        let mut time: timespec64 = Default::default();
        unsafe {
            ktime_get_real_ts64(&mut time);
        }
        let time_millis = (1000 * time.tv_sec) +  (time.tv_nsec / 1000_000);
        time_millis as u128
    }

    pub fn detect(
        &self, 
        data: &Vec<u8>, 
        frame_size: [u32; 2]
    ) -> Result<Vec<f32>> {
        // let mut msg = "";
        
        let time_to_send = RustSdk::prepare_timestamp();
        let listener = TcpListener::try_new(net::init_ns(), &self.socket_addr)?;

        let stream = listener.accept(false)?;

        stream.write(&time_to_send.to_be_bytes(), true)?;

        stream.write(
            frame_size[0].to_be_bytes().as_slice(),
            true
        )?;

        stream.write(
            frame_size[1].to_be_bytes().as_slice(),
            true
        )?;
        
        stream.write(
            (data.len() as u64).to_be_bytes().as_slice(),
            true
        )?;

        stream.write(
            data.as_slice(),
            true
        )?;
        
        let mut len_to_read = [0u8; 8];
        stream.read(&mut len_to_read, true)?;
        // Number of elements of an array of f32 data.
        let data_len = u64::from_be_bytes(len_to_read) as usize;
        let mut data_buf: Vec<u8> = Vec::new();
        data_buf.try_resize(data_len * 4, 0u8)?;
        stream.read(&mut data_buf, true)?;
        let mut data_to_process: Vec<f32> = Vec::new();
        for i in 0..data_len {
            let start_idx = i * 4;
            let end_idx = (i + 1) * 4;
            data_to_process.try_push(
                f32::from_be_bytes(
                    data_buf[start_idx..end_idx]
                        .try_into()
                        .unwrap_or_default()
                )
            )?;
        }
        return Ok(data_to_process);
    }
}

impl kernel::Module for RustSdk {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("RustSdk (init)\n");

        Ok(RustSdk {
            socket_addr: SocketAddr::V4(
                SocketAddrV4::new(
                    Ipv4Addr::LOOPBACK,
                    11111 as u16
                )
            )
        })
    }
}

impl Drop for RustSdk {
    fn drop(&mut self) {
        pr_info!("RustSdk (exit)\n");
    }
}
