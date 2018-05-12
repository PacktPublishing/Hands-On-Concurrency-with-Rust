extern crate feruscore;
extern crate num_cpus;
extern crate rand;

use feruscore::individual::*;
use feruscore::mars::*;
use rand::{thread_rng, Rng};
use std::fs::{self, DirBuilder, File};
use std::io::{self, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::mpsc;
use std::{thread, time};

// configuration
const POPULATION_SIZE: usize = 1_048_576 / 8; // NOTE ideally, a power of two
const CHROMOSOME_SIZE: u16 = 100;
const CORE_SIZE: u16 = 8000;
const GENE_MUTATION_CHANCE: u32 = 100;
const ROUNDS: u16 = 100;

// reporting
static GENERATIONS: AtomicUsize = AtomicUsize::new(0);

fn report() -> () {
    let delay_millis = 1_000;
    let delay = time::Duration::from_millis(delay_millis);
    let adjustment = (delay_millis / 1000) as usize;

    let mut total_battles = 0;
    let mut last_gens = 0;
    let mut runtime: usize = 0;
    let mut fitness = [0; 10];
    loop {
        let generation = GENERATIONS.load(Ordering::Relaxed);
        if generation > last_gens {
            for ft in fitness.iter_mut().take(10) {
                *ft = 0;
            }
            total_battles = 0;
        }
        let battles = BATTLES.swap(0, Ordering::Relaxed);
        fitness[0] += FITNESS_00010.swap(0, Ordering::Relaxed);
        fitness[1] += FITNESS_11020.swap(0, Ordering::Relaxed);
        fitness[2] += FITNESS_21030.swap(0, Ordering::Relaxed);
        fitness[3] += FITNESS_31040.swap(0, Ordering::Relaxed);
        fitness[4] += FITNESS_41050.swap(0, Ordering::Relaxed);
        fitness[5] += FITNESS_51060.swap(0, Ordering::Relaxed);
        fitness[6] += FITNESS_61070.swap(0, Ordering::Relaxed);
        fitness[7] += FITNESS_71080.swap(0, Ordering::Relaxed);
        fitness[8] += FITNESS_81090.swap(0, Ordering::Relaxed);
        fitness[9] += FITNESS_91100.swap(0, Ordering::Relaxed);

        total_battles += battles;

        println!("GENERATION({}):", generation);
        println!("    RUNTIME (sec):  {}", runtime / adjustment);
        println!("    BATTLES:        {}", total_battles);
        println!("       BATTLES/s:   {}", battles / adjustment);
        println!("    FITNESS:");
        println!("        00...10:    {}", fitness[0]);
        println!("        11...20:    {}", fitness[1]);
        println!("        21...30:    {}", fitness[2]);
        println!("        31...40:    {}", fitness[3]);
        println!("        41...50:    {}", fitness[4]);
        println!("        51...60:    {}", fitness[5]);
        println!("        61...60:    {}", fitness[6]);
        println!("        71...70:    {}", fitness[7]);
        println!("        81...80:    {}", fitness[8]);
        println!("        91...100:   {}", fitness[9]);

        last_gens = generation;
        runtime += (delay_millis / 1000) as usize;
        thread::sleep(delay);
    }
}

fn checkpoint(generation: usize, best: &Vec<Individual>) -> io::Result<()> {
    let root: PathBuf = Path::new("/tmp/feruscore/checkpoints").join(format!("{:04}", generation));
    DirBuilder::new().recursive(true).create(&root)?;
    assert!(fs::metadata(&root).unwrap().is_dir());

    for (idx, indv) in best.iter().rev().enumerate().take(32) {
        let path = root.join(format!("{:04}.red", idx));
        let mut fp = BufWriter::new(File::create(&path).expect("could not write lfp"));
        indv.serialize(&mut fp)?;
    }

    Ok(())
}

fn sort_by_tournament(mars: &mut Mars, mut population: Vec<Individual>) -> Vec<Individual> {
    let mut i = population.len() - 1;
    while i >= (population.len() / 2) {
        let left_idx = thread_rng().gen_range(0, i);
        let mut right_idx = left_idx;
        while left_idx == right_idx {
            right_idx = thread_rng().gen_range(0, i);
        }
        match mars.compete(ROUNDS, &population[left_idx], &population[right_idx]) {
            Winner::Right(_) => {
                population.swap(i, right_idx);
            }
            Winner::Left(_) => {
                population.swap(i, left_idx);
            }
            Winner::Tie => {}
        }
        i -= 1;
    }
    population
}

fn island(
    recv: mpsc::Receiver<Vec<Individual>>,
    snd: mpsc::SyncSender<Vec<Individual>>,
    total_children: usize,
) -> () {
    let mut mars = MarsBuilder::default().core_size(CORE_SIZE).freeze();

    while let Ok(mut population) = recv.recv() {
        // tournament, fitness and selection of parents
        population = sort_by_tournament(&mut mars, population);

        // reproduce
        let mut child_idx = 0;
        let mut parent_idx = population.len() - 1;
        while child_idx < total_children {
            let left_idx = parent_idx;
            let right_idx = parent_idx - 1;
            parent_idx -= 2;

            population[child_idx] = population[left_idx].reproduce(&population[right_idx]);
            child_idx += 1;
            population[child_idx] = population[left_idx].reproduce(&population[right_idx]);
            child_idx += 1;
        }
        for i in 0..32 {
            population[child_idx + i] = Individual::new(CHROMOSOME_SIZE)
        }

        // mutation
        for indv in population.iter_mut() {
            indv.mutate(GENE_MUTATION_CHANCE);
        }

        snd.send(population).expect("could not send");
    }
}

fn main() {
    let mut out_recvs = Vec::with_capacity(num_cpus::get());
    let mut in_sends = Vec::with_capacity(num_cpus::get());

    let total_children = 128;

    let thr_portion = POPULATION_SIZE / num_cpus::get();
    for _ in 0..num_cpus::get() {
        let (in_snd, in_rcv) = mpsc::sync_channel(1);
        let (out_snd, out_rcv) = mpsc::sync_channel(1);
        let mut population: Vec<Individual> = (0..thr_portion)
            .into_iter()
            .map(|_| Individual::new(CHROMOSOME_SIZE))
            .collect();
        population.pop();
        population.pop();
        population.push(ringers::imp(CORE_SIZE));
        population.push(ringers::dwarf(CORE_SIZE));
        let _ = thread::spawn(move || island(in_rcv, out_snd, total_children));
        in_snd.send(population).unwrap();
        in_sends.push(in_snd);
        out_recvs.push(out_rcv);
    }

    let _ = thread::spawn(report);

    let mut mars = MarsBuilder::default().core_size(CORE_SIZE).freeze();
    let mut global_population: Vec<Individual> = Vec::with_capacity(POPULATION_SIZE);
    loop {
        for out_rcv in &mut out_recvs {
            let mut pop = out_rcv.recv().unwrap();
            global_population.append(&mut pop);
        }
        assert_eq!(global_population.len(), POPULATION_SIZE);
        thread_rng().shuffle(&mut global_population);

        let generation = GENERATIONS.fetch_add(1, Ordering::Relaxed);
        if generation % 100 == 0 {
            global_population = sort_by_tournament(&mut mars, global_population);
            checkpoint(generation, &global_population).expect("could not checkpoint");
        }

        let split_idx = global_population.len() / num_cpus::get();

        for in_snd in &mut in_sends {
            let idx = global_population.len() - split_idx;
            let pop = global_population.split_off(idx);
            in_snd.send(pop).unwrap();
        }
    }
}
