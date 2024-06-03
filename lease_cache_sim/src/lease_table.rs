use crate::cache::{Cache, CacheBlock};
use crate::virtual_cache::VirtualCache;
use csv::{ReaderBuilder, StringRecord};
use rand::Rng;
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

#[derive(Debug)]

pub struct LeaseTable {
    table: HashMap<u64, (u64, u64, f64)>,
}

impl LeaseTable {
    #[allow(dead_code)]
    pub fn read_lease_look_up_table_from_csv(file_path: &str) -> LeaseTable {
        let file = File::open(file_path).unwrap();
        let mut rdr = ReaderBuilder::new().from_reader(file);
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

    pub fn read_lease_look_up_table_from_txt(file_path: &str) -> LeaseTable {
        let file = File::open(file_path).unwrap();
        let reader = BufReader::new(file);
        let mut result: HashMap<u64, (u64, u64, f64)> = HashMap::new();

        let mut lines = reader.lines().skip(2);

        while let Some(Ok(line)) = lines.next() {
            let parts: Vec<&str> = line.split(',').collect();

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

            result.insert(access_tag, (short_lease, long_lease, short_prob));
        }

        LeaseTable { table: result }
    }

    pub fn new(filename: &str) -> LeaseTable {
        LeaseTable::read_lease_look_up_table_from_txt(filename)
    }

    pub fn query(&self, access_tag: &u64) -> Option<(u64, u64, f64)> {
        self.table.get(access_tag).map(|x| *x)
    }
}

pub struct TraceItem {
    pub access_tag: u64,
    pub reference: u64,
    pub reuse_interval: u64,
}

impl TraceItem {
    pub fn new(access_tag: u64, reference: u64, reuse_interval: u64) -> TraceItem {
        TraceItem {
            access_tag,
            reference,
            reuse_interval,
        }
    }
}

pub struct Trace {
    reader: csv::Reader<BufReader<File>>,
    current_record: Option<csv::Result<StringRecord>>,
}

impl Trace {
    pub fn new(file_path: &str) -> io::Result<Self> {
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
            Some(Err(_)) | None => return None,
        };

        let access_tag =
            u64::from_str_radix(&record[2][2..], 16).expect("Error parsing access_tag");
        let reference = u64::from_str_radix(&record[0][2..], 16).expect("Error parsing reference");
        let reuse_interval =
            u64::from_str_radix(&record[1][2..], 16).expect("Error parsing reuse_interval");
        let item = TraceItem::new(access_tag, reference, reuse_interval);

        self.current_record = self.reader.records().next();

        Some(item)
    }
}

pub fn init_cache_block(
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

    let mut random = rand::thread_rng();
    if random.gen::<f64>() < lease.2 {
        result.remaining_lease = lease.0;
    } else {
        result.remaining_lease = lease.1;
    }

    result.tenancy = 0;
    Ok(result)
}

pub fn run_trace(mut cache: Cache, mut trace: Trace, table: &LeaseTable, offset: u64, set: u64) {
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

    println!("Miss ratio: {}", cache.calculate_miss_ratio());
    println!(
        "Force Eviction: {} / {} ({})",
        cache.forced_eviction_counter,
        cache.step,
        cache.forced_eviction_counter as f64 / cache.step as f64
    );
}

pub fn run_trace_virtual(
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
                // cache
                //     .print("./testVirtual.txt")
                //     .expect("TODO: panic message");
            }
            Err(_) => {
                println!("Error in packing cache block");
            }
        }
    }

    println!("Miss ratio: {}", cache.calculate_miss_ratio());
}
#[allow(unused_variables)]
pub fn run_trace_virtual_predict(mut trace: Trace, table: &LeaseTable) {
    let mut hit: u64 = 0;
    let mut miss: u64 = 0;
    let mut total: u64 = 0;

    while let Some(trace_item) = trace.next() {
        let lease_query = table
            .query(&trace_item.reference)
            .expect("Error in query lease for the access");

        let mut random = rand::thread_rng();
        let current_lease;
        if random.gen::<f64>() < lease_query.2 {
            current_lease = lease_query.0;
        } else {
            current_lease = lease_query.1;
        }


        if &trace_item.reuse_interval < &current_lease {
            hit += 1;
            // println!("HIT Current Lease: {}, Reuse Interval: {}", current_lease, trace_item.reuse_interval);
        } else {
            miss += 1;
            // println!("MISS Current Lease: {}, Reuse Interval: {}", current_lease, trace_item.reuse_interval);
        }

        total += 1;
    }

    println!("Miss ratio: {}", miss as f64 / total as f64);
}
