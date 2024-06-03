use std::time::Instant;

use clap::{Arg, Command};

mod cache;
mod lease_table;
mod lru_sim;
mod virtual_cache;

use cache::Cache;
use lease_table::{run_trace, run_trace_virtual, run_trace_virtual_predict, LeaseTable, Trace};
use lru_sim::{run_lru_simulation};
use virtual_cache::VirtualCache;

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
            Arg::new("mode")
                .short('m')
                .value_name("The mode of the simulator, 0 for physical, 1 for virtual, 2 for virtual with prediction, 3 for LRU")
                .default_value("0"),
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
                .default_value("3"),
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
    let test_trace = Trace::new(trace_path).unwrap();

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
    let num_sets = 1 << set; // Calculate the number of sets based on the set index bits
    let mode = matches
        .get_one::<String>("mode")
        .unwrap_or(&"0".to_string())
        .parse::<u64>()
        .expect("Error in parsing mode");

    print!("Current Parameters:");
    print!("Trace Path: {}  ", trace_path);
    print!("Lease Table Path: {}  ", lease_table_path);
    print!("Associativity: {}  ", associativity);
    print!("Cache Size: {}  ", cache_size);
    print!("Offset: {}  ", offset);
    print!("Set: {}  ", set);
    print!("Number of Sets: {}  ", num_sets); // Print the number of sets
    println!("Running Mode: {}  ", mode);

    let start = Instant::now(); // Start timing

    match mode {
        0 => {
            let test_cache = Cache::new(cache_size, associativity);
            run_trace(test_cache, test_trace, &test_table, offset, set);
        },
        1 => {
            let test_cache = VirtualCache::new(associativity);
            run_trace_virtual(test_cache, test_trace, &test_table, offset, set);
        },
        2 => {
            run_trace_virtual_predict(test_trace, &test_table);
        },
        3 => {
            run_lru_simulation(test_trace, cache_size as usize, num_sets as usize, associativity as usize, offset, set);
        },
        _ => {
            eprintln!("Invalid mode specified");
        }
    }

    let duration = start.elapsed(); // End timing

    println!("Time elapsed is: {:?}", duration);
}
