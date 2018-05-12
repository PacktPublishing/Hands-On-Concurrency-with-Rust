use instruction::Instruction;
use rand::{thread_rng, Rng};
use std::io::{self, Write};
use std::marker::PhantomData;
use std::mem;

pub mod ringers;

#[derive(Debug)]
pub struct IndividualBuilder {
    chromosome: Vec<Instruction>,
    start: usize,
}

impl IndividualBuilder {
    pub fn new() -> IndividualBuilder {
        IndividualBuilder {
            chromosome: Vec::with_capacity(128),
            start: 0,
        }
    }

    pub fn from_chromosome(chromosome: Vec<Instruction>) -> Self {
        IndividualBuilder {
            chromosome,
            start: 0,
        }
    }

    /// If OK, keep pushing. If Err, stop.
    pub fn push(mut self, inst: Instruction) -> Self {
        self.chromosome.push(inst);
        self
    }

    pub fn start(mut self, start: usize) -> Self {
        self.start = start;
        self
    }

    pub fn freeze(mut self) -> Individual {
        let len = self.chromosome.len() as u16;
        let start = self.start % self.chromosome.len();
        if let Some(gene) = self.chromosome.get_mut(start) {
            (*gene).start();
        }
        let mut chromosome = Box::new(self.chromosome);
        let chromosome_ptr = chromosome.as_mut_ptr();
        mem::forget(chromosome);
        Individual {
            chromosome: chromosome_ptr,
            len: len,
        }
    }
}

unsafe impl Send for Individual {}

#[derive(Debug)]
#[repr(C)]
pub struct Individual {
    chromosome: *mut Instruction,
    len: u16,
}

pub struct Iter<'a> {
    cur: isize,
    stop: isize,
    chromosome: *const Instruction,
    phantom: PhantomData<&'a Instruction>,
}

impl<'a> Iterator for Iter<'a> {
    type Item = &'a Instruction;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = (self.stop - self.cur) as usize;
        (rem, Some(rem))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.stop {
            None
        } else {
            unsafe {
                let inst = &*self.chromosome.offset(self.cur);
                self.cur += 1;
                Some(inst)
            }
        }
    }
}

pub struct IterMut<'a> {
    cur: isize,
    stop: isize,
    chromosome: *mut Instruction,
    phantom: PhantomData<&'a Instruction>,
}

impl<'a> Iterator for IterMut<'a> {
    type Item = &'a mut Instruction;

    fn size_hint(&self) -> (usize, Option<usize>) {
        let rem = (self.stop - self.cur) as usize;
        (rem, Some(rem))
    }

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur == self.stop {
            None
        } else {
            unsafe {
                let inst = &mut *self.chromosome.offset(self.cur);
                self.cur += 1;
                Some(inst)
            }
        }
    }
}

impl Individual {
    pub fn new(max_chromosome_size: u16) -> Individual {
        let mut builder = IndividualBuilder::new();
        let len = thread_rng().gen_range(1, max_chromosome_size);
        builder = builder.start(thread_rng().gen_range(0, len as usize));
        for _ in 0..len {
            builder = builder.push(thread_rng().gen::<Instruction>());
        }
        builder.freeze()
    }

    pub fn len(&self) -> usize {
        self.len as usize
    }

    pub fn iter(&self) -> Iter {
        Iter {
            cur: 0,
            stop: self.len as isize,
            chromosome: self.chromosome,
            phantom: PhantomData,
        }
    }

    pub fn iter_mut(&self) -> IterMut {
        IterMut {
            cur: 0,
            stop: self.len as isize,
            chromosome: self.chromosome,
            phantom: PhantomData,
        }
    }

    pub fn mutate(&mut self, mutation_chance: u32) -> () {
        self.iter_mut().for_each(|gene| {
            if thread_rng().gen_weighted_bool(mutation_chance) {
                *gene = thread_rng().gen::<Instruction>();
            }
        });
    }

    pub fn reproduce(&self, partner: &Individual) -> Individual {
        let mut child = IndividualBuilder::new();
        for (lgene, rgene) in self.iter().zip(partner.iter()) {
            child = if thread_rng().gen::<bool>() {
                child.push(*lgene)
            } else {
                child.push(*rgene)
            };
        }
        child.freeze()
    }

    pub fn as_ptr(&self) -> (u16, *const Instruction) {
        (self.len, self.chromosome)
    }

    pub fn serialize(&self, w: &mut Write) -> io::Result<usize> {
        let mut total_wrote = 0;
        for inst in self.iter() {
            total_wrote += inst.serialize(w)?;
        }
        Ok(total_wrote)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use quickcheck::{QuickCheck, TestResult};

    #[test]
    fn children_inherit_shortest_length() {
        fn inner(max_chromosome_size: u16) -> TestResult {
            if max_chromosome_size < 2 {
                return TestResult::discard();
            }
            let left = Individual::new(max_chromosome_size);
            let right = Individual::new(max_chromosome_size);
            let child = left.reproduce(&right);

            let shortest = left.len().min(right.len());
            assert_eq!(child.len(), shortest);
            TestResult::passed()
        }
        QuickCheck::new().quickcheck(inner as fn(u16) -> TestResult);
    }

}
