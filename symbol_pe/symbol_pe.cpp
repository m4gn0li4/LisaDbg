#include <iostream>
#include <fstream>
#include <vector>
#include <windows.h>
#include <dbghelp.h>
#include <filesystem>
#include <dia2.h>


typedef struct Symbol {
    ULONG   Size;
    ULONG64 Value;
    ULONG64 Address;
    ULONG   Tag;
    char* Name;
} Symbol;



std::vector<Symbol> vec_dbg;


extern "C" {
    __declspec(dllexport) const char* GetTagString(DWORD tag) {
        switch (tag) {
        case SymTagNull:
            return "SymTagNull";
        case SymTagExe:
            return "SymTagExe";
        case SymTagCompiland:
            return "SymTagCompiland";
        case SymTagCompilandDetails:
            return "SymTagCompilandDetails";
        case SymTagCompilandEnv:
            return "SymTagCompilandEnv";
        case SymTagFunction:
            return "Function";
        case SymTagBlock:
            return "SymTagBlock";
        case SymTagData:
            return "SymTagData";
        case SymTagAnnotation:
            return "SymTagAnnotation";
        case SymTagLabel:
            return "SymTagLabel";
        case SymTagPublicSymbol:
            return "Function";
        case SymTagUDT:
            return "SymTagUDT";
        case SymTagEnum:
            return "SymTagEnum";
        case SymTagFunctionType:
            return "SymTagFunctionType";
        case SymTagPointerType:
            return "SymTagPointerType";
        case SymTagArrayType:
            return "SymTagArrayType";
        case SymTagBaseType:
            return "SymTagBaseType";
        case SymTagTypedef:
            return "SymTagTypedef";
        case SymTagBaseClass:
            return "SymTagBaseClass";
        case SymTagFriend:
            return "SymTagFriend";
        case SymTagFunctionArgType:
            return "SymTagFunctionArgType";
        case SymTagFuncDebugStart:
            return "Function";
        case SymTagFuncDebugEnd:
            return "SymTagFuncDebugEnd";
        case SymTagUsingNamespace:
            return "SymTagUsingNamespace";
        case SymTagVTableShape:
            return "SymTagVTableShape";
        case SymTagVTable:
            return "SymTagVTable";
        case SymTagCustom:
            return "SymTagCustom";
        case SymTagThunk:
            return "SymTagThunk";
        case SymTagCustomType:
            return "SymTagCustomType";
        case SymTagManagedType:
            return "SymTagManagedType";
        case SymTagDimension:
            return "SymTagDimension";
        default:
            return "Unknown";
        }
    }


    __declspec(dllexport) Symbol* symbole(size_t* len, char* path) {
        HANDLE hProc = GetCurrentProcess();
        if (!SymInitialize(hProc, nullptr, TRUE)) 
            return nullptr;
        
        HANDLE hFile = CreateFileA(path, GENERIC_READ, FILE_SHARE_READ, nullptr, OPEN_EXISTING, FILE_ATTRIBUTE_NORMAL, nullptr);
        if (hFile == INVALID_HANDLE_VALUE) {
            return nullptr;
        }
        DWORD64 baseAddr = SymLoadModuleEx(hProc, hFile, nullptr, nullptr, 0, 0, nullptr, 0);

        if (baseAddr == 0) {
            SymCleanup(hProc);
            return nullptr;
        }

        IMAGEHLP_MODULE64 modInfo = { 0 };
        modInfo.SizeOfStruct = sizeof(IMAGEHLP_MODULE64);

        if (SymGetModuleInfo64(hProc, baseAddr, &modInfo)) {
            SymEnumSymbols(hProc, baseAddr, nullptr, [](PSYMBOL_INFO pInfo, ULONG Size, PVOID Context) -> BOOL {
                if (pInfo != nullptr) {
                    Symbol sd;
                    sd.Address = pInfo->Address;
                    sd.Name = strdup(pInfo->Name);
                    sd.Tag = pInfo->Tag;
                    sd.Value = pInfo->Value;
                    vec_dbg.push_back(sd);
                }
                return TRUE;
                }, nullptr);
        }

        SymUnloadModule64(hProc, baseAddr);
        SymCleanup(hProc);
        *len = vec_dbg.size();
        return vec_dbg.data();
    }

    __declspec(dllexport) void free_symbols() {
        for (auto& sym : vec_dbg) {
            free(sym.Name);
        }
        vec_dbg.clear();
    }
}
