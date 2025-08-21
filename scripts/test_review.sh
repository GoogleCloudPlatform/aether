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

# Automated test result reviewer for AetherScript
# This script runs tests and provides a summary without requiring manual approval

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
LOG_DIR="test_logs"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
LOG_FILE="$LOG_DIR/test_run_$TIMESTAMP.log"

# Create log directory if it doesn't exist
mkdir -p "$LOG_DIR"

# Function to print colored output
print_status() {
    local status=$1
    local message=$2
    case $status in
        "success")
            echo -e "${GREEN}✓${NC} $message"
            ;;
        "error")
            echo -e "${RED}✗${NC} $message"
            ;;
        "warning")
            echo -e "${YELLOW}⚠${NC} $message"
            ;;
        "info")
            echo -e "${BLUE}ℹ${NC} $message"
            ;;
    esac
}

# Function to run tests and capture output
run_tests() {
    local test_type=$1
    local test_cmd=$2
    
    print_status "info" "Running $test_type tests..."
    
    if $test_cmd > "$LOG_DIR/${test_type}_$TIMESTAMP.log" 2>&1; then
        print_status "success" "$test_type tests passed"
        return 0
    else
        print_status "error" "$test_type tests failed (see $LOG_DIR/${test_type}_$TIMESTAMP.log)"
        return 1
    fi
}

# Function to check compilation
check_compilation() {
    print_status "info" "Checking compilation..."
    
    if cargo check --all-targets > "$LOG_DIR/compilation_check_$TIMESTAMP.log" 2>&1; then
        print_status "success" "Compilation check passed"
        return 0
    else
        local error_count=$(grep -c "error\[E[0-9]\+\]" "$LOG_DIR/compilation_check_$TIMESTAMP.log" || true)
        print_status "error" "Compilation failed with $error_count errors"
        return 1
    fi
}

# Function to run specific test suites
run_test_suite() {
    local suite=$1
    case $suite in
        "unit")
            run_tests "unit" "cargo test --lib"
            ;;
        "integration")
            run_tests "integration" "cargo test --test '*' -- --test-threads=1"
            ;;
        "doc")
            run_tests "doc" "cargo test --doc"
            ;;
        "all")
            run_tests "all" "cargo test"
            ;;
    esac
}

# Function to generate summary report
generate_summary() {
    local total_tests=0
    local passed_tests=0
    local failed_tests=0
    
    echo -e "\n${BLUE}Test Summary Report - $(date)${NC}"
    echo "================================================"
    
    # Parse test results from logs
    for log in "$LOG_DIR"/*_$TIMESTAMP.log; do
        if [[ -f "$log" ]]; then
            local test_name=$(basename "$log" | sed "s/_$TIMESTAMP.log//")
            
            # Check for test results
            if grep -q "test result:" "$log" 2>/dev/null; then
                local result=$(grep "test result:" "$log" | tail -1)
                echo "$test_name: $result"
                
                # Extract numbers
                local passed=$(echo "$result" | grep -oP '\d+(?= passed)' || echo 0)
                local failed=$(echo "$result" | grep -oP '\d+(?= failed)' || echo 0)
                
                total_tests=$((total_tests + passed + failed))
                passed_tests=$((passed_tests + passed))
                failed_tests=$((failed_tests + failed))
            fi
        fi
    done
    
    echo "================================================"
    echo "Total: $total_tests tests"
    print_status "success" "Passed: $passed_tests"
    if [[ $failed_tests -gt 0 ]]; then
        print_status "error" "Failed: $failed_tests"
    fi
    
    # Check for compilation errors
    if [[ -f "$LOG_DIR/compilation_check_$TIMESTAMP.log" ]]; then
        local compilation_errors=$(grep -c "error\[E[0-9]\+\]" "$LOG_DIR/compilation_check_$TIMESTAMP.log" || echo 0)
        if [[ $compilation_errors -gt 0 ]]; then
            print_status "error" "Compilation errors: $compilation_errors"
            echo -e "\nTop compilation errors:"
            grep "error\[E[0-9]\+\]" "$LOG_DIR/compilation_check_$TIMESTAMP.log" | head -5
        fi
    fi
    
    echo -e "\nFull logs available in: $LOG_DIR/"
}

# Function to watch for file changes and run tests
watch_mode() {
    print_status "info" "Entering watch mode. Press Ctrl+C to exit."
    
    while true; do
        # Use fswatch if available, otherwise fall back to polling
        if command -v fswatch &> /dev/null; then
            fswatch -1 -r src/ tests/ | while read; do
                clear
                main
            done
        else
            # Simple polling fallback
            local checksum=$(find src/ tests/ -type f -name "*.rs" -exec md5sum {} \; | md5sum)
            sleep 2
            local new_checksum=$(find src/ tests/ -type f -name "*.rs" -exec md5sum {} \; | md5sum)
            
            if [[ "$checksum" != "$new_checksum" ]]; then
                clear
                main
            fi
        fi
    done
}

# Main function
main() {
    echo -e "${BLUE}AetherScript Test Runner${NC}"
    echo "========================"
    
    # Check compilation first
    check_compilation
    local compilation_status=$?
    
    # Run tests based on arguments or default to all
    if [[ $# -eq 0 ]]; then
        run_test_suite "all"
    else
        for arg in "$@"; do
            case $arg in
                --unit)
                    run_test_suite "unit"
                    ;;
                --integration)
                    run_test_suite "integration"
                    ;;
                --doc)
                    run_test_suite "doc"
                    ;;
                --watch)
                    watch_mode
                    ;;
                --summary-only)
                    # Just show existing logs
                    ;;
                *)
                    print_status "warning" "Unknown argument: $arg"
                    ;;
            esac
        done
    fi
    
    # Generate summary
    generate_summary
    
    # Exit with appropriate code
    if [[ $compilation_status -ne 0 ]] || [[ $failed_tests -gt 0 ]]; then
        exit 1
    else
        exit 0
    fi
}

# Handle script arguments
if [[ "$1" == "--help" || "$1" == "-h" ]]; then
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  --unit          Run only unit tests"
    echo "  --integration   Run only integration tests"
    echo "  --doc           Run only documentation tests"
    echo "  --watch         Watch for changes and re-run tests"
    echo "  --summary-only  Show summary of last test run"
    echo "  --help, -h      Show this help message"
    echo ""
    echo "Without options, runs all tests"
    exit 0
fi

# Run main function with all arguments
main "$@"