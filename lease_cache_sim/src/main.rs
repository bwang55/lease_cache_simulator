use std::time::Instant;

use clap::Parser;

use cache::Cache;
use lease_table::{run_trace, run_trace_virtual, run_trace_virtual_predict, LeaseTable, Trace};
use lru_sim::run_lru_simulation;
use virtual_cache::VirtualCache;

mod cache;
mod lease_table;
mod lru_sim;
mod virtual_cache;

#[derive(Parser)]
#[command(
    name = "CLAM Simulator",
    version = "1.1",
    author = "Benjamin Reber, Woody Wu, Boyang Wang",
    about = "Cache Lease Assignment Model Simulator"
)]
struct Cli {
    /// The path of trace file
    #[arg(
        short,
        long,
        value_name = "TRACE_FILE",
        default_value = "testInput/3mm_output.txt"
    )]
    trace: String,

    /// The path of lease table file
    #[arg(
        short,
        long,
        value_name = "LEASE_TABLE_FILE",
        default_value = "testInput/3mm_output_shel_leases"
    )]
    lease_table: String,

    /// The mode of the simulator: 0 for physical, 1 for virtual, 2 for virtual with prediction, 3 for LRU
    #[arg(short, long, value_name = "MODE", default_value = "0")]
    mode: u64,

    /// The associativity of the cache
    #[arg(short, long, value_name = "ASSOCIATIVITY", default_value = "128")]
    associativity: u64,

    /// The length of the block offset
    #[arg(short, long, value_name = "OFFSET", default_value = "3")]
    offset: u64,

    /// The length of set index
    #[arg(short, long, value_name = "SET", default_value = "7")]
    set: u64,

    /// The cache size
    #[arg(short, long, value_name = "CACHE_SIZE", default_value = "128")]
    cache_size: u64,
}

fn main() {
    let cli = Cli::parse();

    let trace_path = &cli.trace;
    let lease_table_path = &cli.lease_table;

    let test_table = LeaseTable::new(lease_table_path);
    let test_trace = Trace::new(trace_path).expect("Error loading trace file");

    let associativity = cli.associativity;
    let cache_size = cli.cache_size;
    let offset = cli.offset;
    let set = cli.set;
    let num_sets = 1 << set; // Calculate the number of sets based on the set index bits
    let mode = cli.mode;

    print!("Current Parameters:");
    println!("Trace Path: {}", trace_path);
    println!("Lease Table Path: {}", lease_table_path);
    print!("Associativity: {}  ", associativity);
    print!("Cache Size: {}  ", cache_size);
    print!("Offset: {}  ", offset);
    print!("Set: {}  ", set);
    println!("Number of Sets: {}", num_sets); // Print the number of sets
    println!("Running Mode: {}", mode);

    let start = Instant::now(); // Start timing

    match mode {
        0 => {
            let test_cache = Cache::new(cache_size, associativity);
            run_trace(test_cache, test_trace, &test_table, offset, set);
        }
        1 => {
            let test_cache = VirtualCache::new(associativity);
            run_trace_virtual(test_cache, test_trace, &test_table, offset, set);
        }
        2 => {
            run_trace_virtual_predict(test_trace, &test_table);
        }
        3 => {
            run_lru_simulation(
                test_trace,
                cache_size as usize,
                num_sets as usize,
                associativity as usize,
                offset,
                set,
            );
        }
        _ => {
            eprintln!("Invalid mode specified");
        }
    }

    let duration = start.elapsed(); // End timing

    println!("Time elapsed is: {:?}", duration);
}
