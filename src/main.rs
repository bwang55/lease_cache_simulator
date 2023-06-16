use rand::Rng;
use std::collections::HashMap;
use std::fs::File;

//read lease table from csv file and store it in a hashmap, waiting for further query
struct LeaseTable {
    table: HashMap<u64, (u64, u64, f64)>,
}

impl LeaseTable {
    fn read_lease_look_up_table_from_csv(file_path: &str) -> LeaseTable {
        let file = File::open(file_path).unwrap();
        let mut rdr = csv::ReaderBuilder::new().from_reader(file);
        let mut result: HashMap<u64, (u64, u64, f64)> = HashMap::new();

        for results in rdr.records() {
            let record = results.expect("Error reading CSV record");

            let access_tag =
                u64::from_str_radix(&record[0][2..], 16).expect("Error parsing access_tag");
            let short_lease =
                u64::from_str_radix(&record[1][2..], 16).expect("Error parsing short_lease");
            let long_lease =
                u64::from_str_radix(&record[2][2..], 16).expect("Error parsing long_lease");
            let short_prob = record[3].parse::<f64>().expect("Error parsing short_prob");

            result.insert(access_tag, (short_lease, long_lease, short_prob));
        }

        LeaseTable { table: result }
    }

    fn new(filename: &str) -> LeaseTable {
        LeaseTable::read_lease_look_up_table_from_csv(filename)
    }

    fn query(&self, access_tag: &u64) -> Option<(u64, u64, f64)> {
        self.table.get(&access_tag).map(|x| *x)
    }
}

struct TraceItem {
    access_tag: u64,
    reference: u64,
}

impl TraceItem {
    fn new(access_tag: u64, reference: u64) -> TraceItem {
        TraceItem {
            access_tag,
            reference,
        }
    }
}

struct Trace {
    accesses: Vec<TraceItem>,
}

impl Trace {
    fn read_from_csv(file_path: &str) -> Trace {
        let file = File::open(file_path).unwrap();
        let mut rdr = csv::ReaderBuilder::new().from_reader(file);
        let mut result: Vec<TraceItem> = Vec::new();
        for results in rdr.records() {
            let record = results.expect("Error reading CSV record");
            let access_tag =
                u64::from_str_radix(&record[0][2..], 16).expect("Error parsing access_tag");
            let reference =
                u64::from_str_radix(&record[1][2..], 16).expect("Error parsing reference");
            result.push(TraceItem::new(access_tag, reference));
        }
        Trace { accesses: result }
    }

    fn new(filename: &str) -> Trace {
        Trace::read_from_csv(filename)
    }
}

#[derive(Debug, Clone, Copy)]
struct CacheBlock {
    _size: u64,
    address: u64,
    tag: u64,
    set_index: u64,
    block_offset: u64,
    remaining_lease: u64,
    tenancy: u64,
}

struct CacheSet {
    block_num: u64,
    blocks: Vec<CacheBlock>,
    forced_eviction: u64,
}

struct Cache {
    size: u64,
    sets: Vec<CacheSet>,
    step: u64,
    forced_eviction_counter: u64,
}

impl CacheSet {
    fn new(size: u64) -> CacheSet {
        CacheSet {
            block_num: size,
            blocks: Vec::new(),
            forced_eviction: 0,
        }
    }

    fn push_to_set(&mut self, block: CacheBlock) {
        //if cacheBlock is in the cache, refresh it
        for item in &mut self.blocks {
            if item.tag == block.tag {
                item.remaining_lease = block.remaining_lease;
                return;
            }
        }
        //if cache is full, evict
        if self.blocks.len() == self.block_num as usize {
            self.random_evict();
            self.blocks.push(block);
        } else {
            self.blocks.push(block);
        }
    }

    fn random_evict(&mut self) -> CacheBlock {
        let mut rng = rand::thread_rng();
        let index = rng.gen_range(0..self.blocks.len());
        self.forced_eviction += 1;
        self.blocks.remove(index)
    }

