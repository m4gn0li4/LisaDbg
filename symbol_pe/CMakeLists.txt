cmake_minimum_required(VERSION 3.8)
project(symbol_pe VERSION 1.0 LANGUAGES CXX)

include_directories("C:/Program Files/Microsoft Visual Studio/2022/Professional/DIA SDK/include")

add_library(symbol_pe SHARED "src/parser.cpp" "src/symbol_pe.h" "src/class/parse_static.cpp" "src/class/static_parse.h")

target_include_directories(symbol_pe PRIVATE "C:/Program Files/Microsoft Visual Studio/2022/Professional/DIA SDK/include")

set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)
target_link_libraries(symbol_pe PRIVATE 
    "C:/Program Files/Microsoft Visual Studio/2022/Professional/DIA SDK/lib/amd64/diaguids.lib" 
    dbghelp
)
