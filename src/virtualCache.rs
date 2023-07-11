use std::io;
use std::io::Write;
use crate::cache::CacheBlock;

struct VirtualCacheSet {
    blocks: Vec<CacheBlock>,
}

impl VirtualCacheSet {
    fn new() -> VirtualCacheSet {
        VirtualCacheSet {
            blocks: Vec::new(),
        }
    }

    /// push a cache block to the cache set. If the cache block is already in the cache, refresh it. Otherwise, push it to the cache set.
    fn push_to_set(&mut self, new_block: CacheBlock) {
        //if cacheBlock is in the cache, refresh it
        for block in &mut self.blocks {
            if block.tag == new_block.tag {
                block.remaining_lease = new_block.remaining_lease;
                return;
            }
        }
        // otherwise, push new_block to the cache set
        self.blocks.push(new_block);
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

pub struct VirtualCache {
    sets: Vec<VirtualCacheSet>,
    step: u64,
}

impl VirtualCache {
    pub fn new(associativity: u64) -> VirtualCache {
        let sets: Vec<VirtualCacheSet> = (0..associativity).map(|_| VirtualCacheSet::new()).collect();
        VirtualCache {
            sets,
            step: 0,
        }
    }

    /// update the cache status
    pub fn update(&mut self, block: CacheBlock) {
        // update all cache blocks in all the sets
        self.sets.iter_mut().for_each(|set| set.update());
        let set_index = block.set_index as usize;
        self.sets[set_index].push_to_set(block);
        self.step += 1;
    }

    pub fn print(&self, output_file: &str) -> io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(output_file)?;


        // Calculate the total num of cache blocks in every set
        let mut total = 0;
        self.sets.iter().for_each(|set| {
            total += set.blocks.len();
        });

        writeln!(file, "---The virtual cache status: step: {}, virtual cache size: {}", self.step, total)?;

        self.sets.iter().for_each(|set| {
            writeln!(file, "------------------------------").expect("TODO: panic message");
            // set.blocks.iter().for_each(|block| block.print());
            set.blocks.iter().for_each(|block| writeln!(file, "{}", block.print()).unwrap());
        });

        Ok(())
    }
}

