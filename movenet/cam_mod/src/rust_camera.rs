//! Provides a support of video driver interaction from kernel space.
//! 
//! Works as a kernel module. The communication between this kernel module and userspace client app requires a well-formed protocol.

use kernel::prelude::*;
use kernel::{
    file::{self, File, SeekFrom}, 
    io_buffer::{IoBufferReader, IoBufferWriter}, 
    miscdev,
    sync::{Ref, RefBorrow, smutex::Mutex},
};
use core::ffi::{self, c_int, c_void, c_ulong};
use core::result::Result::Ok;
use core::ptr;
use core::mem::size_of;
use core::cmp::min;
use kernel::str::CString;
use kernel::bindings::{
    self,
    filp_open,
    filp_close,
    vfs_ioctl,
    O_RDWR
};

module! {
    type: RustCamera,
    name: "rust_camera",
    author: "Yibo Yan",
    description: "A simple module that reads camera input.",
    license: "GPL",
}

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

const V4L2_BUF_TYPE_VIDEO_CAPTURE: u32 = 1;
const V4L2_MEMORY_MMAP: u32 = 1;

// extern "C" {
//     fn get_page_offset() -> u64;
//     fn get_pfn_to_virt(pfn: i64) -> *mut c_void;
// }

const PAGE_SHIFT: u32 = bindings::PAGE_SHIFT;
const PAGE_SIZE: usize = kernel::PAGE_SIZE;
// const PAGE_OFFSET: c_ulong = bindings::page_offset_base;

fn rget_page_offset() -> c_ulong {
    return unsafe { bindings::page_offset_base };
}

// #define __va(x)			((void *)((unsigned long)(x)+PAGE_OFFSET))
fn __va(x: u64) -> *mut c_void {
    let PAGE_OFFSET: c_ulong = rget_page_offset();
    (x + PAGE_OFFSET) as *mut c_int as *mut c_void
}

// #define pfn_to_kaddr(pfn)      __va((pfn) << PAGE_SHIFT)
fn pfn_to_virt(pfn: i64) -> *mut c_void {
    __va(((pfn) << (PAGE_SHIFT as i64)) as u64)
}

// #define _IOC(dir,type,nr,size) \
// 	(((dir)  << _IOC_DIRSHIFT) | \
// 	 ((type) << _IOC_TYPESHIFT) | \
// 	 ((nr)   << _IOC_NRSHIFT) | \
// 	 ((size) << _IOC_SIZESHIFT))
const fn _IOC(dir: u32, tp: u32, nr: u32, size: u32) -> u32 {
    (
        ((dir)  << bindings::_IOC_DIRSHIFT) |
        ((tp)   << bindings::_IOC_TYPESHIFT) |
        ((nr)   << bindings::_IOC_NRSHIFT) |
        ((size) << bindings::_IOC_SIZESHIFT)
    )
}

// #define _IOWR(type,nr,size)	_IOC(_IOC_READ|_IOC_WRITE,(type),(nr),sizeof(size))
const fn _IOWR(tp: u32, nr: u32, size: u32) -> u32 {
    _IOC(
        (bindings::_IOC_READ | bindings::_IOC_WRITE), 
        tp, 
        nr,
        size
    )
}

// #define _IOR(type,nr,size)	_IOC(_IOC_READ,(type),(nr),sizeof(size))
const fn _IOR(tp: u32, nr: u32, size: u32) -> u32 {
    _IOC(
        bindings::_IOC_READ, 
        tp, 
        nr,
        size
    )
}

// #define _IOW(type,nr,size)	_IOC(_IOC_WRITE,(type),(nr),sizeof(size))
const fn _IOW(tp: u32, nr: u32, size: u32) -> u32 {
    _IOC(
        bindings::_IOC_WRITE, 
        tp, 
        nr,
        size
    )
}

// ----- VIDEO IOCTL INTERFACE ----- 
const querycap_cmd: u32 = _IOR(
    'V' as u32, 
    0 as u32, 
    size_of::<v4l2_capability>() as u32
);

const gfmt_cmd: u32 = _IOWR(
    'V' as u32, 
    4 as u32, 
    size_of::<v4l2_format>() as u32
);

const sfmt_cmd: u32 = _IOWR(
    'V' as u32, 
    5 as u32, 
    size_of::<v4l2_format>() as u32
);

const sparm_cmd: u32 = _IOWR(
    'V' as u32, 
    22 as u32, 
    size_of::<v4l2_streamparm>() as u32
);

const gparm_cmd: u32 = _IOWR(
    'V' as u32, 
    21 as u32, 
    size_of::<v4l2_streamparm>() as u32
);

const reqbufs_cmd: u32 = _IOWR(
    'V' as u32, 
    8 as u32, 
    size_of::<v4l2_requestbuffers>() as u32
);

