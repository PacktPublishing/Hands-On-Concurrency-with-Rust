use individual::Individual;
use instruction::Instruction;
use rand::{thread_rng, Rng};
use std::ops::Add;
use std::ptr;
use std::sync::atomic::{AtomicUsize, Ordering};

pub static FITNESS_00010: AtomicUsize = AtomicUsize::new(0);
pub static FITNESS_11020: AtomicUsize = AtomicUsize::new(0);
pub static FITNESS_21030: AtomicUsize = AtomicUsize::new(0);
pub static FITNESS_31040: AtomicUsize = AtomicUsize::new(0);
pub static FITNESS_41050: AtomicUsize = AtomicUsize::new(0);
pub static FITNESS_51060: AtomicUsize = AtomicUsize::new(0);
pub static FITNESS_61070: AtomicUsize = AtomicUsize::new(0);
pub static FITNESS_71080: AtomicUsize = AtomicUsize::new(0);
pub static FITNESS_81090: AtomicUsize = AtomicUsize::new(0);
pub static FITNESS_91100: AtomicUsize = AtomicUsize::new(0);

pub static BATTLES: AtomicUsize = AtomicUsize::new(0);

fn tally_fitness(wins: Winner) -> () {
    let score = match wins {
        Winner::Tie => 0,
        Winner::Right(x) => x,
        Winner::Left(x) => x,
    };
    match score {
        0...10 => FITNESS_00010.fetch_add(1, Ordering::Relaxed),
        11...20 => FITNESS_11020.fetch_add(1, Ordering::Relaxed),
        21...30 => FITNESS_21030.fetch_add(1, Ordering::Relaxed),
        31...40 => FITNESS_31040.fetch_add(1, Ordering::Relaxed),
        41...50 => FITNESS_41050.fetch_add(1, Ordering::Relaxed),
        51...60 => FITNESS_51060.fetch_add(1, Ordering::Relaxed),
        61...70 => FITNESS_61070.fetch_add(1, Ordering::Relaxed),
        71...80 => FITNESS_71080.fetch_add(1, Ordering::Relaxed),
        81...90 => FITNESS_81090.fetch_add(1, Ordering::Relaxed),
        91...100 => FITNESS_91100.fetch_add(1, Ordering::Relaxed),
        _ => unreachable!(),
    };
}

unsafe impl Send for Mars {}

#[link(name = "mars")]
extern "C" {
    fn sim_alloc_bufs(mars: *mut Mars) -> isize;
    fn sim_free_bufs(mars: *mut Mars) -> ();
    fn sim_clear_core(mars: *mut Mars) -> ();
    fn sim_load_warrior(mars: *mut Mars, pos: u32, code: *const Instruction, len: u16) -> isize;
    fn sim_mw(mars: *mut Mars, war_pos_tab: *const u16, death_tab: *mut u32) -> isize;
}

#[repr(C)]
struct WarTable {
    tail: *mut *mut Instruction,
    head: *mut *mut Instruction,
    nprocs: u32,
    succ: *mut WarTable,
    pred: *mut WarTable,
    id: u32,
}

#[repr(C)]
pub struct Mars {
    n_warriors: u32,
    cycles: u32,
    core_size: u16,
    processes: u32,
    max_warrior_length: u16,
    war_tab: *mut WarTable,
    core_mem: *mut Instruction,
    queue_mem: *const *mut Instruction,
}

#[derive(Default)]
pub struct MarsBuilder {
    cycles: Option<u16>,
    core_size: Option<u16>,
    processes: Option<u32>,
    max_warrior_length: Option<u16>,
}

// TODO(blt) must call sim_free_bufs

impl MarsBuilder {
    pub fn freeze(self) -> Mars {
        let mut mars = Mars {
            n_warriors: 2,
            cycles: u32::from(self.cycles.unwrap_or(10_000)),
            core_size: self.core_size.unwrap_or(8_000),
            processes: self.processes.unwrap_or(10_000),
            max_warrior_length: self.max_warrior_length.unwrap_or(100),
            war_tab: ptr::null_mut(),
            core_mem: ptr::null_mut(),
            queue_mem: ptr::null_mut(),
        };
        unsafe {
            sim_alloc_bufs(&mut mars);
        }
        mars
    }

    pub fn core_size(mut self, core_size: u16) -> Self {
        self.core_size = Some(core_size);
        self
    }

    pub fn cycles(mut self, cycles: u16) -> Self {
        self.cycles = Some(cycles);
        self
    }

