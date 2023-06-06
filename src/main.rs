use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::ops::DerefMut;

use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use rand::prelude::ThreadRng;

struct Sampler {
    random: RefCell<ThreadRng>,
    distribution: WeightedIndex<f64>,
    source: Vec<u64>,
}

struct LeaseTable {
    table: HashMap<u64, (u64, u64, f64)>,
}

impl LeaseTable {
    fn read_lease_look_up_table_from_csv(file_path: &str) -> LeaseTable {
        let file = File::open(file_path).unwrap();
        let mut rdr = csv::ReaderBuilder::new()
            .from_reader(file);
        let mut result: HashMap<u64, (u64, u64, f64)> = HashMap::new();
        for results in rdr.records() {
            let record = results.expect("Error reading CSV record");
            let access_tag = record[0].parse::<u64>().unwrap();
            let short = record[1].parse::<u64>().unwrap();
            let long = record[2].parse::<u64>().unwrap();
            let short_prob = record[3].parse::<f64>().unwrap();
            result.insert(access_tag, (short, long, short_prob));
        }
        LeaseTable {
            table: result,
        }
    }

    fn new(filename: &str) -> LeaseTable {
        LeaseTable::read_lease_look_up_table_from_csv(filename)
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
        let mut rdr = csv::ReaderBuilder::new()
            .from_reader(file);
        let mut result: Vec<TraceItem> = Vec::new();
        for results in rdr.records() {
            let record = results.expect("Error reading CSV record");
            let access_tag = record[0].parse::<u64>().unwrap();
            let reference = record[1].parse::<u64>().unwrap();
            result.push(TraceItem::new(access_tag, reference));
        }
        Trace {
            accesses: result,
        }
    }

    fn new(filename: &str) -> Trace {
        Trace::read_from_csv(filename)
    }

    fn query_for_lease(&self, access_tag: u64) -> Result<(u64, u64, f64), (u64, u64, f64)> {
        for item in &self.accesses {
            if item.access_tag == access_tag {
                return Ok((item.access_tag, item.reference, 0.0));
            }
        }
        Err((0, 0, 0.0))
    }
}

struct CacheBlock {
    size: u64,
    address: u64,
    tag: u64,
    set_index: u64,
    block_offset: u64,
    remaining_lease: u64,
    tenancy: u64,
}

struct CacheSet {
    size: u64,
    blocks: Vec<CacheBlock>,
}

struct Cache {
    size: u64,
    sets: Vec<CacheSet>,
}


impl CacheSet {
    fn new(size: u64) -> CacheSet {
        let mut blocks: Vec<CacheBlock> = Vec::new();
        for _ in 0..size {
            blocks.push(CacheBlock::new());
        }
        CacheSet {
            size,
            blocks,
        }
    }
}


impl CacheBlock {
    fn new() -> CacheBlock {
        CacheBlock {
            size: 0,
            address: 0,
            tag: 0,
            set_index: 0,
            block_offset: 0,
            remaining_lease: 0,
            tenancy: 0,
        }
    }


    fn read_from_table(&self, address: u64) -> bool {
        if self.address == address {
            return true;
        }
        return false;
    }
}


// impl Sampler {
//     fn new<T: Iterator<Item = (u64, f64)>>(t: T) -> Sampler {
//         let r = RefCell::new(rand::thread_rng());
//         let vector: Vec<(u64, f64)> = t.into_iter().collect(); //Guarantees our index ordering.
//         let distribution = WeightedIndex::new(vector.iter().map(|(_, weight)| *weight)).unwrap();
//         let source = vector.into_iter().map(|(item, _)| item).collect();
//
//         Sampler {
//             random: r,
//             distribution,
//             source,
//         }
//     }
//
//     fn sample(&self) -> u64 {
//         let index = self
//             .distribution
//             .sample(self.random.borrow_mut().deref_mut());
//         self.source[index]
//     }
// }

// struct Simulator {
//     size: u64,
//     tracker: HashMap<u64, u64>,
//     step: u64,
// }
//
// impl Simulator {
//     fn init() -> Simulator {
//         Simulator {
//             size: 0,
//             tracker: HashMap::new(),
//             step: 0,
//         }
//     }
//
//     fn add_tenancy(&mut self, tenancy: u64) {
//         self.update();
//         self.size += 1;
//         let target = tenancy + self.step;
//         let expirations_at_step = self.tracker.get(&target).copied().unwrap_or(0);
//         self.tracker.insert(target, expirations_at_step + 1);
//     }
//
//     fn update(&mut self) {
//         self.step += 1;
//         self.size -= self.tracker.remove(&self.step).unwrap_or(0);
//     }
//
//
//     fn _get_size(&self) -> u64 {
//         self.size
//     }
// }


