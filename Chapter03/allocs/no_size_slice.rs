use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let values = vec![0, 1, 2, 3, 4, 5, 7, 8, 9, 10];
    let cur: usize = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;
    let cap: usize = cur % values.len();

    let slc: &[u8] = &values[0..cap];

    println!("{:?}", slc);
}
