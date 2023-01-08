use std::io::Read;
use std::io::Seek;
use std::io::SeekFrom;
use std::mem;
use std::env;
use std::fs::File;
use nix::libc::{_SC_PAGESIZE, sysconf};

//   54 #define PM_ENTRY_BYTES      sizeof(uint64_t)
//   55 #define PM_STATUS_BITS      3
//   56 #define PM_STATUS_OFFSET    (64 - PM_STATUS_BITS)
//   57 #define PM_STATUS_MASK      (((1LL << PM_STATUS_BITS) - 1) << PM_STATUS_OFFSET)
//   58 #define PM_STATUS(nr)       (((nr) << PM_STATUS_OFFSET) & PM_STATUS_MASK)
//   59 #define PM_PSHIFT_BITS      6
//   60 #define PM_PSHIFT_OFFSET    (PM_STATUS_OFFSET - PM_PSHIFT_BITS)
//   61 #define PM_PSHIFT_MASK      (((1LL << PM_PSHIFT_BITS) - 1) << PM_PSHIFT_OFFSET)
//   62 #define PM_PSHIFT(x)        (((u64) (x) << PM_PSHIFT_OFFSET) & PM_PSHIFT_MASK)
//   63 #define PM_PFRAME_MASK      ((1LL << PM_PSHIFT_OFFSET) - 1)
//   64 #define PM_PFRAME(x)        ((x) & PM_PFRAME_MASK)
//   65 
//   66 #define PM_PRESENT          PM_STATUS(4LL)
//   67 #define PM_SWAP             PM_STATUS(2LL)

const PM_ENTRY_BYTES: usize = mem::size_of::<u64>();
const PM_STATUS_BITS: usize = 3;
const PM_STATUS_OFFSET: usize = 64 - PM_STATUS_BITS;
const PM_STATUS_MASK: i64 = (((1 as i64) << PM_STATUS_BITS) - 1) << PM_STATUS_OFFSET;
const fn PM_STATUS(nr: i64) -> i64 {
    (nr << PM_STATUS_OFFSET) & PM_STATUS_MASK
}
const PM_PSHIFT_BITS: usize = 6;
const PM_PSHIFT_OFFSET: usize = PM_STATUS_OFFSET - PM_PSHIFT_BITS;
const PM_PSHIFT_MASK: i64 = ((1i64 << PM_PSHIFT_BITS) - 1) << PM_PSHIFT_OFFSET;
const fn PM_PSHIFT(x: i64) -> i64 {
    ((x << PM_PSHIFT_OFFSET) & PM_PSHIFT_MASK) as i64
}
const PM_PFRAME_MASK: i64 = (1i64 << PM_PSHIFT_OFFSET) - 1;
const fn PM_PFRAME(x: i64) -> i64 {
    (x) & PM_PFRAME_MASK
}
const PM_PRESENT: i64 = PM_STATUS(4);
const PM_SWAP: i64 = PM_STATUS(2);

fn main() {
    println!("PM_ENTRY_BYTES:   {}", PM_ENTRY_BYTES);
    println!("PM_STATUS_BITS:   {}", PM_STATUS_BITS);
    println!("PM_STATUS_OFFSET: {}", PM_STATUS_OFFSET);
    println!("PM_STATUS_MASK:   {}", PM_STATUS_MASK);
    println!("PM_STATUS(nr):    {}", PM_STATUS(2));
    println!("PM_PSHIFT_BITS:   {}", PM_PSHIFT_BITS);
    println!("PM_PSHIFT_OFFSET: {}", PM_PSHIFT_OFFSET);
    println!("PM_PSHIFT_MASK:   {}", PM_PSHIFT_MASK);

    println!("PM_PSHIFT(x):     {}", PM_PSHIFT(2));
    println!("PM_PFRAME_MASK:   {}", PM_PFRAME_MASK);
    println!("PM_PFRAME(x):     {}", PM_PFRAME(2));
    println!("PM_PRESENT:       {}", PM_PRESENT);
    println!("PM_SWAP:          {}", PM_SWAP);

    let args: Vec<String> = env::args().collect();
    let virt_addr: u64 = args[2].parse().unwrap();
    
}


// typedef struct pfn_info {
//     uint64_t pfn : 55;
//     bool soft_dirty : 1;
//     bool exclusive : 1;
//     uint8_t padding : 4;
//     bool file_or_shared_anon : 1;
//     bool swapped : 1;
//     bool present : 1;
//   } __attribute__((packed)) pfn_info_t;


fn get_pagesize() -> usize {
    unsafe {
        sysconf(_SC_PAGESIZE) as usize
    }
}

fn get_pagemap(virt_addr: u64) {
    let entry_size: u64 = 8;
    let page_size = get_pagesize() as u64;

    let path = "/proc/self/pagemap";
    let mut f = File::open(path).unwrap();
    
    let page_index = virt_addr / page_size;
    let page_offset = virt_addr % page_size;

    f.seek(SeekFrom::Start(page_index * entry_size))
        .expect("Failed to seek.");
    let mut page_data = [0u8; 8];
    f.read_exact(&mut page_data)
        .expect("Failed to read rge page entry data.");
    
    let page_data_raw = u64::from_ne_bytes(page_data);
    let pfn = PM_PFRAME(page_data_raw as i64);
    println!("Raw result: {:#20x}", page_data_raw);
    println!("Page data: {:#20x}", pfn);
}