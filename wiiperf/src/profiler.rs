use core::{cell::RefCell, cmp::Reverse, ptr};

use arrayvec::ArrayVec;
use wiistd::{ppc::InterruptLock, println, util::ToUsize};

use crate::assembler;

const SAMPLE_COUNT: usize = 1024;

struct Samples {
    // Possible space optimization: low 2 bits are always zero due to word-aligned instructions.
    samples: [u32; SAMPLE_COUNT],
    index: usize,
}

static SAMPLES: InterruptLock<RefCell<Samples>> = InterruptLock::new(RefCell::new(Samples {
    samples: [0; SAMPLE_COUNT],
    index: 0,
}));

const FREQUENCY_MAP_COUNT: usize = 512;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct FreqEntry {
    addr: u32,
    count: u32,
}

#[derive(Debug)]
struct FrequencyMap(ArrayVec<FreqEntry, FREQUENCY_MAP_COUNT>);

static FREQ_MAP: InterruptLock<RefCell<FrequencyMap>> =
    InterruptLock::new(RefCell::new(FrequencyMap(ArrayVec::new_const())));

fn find_fn_start(addr: u32) -> u32 {
    let mut addr = ptr::with_exposed_provenance::<u32>(addr.usize());
    loop {
        let value = unsafe { *addr };

        let is_start_instr = matches!(
            assembler::decode_instr(value),
            Some(assembler::Instruction::Stwu {
                source: 1,
                dest: 1,
                ..
            })
        );
        if is_start_instr {
            return addr as u32;
        } else {
            addr = addr.wrapping_offset(-1);
        }
    }
}

fn populate_freq_map_from_samples(samples: &Samples, freq_map: &mut FrequencyMap) {
    for &address in &samples.samples {
        if address != 0 {
            let fn_addr = find_fn_start(address);

            let freq = freq_map
                .0
                .binary_search_by_key(&fn_addr, |&FreqEntry { addr, .. }| addr);

            match freq {
                Ok(index) => {
                    debug_assert_eq!(freq_map.0[index].addr, fn_addr); // Sanity check that we maintain sorting invariant

                    freq_map.0[index].count += 1;
                }
                Err(insert_idx) => {
                    let entry = FreqEntry {
                        addr: fn_addr,
                        count: 1,
                    };

                    if freq_map.0.try_insert(insert_idx, entry).is_err() {
                        // Freqmap is full. Let's find the entry with the lowest frequency and remove it.
                        // This may result in some inaccuracies if we keep evicting the same entries...
                        let (lowest_idx, _) = freq_map
                            .0
                            .iter()
                            .enumerate()
                            .min_by_key(|&(_, &FreqEntry { count, .. })| count)
                            .unwrap();

                        // Two scenarios:
                        // lowest_idx (L) >= insert_idx (I):         I     L
                        //                                      |_|_|_|_|_|_|_|_|
                        // Rotate everything from I to L one to the right such that L is at I
                        //
                        // lowest_idx (L) < insert_idx (I):          L     I
                        //                                      |_|_|_|_|_|_|_|_|
                        //
                        // Rotate everything from L to I-1 one to the left such that L is removed and I is at I-1
                        if lowest_idx >= insert_idx {
                            freq_map.0[insert_idx..=lowest_idx].rotate_right(1);
                            freq_map.0[insert_idx] = entry;
                        } else {
                            freq_map.0[lowest_idx..insert_idx - 1].rotate_left(1);
                            freq_map.0[insert_idx - 1] = entry;
                        }
                    }
                }
            }
        }
    }
}

pub fn handle_interrupt(srr0: u32) {
    SAMPLES.with_cell_mut(|samples| {
        samples.samples[samples.index] = srr0;
        samples.index = (samples.index + 1) % samples.samples.len();

        if samples.index == 0 {
            FREQ_MAP.with_cell_mut(|freq_map| {
                populate_freq_map_from_samples(samples, freq_map);
            });
            dump_results();
        }
    });
}

pub fn dump_results() {
    FREQ_MAP.with_cell_mut(|freq_map| {
        let mut sorted = ArrayVec::<FreqEntry, 10>::new();

        for entry in &freq_map.0 {
            let index = sorted
                .binary_search_by_key(&Reverse(entry.count), |&FreqEntry { count, .. }| {
                    Reverse(count)
                });

            match index {
                Ok(index) | Err(index) => {
                    if sorted.try_insert(index, *entry).is_err() && index < sorted.len() {
                        sorted[index..].rotate_right(1);
                        sorted[index] = *entry;
                    }
                }
            }
        }

        println!("Address     Count");
        for &FreqEntry { addr, count } in sorted.iter().take(10) {
            println!("{addr:#010x}  {count}");
        }
    });
}
