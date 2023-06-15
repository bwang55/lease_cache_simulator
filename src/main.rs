use std::collections::HashMap;
use std::fs::File;
use std::io;

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

            let access_tag = u64::from_str_radix(&record[0][2..], 16).expect("Error parsing access_tag");
            let short_lease = u64::from_str_radix(&record[1][2..], 16).expect("Error parsing short_lease");
            let long_lease = u64::from_str_radix(&record[2][2..], 16).expect("Error parsing long_lease");
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
            let access_tag = u64::from_str_radix(&record[0][2..], 16).expect("Error parsing access_tag");
            let reference = u64::from_str_radix(&record[1][2..], 16).expect("Error parsing reference");
            result.push(TraceItem::new(access_tag, reference));
        }
        Trace { accesses: result }
    }

    fn new(filename: &str) -> Trace {
        Trace::read_from_csv(filename)
    }
}

#[derive(Debug)]
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
    step: u64,
}

impl CacheSet {
    fn new(size: u64) -> CacheSet {
        let mut blocks: Vec<CacheBlock> = Vec::new();
        for _ in 0..size {
            blocks.push(CacheBlock::new());
        }
        CacheSet { size, blocks }
    }

    fn push(&mut self, block: CacheBlock) {
        //if cache is full, evict
        if self.blocks.len() == self.size as usize {
            Self::evict();
            self.blocks.push(block);
        } else {
            self.blocks.push(block);
        }
    }

    fn evict() -> CacheBlock {
        //Todo: implement eviction policy
        return CacheBlock::new();
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

    // fn print(&self) {
    //     println!("address: {}, tag: {}, set_index: {}, block_offset: {}, remaining_lease: {}, tenancy: {}", self.address, self.tag, self.set_index, self.block_offset, self.remaining_lease, self.tenancy);
    // }
}

impl Cache {
    fn new(size: u64, associativity: u64) -> Cache {
        let mut sets: Vec<CacheSet> = Vec::new();
        for _ in 0..size / associativity {
            sets.push(CacheSet::new(associativity));
        }
        Cache {
            size,
            sets,
            step: 0,
        }
    }

    fn insert(&mut self, block: CacheBlock) {
        //read binary set_index and decide which set to insert
        let set_index = block.set_index as usize;
        self.sets[set_index].push(block);
    }

    fn update(&mut self, block: CacheBlock) {
        let set_index = block.set_index as usize;
        for item in &mut self.sets[set_index].blocks {
            if item.address == block.address {
                item.remaining_lease = block.remaining_lease;
                item.tenancy = block.tenancy;
            }
        }
    }
}

fn pack_to_cache_block(
    input: &TraceItem,
    offset: u64,
    set: u64,
    table: LeaseTable,
) -> Result<CacheBlock, CacheBlock> {
    let mut result = CacheBlock::new();
    result.address = input.access_tag;
    result.block_offset = input.access_tag & ((1 << offset) - 1);
    result.set_index = (input.access_tag >> offset) & ((1 << set) - 1);
    result.tag = input.access_tag >> (offset + set);
    let lease = table
        .query(&input.reference)
        .expect("Error in query lease for the access");
    result.remaining_lease = lease.0;
    result.tenancy = 0;
    Ok(result)
}

fn main() {
    let file_path = "./fakeTable.csv";
    let test_table = LeaseTable::new(file_path);
    print!("{:x?}", test_table.table);

    let trace_path = "./trace.csv";
    let test_trace = Trace::new(trace_path);
    test_trace.accesses.iter().for_each(|x| {
        println!("{:x?}", x.access_tag);
        println!("{:x?}", x.reference);
    });

    let x = test_table.query(&test_trace.accesses[0].reference);
    println!("{:?}", x.unwrap());

    // let test = pack_to_cache_block(&test_trace.accesses[0], 2, 1, test_table);
    // test.unwrap().print();
}
