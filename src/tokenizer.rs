use std::str::FromStr;
use std::fmt::Debug;
use crate::util::AddressingMode;

#[derive(Debug, Clone, PartialEq)]
pub enum Token<T: Copy + Debug>
{
    /*      Section-name Section-Attributes Followed-Tokens */
    Section(String, Option<Vec<String>>, Vec<Token<T>>),
    /*      Directive-name Directive-Attributes */
    Directive(String, Vec<String>),
    /*      Label-name Followed-Tokens */
    Label(String, Vec<Token<T>>),
    /*      Mnemonic Followed-Tokens */
    Mnemonic(String, Vec<Token<T>>),
    /*      Symbol-name */
    Symbol(String),
    /*      Offset-Label Offset */
    Offset
    {
        base: String,
        offset: Option<AddressingMode<T>>
    },
    /*      Register-name */
    Register(String),
    /*      Immediate-value<T> */
    Immediate(T)
}

pub struct Tokenizer<T: Copy + Debug>
{
    pub tokens: Vec<Token<T>>
}

impl<T: Copy + Debug> Tokenizer<T>
{
    pub fn new_from_string(string: &str) -> Result<Self, String>
        where T: FromStr, <T as FromStr>::Err: std::fmt::Debug
    {
        Ok(Tokenizer {
            tokens: Self::tokenize(string)?
        })
    }

