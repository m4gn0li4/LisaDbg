#pragma once

#include <windows.h>
#include <dbghelp.h>


extern "C" {
    __declspec(dllexport) PSYMBOL_INFO* symbole(size_t* len, const char* path);
    __declspec(dllexport) void free_symbols(PSYMBOL_INFO* symbols);
}
