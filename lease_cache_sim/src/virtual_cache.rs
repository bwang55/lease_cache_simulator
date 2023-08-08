use crate::CacheBlock;
use std::io;
use std::io::Write;

pub struct VirtualCache {
    sets: Vec<Vec<CacheBlock>>,
    step: u64,
    miss_counter: u64,
}

impl VirtualCache {
    pub fn new(associativity: u64) -> VirtualCache {
        let sets: Vec<Vec<CacheBlock>> = (0..associativity).map(|_| Vec::new()).collect();
        VirtualCache {
            sets,
            step: 0,
            miss_counter: 0,
        }
    }

    /// update the cache status
    pub fn update(&mut self, block: CacheBlock) {
        // update all cache blocks in all the sets
        self.sets.iter_mut().for_each(|set| {
            set.retain(|block| block.remaining_lease > 1);
            set.iter_mut().for_each(|block| {
                block.tenancy += 1;
                block.remaining_lease -= 1;
            });
        });

        let set_index = block.set_index as usize;

        // check if the block is already in the cache set and update it if it is
        if let Some(existing_block) = self.sets[set_index].iter_mut().find(|b| b.tag == block.tag) {
            existing_block.remaining_lease = block.remaining_lease;
        } else {
            // otherwise, push the block to the cache set
            self.sets[set_index].push(block);
            self.miss_counter += 1;
        }

        self.step += 1;
    }

    #[allow(unused)]
    pub fn print(&self, output_file: &str) -> io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(output_file)?;

        // Calculate the total num of cache blocks in every set
        let total: usize = self.sets.iter().map(|set| set.len()).sum();

        writeln!(
            file,
            "---The virtual cache status: step: {}, virtual cache size: {}, num of misses: {}",
            self.step, total, self.miss_counter
        )?;

        for (set_index, set) in self.sets.iter().enumerate() {
            writeln!(file, "Cache set index: {}", set_index)?;

            for block in set {
                writeln!(file, "{}", block.print())?;
            }
        }

        Ok(())
    }

    pub(crate) fn calculate_miss_ratio(&self) -> f64 {
        self.miss_counter as f64 / self.step as f64
    }
}