const querybufs_cmd: u32 = _IOWR(
    'V' as u32, 
    9 as u32, 
    size_of::<v4l2_buffer>() as u32
);

const streamon_cmd: u32 = _IOW(
    'V' as u32, 
    18 as u32, 
    size_of::<i32>() as u32
);

const streamoff_cmd: u32 = _IOW(
    'V' as u32, 
    19 as u32, 
    size_of::<i32>() as u32
);

const qbuf_cmd: u32 = _IOWR(
    'V' as u32, 
    15 as u32, 
    size_of::<v4l2_buffer>() as u32
);

const dqbuf_cmd: u32 = _IOWR(
    'V' as u32, 
    17 as u32, 
    size_of::<v4l2_buffer>() as u32
);

const MEM_SIZE: usize = 512 * 1024;

struct SharedMemSpace {
    mem: Mutex<Vec<u8>>,
    seek_pos: Mutex<u64>,
    buffers: Mutex<Vec<Vec<u64>>>,
    write_info: Mutex<bool>,
    info_uaddr: Mutex<c_ulong>,
    fd: u64,
    cap: Mutex<c_ulong>,
    format: Mutex<c_ulong>,
    streamparm: Mutex<c_ulong>,
    reqbuffers: Mutex<c_ulong>,
    bufs: Mutex<c_ulong>,
    start_cap_type: Mutex<c_ulong>,
    stop_cap_type: Mutex<c_ulong>
}

impl Drop for SharedMemSpace {
    fn drop(&mut self) {
        unsafe {
            filp_close(
                self.fd as *mut bindings::file, ptr::null_mut()
            );
        }
        pr_info!("Dropped SharedMemSpace\n");
    }
}

impl SharedMemSpace {
    fn try_new() -> Result<Ref<Self>> {
        let mut mem_space = Vec::<u8>::try_with_capacity(MEM_SIZE)?;
        for _ in 0..MEM_SIZE {
            mem_space.try_push(0)?;
        }

        let video_path = CString::try_from_fmt(
            fmt!("/dev/video0")
        ).unwrap();

        let fd_mut_ptr = unsafe { 
            filp_open(
                video_path.as_char_ptr(),
                O_RDWR.try_into().unwrap(), 
                0
            ) 
        };

        pr_info!("init fd ptr: {:?}\n", fd_mut_ptr);
        
        Ref::try_new(SharedMemSpace {
            mem: Mutex::new(mem_space),
            seek_pos: Mutex::new(0),
            buffers: Mutex::new(Vec::new()),
            write_info: Mutex::new(false),
            info_uaddr: Mutex::new(0),
            fd: fd_mut_ptr as *const _ as u64,
            cap: Mutex::new(0),
            format: Mutex::new(0),
            streamparm: Mutex::new(0),
            reqbuffers: Mutex::new(0),
            bufs: Mutex::new(0),
            start_cap_type: Mutex::new(0),
            stop_cap_type: Mutex::new(0)
        })
    }
}

struct WrappedData {
    mem_space: Ref<SharedMemSpace>,
}

struct RustFile;

#[vtable]
impl file::Operations for RustFile {
    type Data = Ref<WrappedData>;
    type OpenData = Ref<SharedMemSpace>;

    fn open(context: &Ref<SharedMemSpace>, _file: &File) -> Result<Self::Data> {
        let cloned = context.clone();

        Ref::try_new(WrappedData {
            mem_space: context.clone(),
        })
    }

    fn read(
        shared: RefBorrow<'_, WrappedData>,
        _: &File,
        data: &mut impl IoBufferWriter,
        _offset: u64,
    ) -> Result<usize> {
        let shared = shared.mem_space.clone();
        let mut total_len: usize = 0;

        let fd = (shared.fd) as *mut bindings::file;
        let buf_addr = shared.bufs.lock();
        let result = unsafe { 
            bindings::vfs_ioctl(fd, dqbuf_cmd, *buf_addr) 
        };
        if result < 0 {
            pr_alert!("Failed to dqbuf. ecode: {}\n", result);
        } else {
            let buf_idx: usize = 0;

            let buffers = shared.buffers.lock();
            let buffer_to_read: &Vec<u64> = &buffers[buf_idx];
            let psize = PAGE_SIZE as u32;
            let vec_size = buffer_to_read.len() * (psize as usize);
            let mut out = Vec::<u8>::new();
            out.try_resize(vec_size, 0u8)?;
            
            let mut written: usize = 0;
            for pfn in buffer_to_read {
                let size_to_write = psize as usize;
                let kaddr = pfn_to_virt(*pfn as i64);

                unsafe {
                    ptr::copy_nonoverlapping(
                        kaddr, 
                        out[written..(written + size_to_write)].as_mut_ptr() as *mut c_void, 
                        size_to_write
                    );
                }
                written += size_to_write;
            }
            total_len = written;
            data.write_slice(&out.as_slice())?;
        }

        let result = unsafe { 
            bindings::vfs_ioctl(fd, qbuf_cmd, *buf_addr) 
        };

        if result < 0 {
            pr_alert!("qbuf failed. ecode: {}\n", result);
        }

        Ok(total_len)
    }

