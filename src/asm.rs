use std::collections::HashMap;
use lazy_static::lazy_static;

use crate::{
    lexer::*, lex, 
 // codec::encoder::*
};

pub struct Object
{
    binary: Vec<u8>,
    relocations: Vec<(Emittable /* Instruction */, usize /* Start address */)>,
    symbols: HashMap<String /* Identifier */, usize /* Start address */>
}

impl Object
{
    pub fn new() -> Self
    {
        Object
        {
            binary: Vec::new(),
            relocations: Vec::new(),
            symbols: HashMap::new()
        }
    }
}

lazy_static!
{
    pub static ref PSEUDO_INSTRUCTIONS: HashMap<&'static str /* Mnemonic */, Vec<Token>> =
    {
        let mut m = HashMap::new();
   
        m.insert("nop",    lex!("add zero, zero, 0").unwrap());
        m.insert("mv",     lex!("addi rd, rs, 0").unwrap());    
        m.insert("not",    lex!("xori rd, rs1, -1").unwrap());
        m.insert("neg",    lex!("sub rd, x0, rs1").unwrap());
        m.insert("negw",   lex!("subw rd, x0, rs1").unwrap());
        m.insert("sext.w", lex!("addiw rd, rs1, 0").unwrap());
        m.insert("seqz",   lex!("sltiu rd, rs1, 1").unwrap());
        m.insert("snez",   lex!("sltu rd, x0, rs1").unwrap());
        m.insert("sltz",   lex!("slt rd, rs1, x0").unwrap());
        m.insert("sgtz",   lex!("slt rd, x0, rs1").unwrap());
        m.insert("fmv.s",  lex!("fsgnj.s frd, frs1, frs1").unwrap());
        m.insert("fabs.s", lex!("fsgnjx.s frd, frs1, frs1").unwrap());
        m.insert("fneg.s", lex!("fsgnjn.s frd, frs1, frs1").unwrap());
        m.insert("fmv.d",  lex!("fsgnj.d frd, frs1, frs1").unwrap());
        m.insert("fabs.d", lex!("fsgnjx.d frd, frs1, frs1").unwrap());
        m.insert("fneg.d", lex!("fsgnjn.d frd, frs1, frs1").unwrap());
        m.insert("beqz",   lex!("beq rs1, x0, offset").unwrap());
        m.insert("bnez",   lex!("bne rs1, x0, offset").unwrap());
        m.insert("blez",   lex!("bge x0, rs1, offset").unwrap());
        m.insert("bgez",   lex!("bge rs1, x0, offset").unwrap());
        m.insert("bltz",   lex!("blt rs1, x0, offset").unwrap());
        m.insert("bgtz",   lex!("blt x0, rs1, offset").unwrap());
        m.insert("bgt",    lex!("blt rt, rs, offset").unwrap());
        m.insert("ble",    lex!("bge rt, rs, offset").unwrap());
        m.insert("bgtu",   lex!("bltu rt, rs, offset").unwrap());
        m.insert("bleu",   lex!("bltu rt, rs, offset").unwrap());
        m.insert("j",      lex!("jal x0, offset").unwrap());
        m.insert("jr",     lex!("jal x1, offset").unwrap());
        m.insert("ret",    lex!("jalr x0, x1, 0").unwrap());
        
        m
    };
}


#[derive(Debug, Clone, PartialEq)]
pub enum AssemblerErr
{
    Encoding(String),
    Lexer(LexerErr),
    Syntax(String),
    Other(String)
}

pub struct Assembler
{
    pub object: Object
}

impl Assembler
{
    pub fn new(code: &str) -> Result<Self, AssemblerErr>
    {
        match lex!(code)
        {
            Ok(mut tokens) =>
            { // expand macros into their respective instructions.
                Self::process_macros(&mut tokens)?;

                Ok(Assembler       
                { // Process data, instructions, relocations, etc.
                    object: Object::new()
                })
            },
            // Propagate lexer errors.
            Err(lexer_err) => Err(AssemblerErr::Lexer(
                lexer_err
            ))
        }
    }

    fn process_macros(tokens: &mut Vec<Token>) -> Result<&mut Vec<Token>, AssemblerErr>
    {        
        let mut iter = tokens.iter().enumerate();
        let mut to_drain = Vec::new();

        while let Some((index, token)) = iter.next()
        {
            match token
            {
                Token::Directive(Directive::Macro(name_str, args)) =>
                {
                    let end_index = match tokens[index+1..]
                        .iter().position(|token| *token == Token::Directive(Directive::Marker("endm".into())))
                        {
                            Some(val) => val,
                            None => return Err(AssemblerErr::Syntax(
                                format!(r#""{}" expected an end marker."#, name_str)
                            ))
                        };
                    to_drain.push((index, end_index));
                },
                // Skip over any non macro tokens.
                _ => {}
            }
        }
        Ok(tokens)
    }

    fn process_instruction(token: Token) -> Result<u32, AssemblerErr>
    {
        match token
        {
            Token::Emittable(Emittable::Instruction(mnemonic, operands)) =>
            { // Check if mnemonic matches any pseudo-instruction.
                if PSEUDO_INSTRUCTIONS.contains_key(mnemonic.as_str())
                {
                    let instruction = &PSEUDO_INSTRUCTIONS[mnemonic.as_str()];

                    match instruction.get(0).unwrap()
                    { // Expansion code.
                        Token::Emittable(Emittable::Instruction(ps_mnemonic, ps_operands )) =>
                        {
                            if ps_operands.len() != operands.len()
                            { // Too few, too many arguments provided.
                                return Err(AssemblerErr::Syntax(
                                    format!(r#""{}" expected {} arguments, {} arguments provided."#, ps_mnemonic, ps_operands.len(), operands.len())
                                ))
                            }
                            
                            // Re-process instruction with corrected token.
                            return Self::process_instruction(
                                Token::Emittable(Emittable::Instruction(
                                    ps_mnemonic.clone(), operands
                                ))
                            )
                        },
                        // Shouldn't be met if lexer! and the static expansions are correct.
                        _ => return Err(AssemblerErr::Other(
                            r#"Failed to parse code for pseudo-instruction: "{}""#.into()
                        ))
                    }
                }

                Ok(0)

            },
            _ => 
            {
                Ok(0)
            }
        }
        

    }
}