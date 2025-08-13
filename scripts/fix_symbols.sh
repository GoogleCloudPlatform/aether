#!/bin/bash

# Script to fix Symbol creation sites in semantic analyzer

# Create backup
cp src/semantic/mod.rs src/semantic/mod.rs.backup

# Fix all Symbol creation sites by adding the missing fields
sed -i '' '
/let.*symbol = Symbol {/,/};/ {
    /declaration_location:.*,$/a\
            is_moved: false,\
            borrow_state: BorrowState::None,
}' src/semantic/mod.rs

echo "Symbol creation sites fixed. Running cargo check..."
cargo check 2>&1 | grep -E "error\[E" | head -10