    fn write(
        shared: RefBorrow<'_, WrappedData>,
        _: &File,
        data: &mut impl IoBufferReader,
        _offset: u64,
    ) -> Result<usize> {
        let shared = shared.mem_space.clone();
        let len = data.len();
        pr_info!("Begin write. Len: {}.\n", len);

        let mut cmd_type_buf = [0u8; 8];
        data.read_slice(&mut cmd_type_buf[..])?;
        let cmd_type = u64::from_ne_bytes(cmd_type_buf);
        pr_info!("cmd_type: {}.\n", cmd_type);

        if cmd_type == 0 { // set user address
            pr_info!("In cmd_type 0.\n");
            let mut set_type_buf = [0u8; 8];
            data.read_slice(&mut set_type_buf[..])?;
            let set_type = u64::from_ne_bytes(set_type_buf);
            pr_info!("set_type: {}.\n", set_type);

            let mut uaddr_buf = [0u8; 8];
            data.read_slice(&mut uaddr_buf[..])?;
            let uaddr = u64::from_ne_bytes(uaddr_buf);
            pr_info!("uaddr: {}.\n", uaddr);

            if set_type == 0 { // set cap
                (*shared.cap.lock()) = uaddr;
            } else if set_type == 1 { // set format
                (*shared.format.lock()) = uaddr;
            } else if set_type == 2 { // set streamparm
                (*shared.streamparm.lock()) = uaddr;
            } else if set_type == 3 { // set reqbuffers
                (*shared.reqbuffers.lock()) = uaddr;
            } else if set_type == 4 { // set bufs
                (*shared.bufs.lock()) = uaddr;
            } else if set_type == 5 { // set start_cap_type
                (*shared.start_cap_type.lock()) = uaddr;
            } else if set_type == 6 { // set stop_cap_type
                (*shared.stop_cap_type.lock()) = uaddr;
            } else {
                pr_info!("Undentified type detected. {}\n", set_type);
            }
        } else if cmd_type == 1 { // set pfns);
            let mut pfns: Vec<u64> = Vec::new();
            for i in (0..(len - 8)).step_by(8) {
                let mut pfn = [0u8; 8];
                data.read_slice(&mut pfn[..])?;
                let pfn_u64 = u64::from_ne_bytes(pfn);
                pfns.try_push(pfn_u64)?;
                // pr_info!("INDEX: {:<5} - PFN: {:#20x}\n", i / 8, pfn_u64);
            }
            pr_info!("number of pfns: {}\n", pfns.len());
            let pos = shared.seek_pos.lock();
            let mut buffers = shared.buffers.lock();
            if ((*pos) as usize) >= buffers.len() {
                buffers.try_push(pfns)?;
            } else {
                buffers[(*pos) as usize] = pfns;
            }
            pr_info!("buffers size: {}\n", buffers.len());
            drop(buffers);
            let buffers_size = shared.buffers.lock().len();
            let buffers_0_size = shared.buffers.lock()[0].len();
        } else if cmd_type == 2 { // prep driver
            pr_info!("In cmd_type 2.\n");
            let fd = (shared.fd) as *mut bindings::file;

            let mut io_type_buf = [0u8; 8];
            data.read_slice(&mut io_type_buf[..])?;
            let io_type = u64::from_ne_bytes(io_type_buf);
            pr_info!("io_type: {}.\n", io_type);

            if io_type == 0 {
                // ----- QUERY CAP ----- //
                let cap_addr = shared.cap.lock();

                let result = unsafe { 
                    bindings::vfs_ioctl(fd, querycap_cmd, *cap_addr) 
                };

                if result < 0 {
                    pr_info!("Failed to query cap. Error: {}\n", result);
                } else {
                    pr_info!("Success to query cap.\n");
                }
            } else if io_type == 1 {
                // ----- G FORMAT ----- //
                let format_addr = shared.format.lock();

                let result = unsafe { 
                    bindings::vfs_ioctl(fd, gfmt_cmd, *format_addr) 
                };

                if result < 0 {
                    pr_info!("Failed to get format. Error: {}\n", result);
                } else {
                    pr_info!("Success to get format.\n");
                }
            } else if io_type == 2 {
                // ----- S FORMAT ----- //
                let format_addr = shared.format.lock();

                let result = unsafe { 
                    bindings::vfs_ioctl(fd, sfmt_cmd, *format_addr) 
                };

                if result < 0 {
                    pr_info!("Failed to set format. Error: {}\n", result);
                } else {
                    pr_info!("Success to set format.\n");
                }
            } else if io_type == 3 {
                // ----- S PARM ----- //
                let streamparm_addr = shared.streamparm.lock();

                let result = unsafe { 
                    bindings::vfs_ioctl(fd, sparm_cmd, *streamparm_addr) 
                };

                if result < 0 {
                    pr_info!("Failed to set streamparm. Error: {}\n", result);
                } else {
                    pr_info!("Success to set streamparm.\n");
                }
            } else if io_type == 4 {
                // ----- G PARM ----- //
                let streamparm_addr = shared.streamparm.lock();

                let result = unsafe { 
                    bindings::vfs_ioctl(fd, gparm_cmd, *streamparm_addr) 
                };

                if result < 0 {
                    pr_info!("Failed to get streamparm. Error: {}\n", result);
                } else {
                    pr_info!("Success to get streamparm.\n");
                }
            } else if io_type == 5 {
                // ----- REQ BUF ----- //
                let reqbuffers_addr = shared.reqbuffers.lock();

                let result = unsafe { 
                    bindings::vfs_ioctl(fd, reqbufs_cmd, *reqbuffers_addr) 
                };

                if result < 0 {
                    pr_info!("Failed to request buffer. Error: {}\n", result);
                } else {
                    pr_info!("Success to request buffer.\n");
                }
            } else if io_type == 6 {
                // ----- QUERY BUF ----- //
                let bufs_addr = shared.bufs.lock();

                let result = unsafe { 
                    bindings::vfs_ioctl(fd, querybufs_cmd, *bufs_addr) 
                };

                if result < 0 {
                    pr_info!("Failed to query buffer. Error: {}\n", result);
                } else {
                    pr_info!("Success to query buffer.\n");
                }
            } else if io_type == 7 {
                // ----- QUEUE BUF ----- //
                let bufs_addr = shared.bufs.lock();

                let result = unsafe { 
                    bindings::vfs_ioctl(fd, qbuf_cmd, *bufs_addr) 
                };

                if result < 0 {
                    pr_info!("Failed to queue buffer. Error: {}\n", result);
                } else {
                    pr_info!("Success to queue buffer.\n");
                }
            } else if io_type == 8 {
                // ----- STREAM ON ----- //
                let start_addr = shared.start_cap_type.lock();

                let result = unsafe { 
                    bindings::vfs_ioctl(fd, streamon_cmd, *start_addr) 
                };

                if result < 0 {
                    pr_info!("Failed to stream on. Error: {}\n", result);
                } else {
                    pr_info!("Success to stream on.\n");
                }
            } else if io_type == 9 {
                // ----- STREAM OFF ----- //
                let stop_addr = shared.stop_cap_type.lock();

                let result = unsafe { 
                    bindings::vfs_ioctl(fd, streamoff_cmd, *stop_addr) 
                };

                if result < 0 {
                    pr_info!("Failed to stream off. Error: {}\n", result);
                } else {
                    pr_info!("Success to stream off.\n");
                }
            }
        }
        Ok(len)
    }