    fn update(&mut self) {
        let mut index = 0;
        while index < self.blocks.len() {
            let remaining_lease = self.blocks[index].remaining_lease;
            if remaining_lease <= 1 {
                self.blocks.remove(index);
            } else {
                self.blocks[index].remaining_lease -= 1;
                self.blocks[index].tenancy += 1;
                index += 1;
            }
        }
    }
}

impl CacheBlock {
    fn new() -> CacheBlock {
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

    fn print(&self) {
        println!(
            "address: {:b}, tag: {:b}, set_index: {:b}, block_offset: {:b}, remaining_lease: {}, tenancy: {}",
            self.address,
            self.tag,
            self.set_index,
            self.block_offset,
            self.remaining_lease,
            self.tenancy
        );
    }
}

impl Cache {
    fn new(size: u64, associativity: u64) -> Cache {
        let mut sets: Vec<CacheSet> = Vec::new();
        for _ in 0..associativity {
            sets.push(CacheSet::new(size / associativity));
        }
        Cache {
            size,
            sets,
            step: 0,
            forced_eviction_counter: 0,
        }
    }

    fn update(&mut self, block: CacheBlock) {
        //update all cache blocks
        for set in &mut self.sets {
            set.update();
        }
        let set_index = block.set_index as usize;
        self.sets[set_index].push_to_set(block);
        self.step += 1;
        self.forced_eviction_counter += self.sets[set_index].forced_eviction;
    }

    fn print(&self) {
        println!("The cache status:");
        println!("******************************");
        //caculate the total num of cache blocks in every set
        let mut total = 0;
        for set in &self.sets {
            total += set.blocks.len();
        }
        //print out the current step, total num of cache blocks, and the total num of forced eviction
        println!(
            "step: {}, physical cache size: {}, num of forced eviction: {}",
            self.step, total, self.forced_eviction_counter
        );
        for set in &self.sets {
            println!("------------------------------");
            for block in &set.blocks {
                block.print();
            }
        }
        for _ in 0..2 {
            println!();
        }
    }

    fn run_trace(&mut self, trace: Trace, table: &LeaseTable, offset: u64, set: u64) {
        for item in trace.accesses {
            let block = pack_to_cache_block(&item, offset, set, table)
                .expect("Error in pack_to_cache_block");
            self.update(block);
            self.print();
        }
    }
}

fn pack_to_cache_block(
    input: &TraceItem,
    offset: u64,
    set: u64,
    table: &LeaseTable,
) -> Result<CacheBlock, CacheBlock> {
    let mut result = CacheBlock::new();
    result.address = input.access_tag;
    result.block_offset = input.access_tag & ((1 << offset) - 1);
    result.set_index = (input.access_tag >> offset) & ((1 << set) - 1);
    result.tag = input.access_tag >> (offset + set);
    let lease = table
        .query(&input.reference)
        .expect("Error in query lease for the access");
    //randomly assign remaining_lease according to probability at lease.3
    let mut random = rand::thread_rng();
    if random.gen::<f64>() < lease.2 {
        result.remaining_lease = lease.0;
    } else {
        result.remaining_lease = lease.1;
    }

    result.tenancy = 0;
    Ok(result)
}

fn main() {
    let file_path = "./fakeTable.csv";
    let test_table = LeaseTable::new(file_path);

    let trace_path = "./trace.csv";
    let test_trace = Trace::new(trace_path);

    let mut test_cache = Cache::new(4, 2);

    test_cache.run_trace(test_trace, &test_table, 2, 1);

    // let test = pack_to_cache_block(&test_trace.accesses[0], 2, 1, &test_table);
    // let test2 = pack_to_cache_block(&test_trace.accesses[1], 2, 1, &test_table);
    // let test3 = pack_to_cache_block(&test_trace.accesses[2], 2, 1, &test_table);
    //
    // let mut test_cache = Cache::new(4, 2);
    // test_cache.update(test.unwrap());
    // test_cache.print();
    // test_cache.update(test2.unwrap());
    // test_cache.print();
    // test_cache.update(test3.unwrap());
    // test_cache.print();
}
