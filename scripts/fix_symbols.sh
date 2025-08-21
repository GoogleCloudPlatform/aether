#!/bin/bash
# Copyright 2025 Google LLC
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.


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