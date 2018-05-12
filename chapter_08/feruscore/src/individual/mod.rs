use instruction::{Instruction, OpCode};
use rand::{thread_rng, Rng};
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, BufWriter, Write};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};
use tempdir::TempDir;

pub mod ringers;

const ROUNDS: usize = 100;
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

#[derive(Debug)]
pub struct Individual {
    chromosome: Vec<Option<Instruction>>,
}

pub enum Winner {
    Left(u16),
    Right(u16),
    Tie,
}

fn tally_fitness(score: usize) -> () {
    assert!(score <= ROUNDS);

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

impl Individual {
    pub fn new(chromosome_size: u16, core_size: u16) -> Individual {
        let mut chromosome = Vec::with_capacity(chromosome_size as usize);
        chromosome.par_extend((0..(chromosome_size as usize)).into_par_iter().map(|_| {
            if thread_rng().gen_weighted_bool(OpCode::total() * 2) {
                None
            } else {
                Some(Instruction::random(core_size))
            }
        }));
        Individual { chromosome }
    }

    pub fn serialize(&self, w: &mut Write) -> io::Result<usize> {
        let mut total_wrote = 0;
        for inst in &self.chromosome {
            if let Some(inst) = inst {
                total_wrote += inst.serialize(w)?;
            } else {
                break;
            }
        }
        Ok(total_wrote)
    }

    pub fn mutate(&mut self, mutation_chance: u32, core_size: u16) -> () {
        self.chromosome.par_iter_mut().for_each(|gene| {
            if thread_rng().gen_weighted_bool(mutation_chance) {
                *gene = if thread_rng().gen::<bool>() {
                    Some(Instruction::random(core_size))
                } else {
                    None
                };
            }
        });
    }

    pub fn reproduce(&self, partner: &Individual, child: &mut Individual) -> () {
        for (idx, (lgene, rgene)) in self.chromosome
            .iter()
            .zip(partner.chromosome.iter())
            .enumerate()
        {
            child.chromosome[idx] = if thread_rng().gen::<bool>() {
                *lgene
            } else {
                *rgene
            };
        }
    }

    pub fn compete(&self, other: &Individual) -> Winner {
        let dir = TempDir::new("simulate").expect("could not make tempdir");

        let l_path = dir.path().join("left.red");
        let mut lfp = BufWriter::new(File::create(&l_path).expect("could not write lfp"));
        self.serialize(&mut lfp).expect("could not serialize");
        drop(lfp);

        let r_path = dir.path().join("right.red");
        let mut rfp = BufWriter::new(File::create(&r_path).expect("could not write rfp"));
        other.serialize(&mut rfp).expect("could not serialize");
        drop(rfp);

        let output = Command::new("pmars")
            .arg("-r") // Rounds to play
            .arg(format!("{}", ROUNDS))
            .arg("-b") // Brief mode (no source listings)
            .arg(&l_path)
            .arg(&r_path)
            .output()
            .expect("failed to execute process");
        let result_line = ::std::str::from_utf8(&output.stdout)
            .expect("could not parse output")
            .lines()
            .last()
            .expect("no output");

        // Lines look like:
        //
        //             right wins
        //             |
        // Results: 10 55 32
        //          |     |
        //  left wins     ties
        let pieces = result_line.split(' ').collect::<Vec<&str>>();

        let l_wins = pieces[1].parse::<u16>().expect("could not parse l_wins");
        let r_wins = pieces[2].parse::<u16>().expect("could not parse r_wins");
        let ties = pieces[3].parse::<u16>().expect("could not parse ties");
        assert_eq!((l_wins + r_wins + ties) as usize, ROUNDS);

        tally_fitness(l_wins as usize);
        tally_fitness(r_wins as usize);

        if l_wins > r_wins {
            Winner::Left(l_wins)
        } else if l_wins < r_wins {
            Winner::Right(r_wins)
        } else {
            Winner::Tie
        }
    }
}
