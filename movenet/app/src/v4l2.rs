//! Rust port of V4L2 support with `ioctl` invocation.
//! 
//! Provides a class `VideoCapture` to interact with video driver from userspace program.

use std::{
    fs::File, 
    os::unix::prelude::AsRawFd, 
    str, 
    io::{self, Write, Error, Seek, SeekFrom, Read}, 
    ptr, 
    ffi::{c_void, c_int}
};
use nix::{
    sys::{
        mman::{self, ProtFlags, MapFlags},
        select,
        time
    },
    libc,
    errno::Errno,
};

use crate::pagemap;

// #define VIDIOC_QUERYCAP		 _IOR('V',  0, struct v4l2_capability)
const VIDIOC_QUERYCAP_MAGIC: u8 = 'V' as u8;
const VIDIOC_QUERYCAP_TYPE_MODE: u8 = 0;

// #define VIDIOC_G_FMT		     _IOWR('V',  4, struct v4l2_format)
const VIDIOC_G_FMT_MAGIC: u8 = 'V' as u8;
const VIDIOC_G_FMT_TYPE_MODE: u8 = 4;

// #define VIDIOC_S_FMT		     _IOWR('V',  5, struct v4l2_format)
const VIDIOC_S_FMT_MAGIC: u8 = 'V' as u8;
const VIDIOC_S_FMT_TYPE_MODE: u8 = 5;

// #define VIDIOC_S_PARM		 _IOWR('V', 22, struct v4l2_streamparm)
const VIDIOC_S_PARM_MAGIC: u8 = 'V' as u8;
const VIDIOC_S_PARM_TYPE_MODE: u8 = 22;

// #define VIDIOC_G_PARM		 _IOWR('V', 21, struct v4l2_streamparm)
const VIDIOC_G_PARM_MAGIC: u8 = 'V' as u8;
const VIDIOC_G_PARM_TYPE_MODE: u8 = 21;

// #define VIDIOC_REQBUFS		_IOWR('V',  8, struct v4l2_requestbuffers)
const VIDIOC_REQBUFS_MAGIC: u8 = 'V' as u8;
const VIDIOC_REQBUFS_TYPE_MODE: u8 = 8;

// #define VIDIOC_QUERYBUF		_IOWR('V',  9, struct v4l2_buffer)
const VIDIOC_QUERYBUF_MAGIC: u8 = 'V' as u8;
const VIDIOC_QUERYBUF_TYPE_MODE: u8 = 9;

// #define VIDIOC_STREAMON		 _IOW('V', 18, int)
const VIDIOC_STREAMON_MAGIC: u8 = 'V' as u8;
const VIDIOC_STREAMON_TYPE_MODE: u8 = 18;

// #define VIDIOC_STREAMOFF	 _IOW('V', 19, int)
const VIDIOC_STREAMOFF_MAGIC: u8 = 'V' as u8;
const VIDIOC_STREAMOFF_TYPE_MODE: u8 = 19;

// #define VIDIOC_QBUF		_IOWR('V', 15, struct v4l2_buffer)
const VIDIOC_QBUF_MAGIC: u8 = 'V' as u8;
const VIDIOC_QBUF_TYPE_MODE: u8 = 15;

// #define VIDIOC_DQBUF		_IOWR('V', 17, struct v4l2_buffer)
const VIDIOC_DQBUF_MAGIC: u8 = 'V' as u8;
const VIDIOC_DQBUF_TYPE_MODE: u8 = 17;

const V4L2_BUF_TYPE_VIDEO_CAPTURE: u32 = 1;
const V4L2_MEMORY_MMAP: u32 = 1;
const MAX_V4L_BUFFERS: usize = 10;
const DEFAULT_STREAM_FPS: usize = 30;

static DEVICE_FILE_PATH: &'static str = "/dev/camdriver";

#[repr(C)]
#[derive(Default)]
pub struct v4l2_capability {
    pub driver: [u8; 16],
    pub card: [u8; 32],
    pub bus_info: [u8; 32],
    pub version: u32,
    pub capabilities: u32,
    pub device_caps: u32,
    pub reserved: [u32; 3],
}

#[repr(C)]
#[derive(Copy)]
#[derive(Clone)]
pub struct v4l2_pix_format {
    pub width: u32,
    pub height: u32,
    pub pixelformat: u32,
    pub others: [u8; 200 - 3*4]
}

impl Default for v4l2_pix_format {
    fn default() -> Self {
        Self { 
            width: Default::default(), 
            height: Default::default(), 
            pixelformat: Default::default(), 
            others: [0u8; 200 - 3*4]
        }
    }
}

