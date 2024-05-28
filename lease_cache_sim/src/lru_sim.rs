use std::collections::{HashMap, VecDeque};
use std::io;
use std::io::Write;
use crate::lease_table::Trace;

#[derive(Debug, Clone)]
pub struct LRUCacheBlock {
    pub tag: u64,
    pub set_index: u64,
    pub valid: bool,
}

impl LRUCacheBlock {
    pub fn new(tag: u64, set_index: u64) -> LRUCacheBlock {
        LRUCacheBlock {
            tag,
            set_index,
            valid: true,
        }
    }

    pub fn print(&self) -> String {
        format!(
            "tag: {:x}, set_index: {:x}, valid: {}",
            self.tag, self.set_index, self.valid
        )
    }
}

// 定义LRU缓存结构体
#[allow(dead_code)]
pub struct LRUCache {
    size: usize,
    sets: Vec<VecDeque<LRUCacheBlock>>,
    cache_map: HashMap<u64, (usize, usize)>, // (set_index, position in VecDeque)
    num_sets: usize,
    associativity: usize,
    miss_counter: u64,
}

impl LRUCache {
    pub fn new(size: usize, num_sets: usize, associativity: usize) -> LRUCache {
        let sets = vec![VecDeque::with_capacity(associativity); num_sets];
        LRUCache {
            size,
            sets,
            cache_map: HashMap::new(),
            num_sets,
            associativity,
            miss_counter: 0,
        }
    }

    pub fn access(&mut self, tag: u64, set_index: usize) {
        if set_index >= self.num_sets {
            panic!("set_index out of bounds");
        }

        if let Some(&(stored_set_index, pos)) = self.cache_map.get(&tag) {
            if stored_set_index == set_index {
                // Cache hit
                if let Some(block) = self.sets[set_index].remove(pos) {
                    self.sets[set_index].push_front(block);
                }
            }
        } else {
            // Cache miss
            self.miss_counter += 1;
            if self.sets[set_index].len() == self.associativity {
                // Evict the least recently used block
                if let Some(lru_block) = self.sets[set_index].pop_back() {
                    self.cache_map.remove(&lru_block.tag);
                }
            }
            let new_block = LRUCacheBlock::new(tag, set_index as u64);
            self.sets[set_index].push_front(new_block);
            self.cache_map.insert(tag, (set_index, 0));
        }

        // Update cache_map with new positions
        for (i, block) in self.sets[set_index].iter().enumerate() {
            self.cache_map.insert(block.tag, (set_index, i));
        }
    }

    #[allow(dead_code)]
    pub fn print(&self, output_file: &str) -> io::Result<()> {
        let mut file = std::fs::OpenOptions::new()
            .append(true)
            .create(true)
            .open(output_file)
            .unwrap();
        writeln!(
            file,
            "LRU Cache status: num of misses: {}",
            self.miss_counter
        )?;
        for (index, set) in self.sets.iter().enumerate() {
            writeln!(file, "*CacheSet index: {}", index)?;
            for block in set {
                writeln!(file, "{}", block.print())?;
            }
        }
        Ok(())
    }

    pub fn calculate_miss_ratio(&self, total_accesses: u64) -> f64 {
        if total_accesses == 0 {
            return 0.0;
        }
        self.miss_counter as f64 / total_accesses as f64
    }
}

pub fn run_lru_simulation(
    trace: Trace,
    cache_size: usize,
    num_sets: usize,
    associativity: usize,
    offset: u64,
    set: u64,
) {
    let mut lru_cache = LRUCache::new(cache_size, num_sets, associativity);
    let mut total_accesses = 0;

    for trace_item in trace {
        let set_index = (trace_item.access_tag >> offset) & ((1 << set) - 1);
        lru_cache.access(trace_item.access_tag, set_index as usize);
        total_accesses += 1;
    }

    // lru_cache.print("lru_cache_output.txt").unwrap();
    println!(
        "Miss ratio: {}",
        lru_cache.calculate_miss_ratio(total_accesses)
    );
}