    pub fn tokenize(string: &str) -> Result<Vec<Token<T>>, String>
        where T: FromStr, <T as FromStr>::Err: std::fmt::Debug
    {
        let mut tokens = Vec::<Token<T>>::new();
        let mut lines_iter = string.lines().peekable();

        while let Some(line) = lines_iter.next()
        {
            // Trim comment from line to avoid unnecessary tokenization.
            let trimmed = line.split("#").next().unwrap_or("").trim();

            if trimmed.is_empty()
            {   // Empty comment line, skip to next line.
                continue;
            }

            // Directives and sections.
            if trimmed.starts_with(".")
            {
                // Split the line at the first space.
                let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();

                // parts[0] now contains the directive/section name.
                let name = parts[0].to_string();

                // If there's more after the directive name, split those by whitespace to get individual labels/symbols.
                let symbols = if parts.len() > 1
                {
                    parts[1].split_whitespace().map(|s| s.to_string()).collect::<Vec<_>>()
                }
                else
                {
                    Vec::new()
                };

                if !symbols.is_empty() && parts[0] != ".section"
                {   // Push the finished directive followed by symbols.
                    tokens.push(Token::Directive(name, symbols));
                }
                else
                {   // Tokenize the next lines under a section.
                    // This is a section. Gather all lines under this section until the next one.
                    let mut section_content = Vec::new();

                    while let Some(&next_line) = lines_iter.peek()
                    {
                        // Trim comment from line to avoid unnecessary tokenization.
                        let next_trimmed = next_line.split("#").next().unwrap_or("").trim();

                        if trimmed.is_empty()
                        {   // Empty comment line, skip to next line.
                            continue;
                        }

                        let next_parts: Vec<&str> = next_trimmed.splitn(2, ' ').collect();

                        // New section.
                        if next_trimmed.starts_with('.') && next_parts.len() == 1
                        || next_trimmed.starts_with(".section")
                        {
                            break;
                        }

                        section_content.push(lines_iter.next().unwrap().to_owned());
                    }

                    let section_tokens = Self::tokenize(&section_content.join("\n"))?;

                    // .section allows for custom attributes.
                    tokens.push(Token::Section(name,
                        if parts[0] == ".section" { Some(symbols) } else { None },
                        section_tokens
                    ));
                }
            } // Labels.
            else if trimmed.contains(":")
            {
                // Split the line at ":".
                let parts: Vec<&str> = trimmed.splitn(2, ':').collect();

                // parts[0] now contains the label name.
                let name = parts[0].to_string();

                let mut label_content = Vec::new();
                label_content.push(parts[1].to_owned());

                while let Some(&next_line) = lines_iter.peek()
                {
                    // Trim comment from line to avoid unnecessary tokenization.
                    let next_trimmed = next_line.split('#').next().unwrap_or("").trim();

                    if trimmed.is_empty()
                    {   // Empty comment line, skip to next line.
                        continue;
                    }

                    let next_parts: Vec<&str> = next_trimmed.splitn(2, ' ').collect();

                    // New section.
                    if next_trimmed.starts_with('.') && next_parts.len() == 1
                    || next_trimmed.starts_with(".section") || next_trimmed.contains(":")
                    {
                        break;
                    }

                    label_content.push(lines_iter.next().unwrap().to_owned());
                };

                let label_tokens = Self::tokenize(&label_content.join("\n"))?;

                tokens.push(Token::Label(name, label_tokens))
            } // Registers.
            // Register offset, pattern of Off(Reg). => Reg + Off
            else if trimmed.contains('(') && trimmed.contains(')') && trimmed.split(' ').count() == 1
            {
                // Split offset and relative label/register at first '('.
                let parts: Vec<&str> = trimmed.splitn(2, '(').collect();      

                // parse offset from string.          
                let offset = 
                    if parts.len() > 1
                    {
                        match parts[0].parse::<T>() 
                        {
                        Ok(value) => {
                            Some(AddressingMode::RegisterOffset(value)),
                        }
                        Err(_) => None,
                        }
                    }
                    else
                    {
                        None
                    };
            
                tokens.push(Token::Offset { base: parts[1].replace(")", "").to_string(), offset });
            }
            else if trimmed.starts_with('x') &&
                // Ignore any offsets that may also start with a register.
                !trimmed.contains('+') && !trimmed.contains('-') {
                let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();

                // Instruction mnemonic.
                let register_name = parts[0].replace(',', "").to_string();

                tokens.push(Token::Register(register_name));

                // Any remaining registers/immediate values on the line.
                if parts.len() > 1
                {
                    tokens.append(&mut Self::tokenize(parts[1]).unwrap_or_default())
                }
            }
            else if trimmed.chars().all(|c| c.is_digit(10) || c == '-' || c == '+')
            {
                let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();

                match trimmed.replace(',', "").parse::<T>() {
                    Ok(value) => tokens.push(Token::Immediate(value)),
                    Err(_) => return Err(format!("Failed to parse immediate value: {}", trimmed)),
                }

                // Any remaining registers/immediate values on the line.
                if parts.len() > 1
                {
                    tokens.append(&mut Self::tokenize(parts[1]).unwrap_or_default())
                }
            }  // Tokenization of an instruction.
            else {
                let parts: Vec<&str> = trimmed.splitn(2, ' ').collect();

                // Instruction mnemonic.
                let mnemonic_name = parts[0].to_string();

                // Instruction registers and immediate values operated on, if any.
                let reg_imm = if parts.len() > 1
                {
                    Self::tokenize(parts[1])?
                }
                else
                {
                    Vec::new()
                };

                tokens.push(Token::Mnemonic(mnemonic_name, reg_imm));
            }
        }
        /*
        for line in string.lines()
        {
            for (index, word) in line.split_whitespace().enumerate()
            {
                if index == 0
                {
                    // For segments.
                    if word.starts_with('.')
                    {
                        tokens.push(Token::Segment(word.to_string()));
                    }
                    // For labels.
                    else if word.ends_with(':')
                    {
                        tokens.push(Token::Label(word.trim_end_matches(':').to_string()));
                    }
                    else
                    { // For mnemonics.
                        tokens.push(Token::Mnemonic(word.to_string()));
                    }
                }
                // Registers always start with an x*.
                else if word.starts_with("x")
                {
                    tokens.push(Token::Register(word.trim_end_matches(",").to_string()))
                }
                // Match any immediate value.
                else if word.chars().all(|c| c.is_digit(10) || c == '-' || c == '+')
                {
                    match word.parse::<T>() {
                        Ok(value) => tokens.push(Token::Immediate(value)),
                        Err(_) => return Err(format!("Failed to parse immediate value: {}", word)),
                    }
                }
                // If it contains both '(' and ')', it's a base + offset addressing.
                else if word.contains('(') && word.contains(')')
                {
                    let parts: Vec<&str> = word.split(['(', ')'].as_ref()).collect();

                    // Ensure there are at least two parts (immediate and base register).
                    if parts.len() != 3 || parts[2] != "" {
                        return Err(format!("Invalid offset format: {}", word));
                    }

                    // Try to parse the immediate part.
                    let offset_val: T = match parts[0].parse()
                    {
                        Ok(val) => val,
                        Err(_) => return Err(format!("Failed to parse offset immediate value: {}", parts[0]))
                    };

                    // The second part is the base register.
                    let base: String = parts[1].to_string();
                    let offset = AddressingMode::RegisterOffset(offset_val);

                    tokens.push(Token::Offset{ base, offset });
                }
                // If it just contains '(', it's only a base addressing or it might be an error.
                else if word.contains('(')
                {
                    let parts: Vec<&str> = word.split(')').collect();
                    if parts.len() != 2 || parts[1] != "" {
                        return Err(format!("Invalid base format: {}", word));
                    }

                    let base: String = parts[0].to_string();
                    let offset = AddressingMode::None; // No offset.

                    tokens.push(Token::Offset{ base, offset });
                }
                // If it doesn't contain any parentheses, it's likely a label or a standalone immediate.
                else if word.chars().all(|c| c.is_alphanumeric() || c == '_')
                {
                    tokens.push(Token::Offset{ base: word.to_string(), offset: AddressingMode::None });
                }
                else if word.starts_with('.')
                {
                    let directive = word.to_string();
                    let mut data_vals = Vec::new();

                    // Capture the next words as the data for this directive until the End or another directive.
                    while let Some(next_word) = iterator.next()
                    {
                        match next_word.parse::<T>()
                        {
                            Ok(val) => data_vals.push(val),
                            Err(_) => {
                                iterator.put_back(next_word); // Assuming you're using an iterator with peek/put_back capability.
                                break;
                            },
                        }
                    }

                    tokens.push(Token::Data(directive, data_vals));
                }
                else
                {
                    match word.parse::<T>()
                    {
                        Ok(immediate_val) => tokens.push(Token::Immediate(immediate_val)),
                        Err(_) => return Err(format!("Unknown token: {}", word)),
                    }
                }
            }
            // Mark the end of the instruction.
            tokens.push(Token::End)
        }*/

        Ok(tokens)
    }
}