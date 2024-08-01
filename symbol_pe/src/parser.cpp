#include "symbol_pe.h"
#include "class/static_parse.h"
#include <iostream>
#include <dia2.h>
#include <DbgHelp.h>





BOOL CALLBACK SymCallback(PSYMBOL_INFO pInfo, ULONG Size, PVOID Context) {
    auto* ctx = static_cast<SymCtx*>(Context);
    auto* symbolVector = ctx->symbolVector;
    HANDLE process = ctx->process;
    LocalSym sd;
    sd.Size = pInfo->Size;
    sd.Address = pInfo->Address;
    sd.Value = pInfo->Value;
    sd.Tag = pInfo->Tag;
    sd.Reg = pInfo->Register;
    sd.Name = _strdup(pInfo->Name);

    IMAGEHLP_LINE64 line = { sizeof(IMAGEHLP_LINE64) };
    DWORD disp = 0;
    if (SymGetLineFromAddr64(process, pInfo->Address, &disp, &line)) {
        sd.line_num = line.LineNumber;
        sd.filename = _strdup(line.FileName ? line.FileName : "Unknown");
    }
    else {
        sd.line_num = 0;
        sd.filename = _strdup("Unknown");
    }
    symbolVector->push_back(sd);
    return TRUE;
}






extern "C" {
    __declspec(dllexport) const char* GetTagString(DWORD tag) {
        switch (tag) {
        case SymTagNull: return "SymTagNull";
        case SymTagExe: return "SymTagExe";
        case SymTagCompiland: return "SymTagCompiland";
        case SymTagCompilandDetails: return "SymTagCompilandDetails";
        case SymTagCompilandEnv: return "SymTagCompilandEnv";
        case SymTagFunction: return "SymTagFunction";
        case SymTagBlock: return "SymTagBlock";
        case SymTagData: return "SymTagData";
        case SymTagAnnotation: return "SymTagAnnotation";
        case SymTagLabel: return "SymTagLabel";
        case SymTagPublicSymbol: return "SymTagPublicSymbol";
        case SymTagUDT: return "SymTagUDT";
        case SymTagEnum: return "SymTagEnum";
        case SymTagFunctionType: return "SymTagFunctionType";
        case SymTagPointerType: return "SymTagPointerType";
        case SymTagArrayType: return "SymTagArrayType";
        case SymTagBaseType: return "SymTagBaseType";
        case SymTagTypedef: return "SymTagTypedef";
        case SymTagBaseClass: return "SymTagBaseClass";
        case SymTagFriend: return "SymTagFriend";
        case SymTagFunctionArgType: return "SymTagFunctionArgType";
        case SymTagFuncDebugStart: return "SymTagFuncDebugStart";
        case SymTagFuncDebugEnd: return "SymTagFuncDebugEnd";
        case SymTagUsingNamespace: return "SymTagUsingNamespace";
        case SymTagVTableShape: return "SymTagVTableShape";
        case SymTagVTable: return "SymTagVTable";
        case SymTagCustom: return "SymTagCustom";
        case SymTagThunk: return "SymTagThunk";
        case SymTagCustomType: return "SymTagCustomType";
        case SymTagManagedType: return "SymTagManagedType";
        case SymTagDimension: return "SymTagDimension";
        default: return "Unknown";
        }
    }



    __declspec(dllexport) Symbol* getSymbols(size_t* len, const char* path) {
        try {
            SymExtract extractor;
            auto symbols = extractor.getSymbols(path);
            *len = symbols.size();
            Symbol* symbol_ar = new Symbol[symbols.size()];
            std::copy(symbols.begin(), symbols.end(), symbol_ar);
            return symbol_ar;
        }
        catch (const std::exception& e) {
            std::cerr << "Error: " << e.what() << std::endl;
            return nullptr;
        }
    }



    __declspec(dllexport) void freeSymbols(Symbol* symbols, size_t len) {
        if (symbols == nullptr) return;
        for (size_t i = 0; i < len; ++i) {
            std::free(symbols[i].Name);
            std::free(symbols[i].filename);
        }
        delete[] symbols;
    }



    __declspec(dllexport) LocalSym* GetLocalVar(HANDLE process, DWORD64 addr_func, size_t* len) {
        std::vector<LocalSym> symbols;
        IMAGEHLP_STACK_FRAME st_frame = { 0 };
        st_frame.InstructionOffset = addr_func;
        if (SymSetContext(process, &st_frame, nullptr) == FALSE) {
            std::cerr << "Error in SymSetContext: " << GetLastError() << std::endl;
            return nullptr;
        }
        SymCtx ctx = { process, &symbols };
        if (!SymEnumSymbols(process, 0, "*", SymCallback, &ctx)) {
            std::cerr << "SymEnumSymbols error: " << GetLastError() << std::endl;
            SymCleanup(process);
            return nullptr;
        }
        SymCleanup(process);
        *len = symbols.size();
        LocalSym* symbol_ar = new LocalSym[symbols.size()];
        std::copy(symbols.begin(), symbols.end(), symbol_ar);
        return symbol_ar;
    }



    __declspec(dllexport) void freeLocalSym(LocalSym* symbols, size_t len) {
        if (symbols == nullptr) return;
        for (size_t i = 0; i < len; ++i) {
            std::free(symbols[i].Name);
            std::free(symbols[i].filename);
        }
        delete[] symbols;
    }
}