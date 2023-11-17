use std::{collections::HashMap, hash::Hash};
use lazy_static::lazy_static;

use crate::{
    lexer::*, lex, arch::*, 
    codec::enc::*, encode
};

pub struct Object
{
    pub binary: Vec<u8>,
    pub relocations: Vec<(Emittable /* Instruction */, usize /* Start address */)>,
    pub symbols: HashMap<String /* Identifier */, usize /* Start address */>
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
    pub static ref PSEUDO_INSTRUCTIONS: HashMap<&'static str, (Vec<&'static str>, Vec<Token>)> =
    {
        let mut m = HashMap::new();
        // Mnemonic         Arguments                   Instruction(s)
        m.insert("nop",    (vec![],                     lex!("addi zero, zero, 0").unwrap()));
        m.insert("mv",     (vec!["rd", "rs"],           lex!("addi rd, rs, 0").unwrap()));    
        m.insert("not",    (vec!["rd", "rs1"],          lex!("xori rd, rs1, -1").unwrap()));
        m.insert("neg",    (vec!["rd", "rs1"],          lex!("sub rd, x0, rs1").unwrap()));
        m.insert("negw",   (vec!["rd", "rs1"],          lex!("subw rd, x0, rs1").unwrap()));
        m.insert("sext.w", (vec!["rd", "rs1"],          lex!("addiw rd, rs1, 0").unwrap()));
        m.insert("seqz",   (vec!["rd", "rs1"],          lex!("sltiu rd, rs1, 1").unwrap()));
        m.insert("snez",   (vec!["rd", "rs1"],          lex!("sltu rd, x0, rs1").unwrap()));
        m.insert("sltz",   (vec!["rd", "rs1"],          lex!("slt rd, rs1, x0").unwrap()));
        m.insert("sgtz",   (vec!["rd", "rs1"],          lex!("slt rd, x0, rs1").unwrap()));
        m.insert("fmv.s",  (vec!["frd", "frs1"],        lex!("fsgnj.s frd, frs1, frs1").unwrap()));
        m.insert("fabs.s", (vec!["frd", "frs1"],        lex!("fsgnjx.s frd, frs1, frs1").unwrap()));
        m.insert("fneg.s", (vec!["frd", "frs1"],        lex!("fsgnjn.s frd, frs1, frs1").unwrap()));
        m.insert("fmv.d",  (vec!["frd", "frs1"],        lex!("fsgnj.d frd, frs1, frs1").unwrap()));
        m.insert("fabs.d", (vec!["frd", "frs1"],        lex!("fsgnjx.d frd, frs1, frs1").unwrap()));
        m.insert("fneg.d", (vec!["frd", "frs1"],        lex!("fsgnjn.d frd, frs1, frs1").unwrap()));
        m.insert("beqz",   (vec!["rs1", "offset"],      lex!("beq rs1, x0, offset").unwrap()));
        m.insert("bnez",   (vec!["rs1", "offset"],      lex!("bne rs1, x0, offset").unwrap()));
        m.insert("blez",   (vec!["rs1", "offset"],      lex!("bge x0, rs1, offset").unwrap()));
        m.insert("bgez",   (vec!["rs1", "offset"],      lex!("bge rs1, x0, offset").unwrap()));
        m.insert("bltz",   (vec!["rs1", "offset"],      lex!("blt rs1, x0, offset").unwrap()));
        m.insert("bgtz",   (vec!["rs1", "offset"],      lex!("blt x0, rs1, offset").unwrap()));
        m.insert("bgt",    (vec!["rs", "rt", "offset"], lex!("blt rt, rs, offset").unwrap()));
        m.insert("ble",    (vec!["rs", "rt", "offset"], lex!("bge rt, rs, offset").unwrap()));
        m.insert("bgtu",   (vec!["rs", "rt", "offset"], lex!("bltu rt, rs, offset").unwrap()));
        m.insert("bleu",   (vec!["rt", "rs", "offset"], lex!("bltu rt, rs, offset").unwrap()));
        m.insert("j",      (vec!["offset"],             lex!("jal x0, offset").unwrap()));
        m.insert("jr",     (vec!["offset"],             lex!("jal x1, offset").unwrap()));
        m.insert("ret",    (vec![],                     lex!("jalr x0, x1, 0").unwrap()));
        // li variations.
        m.insert("li.16",  (vec!["rd", "imm"],          lex!("addi rd, x0, imm").unwrap()));
        m.insert("li.32",  (vec!["rd", "imm"],          lex!("lui rd, %hi(imm)
                                                             addi rd, rd, %lo(imm)").unwrap()));
        m.insert("li.64",  (vec!["rd", "imm"],          lex!("lui rd, %highest(imm)
                                                              addi rd, rd, %higher(imm)
                                                              slli rd, rd, 32
                                                              addi rd, rd, %hi(imm) 
                                                              addi rd, rd, %lo(imm)").unwrap()));     
        // la variations.                                               
        m.insert("la.16",   (vec!["rd", "symbol"],      lex!("auipc rd, %pcrel_hi(symbol)
                                                              addi rd, rd, %pcrel_lo(symbol)").unwrap()));
        m.insert("la.32",   (vec!["rd", "symbol"],      lex!("lui rd, %hi(symbol)
                                                              addi rd, rd, %lo(symbol)").unwrap()));      
        m.insert("la.64",   (vec!["rd", "symbol"],      lex!("lui rd, %highest(symbol)
                                                              addi rd, rd, %higher(symbol)
                                                              slli rd, rd, 32
                                                              addi rd, rd, %hi(symbol)
                                                              addi rd, rd, %lo(symbol)").unwrap()));
                                                              
        m
    };
}


#[derive(Debug, Clone, PartialEq)]
pub enum AssemblerErr
{
    Encoder(EncoderErr),
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
            { // Drain macro tokens from the token stream.
                let macros = Self::drain_macros(&mut tokens)?;

                // expand pseudo-code into their counterparts.
                let t = Self::process_expansions(&mut tokens, &macros)?;

                Ok(Assembler       
                { // Process data, instructions, relocations, etc.
                    object: Self::process_binary(&t)?
                })
            },
            // Propagate lexer errors.
            Err(lexer_err) => Err(AssemblerErr::Lexer(lexer_err))
        }
    }

    fn drain_macros(tokens: &mut Vec<Token>) -> Result<HashMap<String, (Vec<String>, Vec<Token>)>, AssemblerErr>
    {
        let mut to_drain = Vec::new();
        
        // Identify macro directives and their ranges.
        for (index, token) in tokens.iter().enumerate() 
        {
            match token
            {
                Token::Directive(Directive::Macro(name_str, args)) => 
                { // Check if macro name is a reserved keyword.
                    if RV_ISA.contains_key(name_str.as_str()) || PSEUDO_INSTRUCTIONS.contains_key(name_str.as_str())
                    {
                        return Err(AssemblerErr::Syntax(
                            format!(r#""{}" is a reserved keyword."#, name_str)
                        ))
                    }

                    let end_index = tokens[index+1..].iter()
                        .position(|t| matches!(t, Token::Directive(Directive::Marker(m)) if m == "endm"))
                        .ok_or_else(|| AssemblerErr::Syntax(
                            format!(r#""{}" expected an end marker."#, name_str)
                        ))? + index + 1;

                    to_drain.push((name_str.clone(), args.clone(), index, end_index));
                },
                _ => {}
            }
        }
        // Sort ranges in reverse order to avoid index shifting during draining.
        to_drain.sort_by(|a, b| b.2.cmp(&a.2));

        let mut macros = HashMap::new();
    
        // Drain macro tokens into the hash map.
        for (name_str, args, start_index, end_index) in to_drain 
        {
            let mut macro_tokens = tokens.drain(start_index..=end_index).collect::<Vec<Token>>();
            macro_tokens.remove(0); // Remove macro directive.
            macro_tokens.pop();           // Remove endm marker.
    
            macros.insert(name_str, (args.to_vec(), macro_tokens));
        }

        Ok(macros)
    }

    // Take expansive code and splice it into the token stream.
    fn expand_code(arguments: Vec<Operand>, exp_details: &mut (Vec<String>, Vec<Token>)) -> Result<&Vec<Token>, AssemblerErr>
    {
        if arguments.len() != exp_details.0.len()
        { // Too few, too many arguments provided.
            return Err(AssemblerErr::Syntax(
                format!(r#"Expected {} arguments, found {}."#, exp_details.0.len(), arguments.len())
            ))
        }

        for token in &mut exp_details.1
        {
            match token
            {
                Token::Emittable(Emittable::Instruction(_, mm_arguments)) =>
                { // Map placeholder identifiers to actual arguments.
                    for mm_argument in mm_arguments
                    {
                        let get_argument_fn = |identifier: &str| -> Result<Operand, AssemblerErr>
                        {
                            let index = exp_details.0.iter()
                                .position(|arg| arg == identifier)
                                .ok_or_else(|| AssemblerErr::Syntax(
                                    format!(r#"Argument "{}" not found."#, identifier)
                                ))?;
                            Ok(arguments[index].clone())
                        };

                        if let Operand::RValue(RValue::Identifier(identifier)) = mm_argument
                        {
                            *mm_argument = get_argument_fn(&identifier)?;
                        }
                    }
                },
                _ => {}
            }
        }
        
        Ok(&exp_details.1)
    }
    
    fn process_expansions<'a>(tokens: &'a mut Vec<Token>, macros: &'a HashMap<String, (Vec<String>, Vec<Token>)>) -> Result<&'a Vec<Token>, AssemblerErr>
    {
        let mut indices_to_expand = Vec::new();
    
        // Find indices to expand in the token stream.
        for (index, token) in tokens.iter_mut().enumerate() 
        {
            if let Token::Emittable(Emittable::Instruction(mnemonic, arguments)) = token 
            { // Adjust mnemonic to match the width of the operand.
                if mnemonic == "li" || mnemonic == "la"
                {
                    if let Operand::RValue(RValue::Immediate(imm)) = &arguments[1]
                    {
                        let width = match imm 
                        {
                            -32768..=32767 => "16",
                            -2147483648..=2147483647 => "32",
                            _ => "64"
                        };

                        *mnemonic = format!("{}.{}", mnemonic, width);
                    }
                    else
                    {
                        return Err(AssemblerErr::Syntax(
                            format!(r#"Expected immediate operand, found "{:?}"."#, arguments[1])
                        ))
                    }
                }

                if PSEUDO_INSTRUCTIONS.contains_key(mnemonic.as_str()) || macros.contains_key(mnemonic.as_str()) 
                {
                    indices_to_expand.push(index);
                }
            }
        }
        // Sort indices in reverse order to avoid index shifting during expansion.
        indices_to_expand.sort_by(|a, b| b.cmp(&a));
        
        // Expand tokens at the indices.
        for index in indices_to_expand 
        {
            if let Token::Emittable(Emittable::Instruction(mnemonic, arguments)) = &tokens[index] 
            {
                let mut exp_details = 
                    if let Some(details) = PSEUDO_INSTRUCTIONS.get(mnemonic.as_str()) 
                    {
                        (details.0.iter().map(|s| s.to_string()).collect(), details.1.clone())
                    }
                    else if let Some(details) = macros.get(mnemonic.as_str()) 
                    {
                        details.clone()
                    }
                    else
                    {
                        continue;
                    };

                let expanded_tokens = Self::expand_code(arguments.clone(), &mut exp_details)?;
                tokens.splice(index..=index, expanded_tokens.iter().cloned());
            }
        }
        Ok(tokens)
    }


    fn process_binary(tokens: &Vec<Token>) -> Result<Object, AssemblerErr>
    {
        let mut object = Object::new();

        for token in tokens
        {
            match token
            {
                Token::Emittable(Emittable::Instruction(mnemonic, operands)) =>
                {
                    let bytes = &encode!(&mnemonic, &operands).map_err(AssemblerErr::Encoder)?;
                                        
                    object.binary.extend_from_slice(bytes);
                },
                _ => {}
            }
        }
        Ok(object)
    }
}