use bitflags::bitflags;

// Returns `address` aligned to `alignment`.
pub fn align_address(address: usize, alignment: usize) -> usize
{
    (address + (alignment - 1)) & !(alignment - 1)
}

bitflags!
{ // Section attribute flags.
    #[derive(Debug, Clone, PartialEq)]
    pub struct SectionFlags: u8
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
    pub address: usize,
    pub length: usize,
    pub attributes: SectionFlags
}