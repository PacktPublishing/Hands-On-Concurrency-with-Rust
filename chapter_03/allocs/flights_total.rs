fn project_flights() -> Vec<(u16, u8)> {
    let mut t = Vec::new();
    t.push((1968, 2));
    t.push((1969, 4));
    t.push((1970, 1));
    t.push((1971, 2));
    t.push((1972, 2));
    t
}

fn main() {
    let mut total: u8 = 0;
    let flights = project_flights();
    for &(_, flights) in flights.iter() {
        total += flights;
    }
    println!("{}", total);
}
