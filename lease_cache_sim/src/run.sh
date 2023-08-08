#!/bin/bash

# List of test items
test_items=("3mm" "gemm" "gramschmidt" "lu" "matmul" "mvt" "syrk" "trisolv" "trmm")

# Loop through each test item
for item in "${test_items[@]}"; do
    echo "Running with -v 0: cargo run --release -- -l ../testInput/${item}_output_shel_leases -t ../testInput/${item}_output.txt -v 0"
    cargo run --release -- -l "../testInput/${item}_output_shel_leases" -t "../testInput/${item}_output.txt" -v 0

    echo "Running with -v 1: cargo run --release -- -l ../testInput/${item}_output_shel_leases -t ../testInput/${item}_output.txt -v 1"
    cargo run --release -- -l "../testInput/${item}_output_shel_leases" -t "../testInput/${item}_output.txt" -v 1

    echo "--------------------------------------"
done

echo "All tests completed."
