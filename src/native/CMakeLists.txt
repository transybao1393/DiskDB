cmake_minimum_required(VERSION 3.10)
project(diskdb_native C)

set(CMAKE_C_STANDARD 11)
set(CMAKE_C_STANDARD_REQUIRED ON)

# Compiler flags
if(CMAKE_C_COMPILER_ID MATCHES "GNU|Clang")
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -Wall -Wextra -Wpedantic -O3 -march=native")
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fPIC")  # Position independent code for shared library
    
    # Enable threading support
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -pthread")
    
    # Security flags
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -D_FORTIFY_SOURCE=2")
    set(CMAKE_C_FLAGS "${CMAKE_C_FLAGS} -fstack-protector-strong")
endif()

# Source files
set(SOURCES
    src/arena.c
    src/parser.c
)

# Create static library
add_library(diskdb_parser STATIC ${SOURCES})

# Include directories
target_include_directories(diskdb_parser PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

# Create shared library for testing
add_library(diskdb_parser_shared SHARED ${SOURCES})
target_include_directories(diskdb_parser_shared PUBLIC
    ${CMAKE_CURRENT_SOURCE_DIR}/include
)

# Installation
install(TARGETS diskdb_parser
    ARCHIVE DESTINATION lib
    LIBRARY DESTINATION lib
)

install(DIRECTORY include/
    DESTINATION include/diskdb
    FILES_MATCHING PATTERN "*.h"
)