    pub fn processes(mut self, processes: u32) -> Self {
        self.processes = Some(processes);
        self
    }

    pub fn max_warrior_length(mut self, max_warrior_length: u16) -> Self {
        self.max_warrior_length = Some(max_warrior_length);
        self
    }
}

#[derive(Copy, Clone, Debug)]
pub enum Winner {
    Left(u16),
    Right(u16),
    Tie,
}

impl Add for Winner {
    type Output = Winner;

    fn add(self, other: Winner) -> Winner {
        match (self, other) {
            (Winner::Tie, y) => y,
            (x, Winner::Tie) => x,
            (Winner::Right(x), Winner::Right(y)) => Winner::Right(x + y),
            (Winner::Left(x), Winner::Left(y)) => Winner::Left(x + y),
            (Winner::Right(x), Winner::Left(y)) => {
                if x > y {
                    Winner::Right(x)
                } else if x < y {
                    Winner::Left(y)
                } else {
                    Winner::Tie
                }
            }
            (Winner::Left(x), Winner::Right(y)) => {
                if x > y {
                    Winner::Left(x)
                } else if x < y {
                    Winner::Right(y)
                } else {
                    Winner::Tie
                }
            }
        }
    }
}

impl Drop for Mars {
    fn drop(&mut self) {
        // The fields that Rust does not allocate inside the Mars struct are:
        //
        //  * core_mem
        //  * queue_mem
        //  * war_tab
        //
        // We've editted sim_free_bufs to deallocate only those
        // structures. Everything else is dropped by Rust's allocator.
        unsafe { sim_free_bufs(self) }
    }
}

impl Mars {
    /// Competes two Individuals at random locations
    ///
    /// The return of this function indicates the winner. `Winner::Right(12)`
    /// will mean that the 'right' player won 12 more rounds than did
    /// 'left'. This may mean they tied 40_000 times or maybe they only played
    /// 12 rounds and right won each time.
    pub fn compete(&mut self, rounds: u16, left: &Individual, right: &Individual) -> Winner {
        let mut wins = Winner::Tie;
        for _ in 0..rounds {
            let core_size = self.core_size;
            let half_core = (core_size / 2) - self.max_warrior_length;
            let upper_core = core_size - self.max_warrior_length;
            let left_pos = thread_rng().gen_range(0, upper_core);
            let right_pos = if (left_pos + self.max_warrior_length) < half_core {
                thread_rng().gen_range(half_core + 1, upper_core)
            } else {
                thread_rng().gen_range(0, half_core)
            };
            wins = wins + self.compete_inner(left, left_pos, right, right_pos);
        }
        tally_fitness(wins);
        BATTLES.fetch_add(1, Ordering::Relaxed);
        wins
    }

    /// INTERNAL FUNCTION, USE ONLY FOR FUZZING
    pub fn compete_inner(
        &mut self,
        left: &Individual,
        left_pos: u16,
        right: &Individual,
        right_pos: u16,
    ) -> Winner {
        let (left_len, left_code) = left.as_ptr();
        let (right_len, right_code) = right.as_ptr();

        let warrior_position_table: Vec<u16> = vec![left_pos, right_pos];
        let mut deaths: Vec<u32> = vec![u32::max_value(), u32::max_value()];
        unsafe {
            sim_clear_core(self);
            assert_eq!(
                0,
                sim_load_warrior(self, left_pos.into(), left_code, left_len)
            );
            assert_eq!(
                0,
                sim_load_warrior(self, right_pos.into(), right_code, right_len)
            );
            let alive = sim_mw(self, warrior_position_table.as_ptr(), deaths.as_mut_ptr());
            assert_ne!(-1, alive);
        }
        let left_dead = deaths[0] != u32::max_value();
        let right_dead = deaths[1] != u32::max_value();
        match (left_dead, right_dead) {
            (false, false) | (true, true) => Winner::Tie,
            (true, false) => Winner::Right(1),
            (false, true) => Winner::Left(1),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use individual::*;

    #[test]
    fn imp_vs_dwarf() {
        let core_size = 8_000;
        let rounds = 100;
        let mut mars = MarsBuilder::default().core_size(core_size).freeze();
        let imp = ringers::imp(core_size);
        let dwarf = ringers::dwarf(core_size);
        let res = mars.compete(rounds, &imp, &dwarf);
        println!("RES: {:?}", res);
        match res {
            Winner::Left(_) | Winner::Tie => panic!("imp should lose to dwarf"),
            Winner::Right(_) => {}
        }
    }
}
