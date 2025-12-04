// Copyright 2025 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod basic_tests;
mod error_system_tests;

mod multi_file_tests;
mod pattern_tests;
mod resource_tests;
mod test_ffi_structs;
mod test_ffi_function_pointers;
mod test_ffi_arrays;
mod test_llvm_c_api_simulation;
mod test_memory_alloc;
mod test_starling_ffi;
mod test_string_runtime;
mod test_variadic_functions;
mod test_separate_compilation;
mod verification_tests;
