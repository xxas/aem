use lazy_static::lazy_static;
use bitflags::bitflags;
use std::str::FromStr;
use std::fmt::Debug;
use regex::Regex;

lazy_static!
{ // Regex patterns for supported token types.
    static ref LABEL_REGEX: Regex           = Regex::new(r"^\s*[a-zA-Z_][a-zA-Z_0-9]*:\s*").unwrap();
    static ref SECTION_REGEX: Regex         = Regex::new(r"^\s*\.[a-zA-Z_][a-zA-Z_0-9]*(\s+.+)?$").unwrap();
    static ref INSTRUCTION_REGEX: Regex     = Regex::new(r"^[a-zA-Z]+($|\s.+)").unwrap();
    static ref REGISTER_REGEX: Regex        = Regex::new(r"^\s*[xf]\d+\s*$").unwrap();
    static ref OFFSET_REGEX: Regex          = Regex::new(r"(-?\d+)\(([a-zA-Z_][a-zA-Z0-9_]*)\)").unwrap();
    static ref DESTINATION_REGEX: Regex     = Regex::new(r"([a-zA-Z_][a-zA-Z0-9_]*)").unwrap();
    static ref DATA_REGEX: Regex            = Regex::new(r#""[^"]*"|\(0x[0-9a-fA-F]+(?:, ?0x[0-9a-fA-F]+)*\)|\([0-9]+(?:, ?[0-9]+)*\)"#).unwrap();
}

bitflags!
{ // Section attribute flags.
    pub struct SectionFlags: u32
    {
        const ALLOCATE = 0b0000_0001;
        const WRITE = 0b0000_0010;
        const EXECUTE = 0b0000_0100;
        const MERGE = 0b0000_1000;
        const STRING = 0b0001_0000;
        const GROUP = 0b0010_0000;
        const TLS = 0b0100_0000;
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DataType
{
    Byte = 0,   // byte values.
    Half = 1,   // halfword (2 bytes) values.
    Word = 2,   // word (4 bytes) values.
    Dword = 3,  // double word (8 bytes) values.
    Float = 4,  // single-precision floating point values.
    Double = 5, // double-precision floating point values.
    ASCII = 6,  // string without null termination.
    ASCIZ = 7   // .asciz/.string: null-terminated string.
}

#[derive(Debug, Clone, PartialEq)]
pub enum RelativeSymbol
{ // Symbol that an offset is relative to.
    Label(String),
    Register(char, u8)
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<T: Copy + Debug>
{
    Section(String, SectionFlags, Vec<Token<T>>),
    Label(String, Vec<Token<T>>),
    Data(String, DataType, T),
    Instruction(String, Vec<Token<T>>),
    Offset
    {
        base: RelativeSymbol,
        offset: T
    },
    Destination(String),
    Register(char, u8),
    Immediate(T),
    Debug(String)
}

#[derive(Debug)]
enum TokenizeError
{
    InvalidSection(String),
    InvalidSectionFlag(String),
    InvalidLabel(String),
    InvalidInstruction(String),
    InvalidRegister(String),
    InvalidImmediate(String),
    InvalidOffset(String),
    Other(String)
}

pub trait ParseWithRadix
{
    fn from_str_radix(src: &str, radix: u32) -> Result<Self, std::num::ParseIntError>
    where Self: Sized;
}

impl ParseWithRadix for i32
{
    fn from_str_radix( src: &str, radix: u32) -> Result<Self, std::num::ParseIntError>
    {
        i32::from_str_radix(src, radix)
    }
}

impl ParseWithRadix for i64
{
    fn from_str_radix( src: &str, radix: u32) -> Result<Self, std::num::ParseIntError>
    {
        i64::from_str_radix(src, radix)
    }
}

enum ParseValueError<T: FromStr>
{
    IntError(std::num::ParseIntError),
    StrError(<T as FromStr>::Err)
}

fn parse_value<T: ParseWithRadix + std::str::FromStr>(s: &str) -> Result<T, ParseValueError<T>> {
    if s.starts_with("0x")
    {
        T::from_str_radix(&s[2..], 16).map_err(ParseValueError::IntError)
    }
    else
    {
        s.parse::<T>().map_err(ParseValueError::StrError)
    }
}


pub struct Tokenizer<T: FromStr + Copy + Debug + Default>
{
    pub tokens: Vec<Token<T>>
}

impl<T: ParseWithRadix + FromStr + Copy + Debug + Default> Tokenizer<T>
{
    pub fn new_from_string(string: &str) -> Result<Self, String>
    {
        let cleaned_lines: Vec<&str> = string
            .lines()
            .filter_map(|line| line.trim().split('#').next())
            .filter(|&line| !line.is_empty())
            .collect();

        Self::process_block(cleaned_lines)
            .map(|tokens| Tokenizer { tokens })
            .map_err(|e| format!("{:?}", e))
    }

    fn get_label(line: &str) -> Result<(&str, &str), TokenizeError>
    { // Split the label name and the following content at ':'.
        let mut label_parts = line.splitn(2, ':');

        if let (Some(label_name), Some(label_content)) = (label_parts.next(), label_parts.next())
            {
                let label_name = label_name.trim();

                if label_name.is_empty()
                { // Labels are required to have a name to produce references.
                    return Err(TokenizeError::InvalidLabel("Invalid syntax: empty label name.".to_string()))
                }

                return Ok((label_name, label_content.trim()))
        };

        Err(TokenizeError::InvalidLabel(format!("Unable to parse label from line: \"{}\"", line)))
    }

    fn get_section(line: &str) -> Result<(&str, SectionFlags), TokenizeError>
    { // Detect the start of a section
        let mut parts = line.split_whitespace();

        if let Some(directive) = parts.next()
        { // Match the directive to deduce the flags of the section.
            let mut section_flags = SectionFlags::empty();

            match directive
            { // Custom section directive, followed by name and attributes.
                ".section" =>
                {
                    let section_name = parts.next().unwrap_or("").trim_start_matches('.');
                    let section_params = parts.next().unwrap_or("").to_lowercase();

                    for c in section_params.chars()
                    { // bitwise or assign each matched character.
                        match c
                        {
                            'a' => section_flags |= SectionFlags::ALLOCATE,
                            'w' => section_flags |= SectionFlags::WRITE,
                            'x' => section_flags |= SectionFlags::EXECUTE,
                            'm' => section_flags |= SectionFlags::MERGE,
                            's' => section_flags |= SectionFlags::STRING,
                            'g' => section_flags |= SectionFlags::GROUP,
                            't' => section_flags |= SectionFlags::TLS,
                            _   =>
                            { // Failed while parsing a section flag that is unsupported.
                                return Err(TokenizeError::InvalidSectionFlag(format!("Unrecognized section flag identifier: \"{}\"", c)))
                            }
                        }
                    }
                    return Ok((section_name, section_flags))
                }, // Handle sections with pre-defined attributes.
                ".text" =>
                {
                    section_flags |= SectionFlags::EXECUTE;
                    return Ok((directive.trim_start_matches('.'), section_flags))
                },
                ".data" =>
                {
                    section_flags |= SectionFlags::ALLOCATE | SectionFlags::WRITE;
                    return Ok((directive.trim_start_matches('.'), section_flags))
                },
                ".bss" =>
                {
                    section_flags |= SectionFlags::ALLOCATE;
                    return Ok((directive.trim_start_matches('.'), section_flags))
                },
                _ =>
                { // Unmatched directive, unable to deduce the section flags.
                    return Ok((directive.trim_start_matches('.'), section_flags))
                }
            }
        }

        Err(TokenizeError::InvalidSection(format!("Unable to parse section from line: \"{}\"", line)))
    }

    fn get_register(word: &str) -> Result<Token<T>, TokenizeError>
    { // todo: add support for names such as 'zero', 'ra', 'sp', 'gp', 'tp', 't*', 'a*', 's*'.
        match word.chars().next()
        { // Registers either start with 'x' or 'f'.
            Some(prefix @ 'x') | Some(prefix @ 'f') =>
            {
                match &word[1..].parse::<u8>()
                { // Parse the index value.
                    Ok(val) => return Ok(Token::<T>::Register(prefix, *val)),
                    Err(_) => return Err(TokenizeError::InvalidRegister(format!("Failed to parse register: \"{}\"", word))),
                };
            }, // A register is not present.
            _ => return Err(TokenizeError::InvalidRegister(format!("Failed to parse register: \"{}\"", word))),
        }
    }

    fn get_offset(word: &str) -> Result<Token<T>, TokenizeError>
    { // split the offset value and symbol from each other.
        let offset_symbol_split: Vec<&str> = word.trim_end_matches(')').splitn(2, '(').collect();

        if let Some(symbol) = offset_symbol_split.last()
        {

            return Ok(Token::Offset
            {
                base: if REGISTER_REGEX.is_match(symbol)
                {
                    match Self::get_register(symbol)
                    {
                        Ok(Token::Register(char_val, num_val)) => RelativeSymbol::Register(char_val, num_val),
                        _ => return Err(TokenizeError::InvalidOffset("Failed to parse an offset value.".to_string())),
                    }
                }
                else
                {
                    RelativeSymbol::Label(symbol.to_string())
                },
                offset: parse_value::<T>(offset_symbol_split.first().unwrap_or(&"")).unwrap_or_default(),
            });
        }

        Ok(Token::Debug(word.to_string()))
    }

    fn process_instruction(line: &str) -> Result<Token<T>, TokenizeError>
    {
        // Mnemonic and operands split.
        let mnemonic_split: Vec<&str> = line
            .trim().splitn(2, ' ')
            .collect();

        if let Some(mnemonic) = mnemonic_split.first()
        { // todo: differeniating _, f_.s, f_.d instrutions.
            let mut operands = Vec::new();

            for operand in mnemonic_split[1].split(',').map(|s| s.trim())
            {
                if REGISTER_REGEX.is_match(operand)
                {
                    operands.push(Self::get_register(operand)?)
                }
                else if OFFSET_REGEX.is_match(operand)
                {
                    operands.push(Self::get_offset(operand)?)
                }
                // Immediate operands, hexadecimal and decimal values.
                else if operand.chars().all(|c| c.is_ascii_hexdigit() || c == 'x') {
                    match parse_value::<T>(operand)
                    { // Parse the index value.
                        Ok(val) => operands.push(Token::Immediate(val)),
                        Err(_) => return Err(TokenizeError::InvalidImmediate(format!("Failed to parse an immediate operand: \"{}\"", operand)))
                    }
                } // Regex is potentially over-kill but captures syntax perfectly.
                // alphabetic or _ first character followed by alphanumeric or _.
                else if DESTINATION_REGEX.is_match(operand)
                {
                    operands.push(Token::Destination(operand.trim().to_string()))
                }
                else
                {
                    return Err(TokenizeError::InvalidInstruction(format!("Unable to parse an instruction operand: \"{}\"", operand)))
                }
            }

            return Ok(Token::Instruction(mnemonic.to_string(), operands))
        }
        Err(TokenizeError::InvalidInstruction(format!("Unable to parse instruction from line: \"{}\"", line)))
    }

    fn process_constant_data(line: &str) -> Result<Token<T>, TokenizeError>
    {
        Ok(Token::Debug(line.to_string()))
    }

    fn process_line(line: &str) -> Result<Token<T>, TokenizeError>
    {
        if INSTRUCTION_REGEX.is_match(line)
        { // Process as an instruction.
            return Ok(Self::process_instruction(line)?)
        }
        else if DATA_REGEX.is_match(line)
        { // Process as constant data.
            return Ok(Self::process_constant_data(line)?)
        }

        // Failed to process the contents of a line.
        Err(TokenizeError::Other(format!("Unable to parse from line: \"{}\"", line)))
    }

    fn process_block(block: Vec<&str>) -> Result<Vec<Token<T>>, TokenizeError>
    {
        let mut tokens = Vec::new();
        let mut line_iter = block.iter().peekable();

        // Closure to process lines until a specific condition is met.
        let process_lines_until =
            |line_iter: &mut std::iter::Peekable<std::slice::Iter<&str>>, condition: &dyn Fn(&str) -> bool| -> Result<Vec<Token<T>>, TokenizeError>
            {
                let mut current_tokens = Vec::new();

                while let Some(&next_line) = line_iter.peek() {
                    if condition(next_line)
                    { // e.g. is_section, is_label, etc.
                        break;
                    }

                    // Tokenize data or text.
                    current_tokens.push(Self::process_line(next_line)?);

                    // Consume the line.
                    line_iter.next();
                }
                Ok(current_tokens)
            };

        while let Some(&line) = line_iter.peek()
        {
            if SECTION_REGEX.is_match(line)
            { // Process following lines and nest them within the section.
                let (section_name, flags) = Self::get_section(line)?;
                let mut section_tokens = Vec::new();

                 // Consume the section line.
                line_iter.next();

                while let Some(&inner_line) = line_iter.peek() {
                    if SECTION_REGEX.is_match(inner_line)
                    { // End of the current section.
                        break;
                    }
                    else if LABEL_REGEX.is_match(inner_line)
                    { // Label nested in section.
                        let (label_name, label_content) = Self::get_label(inner_line)?;

                        // Consume the label line.
                        line_iter.next();

                        let mut label_tokens = process_lines_until(&mut line_iter,
                        // Ensure there isn't a label or a section within the label that's being processed.
                            &|l|
                                LABEL_REGEX.is_match(l) || !DATA_REGEX.is_match(l) && SECTION_REGEX.is_match(l)
                            )?;

                        if !label_content.is_empty()
                        { // Process the remaining label content on the same line.
                            label_tokens.insert( 0, Self::process_line(label_content)?);
                        }

                        // Tokenize following lines.
                        section_tokens.push(Token::Label(label_name.to_string(), label_tokens));
                    }
                    else
                    { // Process standalone lines within the section.
                        section_tokens.push(Self::process_line(inner_line)?);
                        line_iter.next();
                    }
                }
                tokens.push(Token::Section(section_name.to_string(), flags, section_tokens));
            }
            else
            { // Consume any other lines that aren't sections.
                line_iter.next();
            }
        }

        Ok(tokens)
    }
}