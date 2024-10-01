use crate::pefile::Section;
use crate::symbol::{SymbolFile, SymbolType, IMAGE_BASE, SYMBOLS_V};
use anyhow::Error;
use gimli::{AttributeValue, Dwarf, DwarfSections, EndianSlice, Reader, RunTimeEndian, SectionId};
use std::borrow::Cow;
use std::io;

pub fn target_dwarf_info(sections: &[Section]) -> Result<(), Error> {
    let load_section = |id: SectionId| -> Result<Cow<[u8]>, io::Error> {
        sections.iter().find(|section| section.name == id.name()).map_or(Ok(Cow::Borrowed(&[])), |section| {
                Ok(Cow::Borrowed(&section.content))
            })
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
            symbol_info.types_e = entry.tag().to_string();
            while let Some(attr) = attrs.next()? {
                process_attribute(&attr, &dwarf, &unit, &mut symbol_info)?;
            }
            if symbol_info.offset != 0 && symbol_info.name != "" {
                unsafe { SYMBOLS_V.symbol_file.push(symbol_info) }
            }
        }
    }
    unsafe {
        if !SYMBOLS_V.symbol_file.is_empty() {
            SYMBOLS_V.symbol_type = SymbolType::DWARF
        }
    }
    Ok(())
}

fn process_attribute<'a>(attr: &gimli::Attribute<EndianSlice<'a, RunTimeEndian>>, dwarf: &Dwarf<EndianSlice<RunTimeEndian>>, unit: &gimli::Unit<EndianSlice<'a, RunTimeEndian>>, symbol_info: &mut SymbolFile) -> Result<(), Error> {
    match attr.value() {
        AttributeValue::Exprloc(ref data) => {
            if let AttributeValue::Exprloc(_) = attr.raw_value() {
                symbol_info.size = data.0.len();
                symbol_info.value_str = format!("{:x?}", data.0.to_vec());
            }
            dump_exprloc(unit.encoding(), data, symbol_info)?;
        }
        AttributeValue::Addr(addr) => {
            symbol_info.offset = if addr > unsafe { IMAGE_BASE } {
                (addr - unsafe { IMAGE_BASE }) as i64
            } else {
                addr as i64
            };
        }
        AttributeValue::FileIndex(value) => dump_file_index(value, unit, dwarf, symbol_info)?,
        AttributeValue::String(str_bytes) => {
            symbol_info.name = String::from_utf8_lossy(&str_bytes).to_string()
        }
        AttributeValue::DebugStrRef(offset) => {
            let name = dwarf.debug_str.get_str(offset)?;
            symbol_info.name = String::from_utf8_lossy(&name).to_string();
        }
        _ => {}
    }
    Ok(())
}

fn dump_file_index<R: Reader>(file_index: u64, unit: &gimli::Unit<R>, dwarf: &Dwarf<R>, symbol_file: &mut SymbolFile) -> Result<(), Error> {
    if file_index == 0 && unit.header.version() <= 4 {
        return Ok(());
    }
    let header = match unit.line_program {
        Some(ref program) => program.header(),
        None => return Ok(()),
    };
    let file = match header.file(file_index) {
        Some(file) => file,
        None => return Ok(()),
    };
    if let Some(directory) = file.directory(header) {
        let directory = dwarf.attr_string(unit, directory)?;
        let directory = directory.to_string_lossy()?;
        let directory = if &directory[directory.len() - 2..directory.len() - 1] != "/" {
            format!("{directory}/")
        } else {
            directory.to_string()
        };
        symbol_file.filename = format!("{directory}{}", dwarf.attr_string(unit, file.path_name())?.to_string_lossy()?);
    } else {
        symbol_file.filename = String::from(dwarf.attr_string(unit, file.path_name())?.to_string_lossy()?);
    }
    Ok(())
}

fn dump_exprloc<'a>(encoding: gimli::Encoding, data: &gimli::Expression<EndianSlice<'a, RunTimeEndian>>, symbol: &mut SymbolFile) -> Result<(), Error> {
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
        gimli::Operation::ImplicitValue { data } => {
            let data = data.to_slice()?;
            symbol.value_str = format!("{:x?}", data.to_vec());
        }
        gimli::Operation::ImplicitPointer { value, .. } => {
            symbol.value_str = format!("{:#x}", value.0)
        }
        gimli::Operation::EntryValue { expression } => {
            dump_exprloc(encoding, &gimli::Expression(expression), symbol)?
        }
        gimli::Operation::Address { address } => unsafe {
            symbol.offset = if address >= IMAGE_BASE {
                (address - IMAGE_BASE) as i64
            } else {
                address as i64
            }
        },
        gimli::Operation::FrameOffset { offset } => symbol.offset = offset,
        _ => {}
    }
    Ok(())
}
