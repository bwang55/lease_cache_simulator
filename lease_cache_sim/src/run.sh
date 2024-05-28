#!/bin/bash

# List of test items
#test_items=("3mm" "gemm" "gramschmidt" "lu" "matmul" "mvt" "syrk" "trisolv" "trmm")
test_items=("3mm")

# Loop through each test item
for item in "${test_items[@]}"; do
    echo "Running with -m 0: cargo run --release -- -l ../testInput/${item}_output_shel_leases -t ../testInput/${item}_output.txt -m 0"
    cargo run --release -- -l "../testInput/${item}_output_shel_leases" -t "../testInput/${item}_output.txt" -m 0

    echo "Running with -m 1: cargo run --release -- -l ../testInput/${item}_output_shel_leases -t ../testInput/${item}_output.txt -m 1"
    cargo run --release -- -l "../testInput/${item}_output_shel_leases" -t "../testInput/${item}_output.txt" -m 1

    echo "Running with -m 2: cargo run --release -- -l ../testInput/${item}_output_shel_leases -t ../testInput/${item}_output.txt -m 2"
    cargo run --release -- -l "../testInput/${item}_output_shel_leases" -t "../testInput/${item}_output.txt" -m 2

    echo "Running with -m 3: cargo run --release -- -l ../testInput/${item}_output_shel_leases -t ../testInput/${item}_output.txt -m 3"
    cargo run --release -- -l "../testInput/${item}_output_shel_leases" -t "../testInput/${item}_output.txt" -m 3

    echo "--------------------------------------"
done

echo "All tests completed."
