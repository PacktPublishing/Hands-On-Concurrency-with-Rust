extern crate naive_hashmap;
extern crate rand;

use naive_hashmap::HashMap;
use rand::{Rng, SeedableRng, XorShiftRng};

fn main() {
    let mut rng: XorShiftRng = SeedableRng::from_seed([1981, 1986, 2003, 2011]);
    let mut hash_map = HashMap::new();

    let mut insert_empty = 0;
    let mut insert_present = 0;
    let mut get_fail = 0;
    let mut get_success = 0;

    for _ in 0..100_000 {
        let key = rng.gen::<u16>();
        if rng.gen::<bool>() {
            let value = rng.gen::<u32>();
            if hash_map.insert(key, value).is_none() {
                insert_empty += 1;
            } else {
                insert_present += 1;
            }
        } else {
            if hash_map.get(&key).is_none() {
                get_fail += 1;
            } else {
                get_success += 1;
            }
        }
    }

    println!("INSERT");
    println!("  empty:   {}", insert_empty);
    println!("  present: {}", insert_present);
    println!("LOOKUP");
    println!("  fail:    {}", get_fail);
    println!("  success: {}", get_success);
}
