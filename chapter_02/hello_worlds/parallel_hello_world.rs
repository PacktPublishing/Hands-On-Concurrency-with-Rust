use std::thread;

fn main() {
    thread::spawn(|| println!("GREETINGS, HUMANS"))
        .join()
        .unwrap();
}
