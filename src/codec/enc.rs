use crate::{
    lexer::*, 
    arch::*
};

#[derive(Debug)]
pub enum EncoderErr
{
    Token(String),
    Mnemonic(String),
    Format(String),
    Operands(String),
    FloatRounding(String)
}

pub struct Encoder
{
    pub binary: u32
}

impl Encoder {
    pub fn new(mnemonic: &String, operands: &Vec<Operand>) -> Result<Self, EncoderErr> 
    {
        if !RV_ISA.contains_key(mnemonic.as_str())
        {
            return Err(EncoderErr::Mnemonic(
                format!(r#"Unsupported instruction mnemonic: "{}""#, mnemonic)
            ))
        }        
        
        let instruction = &RV_ISA[mnemonic.as_str()];
        match instruction.opcode
        {                    
            Opcode::Op | Opcode::Op32 | Opcode::Op64 => 
            {
                Ok(Encoder{
                    binary: Self::encode_op(instruction, operands)?
                })
            }
            Opcode::OpFp =>
            {
                Ok(Encoder{
                    binary: Self::encode_fp(instruction, operands)?
                })
            }
            Opcode::Amo => 
            {
                Ok(Encoder{
                    binary: Self::encode_amo(instruction, operands)?
                })
            }
            Opcode::Jalr =>
            {
                Ok(Encoder{
                    binary: Self::encode_jalr(instruction, operands)?
                })            }
            Opcode::Load | Opcode::LoadFp =>
            {
                Ok(Encoder{
                    binary: Self::encode_load(instruction, operands)?
                })     
            }
            Opcode::OpImm | Opcode::OpImm32 | Opcode::OpImm64 =>
            {
                Ok(Encoder{
                    binary: Self::encode_op_imm(instruction, operands)?
                })                 
            }
            Opcode::MiscMem =>
            {
                Ok(Encoder{
                    binary: Self::encode_misc_mem(mnemonic, instruction, operands)?
                })               
            }
            Opcode::System => 
            {
                Ok(Encoder{
                    binary: Self::encode_system(instruction, operands)?
                })     
            }            
            Opcode::Store | Opcode::StoreFp =>
            {
                Ok(Encoder{
                    binary: Self::encode_store(instruction, operands)?
                })     
            }
            Opcode::Branch => 
            {
                Ok(Encoder{
                    binary: Self::encode_branch(instruction, operands)?
                })                 
            }
            Opcode::Lui | Opcode::AuiPC =>
            {
                Ok(Encoder{
                    binary: Self::encode_u_type(instruction, operands)?
                })     
            }
            Opcode::Jal =>
            {
                Ok(Encoder{
                    binary: Self::encode_jal(instruction, operands)?
                })     
            }
            Opcode::MAdd | Opcode::MSub | 
            Opcode::NmAdd | Opcode::NmSub =>
            { // todo: add support for FMA/R4 opcode instructions.
                Err(EncoderErr::Format(r#"Unsupported FMA/R4 opcode instruction."#.to_string()))     
            }
        }
    }

    fn encode_op(instruction: &Instruction, operands: &Vec<Operand>) -> Result<u32, EncoderErr> 
    {
        if let (Ok(RValue::Register(_, rd)), Ok(RValue::Register(_, rs1)), Ok(RValue::Register(_, rs2))) = 
            (&operands[0].try_into(), &operands[1].try_into(), &operands[2].try_into()) 
        {
            let funct7 = instruction.funct7.unwrap() as u32;
            let funct3 = instruction.funct3.unwrap() as u32;
            let opcode = instruction.opcode as u32;
        
            return Ok((funct7 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode)
        } 
        else 
        {
           return Err(EncoderErr::Operands(
                r#"Invalid operands."#.to_string()
            ))
        }
    }

    fn encode_fp(instruction: &Instruction, operands: &Vec<Operand>) -> Result<u32, EncoderErr> 
    { // Todo: Add support for float rounding modes.
        const FRM: u32 = 0b000;

        if let(Ok(RValue::Register(_, rd)), Ok(RValue::Register(_, rs1)), Ok(RValue::Register(_, rs2))) = 
            (&operands[0].try_into(), &operands[1].try_into(), &operands[2].try_into()) 
            {    
            let funct5 = instruction.funct5.unwrap() as u32;
            let opcode = instruction.opcode as u32;
    
            let float_rd = funct5 & 0b10000 != 0;
            let float_rs1 = if funct5 & 0b1000 != 0 {
                funct5 & 0b1000 != 0
            } else {
                !float_rd
            };
    
            let rd = if float_rd { *rd } else { rd & 0b11111 };
            let rs1 = if float_rs1 { *rs1 } else { rs1 & 0b11111 };
            let rs2 = if let Some(rs2_val) = instruction.rs2 {
                rs2_val as u32
            } else {
                rs2 & 0b11111
            };
    
            Ok((funct5 << 25) | (rs2 << 20) | (rs1 << 15) | (FRM << 12) | (rd << 7) | opcode)
        } 
        else 
        {
            Err(EncoderErr::Operands(
                r#"Invalid operands."#.to_string()
            ))
        }
    } 
    
    fn encode_amo(instruction: &Instruction, operands: &Vec<Operand>) -> Result<u32, EncoderErr> 
    {
        if let(Ok(RValue::Register(_, rd)), Ok(RValue::Register(_, rs1)), Ok(RValue::Register(_, rs2))) = 
            (&operands[0].try_into(), &operands[1].try_into(), &operands[2].try_into()) 
        {
            let funct5 = instruction.funct5.unwrap() as u32;
            let funct3 = instruction.funct3.unwrap() as u32;
            let opcode = instruction.opcode as u32;
        
            const AQ: u32 = 0;
            const RL: u32 = 0;

            Ok((funct5 << 27) | (AQ << 26) | (RL << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode)
        } 
        else 
        {
            Err(EncoderErr::Operands(
                r#"Invalid operands."#.to_string()
            ))
        }
    }
  
    fn encode_jalr(instruction: &Instruction, operands: &Vec<Operand>) -> Result<u32, EncoderErr> 
    {
        if let(Ok(RValue::Register(_, rd)), Ok(Operand::Address(RValue::Register(_, rs1), RValue::Immediate(offset)))) = 
            (&operands[0].try_into(), &operands[1].try_into()) 
        {
            let funct3 = instruction.funct3.unwrap() as u32;
            let opcode = instruction.opcode as u32;
            let imm: u32 = (*offset as u32) & 0xFFF;
            
            Ok((imm << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode)
        } 
        else 
        {
            Err(EncoderErr::Operands(
                r#"Invalid operands."#.to_string()
            ))
        }
    }

    fn encode_load(instruction: &Instruction, operands: &Vec<Operand>) -> Result<u32, EncoderErr> 
    {
        if let(Ok(RValue::Register(_, rd)), Ok(Operand::Address(RValue::Register(_, rs1), RValue::Immediate(offset)))) = 
            (&operands[0].try_into(), &operands[1].try_into())
        {
            let funct3 = instruction.funct3.unwrap() as u32;
            let opcode = instruction.opcode as u32;
            let imm: u32 = (*offset as u32) & 0xFFF;
            
            Ok((imm << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode)
        } 
        else 
        {
            Err(EncoderErr::Operands(
                r#"Invalid operands."#.to_string()
            ))
        }
    }

    fn encode_op_imm(instruction: &Instruction, operands: &Vec<Operand>) -> Result<u32, EncoderErr> 
    {
        if let(Ok(RValue::Register(_, rd)), Ok(RValue::Register(_, rs1)), Ok(RValue::Immediate(immediate))) = 
            (&operands[0].try_into(), &operands[1].try_into(), &operands[2].try_into())
        {
            let funct3 = instruction.funct3.unwrap() as u32;
            let opcode = instruction.opcode as u32;
    
            let imm: u32;

            if let Some(shift_type) = &instruction.shift 
            {
                let shamt_width: u32 = match instruction.opcode 
                {
                    Opcode::OpImm32 => 5,  
                    Opcode::OpImm64 => 6,
                    _ => 7
                };
    
                if *immediate < 0 || *immediate >= (1 << shamt_width)
                {
                    return Err(EncoderErr::Operands(format!(
                        r#"Invalid shamt field (out of range): "{}""#, immediate
                    )));
                }
                let imm_11_7: u32 = (0b0 << 4) | (*shift_type as u32);
                let imm_6_0: u32 = (*immediate as u32) & ((1 << shamt_width) - 1);
                imm = (imm_11_7 << 6) | imm_6_0;
            } 
            else 
            {
                imm = (*immediate as u32) & 0xFFF;
            }
    
            Ok((imm << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode)  
        } 
        else 
        {
            Err(EncoderErr::Operands(
                r#"Invalid operands."#.to_string()
            ))
        }
    }

    fn encode_misc_mem(mnemonic: &String, instruction: &Instruction, operands: &Vec<Operand>) -> Result<u32, EncoderErr> 
    {
        let mut imm: u32 = 0;
        let mut rs1: u32 = 0;
        let mut rd: u32 = 0;
    
        if mnemonic == "lq"
        {
            if let (Ok(RValue::Register(_, rd_val)), Ok(Operand::Address(RValue::Register(_, rs1_val), RValue::Immediate(offset)))) = 
                (&operands[0].try_into(), &operands[1].try_into())
            {
                rd = *rd_val;
                rs1 = *rs1_val;
                imm = (*offset as u32) & 0xFFF;
            } 
            else 
            {
                return Err(EncoderErr::Operands(
                    "Invalid operands.".to_string()
                ))
            }
        } 
        else if mnemonic == "fence"
        {
            if let (Ok(RValue::Immediate(pred)), Ok(RValue::Immediate(succ))) = 
                (&operands[0].try_into(), &operands[1].try_into())
            {
                imm = ((*pred as u32) << 4) | (*succ as u32);
            }
        }
    
        let funct3 = instruction.funct3.unwrap() as u32;
        let opcode = instruction.opcode as u32;
        Ok((imm << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode)
    }

    fn encode_system(instruction: &Instruction, operands: &Vec<Operand>) -> Result<u32, EncoderErr> 
    {
        let mut rs1: u32 = 0;
        let mut rd: u32 = 0;
        let mut imm: u32 = instruction.funct12.unwrap() as u32;

        if instruction.isa == ISA::Zicsr {
            if let (Ok(RValue::Register(_, dest)), Ok(csr), Ok(src)) = (
                &operands[0].try_into(), &operands[1].try_into(),
                if instruction.funct3.unwrap() & 0b1000 == 0 
                { 
                    Ok(Self::get_register(&operands[2]).unwrap().1)
                } 
                else 
                {
                    Some((*Self::get_immediate(&operands[2]).unwrap()) as u32)
                }
            ) {
                rd = dest;
                imm = *csr as u32;
                rs1 = src.into();
            }
        }

        let funct3 = instruction.funct3.unwrap() as u32;
        let opcode = instruction.opcode as u32;
        Ok((imm << 20) | (rs1 << 15) | (funct3 << 12) | (rd << 7) | opcode)
    }

    fn encode_store(instruction: &Instruction, operands: &Vec<Operand>) -> Result<u32, EncoderErr> 
    {
        if let(Ok(RValue::Register(_, rs2)), Ok(Operand::Address(RValue::Register(_, rs1), RValue::Immediate(offset)))) = 
            (&operands[0].try_into(), &operands[1].try_into())
        {
            let funct3 = instruction.funct3.unwrap() as u32;
            let opcode = instruction.opcode as u32;
            let imm: u32 = (*offset as u32) & 0xFFF;
            let imm_11_5 = (imm >> 5) & 0x7F;
            let imm_4_0 = imm & 0x1F;

            Ok((imm_11_5 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (imm_4_0 << 7) | opcode)
        } 
        else 
        {
            Err(EncoderErr::Operands(
                "Invalid operands.".to_string()
            ))
        }
    }

    fn encode_branch(instruction: &Instruction, operands: &Vec<Operand>) -> Result<u32, EncoderErr> 
    {
        if let (Some((_, rs1)), Some((_, rs2)), Some(imm)) = (
            Self::get_register(&operands[0]),
            Self::get_register(&operands[1]),
            Self::get_immediate(&operands[2])
        ) {
            let funct3 = instruction.funct3.unwrap() as u32;
            let opcode = instruction.opcode as u32;
            let imm_val = *imm as u32;

            let imm_12 = (imm_val >> 12) & 0x1;
            let imm_11 = (imm_val >> 11) & 0x1;
            let imm_10_5 = (imm_val >> 5) & 0x3F;
            let imm_4_1 = imm_val & 0x1F;

            Ok((imm_12 << 31) | (imm_10_5 << 25) | (rs2 << 20) | (rs1 << 15) | (funct3 << 12) | (imm_4_1 << 7) | (imm_11 << 7) | opcode)
        } 
        else 
        {
            Err(EncoderErr::Operands(
                "Invalid operands.".to_string()
            ))
        }
    }

    fn encode_u_type(instruction: &Instruction, operands: &Vec<Operand>) -> Result<u32, EncoderErr> 
    {
        if let(Ok(RValue::Register(_, rd)), Ok(RValue::Immediate(imm))) = (&operands[0].try_into(), &operands[1].try_into())
        {
            let opcode = instruction.opcode as u32;
            let imm_val = ((*imm as u32) & 0xFFFFF) << 12;

            Ok(imm_val | (rd << 7) | opcode)
        } 
        else 
        {
            Err(EncoderErr::Operands(
                "Invalid operands.".to_string()
            ))
        }
    }

    fn encode_jal(instruction: &Instruction, operands: &Vec<Operand>) -> Result<u32, EncoderErr> 
    {
        if let (Some((_, rd)), Some(imm)) = (
            Self::get_register(&operands[0]),
            Self::get_immediate(&operands[1])
        ) {
            let opcode = instruction.opcode as u32;
            let imm_val = *imm as u32;

            let imm_20 = (imm_val >> 20) & 0x1;
            let imm_19_12 = (imm_val >> 12) & 0xFF;
            let imm_11 = (imm_val >> 11) & 0x1;
            let imm_10_1 = (imm_val >> 1) & 0x3FF;

            Ok((imm_20 << 31) | (imm_19_12 << 20) | (imm_11 << 20) | (imm_10_1 << 1) | (rd << 7) | opcode)
        } 
        else 
        {
            Err(EncoderErr::Operands(
                "Invalid operands.".to_string()
            ))
        }
    }
}