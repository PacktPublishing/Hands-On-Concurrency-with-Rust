fn main() {
    let values = vec![0, 1, 2, 3, 4, 5, 7, 8, 9, 10];
    let slc: &[u8] = &values[0..10_000];

    println!("{:?}", slc);
}
