use std::collections::HashMap;
use lazy_static::lazy_static;

#[derive(Debug, Clone, PartialEq)]
pub enum Format
{
    RType,  // Instructions using 3 register inputs (add, xor, mul).
    IType,  // Instructions w/ Immediate loads (addi, lw, jalr, slli).
    SType,  // Store instructions (sw, sb).
    SBType, // Branch instructions (beq, bge).
    UType,  // Instructions w/ upper immediates (lui, auipc).
    UJType, // Jump instructions.
    R4Type  // Fused multiply-add instructions require three sources and one destination register.
}

#[derive(Debug, Clone, PartialEq)]
pub enum FloatWidth
{
    Single = 0b010,
    Double = 0b011,
    Quad   = 0b100
}

#[derive(Debug, Clone, PartialEq)]
pub enum FloatFormat
{
    Half    = 0b10,
    Single  = 0b00,
    Double  = 0b01,
    Quad    = 0b11
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum Opcode
{
    Load        = 0b0000011,
    LoadFp      = 0b0000111,
    MiscMem     = 0b0001111,
    OpImm       = 0b0010011,
    AuiPC       = 0b0010111,
    OpImm32     = 0b0011011,
    Store       = 0b0100011,
    StoreFp     = 0b0100111,
    Amo         = 0b0101111,
    Op          = 0b0110011,
    Op32        = 0b0111011,
    Lui         = 0b0110111,
    MAdd        = 0b1000011,
    MSub        = 0b1000111,
    NmSub       = 0b1001011,
    NmAdd       = 0b1001111,
    OpFp        = 0b1010011,
    OpImm64     = 0b1011011,
    Branch      = 0b1100011,
    Jalr        = 0b1100111,
    Jal         = 0b1101111,
    System      = 0b1110011,
    Op64        = 0b1111011
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ShiftType
{
    SLL,
    SRL,
    SRA,
    SLLW,
    SRLW,
    SRAW,
    SLLD,
    SRLD,
    SRAD
}

#[derive(Debug, Clone, PartialEq)]
pub enum ISA
{
    RV32I,    // Base Integer Instruction Set (32-bit)
    RV64I,    // Base Integer Instruction Set (64-bit)
    RV128I,   // Base Integer Instruction Set (128-bit)
    RV32E,    // Base Integer Instruction Set (32-bit, reduced registers)
    RV64E,    // Base Integer Instruction Set (64-bit, reduced registers)
    RV128E,   // Base Integer Instruction Set (128-bit, reduced registers)
    ZiFencei, // Memory Fence Instruction
    Zicsr,    // Control and Status Register (CSR) Instructions
    RV32M,    // M-extension (Integer Multiplication and Division) for 32-bit
    RV64M,    // M-extension for 64-bit
    RV128M,   // M-extension for 128-bit
    RV32A,    // A-extension (Atomic Memory Operations) for 32-bit
    RV64A,    // A-extension for 64-bit
    RV128A,   // A-extension for 128-bit
    RV32F,    // F-extension (Single-Precision Floating-Point) for 32-bit
    RV64F,    // F-extension for 64-bit
    RV128F,   // F-extension for 128-bit
    RV32D,    // D-extension (Double-Precision Floating-Point) for 32-bit
    RV64D,    // D-extension for 64-bit
    RV128D,   // D-extension for 128-bit
    RV32Q,    // Q-extension (Quadruple-Precision Floating-Point) for 32-bit
    RV64Q,    // Q-extension for 64-bit
    RV128Q    // Q-extension for 128-bit
}

#[derive(Debug, Clone, PartialEq)]
pub struct Instruction
{
    pub opcode: Opcode,
    pub format: Format,
    pub isa: ISA,
    pub funct3: Option<u8>,
    pub funct5: Option<u8>,
    pub funct7: Option<u8>,
    pub funct12: Option<u16>,
    pub float_format: Option<FloatFormat>,
    pub shift: Option<ShiftType>,
    pub rs2: Option<u8>
}

impl Instruction 
{
    fn new(opcode: Opcode, format: Format, isa: ISA) -> Self
    {
        Instruction
        {
            opcode,
            format,
            isa,
            funct3: None,
            funct5: None,
            funct7: None,
            funct12: None,
            float_format: None,
            shift: None,
            rs2: None
        }
    }

    fn with_funct3(mut self, function: u8) -> Self
    {
        self.funct3 = Some(function);
        self
    }

    fn with_funct5(mut self, function: u8) -> Self
    {
        self.funct5 = Some(function);
        self
    }

    fn with_funct7(mut self, function: u8) -> Self
    {
        self.funct7 = Some(function);
        self
    }

    fn with_funct12(mut self, function: u16) -> Self
    {
        self.funct12 = Some(function);
        self
    }

    fn with_float_format(mut self, fformat: FloatFormat) -> Self
    {
        self.float_format = Some(fformat);
        self
    }

    fn with_shift(mut self, shift: ShiftType) -> Self
    {
        self.shift = Some(shift);
        self
    }

    fn with_rs2(mut self, rs2: u8) -> Self
    {
        self.rs2 = Some(rs2);
        self
    }
}

lazy_static!
{ // RISC-V ISA Superset.
    pub static ref RV_ISA: HashMap<&'static str, Instruction> =
    {
        let mut map = HashMap::new();
        map.insert("lui",       Instruction::new(Opcode::Lui,     Format::UType, ISA::RV32I));
        map.insert("auipc",     Instruction::new(Opcode::AuiPC,   Format::UType, ISA::RV32I));
        map.insert("jal",       Instruction::new(Opcode::Jal,     Format::UJType, ISA::RV32I));
        map.insert("jalr",      Instruction::new(Opcode::Jalr,    Format::IType, ISA::RV32I).with_funct3(0b000));
        map.insert("beq",       Instruction::new(Opcode::Branch,  Format::SBType, ISA::RV32I).with_funct3(0b000));
        map.insert("bne",       Instruction::new(Opcode::Branch,  Format::SBType, ISA::RV32I).with_funct3(0b001));
        map.insert("blt",       Instruction::new(Opcode::Branch,  Format::SBType, ISA::RV32I).with_funct3(0b100));
        map.insert("bge",       Instruction::new(Opcode::Branch,  Format::SBType, ISA::RV32I).with_funct3(0b101));
        map.insert("bltu",      Instruction::new(Opcode::Branch,  Format::SBType, ISA::RV32I).with_funct3(0b110));
        map.insert("bgeu",      Instruction::new(Opcode::Branch,  Format::SBType, ISA::RV32I).with_funct3(0b111));
        map.insert("lb",        Instruction::new(Opcode::Load,    Format::IType, ISA::RV32I).with_funct3(0b000));
        map.insert("lh",        Instruction::new(Opcode::Load,    Format::IType, ISA::RV32I).with_funct3(0b001));
        map.insert("lw",        Instruction::new(Opcode::Load,    Format::IType, ISA::RV32I).with_funct3(0b010));
        map.insert("lbu",       Instruction::new(Opcode::Load,    Format::IType, ISA::RV32I).with_funct3(0b100));
        map.insert("lhu",       Instruction::new(Opcode::Load,    Format::IType, ISA::RV32I).with_funct3(0b101));
        map.insert("sb",        Instruction::new(Opcode::Store,   Format::SType, ISA::RV32I).with_funct3(0b000));
        map.insert("sh",        Instruction::new(Opcode::Store,   Format::SType, ISA::RV32I).with_funct3(0b001));
        map.insert("sw",        Instruction::new(Opcode::Store,   Format::SType, ISA::RV32I).with_funct3(0b010));
        map.insert("addi",      Instruction::new(Opcode::OpImm,   Format::IType, ISA::RV32I).with_funct3(0b000));
        map.insert("slti",      Instruction::new(Opcode::OpImm,   Format::IType, ISA::RV32I).with_funct3(0b010));
        map.insert("sltiu",     Instruction::new(Opcode::OpImm,   Format::IType, ISA::RV32I).with_funct3(0b011));
        map.insert("xori",      Instruction::new(Opcode::OpImm,   Format::IType, ISA::RV32I).with_funct3(0b100));
        map.insert("ori",       Instruction::new(Opcode::OpImm,   Format::IType, ISA::RV32I).with_funct3(0b110));
        map.insert("andi",      Instruction::new(Opcode::OpImm,   Format::IType, ISA::RV32I).with_funct3(0b111));
        map.insert("slli",      Instruction::new(Opcode::OpImm,   Format::IType, ISA::RV32I).with_funct3(0b001).with_shift(ShiftType::SLL));
        map.insert("srli",      Instruction::new(Opcode::OpImm,   Format::IType, ISA::RV32I).with_funct3(0b101).with_shift(ShiftType::SRL));
        map.insert("srai",      Instruction::new(Opcode::OpImm,   Format::IType, ISA::RV32I).with_funct3(0b101).with_shift(ShiftType::SRA));
        map.insert("add",       Instruction::new(Opcode::Op,      Format::RType, ISA::RV32I).with_funct3(0b000).with_funct7(0b0000000));
        map.insert("sub",       Instruction::new(Opcode::Op,      Format::RType, ISA::RV32I).with_funct3(0b000).with_funct7(0b0100000));
        map.insert("sll",       Instruction::new(Opcode::Op,      Format::RType, ISA::RV32I).with_funct3(0b001).with_funct7(0b0000000));
        map.insert("slt",       Instruction::new(Opcode::Op,      Format::RType, ISA::RV32I).with_funct3(0b010).with_funct7(0b0000000));
        map.insert("sltu",      Instruction::new(Opcode::Op,      Format::RType, ISA::RV32I).with_funct3(0b011).with_funct7(0b0000000));
        map.insert("xor",       Instruction::new(Opcode::Op,      Format::RType, ISA::RV32I).with_funct3(0b100).with_funct7(0b0000000));
        map.insert("srl",       Instruction::new(Opcode::Op,      Format::RType, ISA::RV32I).with_funct3(0b101).with_funct7(0b0000000));
        map.insert("sra",       Instruction::new(Opcode::Op,      Format::RType, ISA::RV32I).with_funct3(0b101).with_funct7(0b0100000));
        map.insert("or",        Instruction::new(Opcode::Op,      Format::RType, ISA::RV32I).with_funct3(0b110).with_funct7(0b0000000));
        map.insert("and",       Instruction::new(Opcode::Op,      Format::RType, ISA::RV32I).with_funct3(0b111).with_funct7(0b0000000));
        map.insert("fence",     Instruction::new(Opcode::MiscMem, Format::IType, ISA::RV32I).with_funct3(0b000));
        map.insert("ecall",     Instruction::new(Opcode::System,  Format::IType, ISA::RV32I).with_funct3(0b000).with_funct12(0b000000000000));
        map.insert("ebreak",    Instruction::new(Opcode::System,  Format::IType, ISA::RV32I).with_funct3(0b000).with_funct12(0b000000000001));

        map.insert("addiw",     Instruction::new(Opcode::OpImm32, Format::IType, ISA::RV64I).with_funct3(0b000));
        map.insert("slliw",     Instruction::new(Opcode::OpImm32, Format::IType, ISA::RV64I).with_funct3(0b001).with_shift(ShiftType::SLLW));
        map.insert("srliw",     Instruction::new(Opcode::OpImm32, Format::IType, ISA::RV64I).with_funct3(0b101).with_shift(ShiftType::SRLW));
        map.insert("sraiw",     Instruction::new(Opcode::OpImm32, Format::IType, ISA::RV64I).with_funct3(0b101).with_shift(ShiftType::SRAW));
        map.insert("addw",      Instruction::new(Opcode::Op32,    Format::RType, ISA::RV64I).with_funct3(0b000).with_funct7(0b0000000));
        map.insert("subw",      Instruction::new(Opcode::Op32,    Format::RType, ISA::RV64I).with_funct3(0b000).with_funct7(0b0100000));
        map.insert("sllw",      Instruction::new(Opcode::Op32,    Format::RType, ISA::RV64I).with_funct3(0b001).with_funct7(0b0000000));
        map.insert("srlw",      Instruction::new(Opcode::Op32,    Format::RType, ISA::RV64I).with_funct3(0b101).with_funct7(0b0000000));
        map.insert("sraw",      Instruction::new(Opcode::Op32,    Format::RType, ISA::RV64I).with_funct3(0b101).with_funct7(0b0100000));
        map.insert("ld",        Instruction::new(Opcode::Load,    Format::IType, ISA::RV64I).with_funct3(0b011));
        map.insert("lwu",       Instruction::new(Opcode::Load,    Format::IType, ISA::RV64I).with_funct3(0b110));
        map.insert("sd",        Instruction::new(Opcode::Store,   Format::SType, ISA::RV64I).with_funct3(0b011));

        map.insert("addid",     Instruction::new(Opcode::OpImm64, Format::IType, ISA::RV128I).with_funct3(0b000));
        map.insert("sllid",     Instruction::new(Opcode::OpImm64, Format::IType, ISA::RV128I).with_funct3(0b001).with_shift(ShiftType::SLLD));
        map.insert("srlid",     Instruction::new(Opcode::OpImm64, Format::IType, ISA::RV128I).with_funct3(0b101).with_shift(ShiftType::SRLD));
        map.insert("sraid",     Instruction::new(Opcode::OpImm64, Format::IType, ISA::RV128I).with_funct3(0b101).with_shift(ShiftType::SRAD));
        map.insert("addd",      Instruction::new(Opcode::Op64,    Format::RType, ISA::RV128I).with_funct3(0b000).with_funct7(0b0000000));
        map.insert("subd",      Instruction::new(Opcode::Op64,    Format::RType, ISA::RV128I).with_funct3(0b000).with_funct7(0b0100000));
        map.insert("slld",      Instruction::new(Opcode::Op64,    Format::RType, ISA::RV128I).with_funct3(0b001).with_funct7(0b0000000));
        map.insert("srld",      Instruction::new(Opcode::Op64,    Format::RType, ISA::RV128I).with_funct3(0b101).with_funct7(0b0000000));
        map.insert("srad",      Instruction::new(Opcode::Op64,    Format::RType, ISA::RV128I).with_funct3(0b101).with_funct7(0b0100000));
        map.insert("lq",        Instruction::new(Opcode::MiscMem, Format::IType, ISA::RV128I).with_funct3(0b010));
        map.insert("ldu",       Instruction::new(Opcode::Load,    Format::IType, ISA::RV128I).with_funct3(0b111));
        map.insert("sq",        Instruction::new(Opcode::Store,   Format::SType, ISA::RV128I).with_funct3(0b100));

        map.insert("fence.i",   Instruction::new(Opcode::MiscMem, Format::SType, ISA::ZiFencei).with_funct3(0b001));

        map.insert("csrrw",     Instruction::new(Opcode::System,  Format::IType, ISA::Zicsr).with_funct3(0b001));
        map.insert("csrrs",     Instruction::new(Opcode::System,  Format::IType, ISA::Zicsr).with_funct3(0b010));
        map.insert("csrrc",     Instruction::new(Opcode::System,  Format::IType, ISA::Zicsr).with_funct3(0b011));
        map.insert("csrrwi",    Instruction::new(Opcode::System,  Format::IType, ISA::Zicsr).with_funct3(0b101));
        map.insert("csrrsi",    Instruction::new(Opcode::System,  Format::IType, ISA::Zicsr).with_funct3(0b110));
        map.insert("csrrci",    Instruction::new(Opcode::System,  Format::IType, ISA::Zicsr).with_funct3(0b111));

        map.insert("mul",       Instruction::new(Opcode::Op,      Format::RType, ISA::RV32M).with_funct3(0b000).with_funct7(0b0000001));
        map.insert("mulh",      Instruction::new(Opcode::Op,      Format::RType, ISA::RV32M).with_funct3(0b001).with_funct7(0b0000001));
        map.insert("mulhsu",    Instruction::new(Opcode::Op,      Format::RType, ISA::RV32M).with_funct3(0b010).with_funct7(0b0000001));
        map.insert("mulhu",     Instruction::new(Opcode::Op,      Format::RType, ISA::RV32M).with_funct3(0b011).with_funct7(0b0000001));
        map.insert("div",       Instruction::new(Opcode::Op,      Format::RType, ISA::RV32M).with_funct3(0b100).with_funct7(0b0000001));
        map.insert("divu",      Instruction::new(Opcode::Op,      Format::RType, ISA::RV32M).with_funct3(0b101).with_funct7(0b0000001));
        map.insert("rem",       Instruction::new(Opcode::Op,      Format::RType, ISA::RV32M).with_funct3(0b110).with_funct7(0b0000001));
        map.insert("remu",      Instruction::new(Opcode::Op,      Format::RType, ISA::RV32M).with_funct3(0b111).with_funct7(0b0000001));

        map.insert("mulw",      Instruction::new(Opcode::Op32,    Format::RType, ISA::RV64M).with_funct3(0b000).with_funct7(0b0000001));
        map.insert("divw",      Instruction::new(Opcode::Op32,    Format::RType, ISA::RV64M).with_funct3(0b100).with_funct7(0b0000001));
        map.insert("divuw",     Instruction::new(Opcode::Op32,    Format::RType, ISA::RV64M).with_funct3(0b101).with_funct7(0b0000001));
        map.insert("remw",      Instruction::new(Opcode::Op32,    Format::RType, ISA::RV64M).with_funct3(0b110).with_funct7(0b0000001));
        map.insert("remuw",     Instruction::new(Opcode::Op32,    Format::RType, ISA::RV64M).with_funct3(0b111).with_funct7(0b0000001));

        map.insert("muld",      Instruction::new(Opcode::Op64,    Format::RType, ISA::RV128M).with_funct3(0b000).with_funct7(0b0000001));
        map.insert("divd",      Instruction::new(Opcode::Op64,    Format::RType, ISA::RV128M).with_funct3(0b100).with_funct7(0b0000001));
        map.insert("divud",     Instruction::new(Opcode::Op64,    Format::RType, ISA::RV128M).with_funct3(0b101).with_funct7(0b0000001));
        map.insert("remd",      Instruction::new(Opcode::Op64,    Format::RType, ISA::RV128M).with_funct3(0b110).with_funct7(0b0000001));
        map.insert("remud",     Instruction::new(Opcode::Op64,    Format::RType, ISA::RV128M).with_funct3(0b111).with_funct7(0b0000001));

        map.insert("lr.w",      Instruction::new(Opcode::Amo,     Format::RType, ISA::RV32A).with_funct3(0b010).with_funct5(0b00010));
        map.insert("sc.w",      Instruction::new(Opcode::Amo,     Format::RType, ISA::RV32A).with_funct3(0b010).with_funct5(0b00011));
        map.insert("amoswap.w", Instruction::new(Opcode::Amo,     Format::RType, ISA::RV32A).with_funct3(0b010).with_funct5(0b00001));
        map.insert("amoadd.w",  Instruction::new(Opcode::Amo,     Format::RType, ISA::RV32A).with_funct3(0b010).with_funct5(0b00000));
        map.insert("amoxor.w",  Instruction::new(Opcode::Amo,     Format::RType, ISA::RV32A).with_funct3(0b010).with_funct5(0b00100));
        map.insert("amoand.w",  Instruction::new(Opcode::Amo,     Format::RType, ISA::RV32A).with_funct3(0b010).with_funct5(0b01100));
        map.insert("amoor.w",   Instruction::new(Opcode::Amo,     Format::RType, ISA::RV32A).with_funct3(0b010).with_funct5(0b01000));
        map.insert("amomin.w",  Instruction::new(Opcode::Amo,     Format::RType, ISA::RV32A).with_funct3(0b010).with_funct5(0b10000));
        map.insert("amomax.w",  Instruction::new(Opcode::Amo,     Format::RType, ISA::RV32A).with_funct3(0b010).with_funct5(0b10100));
        map.insert("amominu.w", Instruction::new(Opcode::Amo,     Format::RType, ISA::RV32A).with_funct3(0b010).with_funct5(0b11000));
        map.insert("amomaxu.w", Instruction::new(Opcode::Amo,     Format::RType, ISA::RV32A).with_funct3(0b010).with_funct5(0b11100));

        map.insert("lr.d",      Instruction::new(Opcode::Amo,     Format::RType, ISA::RV64A).with_funct3(0b011).with_funct5(0b00010));
        map.insert("sc.d",      Instruction::new(Opcode::Amo,     Format::RType, ISA::RV64A).with_funct3(0b011).with_funct5(0b00011));
        map.insert("amoswap.d", Instruction::new(Opcode::Amo,     Format::RType, ISA::RV64A).with_funct3(0b011).with_funct5(0b00001));
        map.insert("amoadd.d",  Instruction::new(Opcode::Amo,     Format::RType, ISA::RV64A).with_funct3(0b011).with_funct5(0b00000));
        map.insert("amoxor.d",  Instruction::new(Opcode::Amo,     Format::RType, ISA::RV64A).with_funct3(0b011).with_funct5(0b00100));
        map.insert("amoand.d",  Instruction::new(Opcode::Amo,     Format::RType, ISA::RV64A).with_funct3(0b011).with_funct5(0b01100));
        map.insert("amoor.d",   Instruction::new(Opcode::Amo,     Format::RType, ISA::RV64A).with_funct3(0b011).with_funct5(0b01000));
        map.insert("amomin.d",  Instruction::new(Opcode::Amo,     Format::RType, ISA::RV64A).with_funct3(0b011).with_funct5(0b10000));
        map.insert("amomax.d",  Instruction::new(Opcode::Amo,     Format::RType, ISA::RV64A).with_funct3(0b011).with_funct5(0b10100));
        map.insert("amominu.d", Instruction::new(Opcode::Amo,     Format::RType, ISA::RV64A).with_funct3(0b011).with_funct5(0b11000));
        map.insert("amomaxu.d", Instruction::new(Opcode::Amo,     Format::RType, ISA::RV64A).with_funct3(0b011).with_funct5(0b11100));

        map.insert("lr.q",      Instruction::new(Opcode::Amo,     Format::RType, ISA::RV128A).with_funct3(0b100).with_funct5(0b00010));
        map.insert("sc.q",      Instruction::new(Opcode::Amo,     Format::RType, ISA::RV128A).with_funct3(0b100).with_funct5(0b00011));
        map.insert("amoswap.q", Instruction::new(Opcode::Amo,     Format::RType, ISA::RV128A).with_funct3(0b100).with_funct5(0b00001));
        map.insert("amoadd.q",  Instruction::new(Opcode::Amo,     Format::RType, ISA::RV128A).with_funct3(0b100).with_funct5(0b00000));
        map.insert("amoxor.q",  Instruction::new(Opcode::Amo,     Format::RType, ISA::RV128A).with_funct3(0b100).with_funct5(0b00100));
        map.insert("amoand.q",  Instruction::new(Opcode::Amo,     Format::RType, ISA::RV128A).with_funct3(0b100).with_funct5(0b01100));
        map.insert("amoor.q",   Instruction::new(Opcode::Amo,     Format::RType, ISA::RV128A).with_funct3(0b100).with_funct5(0b01000));
        map.insert("amomin.q",  Instruction::new(Opcode::Amo,     Format::RType, ISA::RV128A).with_funct3(0b100).with_funct5(0b10000));
        map.insert("amomax.q",  Instruction::new(Opcode::Amo,     Format::RType, ISA::RV128A).with_funct3(0b100).with_funct5(0b10100));
        map.insert("amominu.q", Instruction::new(Opcode::Amo,     Format::RType, ISA::RV128A).with_funct3(0b100).with_funct5(0b11000));
        map.insert("amomaxu.q", Instruction::new(Opcode::Amo,     Format::RType, ISA::RV128A).with_funct3(0b100).with_funct5(0b11100));

        map.insert("flw",       Instruction::new(Opcode::LoadFp,  Format::IType, ISA::RV32F).with_funct3(FloatWidth::Single as u8));
        map.insert("fsw",       Instruction::new(Opcode::StoreFp, Format::SType, ISA::RV32F).with_funct3(FloatWidth::Single as u8));
        map.insert("fmadd.s",   Instruction::new(Opcode::MAdd,    Format::R4Type, ISA::RV32F).with_float_format(FloatFormat::Single));
        map.insert("fmsub.s",   Instruction::new(Opcode::MSub,    Format::R4Type, ISA::RV32F).with_float_format(FloatFormat::Single));
        map.insert("fnmadd.s",  Instruction::new(Opcode::NmAdd,   Format::R4Type, ISA::RV32F).with_float_format(FloatFormat::Single));
        map.insert("fnmsub.s",  Instruction::new(Opcode::NmSub,   Format::R4Type, ISA::RV32F).with_float_format(FloatFormat::Single));
        map.insert("fadd.s",    Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32F).with_funct5(0b00000).with_float_format(FloatFormat::Single));
        map.insert("fsub.s",    Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32F).with_funct5(0b00001).with_float_format(FloatFormat::Single));
        map.insert("fmul.s",    Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32F).with_funct5(0b00010).with_float_format(FloatFormat::Single));
        map.insert("fdiv.s",    Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32F).with_funct5(0b00011).with_float_format(FloatFormat::Single));
        map.insert("fsqrt.s",   Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32F).with_funct5(0b01011).with_rs2(0b00000).with_float_format(FloatFormat::Single));
        map.insert("fsgnj.s",   Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32F).with_funct5(0b00100).with_funct3(0b000).with_float_format(FloatFormat::Single));
        map.insert("fsgnjn.s",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32F).with_funct5(0b00100).with_funct3(0b001).with_float_format(FloatFormat::Single));
        map.insert("fsgnjx.s",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32F).with_funct5(0b00100).with_funct3(0b010).with_float_format(FloatFormat::Single));
        map.insert("fmin.s",    Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32F).with_funct5(0b00101).with_funct3(0b000).with_float_format(FloatFormat::Single));
        map.insert("fmax.s",    Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32F).with_funct5(0b00101).with_funct3(0b001).with_float_format(FloatFormat::Single));
        map.insert("feq.s",     Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32F).with_funct5(0b10100).with_funct3(0b010).with_float_format(FloatFormat::Single));
        map.insert("flt.s",     Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32F).with_funct5(0b10100).with_funct3(0b001).with_float_format(FloatFormat::Single));
        map.insert("fle.s",     Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32F).with_funct5(0b10100).with_funct3(0b000).with_float_format(FloatFormat::Single));
        map.insert("fcvt.w.s",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32F).with_funct5(0b11000).with_rs2(0b00000).with_float_format(FloatFormat::Single));
        map.insert("fcvt.wu.s", Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32F).with_funct5(0b11000).with_rs2(0b00001).with_float_format(FloatFormat::Single));
        map.insert("fcvt.s.w",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32F).with_funct5(0b11010).with_rs2(0b00000).with_float_format(FloatFormat::Single));
        map.insert("fcvt.s.wu", Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32F).with_funct5(0b11010).with_rs2(0b00001).with_float_format(FloatFormat::Single));
        map.insert("fclass.s",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32F).with_funct5(0b11100).with_rs2(0b00000).with_funct3(0b001).with_float_format(FloatFormat::Single));

        map.insert("fmv.x.w",   Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32F).with_funct5(0b11100).with_rs2(0b00000).with_funct3(0b000).with_float_format(FloatFormat::Single));
        map.insert("fmv.w.x",   Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32F).with_funct5(0b11110).with_rs2(0b00000).with_funct3(0b000).with_float_format(FloatFormat::Single));
        map.insert("fcvt.l.s",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV64F).with_funct5(0b11000).with_rs2(0b00010).with_float_format(FloatFormat::Single));
        map.insert("fcvt.lu.s", Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV64F).with_funct5(0b11000).with_rs2(0b00011).with_float_format(FloatFormat::Single));
        map.insert("fcvt.s.l",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV64F).with_funct5(0b11010).with_rs2(0b00010).with_float_format(FloatFormat::Single));
        map.insert("fcvt.s.lu", Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV64F).with_funct5(0b11010).with_rs2(0b00011).with_float_format(FloatFormat::Single));

        map.insert("fcvt.t.s",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV128F).with_funct5(0b11000).with_rs2(0b00100).with_float_format(FloatFormat::Single));
        map.insert("fcvt.tu.s", Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV128F).with_funct5(0b11000).with_rs2(0b00101).with_float_format(FloatFormat::Single));
        map.insert("fcvt.s.t",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV128F).with_funct5(0b11010).with_rs2(0b00100).with_float_format(FloatFormat::Single));
        map.insert("fcvt.s.tu", Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV128F).with_funct5(0b11010).with_rs2(0b00101).with_float_format(FloatFormat::Single));

        map.insert("fld",       Instruction::new(Opcode::LoadFp,  Format::IType, ISA::RV32D).with_funct3(FloatWidth::Double as u8));
        map.insert("fsd",       Instruction::new(Opcode::StoreFp, Format::SType, ISA::RV32D).with_funct3(FloatWidth::Double as u8));

        map.insert("fmadd.d",   Instruction::new(Opcode::MAdd,    Format::R4Type, ISA::RV32D).with_float_format(FloatFormat::Double));
        map.insert("fmsub.d",   Instruction::new(Opcode::MSub,    Format::R4Type, ISA::RV32D).with_float_format(FloatFormat::Double));
        map.insert("fnmadd.d",  Instruction::new(Opcode::NmAdd,   Format::R4Type, ISA::RV32D).with_float_format(FloatFormat::Double));
        map.insert("fnmsub.d",  Instruction::new(Opcode::NmSub,   Format::R4Type, ISA::RV32D).with_float_format(FloatFormat::Double));

        map.insert("fadd.d",    Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32D).with_funct5(0b00000).with_float_format(FloatFormat::Double));
        map.insert("fsub.d",    Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32D).with_funct5(0b00001).with_float_format(FloatFormat::Double));
        map.insert("fmul.d",    Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32D).with_funct5(0b00010).with_float_format(FloatFormat::Double));
        map.insert("fdiv.d",    Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32D).with_funct5(0b00011).with_float_format(FloatFormat::Double));

        map.insert("fsqrt.d",   Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32D).with_funct5(0b01011).with_rs2(0b00000).with_float_format(FloatFormat::Double));

        map.insert("fsgnj.d",   Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32D).with_funct5(0b00100).with_funct3(0b000).with_float_format(FloatFormat::Double));
        map.insert("fsgnjn.d",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32D).with_funct5(0b00100).with_funct3(0b001).with_float_format(FloatFormat::Double));
        map.insert("fsgnjx.d",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32D).with_funct5(0b00100).with_funct3(0b010).with_float_format(FloatFormat::Double));
        map.insert("fmin.d",    Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32D).with_funct5(0b00101).with_funct3(0b000).with_float_format(FloatFormat::Double));
        map.insert("fmax.d",    Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32D).with_funct5(0b00101).with_funct3(0b001).with_float_format(FloatFormat::Double));

        map.insert("feq.d",     Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32D).with_funct5(0b10100).with_funct3(0b010).with_float_format(FloatFormat::Double));
        map.insert("flt.d",     Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32D).with_funct5(0b10100).with_funct3(0b001).with_float_format(FloatFormat::Double));
        map.insert("fle.d",     Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32D).with_funct5(0b10100).with_funct3(0b000).with_float_format(FloatFormat::Double));

        map.insert("fcvt.w.d",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32D).with_funct5(0b11000).with_rs2(0b00000).with_float_format(FloatFormat::Double));
        map.insert("fcvt.wu.d", Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32D).with_funct5(0b11000).with_rs2(0b00001).with_float_format(FloatFormat::Double));
        map.insert("fcvt.d.w",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32D).with_funct5(0b11010).with_rs2(0b00000).with_float_format(FloatFormat::Double));
        map.insert("fcvt.d.wu", Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32D).with_funct5(0b11010).with_rs2(0b00001).with_float_format(FloatFormat::Double));

        map.insert("fcvt.s.d",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32D).with_funct5(0b01000).with_rs2(0b00000).with_float_format(FloatFormat::Single));
        map.insert("fcvt.d.s",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32D).with_funct5(0b01000).with_rs2(0b00000).with_float_format(FloatFormat::Double));

        map.insert("fclass.d",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32D).with_funct5(0b11100).with_rs2(0b00000).with_funct3(0b001).with_float_format(FloatFormat::Double));

        map.insert("fmv.x.d",   Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV64D).with_funct5(0b11100).with_rs2(0b00000).with_funct3(0b000).with_float_format(FloatFormat::Double));
        map.insert("fmv.d.x",   Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV64D).with_funct5(0b11110).with_rs2(0b00000).with_funct3(0b000).with_float_format(FloatFormat::Double));

        map.insert("fcvt.l.d",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV64D).with_funct5(0b11000).with_rs2(0b00010).with_float_format(FloatFormat::Double));
        map.insert("fcvt.lu.d", Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV64D).with_funct5(0b11000).with_rs2(0b00011).with_float_format(FloatFormat::Double));
        map.insert("fcvt.d.l",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV64D).with_funct5(0b11010).with_rs2(0b00010).with_float_format(FloatFormat::Double));
        map.insert("fcvt.d.lu", Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV64D).with_funct5(0b11010).with_rs2(0b00011).with_float_format(FloatFormat::Double));

        map.insert("fcvt.t.d",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV128D).with_funct5(0b11000).with_rs2(0b00100).with_float_format(FloatFormat::Double));
        map.insert("fcvt.tu.d", Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV128D).with_funct5(0b11000).with_rs2(0b00101).with_float_format(FloatFormat::Double));
        map.insert("fcvt.d.t",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV128D).with_funct5(0b11010).with_rs2(0b00100).with_float_format(FloatFormat::Double));
        map.insert("fcvt.d.tu", Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV128D).with_funct5(0b11010).with_rs2(0b00101).with_float_format(FloatFormat::Double));

        map.insert("flq",       Instruction::new(Opcode::LoadFp,  Format::IType, ISA::RV32Q).with_funct3(FloatWidth::Quad as u8));
        map.insert("fsq",       Instruction::new(Opcode::StoreFp, Format::SType, ISA::RV32Q).with_funct3(FloatWidth::Quad as u8));

        map.insert("fmadd.q",   Instruction::new(Opcode::MAdd,    Format::R4Type, ISA::RV32Q).with_float_format(FloatFormat::Quad));
        map.insert("fmsub.q",   Instruction::new(Opcode::MSub,    Format::R4Type, ISA::RV32Q).with_float_format(FloatFormat::Quad));
        map.insert("fnmadd.q",  Instruction::new(Opcode::NmAdd,   Format::R4Type, ISA::RV32Q).with_float_format(FloatFormat::Quad));
        map.insert("fnmsub.q",  Instruction::new(Opcode::NmSub,   Format::R4Type, ISA::RV32Q).with_float_format(FloatFormat::Quad));

        map.insert("fadd.q",    Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b00000).with_float_format(FloatFormat::Quad));
        map.insert("fsub.q",    Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b00001).with_float_format(FloatFormat::Quad));
        map.insert("fmul.q",    Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b00010).with_float_format(FloatFormat::Quad));
        map.insert("fdiv.q",    Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b00011).with_float_format(FloatFormat::Quad));

        map.insert("fsqrt.q",   Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b01011).with_rs2(0b00000).with_float_format(FloatFormat::Quad));

        map.insert("fsgnj.q",   Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b00100).with_funct3(0b000).with_float_format(FloatFormat::Quad));
        map.insert("fsgnjn.q",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b00100).with_funct3(0b001).with_float_format(FloatFormat::Quad));
        map.insert("fsgnjx.q",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b00100).with_funct3(0b010).with_float_format(FloatFormat::Quad));
        map.insert("fmin.q",    Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b00101).with_funct3(0b000).with_float_format(FloatFormat::Quad));
        map.insert("fmax.q",    Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b00101).with_funct3(0b001).with_float_format(FloatFormat::Quad));

        map.insert("feq.q",     Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b10100).with_funct3(0b010).with_float_format(FloatFormat::Quad));
        map.insert("flt.q",     Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b10100).with_funct3(0b001).with_float_format(FloatFormat::Quad));
        map.insert("fle.q",     Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b10100).with_funct3(0b000).with_float_format(FloatFormat::Quad));

        map.insert("fcvt.w.q",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b11000).with_rs2(0b00000).with_float_format(FloatFormat::Quad));
        map.insert("fcvt.wu.q", Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b11000).with_rs2(0b00001).with_float_format(FloatFormat::Quad));
        map.insert("fcvt.q.w",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b11010).with_rs2(0b00000).with_float_format(FloatFormat::Quad));
        map.insert("fcvt.q.wu", Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b11010).with_rs2(0b00001).with_float_format(FloatFormat::Quad));

        map.insert("fcvt.s.q",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b01000).with_rs2(0b00000).with_float_format(FloatFormat::Single));
        map.insert("fcvt.q.s",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b01000).with_rs2(0b00000).with_float_format(FloatFormat::Quad));
        map.insert("fcvt.d.q",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b01000).with_rs2(0b00000).with_float_format(FloatFormat::Double));
        map.insert("fcvt.q.d",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b01000).with_rs2(0b00000).with_float_format(FloatFormat::Quad));

        map.insert("fclass.q",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV32Q).with_funct5(0b11100).with_rs2(0b00000).with_funct3(0b001).with_float_format(FloatFormat::Quad));

        map.insert("fcvt.l.q",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV64Q).with_funct5(0b11000).with_rs2(0b00010).with_float_format(FloatFormat::Quad));
        map.insert("fcvt.lu.q", Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV64Q).with_funct5(0b11000).with_rs2(0b00011).with_float_format(FloatFormat::Quad));
        map.insert("fcvt.q.l",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV64Q).with_funct5(0b11010).with_rs2(0b00010).with_float_format(FloatFormat::Quad));
        map.insert("fcvt.q.lu", Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV64Q).with_funct5(0b11010).with_rs2(0b00011).with_float_format(FloatFormat::Quad));

        map.insert("fmv.x.q",   Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV128Q).with_funct5(0b11100).with_rs2(0b00000).with_funct3(0b000).with_float_format(FloatFormat::Quad));
        map.insert("fmv.q.x",   Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV128Q).with_funct5(0b11110).with_rs2(0b00000).with_funct3(0b000).with_float_format(FloatFormat::Quad));

        map.insert("fcvt.t.q",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV128Q).with_funct5(0b11000).with_rs2(0b00100).with_float_format(FloatFormat::Quad));
        map.insert("fcvt.tu.q", Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV128Q).with_funct5(0b11000).with_rs2(0b00101).with_float_format(FloatFormat::Quad));
        map.insert("fcvt.q.t",  Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV128Q).with_funct5(0b11010).with_rs2(0b00100).with_float_format(FloatFormat::Quad));
        map.insert("fcvt.q.tu", Instruction::new(Opcode::OpFp,    Format::RType, ISA::RV128Q).with_funct5(0b11010).with_rs2(0b00101).with_float_format(FloatFormat::Quad));

        map
    };

    pub static ref CONVENTIONAL_TO_ABI: HashMap<&'static str, &'static str> = 
    {
        let mut map = HashMap::new();

        map.insert("zero", "x0");
        map.insert("ra", "x1");
        map.insert("sp", "x2");
        map.insert("gp", "x3");
        map.insert("tp", "x4");
        map.insert("t0", "x5");
        map.insert("t1", "x6");
        map.insert("t2", "x7");
        map.insert("s0", "x8");
        map.insert("s1", "x9");
        map.insert("a0", "x10");
        map.insert("a1", "x11");
        map.insert("a2", "x12");
        map.insert("a3", "x13");
        map.insert("a4", "x14");
        map.insert("a5", "x15");
        map.insert("a6", "x16");
        map.insert("a7", "x17");
        map.insert("s2", "x18");
        map.insert("s3", "x19");
        map.insert("s4", "x20");
        map.insert("s5", "x21");
        map.insert("s6", "x22");
        map.insert("s7", "x23");
        map.insert("s8", "x24");
        map.insert("s9", "x25");
        map.insert("s10", "x26");
        map.insert("s11", "x27");
        map.insert("t3", "x28");
        map.insert("t4", "x29");
        map.insert("t5", "x30");
        map.insert("t6", "x31");
        map.insert("fp", "x8");      
        map.insert("ft0", "f0");
        map.insert("ft1", "f1");
        map.insert("ft2", "f2");
        map.insert("ft3", "f3");
        map.insert("ft4", "f4");
        map.insert("ft5", "f5");
        map.insert("ft6", "f6");
        map.insert("ft7", "f7");
        map.insert("fs0", "f8");
        map.insert("fs1", "f9");
        map.insert("fa0", "f10");
        map.insert("fa1", "f11");
        map.insert("fa2", "f12");
        map.insert("fa3", "f13");
        map.insert("fa4", "f14");
        map.insert("fa5", "f15");
        map.insert("fa6", "f16");
        map.insert("fa7", "f17");
        map.insert("fs2", "f18");
        map.insert("fs3", "f19");
        map.insert("fs4", "f20");
        map.insert("fs5", "f21");
        map.insert("fs6", "f22");
        map.insert("fs7", "f23");
        map.insert("fs8", "f24");
        map.insert("fs9", "f25");
        map.insert("fs10", "f26");
        map.insert("fs11", "f27");
        map.insert("ft8", "f28");
        map.insert("ft9", "f29");
        map.insert("ft10", "f30");
        map.insert("ft11", "f31");

        map
    };
}