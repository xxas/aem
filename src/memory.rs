use std::collections::HashMap;
use bitflags::bitflags;

pub type Address = usize;
pub type Binary = Vec<u8>;

// Returns `address` aligned to `alignment`.
pub fn align_address(address: Address, alignment: usize) -> usize
{
    (address + (alignment - 1)) & !(alignment - 1)
}

bitflags!
{ // Section attribute flags.
    pub struct SectionFlags: u32
    {
        const ALLOCATE  = 0b0000_0001;
        const WRITE     = 0b0000_0010;
        const EXECUTE   = 0b0000_0100;
        const MERGE     = 0b0000_1000;
        const STRING    = 0b0001_0000;
        const GROUP     = 0b0010_0000;
        const TLS       = 0b0100_0000;
    }
}

pub struct Section
{ // Section address, length and attributes.
    pub name: String,
    pub address: Address,
    pub length: usize,
    pub attributes: SectionFlags
}

#[derive(Debug)]
pub enum SymbolTableErr
{
    Unmatched(String),
    Duplicate(String) // Duplicate symbol table entry.
}

pub struct SymbolTable
{
    pub table: HashMap<String, Address>
}

impl SymbolTable
{
    pub fn new() -> Self
    {
        Self
        {
            table: HashMap::new()
        }
    }

    pub fn insert(&mut self, label: &str, address: Address) -> Result<(), SymbolTableErr>
    {
        if self.table.contains_key(label)
        {
            Err(SymbolTableErr::Duplicate(
                format!(r#"Duplicate label: "{}""#, label)
            ))
        }
        else
        {
            self.table.insert(label.to_string(), address);
            Ok(())
        }
    }

    pub fn lookup(&self, label: &str) -> Option<Address> 
    {
        if let Some(&value) = self.table.get(label)
        {
            return Some(value);
        }

        None
    }
}

pub struct Object 
{
    pub binary: Binary,             // Compiled binary.
    pub sections: Vec<Section>,     // Where sections are located in compiled binary and attributes.
    pub symbols: SymbolTable        // Where symbols are located in compiled binary.
}

impl Object
{
    pub fn new() -> Self
    {
        Self
        {
            binary: Binary::new(),
            sections: Vec::new(),
            symbols: SymbolTable::new(),
        }
    }
}