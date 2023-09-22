use std::collections::HashMap;
use std::fs::File;
use std::io;
use std::io::{BufRead, BufReader};
use std::time::Instant;

use clap::{Arg, Command};
use csv::{ReaderBuilder, StringRecord};
use rand::Rng;

use crate::cache::{Cache, CacheBlock};
use crate::virtual_cache::VirtualCache;

mod cache;
mod virtual_cache;

//read lease table from csv file and store it in a hashmap, waiting for further query
#[derive(Debug)]
struct LeaseTable {
    table: HashMap<u64, (u64, u64, f64)>,
}

impl LeaseTable {
    #[allow(unused)]
    fn read_lease_look_up_table_from_csv(file_path: &str) -> LeaseTable {
        let file = File::open(file_path).unwrap();
        let mut rdr = csv::ReaderBuilder::new().from_reader(file);
        let mut result: HashMap<u64, (u64, u64, f64)> = HashMap::new();

        for results in rdr.records() {
            let record = results.expect("Error reading CSV record");

            // Convert the strings from base 16 to to numbers with base of 10
            let access_tag =
                u64::from_str_radix(&record[0][2..], 16).expect("Error parsing access_tag");
            let short_lease =
                u64::from_str_radix(&record[1][2..], 16).expect("Error parsing short_lease");
            let long_lease =
                u64::from_str_radix(&record[2][2..], 16).expect("Error parsing long_lease");
            let short_prob = record[3].parse::<f64>().expect("Error parsing short_prob");

            // println!("access_tag: {:x}, short_lease: {:x}, long_lease: {:x}, short_prob: {}", access_tag, short_lease, long_lease, short_prob);

            result.insert(access_tag, (short_lease, long_lease, short_prob));
        }

        LeaseTable { table: result }
    }

    fn read_lease_look_up_table_from_txt(file_path: &str) -> LeaseTable {
        let file = File::open(file_path).unwrap();
        let reader = BufReader::new(file);
        let mut result: HashMap<u64, (u64, u64, f64)> = HashMap::new();

        //skip the first two line
        let mut lines = reader.lines().skip(2);

        // Now, start reading the actual data
        while let Some(Ok(line)) = lines.next() {
            let parts: Vec<&str> = line.split(',').collect();

            // Parse the data
            let access_tag =
                u64::from_str_radix(parts[1].trim_start(), 16).expect("Error parsing access_tag");
            let short_lease =
                u64::from_str_radix(parts[2].trim_start(), 16).expect("Error parsing short_lease");
            let long_lease =
                u64::from_str_radix(parts[3].trim_start(), 16).expect("Error parsing long_lease");
            let short_prob = parts[4]
                .trim()
                .parse::<f64>()
                .expect("Error parsing short_prob");

            // println!("access_tag: {:x}, short_lease: {:x}, long_lease: {:x}, short_prob: {}", access_tag, short_lease, long_lease, short_prob);

            result.insert(access_tag, (short_lease, long_lease, short_prob));
        }

        LeaseTable { table: result }
    }

    fn new(filename: &str) -> LeaseTable {
        LeaseTable::read_lease_look_up_table_from_txt(filename)
    }

    fn query(&self, access_tag: &u64) -> Option<(u64, u64, f64)> {
        self.table.get(&access_tag).map(|x| *x)
    }
}

struct TraceItem {
    access_tag: u64,
    reference: u64,
    reuse_interval: u64,
}

impl TraceItem {
    fn new(access_tag: u64, reference: u64, reuse_interval:u64) -> TraceItem {
        TraceItem {
            access_tag,
            reference,
            reuse_interval
        }
    }
}

struct Trace {
    reader: csv::Reader<BufReader<File>>,
    current_record: Option<csv::Result<StringRecord>>,
}

impl Trace {
    fn new(file_path: &str) -> io::Result<Self> {
        let file = File::open(file_path)?;
        let mut reader = ReaderBuilder::new().from_reader(BufReader::new(file));
        let current_record = reader.records().next();
        Ok(Trace {
            reader,
            current_record,
        })
    }
}

impl Iterator for Trace {
    type Item = TraceItem;

    fn next(&mut self) -> Option<Self::Item> {
        let record = match &self.current_record {
            Some(Ok(record)) => record,
            Some(Err(_)) | None => return None, // error reading or no more records in CSV file
        };

        // Parsing only the necessary fields
        let access_tag =
            u64::from_str_radix(&record[2][2..], 16).expect("Error parsing access_tag");
        let reference = u64::from_str_radix(&record[0][2..], 16).expect("Error parsing reference");
        let reuse_interval = u64::from_str_radix(&record[1][2..], 16).expect("Error parsing reuse_interval");
        let item = TraceItem::new(access_tag, reference, reuse_interval);

        // move to the next record in CSV file
        self.current_record = self.reader.records().next();

        Some(item)
    }
}

