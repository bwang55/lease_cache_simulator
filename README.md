# CLAM simulator🔖

CLAMsimulator is a lease cache simulator written in Rust. It allows you to simulate the behavior of a cache and analyze cache hits, misses, and forced evictions based on the given input trace and cache configuration.

## Features

- **Cache Behavior Simulation**: Simulate the behavior of a cache with customizable configurations.
- **Cache Analysis**: Analyze cache hits, misses, and forced evictions based on your input trace.
- **Customizable Cache Block Size & Replacement Policies**: Customize your cache block size and replacement policies to suit your specific needs.
- **Access Trace Generation**: Generate Access Trace using [DACE](https://github.com/dcompiler/dace.git).
- **Output File**: Conveniently output simulation results to a file for further in-depth analysis.

## Prerequisites

- Rust (https://www.rust-lang.org/tools/install)
- Dace (https://github.com/dcompiler/dace)

## Usage

### Command Line Options

The simulator accepts several command-line options to configure the cache simulation:

- `-t`, `--trace`: The path to the trace file (default: `../testInput/trace.txt`)
- `-l`, `--lease_table`: The path to the lease table file (default: `../testInput/testTable.txt`)
- `-m`, `--mode`: The mode of the simulator (0 for physical, 1 for virtual, 2 for virtual with prediction, 3 for LRU) (default: 0)
- `-a`, `--associativity`: The associativity of the cache (default: 128)
- `-o`, `--offset`: The length of the block offset (default: 2)
- `-s`, `--set`: The length of the set index (default: 7)
- `-c`, `--cache_size`: The cache size (default: 128)


## Example Command

To run the simulator with a trace file and lease table, simulating a physical cache with the default parameters:

```sh
cargo run --release -- -t ../testInput/trace.txt -l ../testInput/testTable.txt -m 0
```

To simulate an LRU cache:

```sh
cargo run --release -- -t ../testInput/trace.txt -l ../testInput/testTable.txt -m 3
```


## project structure

```sh
.
├── Cargo.lock
├── Cargo.toml
├── README.md
├── lease_cache_sim
│   ├── Cargo.toml
│   ├── src
│   │   ├── cache.rs
│   │   ├── lease_table.rs
│   │   ├── lru_sim.rs
│   │   ├── main.rs
│   │   ├── note
│   │   ├── run.sh
│   │   └── virtual_cache.rs
│   └── testInput
│       ├── 3mm_output.txt
│       ├── 3mm_output_shel_leases
│       ├── gemm_output.txt
│       ├── gemm_output_shel_leases
│       ├── gramschmidt_output.txt
│       ├── gramschmidt_output_shel_leases
│       ├── lu_output.txt
│       ├── lu_output_shel_leases
│       ├── matmul_output.txt
│       ├── matmul_output_shel_leases
│       ├── mvt_output.txt
│       ├── mvt_output_shel_leases
│       ├── syrk_output.txt
│       ├── syrk_output_shel_leases
│       ├── trisolv_output.txt
│       ├── trisolv_output_shel_leases
│       ├── trmm_output.txt
│       ├── trmm_output_shel_leases
│       └── trmm_output_short.txt
└── trace_gen
    ├── Cargo.toml
    └── src
        ├── main.rs
        ├── ri_utils.rs
        └── sampling.rs

```

## Access Trace Generation

To generate access traces, you can use [DACE](https://github.com/dcompiler/dace.git). Follow the instructions in the DACE repository to generate access traces that can be used as input for the CLAM Simulator.

**Happy caching!**