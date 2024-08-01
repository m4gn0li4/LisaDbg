#ifndef SYMBOL_EXTRACTOR_H
#define SYMBOL_EXTRACTOR_H

#include <vector>
#include <string>
#include <windows.h>

typedef struct Symbol {
    ULONG   Size;
    ULONG64 Value;
    ULONG64 Address;
    ULONG   Tag;
    char* Name;
    char* filename;
    DWORD   line_num;
} Symbol;



typedef struct LocalSym {
    ULONG Size;
    ULONG64 Value;
    ULONG64 Address;
    ULONG Tag;
    char* Name;
    char* filename;
    DWORD line_num;
    ULONG Reg;
} LocalSym;



class SymExtract {
public:
    SymExtract();
    ~SymExtract();
    std::vector<Symbol> getSymbols(const std::string& path);
private:
    HANDLE hproc;
};

#endif