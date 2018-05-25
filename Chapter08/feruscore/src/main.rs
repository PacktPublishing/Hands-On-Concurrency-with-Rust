extern crate feruscore;
extern crate rand;
extern crate rayon;

use feruscore::individual::*;
use rand::{thread_rng, Rng};
use rayon::prelude::*;
use std::collections::VecDeque;
use std::fs::File;
use std::fs::{self, DirBuilder};
use std::io::{self, BufWriter};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::{thread, time};

// configuration
const POPULATION_SIZE: u16 = 256; // NOTE must be powers of two
const CHROMOSOME_SIZE: u16 = 25; // Must be <= 100
const PARENTS: u16 = POPULATION_SIZE / 8;
const CHILDREN: u16 = PARENTS * 2;
const RANDOS: u16 = POPULATION_SIZE - (PARENTS + CHILDREN);
const CORE_SIZE: u16 = 8000;
const GENE_MUTATION_CHANCE: u32 = 100;

// reporting
static GENERATIONS: AtomicUsize = AtomicUsize::new(0);
static REGIONAL_BATTLES: AtomicUsize = AtomicUsize::new(0);
static FINALS_BATTLES: AtomicUsize = AtomicUsize::new(0);
static STATE: AtomicUsize = AtomicUsize::new(0);
static NEW_PARENTS: AtomicUsize = AtomicUsize::new(0);

#[derive(Debug)]
enum State {
    Tournament,
    Reproduce,
    Mutate,
}

fn report() -> () {
    let delay_millis = 1_000;
    let delay = time::Duration::from_millis(delay_millis);
    let adjustment = (delay_millis / 1000) as usize;

    let mut total_battles = 0;
    let mut last_gens = 0;
    let mut runtime: usize = 0;
    let mut total_parents = 0;
    let mut fitness = [0; 10];
    loop {
        let generation = GENERATIONS.load(Ordering::Relaxed);
        if generation > last_gens {
            total_parents = 0;
            for i in 0..10 {
                fitness[i] = 0;
            }
            total_battles = 0;
        }
        let regional_battles = REGIONAL_BATTLES.swap(0, Ordering::Relaxed);
        let finals_battles = FINALS_BATTLES.swap(0, Ordering::Relaxed);
        let parents = NEW_PARENTS.swap(0, Ordering::Relaxed);
        let gens_per_second = (generation - last_gens) / adjustment;
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

        total_battles += regional_battles + finals_battles;
        total_parents += parents;

        let state = match STATE.load(Ordering::Relaxed) {
            0 => State::Tournament,
            1 => State::Reproduce,
            2 => State::Mutate,
            _ => unreachable!(),
        };
        println!("GENERATION({}):", generation);
        println!("    STATE:          {:?}", state);
        println!("    RUNTIME (sec):  {}", runtime / adjustment);
        println!("    GENS/s:         {}", gens_per_second);
        println!("    PARENTS:        {}", total_parents);
        println!("       PARENTS/s:   {}", parents / adjustment);
        println!("    BATTLES:        {}", total_battles);
        println!("       R_BATTLES/s: {}", regional_battles / adjustment);
        println!("       F_BATTLES/s: {}", finals_battles / adjustment);
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

fn regional_tournament(
    (chmp, mut population): (Option<Individual>, Vec<Individual>),
    indv: Individual,
) -> (Option<Individual>, Vec<Individual>) {
    if let Some(chmp) = chmp {
        REGIONAL_BATTLES.fetch_add(1, Ordering::Relaxed);
        match chmp.compete(&indv) {
            Winner::Left(_) | Winner::Tie => {
                population.push(indv);
                (Some(chmp), population)
            }
            Winner::Right(_) => {
                population.push(chmp);
                (Some(indv), population)
            }
        }
    } else {
        (Some(indv), population)
    }
}

fn finals_tournament(
    (left, mut lpop): (Option<Individual>, Vec<Individual>),
    (right, rpop): (Option<Individual>, Vec<Individual>),
) -> (Option<Individual>, Vec<Individual>) {
    if let Some(left) = left {
        lpop.extend(rpop);
        if let Some(right) = right {
            FINALS_BATTLES.fetch_add(1, Ordering::Relaxed);
            match left.compete(&right) {
                Winner::Left(_) | Winner::Tie => {
                    lpop.push(right);
                    (Some(left), lpop)
                }
                Winner::Right(_) => {
                    lpop.push(left);
                    (Some(right), lpop)
                }
            }
        } else {
            (Some(left), lpop)
        }
    } else {
        assert!(lpop.is_empty());
        (right, rpop)
    }
}

fn checkpoint(generation: usize, best: &VecDeque<Individual>) -> io::Result<()> {
    let root: PathBuf = Path::new("/tmp/feruscore/checkpoints").join(generation.to_string());
    DirBuilder::new().recursive(true).create(&root)?;
    assert!(fs::metadata(&root).unwrap().is_dir());

    for (idx, indv) in best.iter().enumerate() {
        let path = root.join(format!("{:04}.red", idx));
        let mut fp = BufWriter::new(File::create(&path).expect("could not write lfp"));
        indv.serialize(&mut fp)?;
    }

    Ok(())
}

fn main() {
    let _ = thread::spawn(report);

    // initial, random generation
    let mut population: Vec<Individual> = Vec::with_capacity(POPULATION_SIZE as usize);
    let mut children: Vec<Individual> = Vec::with_capacity(CHILDREN as usize);
    let mut parents: VecDeque<Individual> = VecDeque::with_capacity(PARENTS as usize);
    population.par_extend(
        (0..POPULATION_SIZE)
            .into_par_iter()
            .map(|_| Individual::new(CHROMOSOME_SIZE, CORE_SIZE)),
    );
    population.pop();
    population.pop();
    population.push(ringers::imp(CHROMOSOME_SIZE));
    population.push(ringers::dwarf(CHROMOSOME_SIZE));

    loop {
        // tournament, fitness and selection of parents
        STATE.store(0, Ordering::Release);
        while parents.len() < PARENTS as usize {
            thread_rng().shuffle(&mut population);
            let res = population
                .into_par_iter()
                .fold(|| (None, Vec::new()), regional_tournament)
                .reduce(|| (None, Vec::new()), finals_tournament);
            population = res.1;
            parents.push_back(res.0.unwrap());
            NEW_PARENTS.fetch_add(1, Ordering::Relaxed);
        }

        // reproduce
        STATE.store(1, Ordering::Release);
        while children.len() < CHILDREN as usize {
            let parent0 = parents.pop_front().expect("no parent0");
            let parent1 = parents.pop_front().expect("no parent1");
            let mut child = population.pop().expect("no child");
            parent0.reproduce(&parent1, &mut child);
            children.push(child);
            parents.push_back(parent0);
            parents.push_back(parent1);
        }

        assert_eq!(children.len(), CHILDREN as usize);
        assert_eq!(parents.len(), PARENTS as usize);
        assert_eq!(population.len(), RANDOS as usize);

        population.append(&mut children);

        // mutation
        STATE.store(2, Ordering::Release);
        population.par_iter_mut().for_each(|indv| {
            let _ = indv.mutate(GENE_MUTATION_CHANCE, CORE_SIZE);
        });

        let generation = GENERATIONS.fetch_add(1, Ordering::Relaxed);
        if generation % 100 == 0 {
            checkpoint(generation, &parents).expect("could not checkpoint");
        }

        for parent in parents.drain(..) {
            population.push(parent);
        }
    }
}
