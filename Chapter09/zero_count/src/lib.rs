#[no_mangle]
pub extern "C" fn tail_zero_count(arr: *const u16, len: usize) -> u64 {
    let mut zeros: u64 = 0;
    unsafe {
        for i in 0..len {
            zeros += (*arr.offset(i as isize)).trailing_zeros() as u64
        }
    }
    zeros
}