#[repr(C)]
#[derive(Copy)]
#[derive(Clone)]
pub struct v4l2_pix_format_mplane {
    pub width: u32,
    pub height: u32,
    pub pixelformat: u32,
    pub others: [u8; 200 - 3*4]
}

impl Default for v4l2_pix_format_mplane {
    fn default() -> Self {
        Self { 
            width: Default::default(), 
            height: Default::default(), 
            pixelformat: Default::default(), 
            others: [0u8; 200 - 3*4]
        }
    }
}

#[repr(C)]
pub enum Fmt {
    Pix(v4l2_pix_format)
}

impl Default for Fmt {
    fn default() -> Self {
        Self::Pix(Default::default())
    }
}

#[repr(C)]
pub struct v4l2_format {
    pub r#type: u32,
    pub fmt: Fmt,
}

impl Default for v4l2_format {
    fn default() -> Self {
        Self { 
            r#type: Default::default(), 
            fmt: Default::default()
        }
    }
}

#[repr(C)]
#[derive(Default)]
#[derive(Copy, Clone)]
pub struct v4l2_fract {
    pub numerator: u32,
    pub denominator: u32,
}

#[repr(C)]
#[derive(Default)]
#[derive(Copy, Clone)]
pub struct v4l2_captureparm {
    pub capability: u32,
    pub capturemode: u32,
    pub timeperframe: v4l2_fract,
    pub extendedmode: u32,
    pub readbuffers: u32,
    pub reserved: [u32; 4]
}

#[repr(C)]
pub union Parm {
    pub capture: v4l2_captureparm,
    pub raw_data: [u8; 200]
}

impl Default for Parm {
    fn default() -> Self {
        Self {
            capture: Default::default()
        }
    }
}

#[repr(C)]
pub struct v4l2_streamparm {
    pub r#type: u32,
    pub capability: u32,
    pub capturemode: u32,
    pub numerator: u32,
    pub denominator: u32,
    pub extendedmode: u32,
    pub readbuffers: u32,
    pub others: [u8; 204 - 7 * 4]
}

