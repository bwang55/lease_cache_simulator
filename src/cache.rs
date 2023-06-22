use std::io;
use std::io::Write;

use rand::Rng;

// use crate::{LeaseTable, pack_to_cache_block, Trace};

#[derive(Debug, Clone, Copy)]
pub struct CacheBlock {
    _size: u64,
    pub address: u64,
    pub tag: u64,
    pub set_index: u64,
    pub block_offset: u64,
    pub remaining_lease: u64,
    pub tenancy: u64,
}

impl CacheBlock {
    pub fn new() -> CacheBlock {
        CacheBlock {
            _size: 0,
            address: 0,
            tag: 0,
            set_index: 0,
            block_offset: 0,
            remaining_lease: 0,
            tenancy: 0,
        }
    }

    pub fn print(&self) -> String {
        format!(
            "address: {:b}, tag: {:b}, set_index: {:b}, block_offset: {:b}, remaining_lease: {}, tenancy: {}",
            self.address,
            self.tag,
            self.set_index,
            self.block_offset,
            self.remaining_lease,
            self.tenancy
        )
    }
}

struct CacheSet {
    block_num: u64,
    blocks: Vec<CacheBlock>,
    forced_eviction: u64,
}

impl CacheSet {
    fn new(size: u64) -> CacheSet {
        CacheSet {
            block_num: size,
            blocks: Vec::new(),
            forced_eviction: 0,
        }
    }

    /// push a cache block to the cache set. If the cache set is full, evict a cache block randomly. If the cache block is already in the cache, refresh it. Otherwise, push it to the cache set.
    fn push_to_set(&mut self, new_block: CacheBlock) {
        //if cacheBlock is in the cache, refresh it
        for block in &mut self.blocks {
            if block.tag == new_block.tag {
                block.remaining_lease = new_block.remaining_lease;
                return;
            }
        }


        // if cache is full, evict
        if self.blocks.len() == self.block_num as usize {
            self.random_evict();
            self.blocks.push(new_block);
        } else {
            self.blocks.push(new_block);
        }
    }

    fn random_evict(&mut self) -> CacheBlock {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.blocks.len());
        self.forced_eviction += 1;
        self.blocks.remove(index)
    }

    /// update the remaining lease of each cache block in the cache set
    fn update(&mut self) {
        self.blocks.retain(|block| block.remaining_lease > 1);
        self.blocks.iter_mut().for_each(|block| {
            block.tenancy += 1;
            block.remaining_lease -= 1;
        });
    }
}

pub struct Cache {
    _size: u64,
    sets: Vec<CacheSet>,
    step: u64,
    forced_eviction_counter: u64,
}

impl Cache {
    pub fn new(size: u64, associativity: u64) -> Cache {
        let sets: Vec<CacheSet> = (0..associativity).map(|_| CacheSet::new(size / associativity)).collect();
        Cache {
            _size: size,
            sets,
            step: 0,
            forced_eviction_counter: 0,
        }
    }

    /// update the cache status
    pub fn update(&mut self, block: CacheBlock) {
        // update all cache blocks in all the sets
        self.sets.iter_mut().for_each(|set| set.update());
        let set_index = block.set_index as usize;
        self.sets[set_index].push_to_set(block);
        self.step += 1;
        self.forced_eviction_counter += self.sets[set_index].forced_eviction;
    }

    pub fn print(&self, output_file: &str) -> io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(output_file)
            .unwrap();

        writeln!(file, "The cache status:")?;
        writeln!(file, "******************************").unwrap();
        //calculate the total num of cache blocks in every set
        let mut total = 0;
        self.sets.iter().for_each(|set| {
            total += set.blocks.len();
        });

        //print out the current step, total num of cache blocks, and the total num of forced eviction
        writeln!(
            file,
            "step: {}, physical cache size: {}, num of forced eviction: {}",
            self.step, total, self.forced_eviction_counter
        ).unwrap();

        self.sets.iter().for_each(|set| {
            writeln!(file, "------------------------------").expect("TODO: panic message");
            // set.blocks.iter().for_each(|block| block.print());
            set.blocks.iter().for_each(|block| writeln!(file, "{}", block.print()).unwrap());
        });

        // write two empty lines
        writeln!(file, "\n\n")?;

        Ok(())
    }
}