impl Cache {
    fn new(size: u64, associativity: u64) -> Cache {
        let mut sets: Vec<CacheSet> = Vec::new();
        for _ in 0..size / associativity {
            sets.push(CacheSet::new(associativity));
        }
        Cache {
            size,
            sets,
        }
    }


    fn insert(&mut self, address: u64, tenancy: u64, lease: u64) {
        let set_index = address % self.size;
        let mut set = &mut self.sets[set_index as usize];
        let mut block = &mut set.blocks[0];
        block.address = address;
        block.tenancy = tenancy;
        block.remaining_lease = lease;
    }
}


fn pack_to_cache_block(input: TraceItem, offset: u64, set: u64, trace: Trace) -> Result<CacheBlock, CacheBlock> {
    let mut result = CacheBlock::new();
    result.address = input.access_tag;
    result.block_offset = input.access_tag & ((1 << offset) - 1);
    result.set_index = (input.access_tag >> offset) & ((1 << set) - 1);
    result.tag = input.access_tag >> (offset + set);
    let lease = trace.query_for_lease(input.reference).expect("Error in query lease for the access");
    result.remaining_lease = lease.0;
    result.tenancy = 0;
    Ok(result)
}


// fn caching(ten_dist: Sampler, _cache_size: u64, _delta: f64, length:usize) -> Vec<u64> {
//     let mut cache = Simulator::init();
//     let samples_to_issue: u64 = length as u64;
//     let mut prev_output: Vec<u64> = vec![0; length + 1];
//     let mut dcsd_observed = vec![0; length + 1];
//     let mut time = 0;
//     loop {
//
//         //this part of code is for warmup cycles, but currently unused.
//         if time >= 0{
//             break
//         }
//         let tenancy = ten_dist.sample();
//         cache.add_tenancy(tenancy);
//         time += 1;
//     }
//
// //
//     let mut cycles = 0;
//     loop {
//         if cycles > 100000{//this is the main loop, larger numbers of loop gives higher precisions
//             return dcsd_observed.clone();
//         }
//         for _ in 0..samples_to_issue -1 {
//             let tenancy = ten_dist.sample();
//             cache.add_tenancy(tenancy);
//             dcsd_observed[cache.size as usize] += 1;
//         }
//
//         prev_output = dcsd_observed.clone();
//         cycles += 1;
//     }
// }


// fn get_sum(input:&Vec<u64>) -> u128{
//     let mut sum:u128 = 0;
//     let mut index:usize = 0;
//     for k in input{
//         sum += *k as u128;
//         if index == input.len(){
//             break;
//         }
//         index += 1;
//     }
//     if sum == 0{
//         return 1;
//     }
//     return sum;
// }


fn input_to_hashmap() -> (HashMap<u64, f64>, usize) {
    let mut rdr = csv::ReaderBuilder::new()
        .from_reader(io::stdin());
    let mut _result: HashMap<u64, f64> = HashMap::new();
    let mut largest = 0;
    for result in rdr.records() {
        let record = result.unwrap();
        if record.get(0).unwrap().parse::<usize>().unwrap() > largest {
            largest = record.get(0).unwrap().parse().unwrap();
        }
        _result.insert(record.get(0).unwrap().parse().unwrap(), record.get(1).unwrap().parse().unwrap());
    }
    return (_result, largest);
}


fn write(output: Vec<u64>) {
    let sum = get_sum(&output);
    let mut wtr = csv::Writer::from_writer(io::stdout());
    let mut index: usize = 0;
    wtr.write_record(&["DCS", "probability"]).expect("cannot write");
    for key in output {
        wtr.write_record(&[index.to_string(), ((key as f64) / sum as f64).to_string()]).expect("cannot write");
        index += 1;
    }
}


fn main() {
    //
    // let test = input_to_hashmap();
    // let test_1 = caching(Sampler::new(test.0.into_iter()), 10, 0.005, test.1);
    // write(test_1);
    let file_path = "./fakeTable.csv";
    let test_table = LeaseTable::new(file_path);
    //print out the test table
    for _i in test_table.table.iter() {
        println!("1");
    }
}