impl Default for v4l2_streamparm {
    fn default() -> Self {
        Self { 
            r#type: Default::default(), 
            capability: Default::default(), 
            capturemode: Default::default(), 
            numerator: Default::default(), 
            denominator: Default::default(), 
            extendedmode: Default::default(), 
            readbuffers: Default::default(), 
            others: [0u8; 204 - 7 * 4] 
        }
    }
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct timeval {
    tv_sec: i64,
    tv_usec: i64
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct v4l2_timecode {
    r#type: u32,
    flags: u32,
    frames: u8,
    seconds: u8,
    minutes: u8,
    hours: u8,
    userbits: [u8; 4]
}

#[repr(C)]
#[derive(Default, Clone, Copy, Debug)]
pub struct v4l2_buffer {
    pub index: u32,
    pub r#type: u32,
    pub bytesused: u32,
    pub flags: u32,
    pub field: u32,
    pub timestamp: timeval,
    pub timecode: v4l2_timecode,
    pub sequence: u32,
    pub memory: u32,
    pub offset: u32,
    pub aligned: u32,
    pub length: u32,
    pub reserved2: u32,
    pub reserved: u32,
}

#[repr(C)]
#[derive(Default)]
pub struct v4l2_requestbuffers {
    pub count: u32,
    pub r#type: u32,
    pub memory: u32,
    pub capabilities: u32,
    pub reserved: [u32; 1]
}

#[repr(C)]
#[derive(Default, Clone, Copy)]
struct Memory {
    start: u64,
    length: usize
}

#[repr(C)]
#[derive(Default, Copy, Clone)]
pub struct Buffer {
    memories: Memory,
    bytesused: u32,
    buffer: v4l2_buffer
}

impl Buffer {
    pub fn new() -> Self {
        Default::default()
    }
}

pub struct VideoCapture {
    is_open: bool,
    _device_file: File,
    raw_fd: i32,
    fps: u32,
    buffers: Vec<Buffer>,
    dev_file: Result<File, Error>,
    cap: v4l2_capability,
    format: v4l2_format,
    streamparm: v4l2_streamparm,
    reqbuffers: v4l2_requestbuffers,
    bufs: v4l2_buffer,
    start_cap_type: u32,
    stop_cap_type: u32
}

impl VideoCapture {
    /// Open the default video device on index 0.
    pub fn new() -> io::Result<Self> {
        let file = File::options()
                                .write(true)
                                .read(true)
                                .open("/dev/video0")?;

        let dev_file = File::options()
                                .read(true)
                                .write(true)
                                .open(DEVICE_FILE_PATH);
        
                                let fd = file.as_raw_fd();
        println!("camera fd = {}", fd);
        Ok(Self {
            is_open: true,
            _device_file: file,
            raw_fd: fd,
            fps: Default::default(),
            buffers: Vec::new(),
            dev_file,
            cap: v4l2_capability::default(),
            format: v4l2_format::default(),
            streamparm: v4l2_streamparm::default(),
            reqbuffers: v4l2_requestbuffers::default(),
            bufs: v4l2_buffer::default(),
            start_cap_type: V4L2_BUF_TYPE_VIDEO_CAPTURE,
            stop_cap_type: V4L2_BUF_TYPE_VIDEO_CAPTURE
        })
    }

    fn write_uaddr(&mut self, set_type: u64, uaddr: std::ffi::c_ulong) {
        match &mut self.dev_file {
            Ok(f) => {
                let mut result: Vec<u8> = Vec::new();
                let cmd_bytes = 0u64.to_ne_bytes();
                println!("cmd_bytes: {:?}", cmd_bytes);
                for byte in cmd_bytes {
                    result.push(byte);
                }
                for byte in set_type.to_ne_bytes() {
                    result.push(byte);
                }
                let uaddr_bytes = uaddr.to_ne_bytes();
                for byte in uaddr_bytes {
                    result.push(byte);
                }
                println!("results: {:?}", result);
                f.write(&result).unwrap();
                f.flush().unwrap();
            }
            Err(_) => {
                println!("Failed to open camdriver device file. SKIP.")
            }
        }
    }

    pub fn setup_module(
        &mut self
    ) {
        println!("Write cap.");
        let cap = &mut self.cap as * const _ as std::ffi::c_ulong;
        self.write_uaddr(0, cap);

        println!("Write format.");
        let format = &mut self.format as * const _ as std::ffi::c_ulong;
        self.write_uaddr(1, format);

        println!("Write streamparm.");
        let streamparm = &mut self.streamparm as * const _ as std::ffi::c_ulong;
        self.write_uaddr(2, streamparm);

        println!("Write reqbuffers.");
        let reqbuffers = &mut self.reqbuffers as * const _ as std::ffi::c_ulong;
        self.write_uaddr(3, reqbuffers);

        println!("Write bufs.");
        let bufs = &mut self.bufs as * const _ as std::ffi::c_ulong;
        self.write_uaddr(4, bufs);

        println!("Write start_cap_type.");
        let start_cap_type = &mut self.start_cap_type as * const _ as std::ffi::c_ulong;
        self.write_uaddr(5, start_cap_type);

        let stop_cap_type = &mut self.stop_cap_type as * const _ as std::ffi::c_ulong;
        self.write_uaddr(6, stop_cap_type);
    }

    // // Command module to prepare the driver.
    // pub fn prep_module(&mut self) {
        // match &mut self.dev_file {
        //     Ok(f) => {
        //         let cmd_bytes = 2u64.to_ne_bytes();
        //         f.write(&cmd_bytes).unwrap();
        //         f.flush().unwrap();
        //     }
        //     Err(_) => {
        //         println!("Failed to open camdriver device file. SKIP.")
        //     }
        // }

    // }

    pub fn start_ioctl(&mut self, io_type: u64) {
        match &mut self.dev_file {
            Ok(f) => {
                let mut result: Vec<u8> = Vec::new();
                let cmd_bytes = 2u64.to_ne_bytes();
                for byte in cmd_bytes {
                    result.push(byte);
                }
                let io_bytes = io_type.to_ne_bytes();
                for byte in io_bytes {
                    result.push(byte);
                }
                println!("Start IOCTL with io_type: {}", io_type);
                f.write(&result).unwrap();
                f.flush().unwrap();
            }
            Err(_) => {
                println!("Failed to open camdriver device file. SKIP.")
            }
        }
    }

    pub fn prep_stream(
        &mut self, 
        buffer_count: Option<usize>,
        fps: Option<usize>
    ) -> (u32, u32) {
        println!("Started SETUP MODULE");
        self.setup_module();
        println!("Started QUERY_CAP");
        self.query_cap();
        println!("Started CHECK_FORMAT");
        self.check_img_format();
        let (frame_width, frame_height) = match self.format.fmt {
            Fmt::Pix(pix_format) => {
                (pix_format.width, pix_format.height)
            }
        };
        println!("Started SWITCH");
        self.switch_to_yuyv();
        println!("Started SET_FPS");
        self.set_fps(fps.unwrap_or(DEFAULT_STREAM_FPS) as u32);
        println!("Started REQ_BUFS");
        match self.request_buffer(buffer_count.unwrap_or(MAX_V4L_BUFFERS) as u32) {
            Ok(size) => {
                println!("{} buffers are requested", size);
            }
            Err(e) => {
                panic!("request buffer error {}", e);
            }
        };
        println!("Started QUERY_BUF");
        self.query_buffer().unwrap();
        println!("Started STREAM");
        self.start_stream();
        println!("Started QUEUE_BUFFER");
        self.queue_buffer().unwrap();

        (frame_width, frame_height)
    }

    pub fn query_cap(&mut self) {
        if !self.is_open {
            println!("File is opened.");
            return;
        }

        self.start_ioctl(0);

        let info = &self.cap;

        println!("driver: {:?}", str::from_utf8(&info.driver));
        println!("card: {:?}", str::from_utf8(&info.card));
        println!("bus_info: {:?}", str::from_utf8(&info.bus_info));
    }

    pub fn check_img_format(&mut self) {
        self.format.r#type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
        self.start_ioctl(1);

        match self.format.fmt {
            Fmt::Pix(pix_format) => {
                println!("width: {}", pix_format.width);
                println!("height: {}", pix_format.height);
            }
        }
    }

    pub fn switch_to_yuyv(&mut self) {
        match self.format.fmt {
            Fmt::Pix(mut pix_format) => {
                pix_format.pixelformat = 0x56595559;
                self.start_ioctl(2);
            }
        }
    }

    pub fn set_fps(&mut self, value: u32) {
        self.streamparm.r#type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
        self.streamparm.numerator = 1;
        self.streamparm.denominator = value;

        self.start_ioctl(3);
        self.start_ioctl(4);

        println!("FPS: {}/{}", self.streamparm.denominator, self.streamparm.numerator);
        self.fps = self.streamparm.denominator;
    }

    pub fn request_buffer(&mut self, count: u32) -> Result<usize, Errno> {
        self.reqbuffers.count = 1;
        self.reqbuffers.r#type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
        self.reqbuffers.memory = V4L2_MEMORY_MMAP;

        self.start_ioctl(5);

        // if reqbufs.count < 5 {
        //     println!("request buffers: not enough buffer available ({} available) [FAILED] ", reqbufs.count);
        //     return Err(Errno::EADDRNOTAVAIL);
        // }

        // println!("reqbufs count: {}", reqbufs.count);
        // println!("reqbufs memory: {}", reqbufs.memory);

        println!("reqbufs count: {}", self.reqbuffers.count);
        println!("reqbufs memory: {}", self.reqbuffers.memory);
        
        let buf_count = self.reqbuffers.count as usize;
        self.buffers = vec![Default::default(); buf_count];
        Ok(buf_count)
    }

    pub fn query_buffer(&mut self) -> Result<(), Errno> {
        self.bufs.r#type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
        self.bufs.memory = V4L2_MEMORY_MMAP;
        self.bufs.index = 0 as u32;

        self.start_ioctl(6);

        println!("buffer[{}] length: {}", 0, self.bufs.length);
        println!("buffer[{}] offset: {}", 0, self.bufs.offset);

        unsafe {
            let data = mman::mmap(
                ptr::null_mut(), 
                self.bufs.length as usize, 
                ProtFlags::PROT_READ | ProtFlags::PROT_WRITE, 
                MapFlags::MAP_SHARED, 
                self.raw_fd, 
                self.bufs.offset as i64
            );

            match data {
                Ok(val) => {
                    let addr = val as *const _ as u64;
                    // println!("");
                    // println!("Mapped address start: {:#20x}", addr);
                    // println!("Mapped address end: {:#20x}", addr + buf.length as u64);
                    let pfns = pagemap::get_pagemap(addr, self.bufs.length as u64);
                    self.inform_pfns(pfns, 0);
                    self.buffers[0].memories = Memory {
                        start: val as *const _ as u64,
                        length: self.bufs.length as usize,
                    };
                    self.buffers[0].bytesused = self.bufs.bytesused;
                    self.buffers[0].buffer = self.bufs;
                }
                Err(e) => {
                    println!("mem map failed for buffer {}. {}", 0, e);
                    return Err(Errno::EADDRNOTAVAIL)
                }
            }
        }
        
        println!("query buffers [OK]");
        Ok(())
    }

    fn inform_pfns(&mut self, pfns: Vec<i64>, index: usize) {
        match &mut self.dev_file {
            Ok(f) => {
                f.seek(SeekFrom::Start(index as u64)).unwrap();
                let mut result: Vec<u8> = Vec::new();
                let cmd_bytes = 1u64.to_ne_bytes();
                for byte in cmd_bytes {
                    result.push(byte);
                }
                for pfn in pfns {
                    let pfn_bytes = pfn.to_ne_bytes();
                    for byte in pfn_bytes {
                        result.push(byte);
                    }
                }
                // let pfns: Vec<u8> = pfns.iter().flat_map(|x| x.to_ne_bytes()).collect();
                f.write_all(&result).unwrap();
                f.flush().unwrap();
            }
            Err(_) => {
                println!("Failed to open camdriver device file. SKIP.")
            }
        }
    }

    pub fn start_stream(&mut self) {
        self.start_ioctl(8);
    }

    pub fn stop_stream(&mut self) {
        self.start_ioctl(9);
    }

    pub fn queue_buffer(&mut self) -> Result<(), Errno> {
        self.bufs.r#type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
        self.bufs.memory = V4L2_MEMORY_MMAP;
        self.bufs.index = 0;

        self.start_ioctl(7);
        // println!("queue buffer [OK]");
        Ok(())
    }

    pub fn read_frame(&mut self) -> Result<Vec<u8>, Errno> {
        // // ioctl_readwrite!(vidioc_dqbuf, VIDIOC_DQBUF_MAGIC, VIDIOC_DQBUF_TYPE_MODE, v4l2_buffer);
        // ioctl_readwrite!(vidioc_qbuf, VIDIOC_QBUF_MAGIC, VIDIOC_QBUF_TYPE_MODE, v4l2_buffer);

        // let mut buf: v4l2_buffer = Default::default();
        // println!("Started read frame.");
        self.bufs.r#type = V4L2_BUF_TYPE_VIDEO_CAPTURE;
        self.bufs.memory = V4L2_MEMORY_MMAP;

        let mut result = vec![0u8; self.bufs.length as usize];
        match &mut self.dev_file {
            Ok(f) => {
                match f.read(&mut result) {
                    Ok(_) => {  },
                    Err(_) => { return Err(Errno::EADDRNOTAVAIL); }
                }
            }
            Err(_) => {
                println!("Failed to open camdriver device file. SKIP.")
            }
        }

        Ok(result)
    }

    pub fn read(&mut self) -> Result<Vec<u8>, Errno> {
        loop {
            let mut fd_set = select::FdSet::new();
            let mut tv = time::TimeVal::new(2, 0);

            fd_set.clear();
            fd_set.insert(self.raw_fd);

            let result = select::select(
                self.raw_fd + 1, 
                &mut fd_set, 
                None, 
                None, 
                &mut tv
            );

            match result {
                Ok(_) => {
                    let read_result = self.read_frame();
                    match read_result {
                        Ok(out) => { return Ok(out); },
                        Err(e) => {
                            if e == Errno::EAGAIN {
                                continue;
                            }
                            println!("read frame failed in select [FAILED]");
                            return Err(e);
                        }
                    }
                },
                Err(e) => {
                    if e == Errno::EINTR {
                        continue;
                    }
                    println!("select failed [FAILED]");
                    return Err(e);
                }
            }
        }
    }

    pub fn read_out_data(&self, start: u64, size: u32) -> Vec<u8> {
        let size = size as usize;
        let mut out = vec![0u8; size];
        let src = start as *mut c_int as *mut c_void;
        unsafe {
            ptr::copy_nonoverlapping(
                src, 
                out.as_mut_ptr() as *mut c_void, 
                size
            )
        }
        out
    }
    
    pub fn write_to_file(&self, start: u64, size: u32) {
        let size = size as usize;

        let file = File::options()
                                .write(true)
                                .read(true)
                                .create(true)
                                .open("result_frame")
                                .unwrap();
        unsafe {
            let data = start as *mut c_int as *mut c_void;
            let result = libc::write(
                file.as_raw_fd(), 
                data, 
                size
            );
            println!("Write data with len: {}", result);
        }
    }
}

impl Drop for VideoCapture {
    fn drop(&mut self) {
        self.stop_stream();
        for buffer in &self.buffers {
            let mem = buffer.memories;
            unsafe {
                let result = mman::munmap(
                    mem.start as *mut c_int as *mut c_void, 
                    mem.length
                );
                match result {
                    Ok(()) => {},
                    Err(e) => {
                        println!("Failed to munmap. [FAILED] {}", e);
                    }
                }
            }
        }

        println!("munmap. [OK]");
    }
}