    fn seek(
        shared: RefBorrow<'_, WrappedData>,
        _file: &File,
        offset: SeekFrom
    ) -> Result<u64> {
        let shared = shared.mem_space.clone();

        match offset {
            SeekFrom::Start(off) => {
                let mut buffers = shared.buffers.lock();
                let off_usize = off as usize;
                if off_usize < buffers.len() {
                    buffers[off_usize].clear();
                }
                *(shared.seek_pos.lock()) = off;
            }
            _ => { }
        }
        let pos = shared.seek_pos.lock();
        Ok(*pos)
    }

    fn release(_shared: Self::Data, _file: &File) { /* Nothing fancy here. */ }
}

struct RustCamera {
    _dev: Pin<Box<miscdev::Registration<RustFile>>>,
}

impl kernel::Module for RustCamera {
    fn init(_name: &'static CStr, _module: &'static ThisModule) -> Result<Self> {
        pr_info!("RustCamera (init)\n");;
        pr_info!("Rust PAGE_OFFSET: {}\n", rget_page_offset() as u64);
        
        let mut shared_mem = SharedMemSpace::try_new()?;

        let miscdev_reg = miscdev::Options::new()
                .mode(0o666)
                .register_new(
                    fmt!("camdriver"), 
                    shared_mem
                )?;

        Ok(RustCamera {
            _dev: miscdev_reg
        })
    }
}

impl Drop for RustCamera {
    fn drop(&mut self) {
        pr_info!("RustCamera (exit)\n");
    }
}