fn init_cache_block(
    //change the name here
    input: &TraceItem,
    offset: u64,
    set: u64,
    table: &LeaseTable,
) -> Result<CacheBlock, CacheBlock> {
    let mut result = CacheBlock::new();
    result.address = input.access_tag;
    result.block_offset = input.access_tag & ((1 << offset) - 1); // ((1 << offset) - 1) = 11
                                                                  // println!("block_offset: {:b}, input.access_tag: {:b}, thing: {:b}", result.block_offset, input.access_tag, ((1 << offset) - 1));
    result.set_index = (input.access_tag >> offset) & ((1 << set) - 1);
    // println!("set_index: {:b}, input.access_tag: {:b}, thing: {:b}", result.set_index, input.access_tag, ((1 << set) - 1));
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

fn run_trace(mut cache: Cache, mut trace: Trace, table: &LeaseTable, offset: u64, set: u64) {
    while let Some(trace_item) = trace.next() {
        let result = init_cache_block(&trace_item, offset, set, table);
        match result {
            Ok(block) => {
                cache.update(block);
                // cache.print("./test.txt").expect("TODO: panic message");
            }
            Err(_) => {
                println!("Error in packing cache block");
            }
        }
    }
    //print miss ratio
    println!("Miss ratio: {}", cache.calculate_miss_ratio());
    println!("Force Eviction: {} / {} ({})", cache.forced_eviction_counter, cache.step, cache.forced_eviction_counter as f64 / cache.step as f64);
}

fn run_trace_virtual(
    mut cache: VirtualCache,
    mut trace: Trace,
    table: &LeaseTable,
    offset: u64,
    set: u64,
) {
    while let Some(trace_item) = trace.next() {
        let result = init_cache_block(&trace_item, offset, set, table);
        match result {
            Ok(block) => {
                cache.update(block);
                // cache.print("./testVirtual.txt").expect("TODO: panic message");
            }
            Err(_) => {
                println!("Error in packing cache block");
            }
        }
    }
    //print miss ratio
    println!("Miss ratio: {}", cache.calculate_miss_ratio());
}

fn run_trace_virtual_predict(
    mut trace: Trace,
    table: &LeaseTable,
) {
    let mut hit: u64 = 0;
    let mut miss: u64 = 0;
    let mut total: u64 = 0;
    while let Some(trace_item) = trace.next() {
        let lease = table
            .query(&trace_item.reference)
            .expect("Error in query lease for the access");
        //randomly assign remaining_lease according to probability at lease.3
        let mut random = rand::thread_rng();
        let mut lease_assigned = 0;
        if random.gen::<f64>() < lease.2 {
            lease_assigned = lease.0;
        } else {
            lease_assigned = lease.1;
        }
        //Any RI which is greater than the lease is a miss, any one which is less than or equal to the lease is a hit
        if &trace_item.reuse_interval <= &lease_assigned {
            hit += 1;
        } else {
            miss += 1;
        }
        total += 1;

    }

    if hit + miss != total {
        println!("Error in hit/miss calculation");
        panic!("Error in hit/miss calculation")
    }
    println!("Miss ratio: {}", miss as f64 / total as f64);
}

fn main() {
    let m = Command::new("CLAM Simulator")
        .author("Benjamin Reber, Woody Wu, Boyang Wang")
        .version("1.1")
        .arg(
            Arg::new("trace")
                .short('t')
                .value_name("The path of trace file")
                .default_value("../testInput/trace.txt"),
        )
        .arg(
            Arg::new("lease_table")
                .short('l')
                .default_value("../testInput/testTable.txt")
                .value_name("The path of lease table file"),
        )
        .arg(
            Arg::new("Mode")
                .short('m')
                .value_name("The mode of the simulator, 0 for physical, 1 for virtual, 2 for virtual with prediction")
        )
        .arg(
            Arg::new("associativity")
                .short('a')
                .value_name("The associativity of the cache")
                .default_value("128"),
        )
        .arg(
            Arg::new("offset")
                .short('o')
                .value_name("The length of the block offset")
                .default_value("2"),
        )
        .arg(
            Arg::new("set")
                .short('s')
                .value_name("The length of set index")
                .default_value("7"),
        )
        .arg(
            Arg::new("cache_size")
                .short('c')
                .value_name("The Cache Size")
                .default_value("128"),
        );

    let matches = m.get_matches();

    let trace_path = matches
        .get_one::<String>("trace")
        .expect("Trace File Not Found");
    let lease_table_path = matches
        .get_one::<String>("lease_table")
        .expect("lease_table File Not Found");

    let test_table = LeaseTable::new(lease_table_path);
    // println!("lease table: {:?}", test_table);

    let test_trace = Trace::new(trace_path);
    let associativity = matches
        .get_one::<String>("associativity")
        .expect("Error in getting associativity")
        .parse::<u64>()
        .expect("Error in parsing associativity");
    let cache_size = matches
        .get_one::<String>("cache_size")
        .expect("Error in getting cache size")
        .parse::<u64>()
        .expect("Error in parsing cache size");
    let offset = matches
        .get_one::<String>("offset")
        .expect("Error in getting offset")
        .parse::<u64>()
        .expect("Error in parsing offset");
    let set = matches
        .get_one::<String>("set")
        .expect("Error in getting set")
        .parse::<u64>()
        .expect("Error in parsing set");
    let is_virtual = matches
        .get_one::<String>("Mode")
        .unwrap_or(&"0".to_string())
        .parse::<u64>()
        .expect("Error in parsing virtual");

    print!("Current Parameters:");
    print!("Trace Path: {}  ", trace_path);
    print!("Lease Table Path: {}  ", lease_table_path);
    print!("Associativity: {}  ", associativity);
    print!("Cache Size: {}  ", cache_size);
    print!("Offset: {}  ", offset);
    print!("Set: {}  ", set);
    println!("Running Mode: {}  ", is_virtual);

    let start = Instant::now(); // Start timing

    if is_virtual == 0 {
        let test_cache = VirtualCache::new(associativity);
        run_trace_virtual(test_cache, test_trace.unwrap(), &test_table, offset, set);
    } else if is_virtual == 1 {
        let test_cache = Cache::new(cache_size, associativity);
        run_trace(test_cache, test_trace.unwrap(), &test_table, offset, set);
    }else if is_virtual == 2{
        run_trace_virtual_predict(test_trace.unwrap(), &test_table);
    }

    let duration = start.elapsed(); // End timing

    println!("Time elapsed is: {:?}", duration);
}
