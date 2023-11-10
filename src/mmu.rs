use bitflags::bitflags;
use super::memory::*;

bitflags!
{ // Memory protection flags.
    pub struct Protection: u32
    {
        const READ      = 0b0000_0001;
        const WRITE     = 0b0000_0010;
        const EXECUTE   = 0b0000_0100;
        const USER      = 0b0000_1000;
        const GLOBAL    = 0b0001_0000;
        const ACCESSED  = 0b0010_0000;
        const DIRTY     = 0b0100_0000;
        const NOEXECUTE = 0b1000_0000;
    }
}

// Converts section attributes to a memory protection flags.
pub fn attributes_to_protection(section_flags: SectionFlags) -> Protection 
{
    let mut protection_flags = Protection::empty();

    if section_flags.contains(SectionFlags::ALLOCATE) 
    {
        protection_flags |= Protection::READ | Protection::WRITE;
    }

    if section_flags.contains(SectionFlags::EXECUTE) 
    {
        protection_flags |= Protection::EXECUTE;
    }

    protection_flags
}

pub struct MemoryPage
{
    pub start: Address,
    pub end: Address,
    pub protection: Protection
}

impl MemoryPage
{
    fn contains(&self, addr: Address) -> bool
    {
        addr >= self.start && addr <= self.end
    }
}

pub enum MMUErr
{
    AccessViolation(String),
    MisalignedAccess(String),
    OutOfBounds(String)
}

pub struct MMU
{
    pub memory: Vec<u8>,
    pub pages: Vec<MemoryPage>
}

impl MMU
{
    pub fn new(size: usize) -> Self
    {
        Self
        {
            memory: vec![0; size],
            pages: Vec::new()
        }
    }

    pub fn protect(&mut self, start: Address, end: Address, protection: Protection) -> Result<(), MMUErr>
    {
        for page in &self.pages {
            // New page is within an already present page, or end is within the page.
            if (start >= page.start && start < page.end) || (end > page.start && end <= page.end) || (start <= page.start && end >= page.end)
            {
                return Err(MMUErr::AccessViolation(format!(r#"Memory page overlap between addresses: {} - {}"#, start, end)));
            }
        }

        self.pages.push(MemoryPage{ start, end, protection });
        Ok(())
    }

    pub fn query(&self, addr: Address) -> Option<Protection>
    {
        for page in &self.pages
        {
            if page.contains(addr)
            {
                return Some(page.protection);
            }
        }
        None
    }

    pub fn read_byte(&self, address: Address) -> Result<u8, MMUErr>
    {
        if let Some(flags) = self.query(address)
        {
            if flags.contains(Protection::EXECUTE) || flags.contains(Protection::READ)
            {
                Ok(self.memory[address])
            }
            else
            {
                Err(MMUErr::AccessViolation(format!("Memory read violation at address: {}", address)))
            }
        } else
        {
            Err(MMUErr::OutOfBounds(format!("Address out of bounds: {}", address)))
        }
    }

    pub fn write_byte(&mut self, address: Address, value: u8) -> Result<(), MMUErr>
    {
        if let Some(flags) = self.query(address)
        {
            if flags.contains(Protection::WRITE)
            {
                self.memory[address] = value;
                Ok(())
            }
            else
            {
                Err(MMUErr::AccessViolation(format!("Memory write violation at address: {}", address)))
            }
        } else
        {
            Err(MMUErr::OutOfBounds(format!("Address out of bounds: {}", address)))
        }
    }

    pub fn write<T>(&mut self, address: Address, value: T) -> Result<(), MMUErr> 
        where T: Sized + Copy 
    {
        if address % std::mem::align_of::<T>() != 0 
        {
            return Err(MMUErr::MisalignedAccess(format!("Misaligned memory access: {}", address)))
        }

        if address + std::mem::size_of::<T>() > self.memory.len() 
        {
            return Err(MMUErr::OutOfBounds(format!("Address out of bounds: {}", address)))
        }

        let bytes = &value as *const _ as *const u8;
        for i in 0..std::mem::size_of::<T>() 
        {
            self.write_byte(address + i, unsafe { *bytes.add(i) })?;
        }
        Ok(())
    }

    pub fn read<T>(&self, address: Address) -> Result<T, MMUErr> 
        where T: Sized + Default 
    {
        // Check alignment.
        if address % std::mem::align_of::<T>() != 0 
        {
            return Err(MMUErr::MisalignedAccess(format!("Misaligned memory access: {}", address)))
        }

        // Check bounds.
        if address + std::mem::size_of::<T>() > self.memory.len() 
        {
            return Err(MMUErr::OutOfBounds(format!("Address out of bounds: {}", address)))
        }

        let mut value = T::default();
        let value_bytes = unsafe {
            std::slice::from_raw_parts_mut(&mut value as *mut _ as *mut u8, std::mem::size_of::<T>())
        };

        for i in 0..std::mem::size_of::<T>() {
            value_bytes[i] = self.read_byte(address + i)?;
        }

        Ok(value)
    }
}