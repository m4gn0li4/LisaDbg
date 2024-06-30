use std::borrow::Cow;
use std::io;
use anyhow::Error;
use gimli::{AttributeValue, Dwarf, DwarfSections, EndianSlice, Reader, RunTimeEndian, SectionId};
use crate::pefile::Section;
use crate::symbol::{IMAGE_BASE, SymbolFile, SYMBOLS_V, SymbolType};

pub fn target_dwarf_info(sections: &[Section]) -> Result<(), Error> {
    let load_section = |id: SectionId| -> Result<Cow<[u8]>, io::Error> {
        sections.iter().find(|section| section.name == id.name())
            .map_or(Ok(Cow::Borrowed(&[])), |section| Ok(Cow::Borrowed(&section.content)))
    };
    let dwarf_cow = DwarfSections::load(load_section)?;
    let b_section: &dyn for<'a> Fn(&'a Cow<[u8]>) -> EndianSlice<'a, RunTimeEndian> = &|section| EndianSlice::new(section, RunTimeEndian::Little);
    let dwarf = DwarfSections::borrow(&dwarf_cow, b_section);
    let mut unit_iter = dwarf.units();
    while let Ok(Some(header)) = unit_iter.next() {
        let unit = dwarf.unit(header)?;
        let mut entries = unit.entries();
        while let Some((_, entry)) = entries.next_dfs()? {
            let mut symbol_info = SymbolFile::default();
            let mut attrs = entry.attrs();
            while let Some(attr) = attrs.next()? {
                process_attribute(&attr, &dwarf, &unit, &mut symbol_info)?;
            }
            unsafe { SYMBOLS_V.symbol_file.push(symbol_info) }
        }
    }

    unsafe {
        if !SYMBOLS_V.symbol_file.is_empty() {
            SYMBOLS_V.symbol_type = SymbolType::DWARF;
        }
    }

    Ok(())
}

fn process_attribute<'a>(attr: &gimli::Attribute<EndianSlice<'a, RunTimeEndian>>, dwarf: &Dwarf<EndianSlice<RunTimeEndian>>, unit: &gimli::Unit<EndianSlice<'a, RunTimeEndian>>, symbol_info: &mut SymbolFile) -> Result<(), Error> {
    match attr.value() {
        AttributeValue::Exprloc(ref data) => {
            if let AttributeValue::Exprloc(_) = attr.raw_value() {
                symbol_info.size = data.0.len();
                symbol_info.value_str = format!("{:x?}", data.0);
            }
            dump_exprloc(unit.encoding(), data, symbol_info)?;
        }
        AttributeValue::Addr(addr) => {
            symbol_info.addr = if addr > unsafe { IMAGE_BASE } {
                addr - unsafe { IMAGE_BASE }
            } else {
                addr
            };
        }
        AttributeValue::String(str_bytes) => symbol_info.name = String::from_utf8_lossy(&str_bytes).to_string(),
        AttributeValue::DebugStrRef(offset) => {
            let name = dwarf.debug_str.get_str(offset)?;
            symbol_info.name = String::from_utf8_lossy(&name).to_string();
        }
        _ => {}
    }
    Ok(())
}



fn dump_exprloc<'a>(encoding: gimli::Encoding, data: &gimli::Expression<EndianSlice<'a, RunTimeEndian>>, symbol: &mut SymbolFile, ) -> Result<(), Error> {
    let mut pc = data.0.clone();
    while !pc.is_empty() {
        let pc_clone = pc.clone();
        if let Ok(op) = gimli::Operation::parse(&mut pc, encoding) {
            dump_op(encoding, pc_clone, op, symbol)?;
        } else {
            return Ok(());
        }
    }
    Ok(())
}



fn dump_op<'a>(encoding: gimli::Encoding, mut pc: EndianSlice<'a, RunTimeEndian>, op: gimli::Operation<EndianSlice<'a, RunTimeEndian>>, symbol: &mut SymbolFile) -> Result<(), Error> {
    let wop = gimli::DwOp(pc.read_u8()?);
    match op {
        gimli::Operation::Deref { size, .. } => {
            if wop == gimli::DW_OP_deref_size || wop == gimli::DW_OP_xderef_size {
                symbol.size = size as usize;
            }
        }
        gimli::Operation::PlusConstant { value } => symbol.value_str = format!("{:#x}", value),

        gimli::Operation::SignedConstant { value } => {
            if matches!(
                wop,
                gimli::DW_OP_const1s
                    | gimli::DW_OP_const2s
                    | gimli::DW_OP_const4s
                    | gimli::DW_OP_const8s
                    | gimli::DW_OP_consts
            ) {
                symbol.value_str = value.to_string();
            }
        }
        gimli::Operation::UnsignedConstant { value } => {
            if matches!(
                wop,
                gimli::DW_OP_const1u
                    | gimli::DW_OP_const2u
                    | gimli::DW_OP_const4u
                    | gimli::DW_OP_const8u
                    | gimli::DW_OP_constu
            ) {
                symbol.value_str = value.to_string();
            }
        }
        gimli::Operation::ImplicitValue { data } => {
            let data = data.to_slice()?;
            symbol.value_str = format!("{:x?}", data);
        }
        gimli::Operation::ImplicitPointer { value, .. } => symbol.value_str = format!("{:#x}", value.0),
        gimli::Operation::EntryValue { expression } => dump_exprloc(encoding, &gimli::Expression(expression), symbol)?,
        gimli::Operation::Address { address } => symbol.addr = address,

        _ => {}
    }
    Ok(())
}
