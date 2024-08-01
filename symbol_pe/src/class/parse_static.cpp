#include "static_parse.h"
#include <dbghelp.h>
#include <stdexcept>
#include <iostream>
#include <memory>
#include <string>



SymExtract::SymExtract() : hproc(GetCurrentProcess()) {
    if (!SymInitialize(hproc, nullptr, TRUE)) {
        throw std::runtime_error("SymInitialize error: " + std::to_string(GetLastError()));
    }
}

SymExtract::~SymExtract() {
    SymCleanup(hproc);
}

std::vector<Symbol> SymExtract::getSymbols(const std::string& path) {
    std::vector<Symbol> symbols;
    HANDLE hFile = CreateFileA(path.c_str(), GENERIC_READ, FILE_SHARE_READ, nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr);
    if (hFile == INVALID_HANDLE_VALUE) {
        throw std::runtime_error("CreateFileA error: " + std::to_string(GetLastError()));
    }

    DWORD64 baseAddr = SymLoadModuleEx(hproc, hFile, nullptr, nullptr, 0, 0, nullptr, 0);
    if (baseAddr == 0) {
        CloseHandle(hFile);
        throw std::runtime_error("SymLoadModuleEx error: " + std::to_string(GetLastError()));
    }

    IMAGEHLP_MODULE64 modInfo;
    modInfo.SizeOfStruct = sizeof(IMAGEHLP_MODULE64);
    if (!SymGetModuleInfo64(hproc, baseAddr, &modInfo)) {
        SymUnloadModule64(hproc, baseAddr);
        CloseHandle(hFile);
        throw std::runtime_error("SymGetModuleInfo64 error: " + std::to_string(GetLastError()));
    }

    if (!SymEnumSymbols(hproc, baseAddr, nullptr, [](PSYMBOL_INFO pInfo, ULONG Size, PVOID Context) -> BOOL {
        auto* symbols = static_cast<std::vector<Symbol>*>(Context);
        Symbol sd;
        sd.Size = pInfo->Size;
        sd.Address = pInfo->Address;
        sd.Name = _strdup(pInfo->Name);
        sd.Tag = pInfo->Tag;
        sd.Value = pInfo->Value;
        IMAGEHLP_LINE64 line = { sizeof(IMAGEHLP_LINE64) };
        DWORD disp = 0;
        if (SymGetLineFromAddr64(GetCurrentProcess(), pInfo->Address, &disp, &line)) {
            sd.line_num = line.LineNumber;
            sd.filename = _strdup(line.FileName ? line.FileName : "Unknown");
        }
        else {
            sd.line_num = 0;
            sd.filename = _strdup("Unknown");
        }
        symbols->push_back(sd);
        return TRUE;
        }, &symbols)) {
        SymUnloadModule64(hproc, baseAddr);
        CloseHandle(hFile);
        throw std::runtime_error("SymEnumSymbols error: " + std::to_string(GetLastError()));
    }

    SymUnloadModule64(hproc, baseAddr);
    CloseHandle(hFile);
    return symbols;
}