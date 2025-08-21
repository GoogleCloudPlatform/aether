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

# Build the runtime library

echo "Building AetherScript runtime library..."

# Create a temporary directory for object files
mkdir -p target/runtime

# Compile the runtime module to object file
rustc --crate-type staticlib \
      --crate-name aether_runtime \
      -C opt-level=3 \
      -C codegen-units=1 \
      -o target/runtime/libaether_runtime.a \
      src/runtime/mod.rs

echo "Runtime library built at target/runtime/libaether_runtime.a"