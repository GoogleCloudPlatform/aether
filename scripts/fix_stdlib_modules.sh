#!/bin/bash

# Script to fix stdlib module external functions to use create_external_function_named

echo "Fixing stdlib module external functions..."

# List of stdlib modules to fix
MODULES=(
    "io"
    "collections"
    "console"
    "http"
    "json"
    "memory"
    "network"
)

for module in "${MODULES[@]}"; do
    echo "Fixing $module module..."
    FILE="src/stdlib/${module}.rs"
    
    # First, replace create_external_function with create_external_function_named
    sed -i '' 's/create_external_function(/create_external_function_named(/g' "$FILE"
    
    # Now we need to add the AetherScript name as the first parameter
    # This is module-specific and needs careful handling
    
    case "$module" in
        "io")
            # Fix io module functions
            sed -i '' 's/create_external_function_named(\s*"aether_io_open_file"/create_external_function_named("open_file", "aether_io_open_file"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_io_close_file"/create_external_function_named("close_file", "aether_io_close_file"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_io_read_file"/create_external_function_named("read_file", "aether_io_read_file"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_io_write_file"/create_external_function_named("write_file", "aether_io_write_file"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_io_file_exists"/create_external_function_named("file_exists", "aether_io_file_exists"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_io_file_size"/create_external_function_named("file_size", "aether_io_file_size"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_io_create_directory"/create_external_function_named("create_directory", "aether_io_create_directory"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_io_remove_file"/create_external_function_named("remove_file", "aether_io_remove_file"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_io_list_directory"/create_external_function_named("list_directory", "aether_io_list_directory"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_io_print"/create_external_function_named("print", "aether_io_print"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_io_println"/create_external_function_named("println", "aether_io_println"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_io_input"/create_external_function_named("input", "aether_io_input"/g' "$FILE"
            ;;
        "collections")
            # Fix collections module functions
            sed -i '' 's/create_external_function_named(\s*"aether_array_new"/create_external_function_named("array_new", "aether_array_new"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_array_length"/create_external_function_named("array_length", "aether_array_length"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_array_push"/create_external_function_named("array_push", "aether_array_push"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_array_pop"/create_external_function_named("array_pop", "aether_array_pop"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_array_get"/create_external_function_named("array_get", "aether_array_get"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_array_set"/create_external_function_named("array_set", "aether_array_set"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_map_new"/create_external_function_named("map_new", "aether_map_new"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_map_insert"/create_external_function_named("map_insert", "aether_map_insert"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_map_get"/create_external_function_named("map_get", "aether_map_get"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_map_contains"/create_external_function_named("map_contains", "aether_map_contains"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_map_remove"/create_external_function_named("map_remove", "aether_map_remove"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_map_size"/create_external_function_named("map_size", "aether_map_size"/g' "$FILE"
            ;;
        "console")
            # Fix console module functions
            sed -i '' 's/create_external_function_named(\s*"aether_console_write"/create_external_function_named("console_write", "aether_console_write"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_console_writeln"/create_external_function_named("console_writeln", "aether_console_writeln"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_console_read"/create_external_function_named("console_read", "aether_console_read"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_console_readln"/create_external_function_named("console_readln", "aether_console_readln"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_console_clear"/create_external_function_named("console_clear", "aether_console_clear"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_console_set_color"/create_external_function_named("console_set_color", "aether_console_set_color"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_console_reset_color"/create_external_function_named("console_reset_color", "aether_console_reset_color"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_console_move_cursor"/create_external_function_named("console_move_cursor", "aether_console_move_cursor"/g' "$FILE"
            ;;
        "http")
            # Fix http module functions
            sed -i '' 's/create_external_function_named(\s*"aether_http_request"/create_external_function_named("http_request", "aether_http_request"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_http_get"/create_external_function_named("http_get", "aether_http_get"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_http_post"/create_external_function_named("http_post", "aether_http_post"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_http_put"/create_external_function_named("http_put", "aether_http_put"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_http_delete"/create_external_function_named("http_delete", "aether_http_delete"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_http_head"/create_external_function_named("http_head", "aether_http_head"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_http_server_create"/create_external_function_named("http_server_create", "aether_http_server_create"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_http_server_listen"/create_external_function_named("http_server_listen", "aether_http_server_listen"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_http_server_stop"/create_external_function_named("http_server_stop", "aether_http_server_stop"/g' "$FILE"
            ;;
        "json")
            # Fix json module functions
            sed -i '' 's/create_external_function_named(\s*"aether_json_parse"/create_external_function_named("json_parse", "aether_json_parse"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_json_stringify"/create_external_function_named("json_stringify", "aether_json_stringify"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_json_validate"/create_external_function_named("json_validate", "aether_json_validate"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_json_format"/create_external_function_named("json_format", "aether_json_format"/g' "$FILE"
            ;;
        "memory")
            # Fix memory module functions
            sed -i '' 's/create_external_function_named(\s*"aether_memory_allocate"/create_external_function_named("memory_allocate", "aether_memory_allocate"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_memory_reallocate"/create_external_function_named("memory_reallocate", "aether_memory_reallocate"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_memory_free"/create_external_function_named("memory_free", "aether_memory_free"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_memory_copy"/create_external_function_named("memory_copy", "aether_memory_copy"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_memory_set"/create_external_function_named("memory_set", "aether_memory_set"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_memory_compare"/create_external_function_named("memory_compare", "aether_memory_compare"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_memory_usage"/create_external_function_named("memory_usage", "aether_memory_usage"/g' "$FILE"
            ;;
        "network")
            # Fix network module functions
            sed -i '' 's/create_external_function_named(\s*"aether_socket_create"/create_external_function_named("socket_create", "aether_socket_create"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_socket_bind"/create_external_function_named("socket_bind", "aether_socket_bind"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_socket_listen"/create_external_function_named("socket_listen", "aether_socket_listen"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_socket_accept"/create_external_function_named("socket_accept", "aether_socket_accept"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_socket_connect"/create_external_function_named("socket_connect", "aether_socket_connect"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_socket_send"/create_external_function_named("socket_send", "aether_socket_send"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_socket_receive"/create_external_function_named("socket_receive", "aether_socket_receive"/g' "$FILE"
            sed -i '' 's/create_external_function_named(\s*"aether_socket_close"/create_external_function_named("socket_close", "aether_socket_close"/g' "$FILE"
            ;;
    esac
done

echo "Done fixing stdlib modules!"