cmake_minimum_required(VERSION 3.8)


project("symbol_pe" VERSION 1.0 LANGUAGES CXX)


add_library(symbol_pe SHARED "symbol_pe.cpp" "symbol_pe.h")
target_include_directories(symbol_pe PRIVATE "C:/Program Files/Microsoft Visual Studio/2022/Professional/DIA SDK/include")
target_include_directories(symbol_pe PRIVATE "C:/MinGW/include")
set(CMAKE_CXX_STANDARD 23)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_CXX_EXTENSIONS OFF)

target_link_libraries(symbol_pe PRIVATE dbghelp)

