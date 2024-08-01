#ifndef EXPORTED_FUNCTIONS_H
#define EXPORTED_FUNCTIONS_H

#include <cstddef>
#include <windows.h>
#include "class/static_parse.h"


struct SymCtx {
    HANDLE process;
    std::vector<LocalSym>* symbolVector;
};

extern "C" {
    __declspec(dllexport) const char* GetTagString(DWORD tag);
    __declspec(dllexport) Symbol* getSymbols(size_t* len, const char* path);
    __declspec(dllexport) LocalSym* GetLocalVar(HANDLE process, DWORD64 addr_func, size_t* len);
    __declspec(dllexport) void freeSymbols(Symbol* symbols, size_t len);
    __declspec(dllexport) void freeLocalSym(LocalSym* symbols, size_t len);
}

#endif
