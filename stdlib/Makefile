# AetherScript Standard Library Build System
# Compiles stdlib modules to .o + .abi for separate compilation

# Compiler location (relative to stdlib directory)
COMPILER := ../target/release/aether-compiler
COMPILER_DEBUG := ../target/debug/aether-compiler

# Use debug compiler if release not available
ifeq ($(wildcard $(COMPILER)),)
    COMPILER := $(COMPILER_DEBUG)
endif

# Compiler flags for library compilation
CFLAGS := compile -c --emit-abi

# Output directory for compiled files
BUILD_DIR := build
INSTALL_DIR := $(HOME)/.aether/stdlib

# Runtime path for compiler
export AETHER_RUNTIME_PATH := ../runtime/target/release

# Stdlib modules (in dependency order - independent modules first)
# These have no inter-dependencies, all use extern FFI to aether_runtime
MODULES := \
    io \
    math \
    string \
    stringview \
    json \
    collections

# Generated file patterns
SOURCES := $(MODULES:=.aether)
OBJECTS := $(addprefix $(BUILD_DIR)/,$(MODULES:=.o))
ABIS := $(addprefix $(BUILD_DIR)/,$(MODULES:=.abi))

.PHONY: all clean install check debug help

# Default target
all: $(BUILD_DIR) $(OBJECTS) $(ABIS)
	@echo "Standard library built successfully."
	@echo "Object files: $(BUILD_DIR)/*.o"
	@echo "ABI files: $(BUILD_DIR)/*.abi"

# Create build directory
$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

# Pattern rule for compiling .aether to .o and .abi
$(BUILD_DIR)/%.o $(BUILD_DIR)/%.abi: %.aether | $(BUILD_DIR)
	@echo "Compiling $<..."
	$(COMPILER) $(CFLAGS) -o $(BUILD_DIR)/$*.o $<
	@# Verify both files were created
	@test -f $(BUILD_DIR)/$*.o || (echo "Error: Object file not created for $<" && exit 1)
	@test -f $(BUILD_DIR)/$*.abi || (echo "Error: ABI file not created for $<" && exit 1)

# Install compiled stdlib to user's home directory
install: all
	@echo "Installing stdlib to $(INSTALL_DIR)..."
	mkdir -p $(INSTALL_DIR)
	cp $(BUILD_DIR)/*.o $(INSTALL_DIR)/
	cp $(BUILD_DIR)/*.abi $(INSTALL_DIR)/
	@echo "Stdlib installed to $(INSTALL_DIR)"

# Clean build artifacts
clean:
	rm -rf $(BUILD_DIR)
	@echo "Build directory cleaned."

# Check syntax only (no code generation)
check:
	@echo "Checking stdlib syntax..."
	@for src in $(SOURCES); do \
		if [ -f $$src ]; then \
			echo "Checking $$src..."; \
			$(COMPILER) check $$src || exit 1; \
		fi; \
	done
	@echo "All modules pass syntax check."

# Build with debug compiler
debug: COMPILER := $(COMPILER_DEBUG)
debug: all

# Show help
help:
	@echo "AetherScript Standard Library Build System"
	@echo ""
	@echo "Targets:"
	@echo "  all      - Build all stdlib modules (default)"
	@echo "  install  - Install to ~/.aether/stdlib/"
	@echo "  clean    - Remove build artifacts"
	@echo "  check    - Check syntax without compiling"
	@echo "  debug    - Build using debug compiler"
	@echo "  help     - Show this help message"
	@echo ""
	@echo "Modules: $(MODULES)"
	@echo ""
	@echo "Environment:"
	@echo "  COMPILER=$(COMPILER)"
	@echo "  BUILD_DIR=$(BUILD_DIR)"
	@echo "  INSTALL_DIR=$(INSTALL_DIR)"
