use std::collections::HashMap;
use std::fs::File;

use clap::{Arg, Command};
use rand::Rng;

use crate::cache::{Cache, CacheBlock};

mod cache;

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


fn pack_to_cache_block(
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

fn run_trace(mut cache: Cache, trace: Trace, table: &LeaseTable, offset: u64, set: u64) {
    // let mut test_cache = cache;
    for i in 0..trace.accesses.len() {
        let result = pack_to_cache_block(&trace.accesses[i], offset, set, table);
        match result {
            Ok(block) => {
                cache.update(block);
                cache.print("./test.txt").expect("TODO: panic message");
            }
            Err(_) => {
                println!("Error in packing cache block");
            }
        }
    }
}

fn main() {
    let m = Command::new("CLAM Simulator")
        .author("_intentionally leave for blank")
        .version("1.0")
        .arg(
            Arg::new("trace")
                .short('t')
                .value_name("The path of trace file").default_value("./trace.csv"),
        )
        .arg(
            Arg::new("lease_table")
                .short('l').default_value("./fakeTable.csv")
                .value_name("The path of lease table file"),
        );
    let matches = m.get_matches();

    let trace_path = matches
        .get_one::<String>("trace")
        .expect("Trace File Not Found");
    let _lease_table_path = matches
        .get_one::<String>("lease_table")
        .expect("lease_table File Not Found");

    // let file_path = lease_table_path.as_str();
    let file_path = "./fakeTable.csv";

    let test_table = LeaseTable::new(file_path);

    // let trace_path = "./trace.csv";

    let test_trace = Trace::new(trace_path);

    let test_cache = Cache::new(4, 2);

    // test_cache.run_trace(test_trace, &test_table, 2, 1);
    run_trace(test_cache, test_trace, &test_table, 2, 1);
}
