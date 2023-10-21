use crate::{
    utility::*,
    codec::architecture::*,
    memory::*
};

use lazy_static::lazy_static;
use std::str::FromStr;
use regex::Regex;

lazy_static!
{ // Matches strings representing directives (e.g. ".section", ".align 0x4, 0xff").
    static ref DIRECTIVE_REGEX: Regex          = Regex::new(r#"^\.[a-zA-Z0-9_]+"#).unwrap();

    // Matches strings representing labels (e.g. "label:")
    static ref LABEL_REGEX: Regex              = Regex::new(r#"^[a-zA-Z0-9_]+:"#).unwrap();

    // Matches strings representing ABI/Conventional register namings (e.g. "x0" or "zero").
    static ref REGISTER_REGEX: Regex           = Regex::new(r#"^\s*(x\d+|zero|ra|sp|gp|tp|t[0-6]|s[0-1][0-1]?|a[0-7]|f\d+|ft[0-7]|fs[0-1][0-1]?|fa[0-7])\s*$"#).unwrap();

    // Matches strings representing relative addressing (e.g. "-4(Symbol)")
    static ref RELATIVE_ADDRESS_REGEX: Regex   = Regex::new(r#"(-?\d+)\(([a-zA-Z_][a-zA-Z0-9_]*)\)"#).unwrap();

    // Matches strings representing relocation functions (e.g. "%hi(Symbol)").
    static ref RELOCATION_REGEX: Regex         = Regex::new(r#"%((?:pc|tp)?rel_)?(hi|lo|add)\([^)]+\)"#).unwrap();

    /* Matches strings representing:
     *      Negative decimal numbers (e.g. "-123").
     *      Positive decimal numbers (e.g. "123").
     *      Hexadecimal numbers with either lowercase or uppercase letters (e.g. "0xff" or "0X1A"). */
    static ref SIGNED_REGEX: Regex          = Regex::new(r"^(-\d+|\d+|0x[0-9a-fA-F]+)$").unwrap();

    // Matches strings following allowed identifier characters.
    static ref IDENTIFIER_REGEX: Regex      = Regex::new(r#"^[a-zA-Z0-9_]*$"#).unwrap();

    // Matches strings within quotations (e.g. r#""Hello World!""#).
    static ref STRING_REGEX: Regex          = Regex::new(r#""(.*?)""#).unwrap();
}

#[derive(Debug, Clone, PartialEq)]
pub enum RValue
{
    Register(char /* Type */, u32 /* Index */),
    Identifier(String /* Macro parameters, Symbols. */),
    Immediate(i32 /* Immediate value */)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Operand
{
    RValue(RValue),
    RelocationFn(String /* Function name */, RValue /* Symbol */),
    Address(RValue /* Relative Symbol/register */, RValue /* Offset value */)
}

impl From<RValue> for Operand
{
    fn from(rvalue: RValue) -> Self
    {
        Operand::RValue(rvalue)
    }
}

impl std::convert::TryFrom<Operand> for RValue {
    type Error = ();

    fn try_from(operand: Operand) -> Result<Self, Self::Error>
    {
        match operand 
        {
            Operand::RValue(rvalue) => Ok(rvalue),
            _ => Err(()),
        }
    }
}

// Types that directly emit to binary.
#[derive(Debug, Clone, PartialEq)]
pub enum Emittable
{
    Byte(Vec<u8>   /*   8-bit values seperated by commas. */),
    Half(Vec<u16>  /*  16-bit values seperated by commas. */),
    Word(Vec<u32>  /*  32-bit values seperated by commas. */),
    Dword(Vec<u64> /*  64-bit values seperated by commas. */),
    String(String  /*  Null terminated string. */),
    Instruction(String /* Mnemonic */, Vec<Operand> /* Operands (if applicable). */)
}

macro_rules! impl_from_for_emittable
{
    ($($variant:ident($inner_type:ty),)*) =>
    {
        $(
            impl From<$inner_type> for Emittable
            {
                fn from(values: $inner_type) -> Self
                {
                    Emittable::$variant(values)
                }
            }
        )*
    }
}

impl_from_for_emittable!(
    Byte(Vec<u8>),
    Half(Vec<u16>),
    Word(Vec<u32>),
    Dword(Vec<u64>),
);

// Directive to set Symbol visibility (local/global scope).
#[derive(Debug, Clone, PartialEq)]
pub enum Visibility
{
    Local(String  /* Symbol name */),
    Global(String /* Symbol name */)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Align
{
    AsPow(u32 /* alignment (x^2) */, u32 /* Padding value */, u32 /* Max padding value */),
    AsBytes(u32 /* Byte alignment value */, u32 /* Padding value */)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Directive
{
    Alignment(Align),
    Section(String, SectionFlags, u32),
    Equal(String /* Symbol Name */, RValue /* Constant Value */),
    Scope(Visibility /* Symbol visibility (e.g. local, global scope) */),
    Macro(String /* Macro name */, Vec<String> /* Macro arguments */),
    Marker(String /* Name */)
}

impl From<Visibility> for Directive
{
    fn from(vis: Visibility) -> Self
    {
        Directive::Scope(vis)
    }
}

impl From<Align> for Directive
{
    fn from(align: Align) -> Self
    {
        Directive::Alignment(align)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token
{
    Emittable(Emittable /* Emittable */),
    Directive(Directive /* Directive */),
    Label(String /* Symbol name */)
}

impl From<Emittable> for Token
{
    fn from(emittable: Emittable) -> Self
    {
        Token::Emittable(emittable)
    }
}

impl From<Directive> for Token
{
    fn from(directive: Directive) -> Self
    {
        Token::Directive(directive)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum LexerErr
{
    Syntax(String),
    Parsing(String)
}

pub struct Lexer
{
    pub tokens: Vec<Token>
}

impl Lexer
{
    pub fn new(code: &str) -> Result<Self, LexerErr>
    { // Trim comments (denoted by '#'), split labels, filter empty lines.
        let cleansed: Vec<&str> = code.lines()
            .filter_map(|line| line.split('#').next())
            .flat_map(|line|
            {
                let mut parts: Vec<&str> = line.split(':').collect();
                if parts.len() > 1
                {
                    for i in 0..(parts.len() - 1)
                    {
                        if let Some(start) = line.find(parts[i])
                        {
                            let end = start + parts[i].len() + 1;
                            parts[i] = &line[start..end]
                        }
                    }
                }
                parts
            }).map(|s: &str| s.trim())
            .filter(|&s| !s.is_empty())
            .collect();

        Ok(Self{
            tokens: Self::process(cleansed)?
        })
    }

    fn process(code: Vec<&str>) -> Result<Vec<Token>, LexerErr>
    {
        let mut tokens: Vec<Token> = Vec::<Token>::new();

        for line in code
        { // Each label should be on a separate line.
            if LABEL_REGEX.is_match(line)
            { // Tokenize labels.
                tokens.push(Token::Label(line.trim_end_matches(':').into()))
            }
            else if DIRECTIVE_REGEX.is_match(line)
            { // shorten length of directive.
                let directive_str = line.trim_start_matches('.');

                if let Ok(emittable) = Self::get_emittable_directive(directive_str)
                { // Tokenize data emitting directives (e.g. ".string" or ".word").
                    tokens.push(emittable.into())
                }
                else
                { // Tokenize high level directives.
                    tokens.push(Self::get_directive(directive_str)?.into())
                }
            }
            else
            { // Tokenize instructions.
                tokens.push(Self::get_instruction(line)?.into())
            }
        }
        Ok(tokens)
    }

    fn get_directive(line: &str) -> Result<Directive, LexerErr>
    {
        if let Some((directive_str, args_str)) = line.split_once(' ')
        {
            match directive_str
            {
                "global" | "globl" => Ok(Visibility::Global(args_str.into()).into()),
                "local"            => Ok(Visibility::Local(args_str.into()).into()),
                "align" | "p2align" =>
                { // Split arguments at ',', trim and filter words with SIGNED_REGEX.
                    let args_split: Vec<&str> = args_str.split(',')
                        .map(|word| word.trim()).filter(|word| SIGNED_REGEX.is_match(word))
                        .collect();

                    // Too few or too many arguments provided.
                    if args_split.len() <= 0 || args_split.len() > 3
                    {
                        return Err(LexerErr::Syntax(
                            format!(r#"Expected 1-3 arguments. {} arguments were provided."#, args_split.len())
                        ))
                    }

                    let mut args_iter = args_split.iter();
                    let mut extract_or = |default_val: Option<u32>| -> Result<u32, LexerErr>
                    { // Advance, extract provided argument value or default value.
                        args_iter.next().map_or_else(|| default_val.ok_or(LexerErr::Parsing(
                                "Unable to parse alignment value from arguments.".into()
                            )), |arg_str| parse_value::<u32>(arg_str)
                            .or_else(|_| default_val.ok_or(LexerErr::Parsing(
                                    "Unable to parse alignment value from arguments.".into()
                            ))))
                    };

                    // Extract each argument value or it's corresponding default value (0).
                    Ok(Align::AsPow(extract_or(None)?,
                        extract_or(Some(0))?, extract_or(Some(0))?
                    ).into())
                },
                "section" =>
                {
                    let mut flags: SectionFlags = SectionFlags::empty();

                    if let Some(matched) = STRING_REGEX.captures(args_str).and_then(|capture| capture.get(1)) 
                    {
                        for c in matched.as_str().chars() 
                        {
                            flags |= match c {
                                'a' => SectionFlags::ALLOCATE,
                                'w' => SectionFlags::WRITE,
                                'x' => SectionFlags::EXECUTE,
                                'm' => SectionFlags::MERGE,
                                's' => SectionFlags::STRING,
                                'g' => SectionFlags::GROUP,
                                't' => SectionFlags::TLS,
                                _   => return Err(LexerErr::Parsing(
                                    format!(r#"Unexpected section flag identifier: "{}""#, c)
                                )),
                            };
                        } 
                    }
                    Ok(Directive::Section(directive_str.into(), flags, 4))
                },
                _ => Err(LexerErr::Parsing(
                    format!(r#"Unable to parse directive: "{}""#, directive_str)
                ))
            }
        }
        else
        {
            let directive = line.trim();

            match directive
            {
                "text" | "init" | "fini"   => Ok(Directive::Section(directive.into(), SectionFlags::EXECUTE, 4)),
                "bss"  | "sbss" | "rodata" => Ok(Directive::Section(directive.into(), SectionFlags::ALLOCATE, 4)),
                "data" | "sdata" => Ok(Directive::Section(directive.into(), SectionFlags::ALLOCATE | SectionFlags::WRITE, 4)),
                "endm"           => Ok(Directive::Marker(directive.into())),
                _ => Err(LexerErr::Parsing(
                    format!(r#"Unable to match directive: "{}""#, directive)
                ))
            }
        }
    }

    fn get_emittable_directive(line: &str) -> Result<Emittable, LexerErr>
    {
        match line.split_once(' ')
        {
            Some((directive_str, args_str)) =>
            { // Extract each value of 'V' separated by ','.
                fn extract_values<V: ParseWithRadix + FromStr>(arguments: &str) -> Result<Vec<V>, LexerErr>
                {
                    arguments.split(',').map(|s|parse_value::<V>(s.trim())
                        .map_err(|_| LexerErr::Parsing(
                            format!(r#"Unable to parse data directive value: {}"#, s)
                        ))).collect()
                }

                match directive_str
                {
                    "byte"  => Ok(extract_values::<u8>(args_str)?.into()),
                    "half"  => Ok(extract_values::<u16>(args_str)?.into()),
                    "word"  => Ok(extract_values::<u32>(args_str)?.into()),
                    "dword" => Ok(extract_values::<u64>(args_str)?.into()),
                    "string" | "asciz" =>
                    {
                        STRING_REGEX.captures(args_str)
                            .and_then(|capture|
                                capture.get(1).map(|matched|
                                    Emittable::String(matched.as_str().into())
                                )).ok_or_else(|| LexerErr::Parsing(
                                    format!("Invalid arguments provided for .string directive: {}", args_str)
                                ))
                    },
                    "zero" =>
                    {
                        parse_value::<usize>(args_str)
                            .map(|size_val| Emittable::Byte(vec![0; size_val]))
                            .map_err(|_| LexerErr::Parsing(
                                format!(r#"Invalid arguments provided for .zero directive: {}"#, args_str)
                            ))
                    }, // Unmatched directive.
                    _ => Err(LexerErr::Parsing(
                        format!(r#"Unable to parse directive: "{}""#, directive_str)
                    ))
                }
            }, // Arguments weren't provided with a data emitting directive.
            _ => Err(LexerErr::Syntax(
                format!(r#"Expected arguments following directive: "{}""#, line)
            ))
        }
    }

    fn get_instruction(line: &str) -> Result<Emittable, LexerErr>
    { // split instruction mnemonic and operands.
        if let Some((mnemonic_str, operands_str)) = line.split_once(' ')
        {
            let mut tokens = Vec::new();

            // Split at each ',' and tokenize each split operand.
            for operand_str in operands_str.split(',').map(|s|s.trim())
            {
                if REGISTER_REGEX.is_match(operand_str)
                {
                    tokens.push(Self::get_register(operand_str)?.into())
                }
                else if RELATIVE_ADDRESS_REGEX.is_match(operand_str)
                {
                    tokens.push(Self::get_relative_address(operand_str)?)
                }
                else if RELOCATION_REGEX.is_match(operand_str)
                {
                    tokens.push(Self::get_relocation_function(operand_str)?)
                }
                else if SIGNED_REGEX.is_match(operand_str)
                {
                    parse_value::<i32>(operand_str)
                        .map(|val| tokens.push(RValue::Immediate(val).into()))
                        .map_err(|_| LexerErr::Parsing(
                            format!(r#"Unable to parse immediate value: "{}""#, operand_str)
                        ))?
                }
                else if IDENTIFIER_REGEX.is_match(operand_str)
                {
                    tokens.push(RValue::Identifier(operand_str.into()).into())
                }
                else
                {
                    return Err(LexerErr::Syntax(
                        format!("Unexpected instruction operand: {}", operand_str)
                    ))
                }
            }
            Ok(Emittable::Instruction(mnemonic_str.into(), tokens))
        }
        else
        { // Some instructions have no operands (e.g. "nop" or "ecall").
            Ok(Emittable::Instruction(line.trim().into(), vec![]))
        }
    }

    fn get_relative_address(operand: &str) -> Result<Operand, LexerErr>
    { // Either an address stored within a register or an identifier resolved during linking.
        let extract_or_err = |offset_val, ref_str| -> Result<Operand, LexerErr>
        {
            if REGISTER_REGEX.is_match(ref_str)
            {
                Ok(Operand::Address(
                    Self::get_register(ref_str)?, RValue::Immediate(offset_val)
                ))
            }
            else if IDENTIFIER_REGEX.is_match(ref_str)
            {
                Ok(Operand::Address(
                    RValue::Identifier(ref_str.into()), RValue::Immediate(offset_val)
                ))
            }
            else
            {
                Err(LexerErr::Syntax(
                    format!("Unexpected relative address operand: {}", operand)
                ))
            }
        };

        // Trim ending ')' and split at first '('.
        let mut operand_splits = operand.trim_end_matches(')').split('(');

        match operand_splits.next()
        {
            Some(first_str) => match parse_value::<i32>(first_str)
            {
                Ok(offset_val) => match operand_splits.next()
                {
                    Some(second_str) => Ok(
                        extract_or_err(offset_val, second_str)?
                    ),
                    // An offset was provided but an identifier is not present.
                    None => Err(LexerErr::Syntax(
                        format!(r#"Relative address expected an identifier following offset value: "{}""#, operand)
                    ))
                },
                Err(_) =>
                {
                    match operand_splits.next()
                    {
                        Some(second_str) => Ok(
                            extract_or_err(0, second_str)?
                        ),
                        None => Err(LexerErr::Syntax(
                            format!(r#"Relative address expected an identifier: "{}""#, operand)
                        ))
                    }
                }
            },
            None => Err(LexerErr::Syntax(
                format!(r#"Invalid syntax provided for relative address: "{}""#, operand)
            ))
        }
    }

    fn get_relocation_function(operand: &str) -> Result<Operand, LexerErr>
    { // Trim '%' and ending ')' then split between function and symbol. Ex: %hi(Symbol)
        match operand.trim_start_matches('%').trim_end_matches(')').split_once('(')
        {
            Some((func_str, symbol_str)) =>
            {
                if !IDENTIFIER_REGEX.is_match(symbol_str)
                {
                    return Err(LexerErr::Syntax(
                        format!(r#"Relocation function expected an identifier: "{}""#, symbol_str)
                    ))
                }
                Ok(Operand::RelocationFn(
                    func_str.into(), RValue::Identifier(symbol_str.into())
                ))
            },
            None => Err(LexerErr::Syntax(
                format!(r#"Incomplete relocation function: "{}""#, operand)
            ))
        }
    }

    fn get_register(register: &str) -> Result<RValue, LexerErr>
    {
        if CONVENTIONAL_TO_ABI.contains_key(register)
        { // Conventional register names to ABI names.
            Ok(Self::get_register(
                CONVENTIONAL_TO_ABI[register]
            )?)
        }
        else
        { // Match ABI register naming.
            match register.chars().next()
            { // Supported ABI registers either start with 'x' or 'f' (integral or floating point).
                Some(prefix @ 'x') | Some(prefix @ 'f') =>
                {
                    register[1..].parse::<u32>()
                        .map(|val| RValue::Register(prefix, val))
                        .map_err(|_| LexerErr::Parsing(
                            format!(r#"Unable to parse ABI register index: "{}""#, register)
                        ))
                }, // Register prefix is unsupported.
                _ => Err(LexerErr::Parsing(
                    format!(r#"Unexpected ABI register prefix: "{}""#, register)
                ))
            }
        }
    }
}