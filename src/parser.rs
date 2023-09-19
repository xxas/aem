use std::fmt::Debug;

use crate::util::AddressingMode;
use crate::tokenizer::Token;
use crate::tokenizer::Tokenizer;

// Contents of an instruction.
#[derive(Debug)]
pub struct Instruction<T: Copy + Debug>
{
    pub mnemonic: String,
    pub     dest: Option<u8>,
    pub     src0: Option<u8>,
    pub     src1: Option<u8>,
    pub      imm: Option<AddressingMode<T>>
}

// Contents of a label.
#[derive(Debug)]
pub enum LabelContents<T: Copy + Debug> 
{
    Function(Vec<Instruction<T>>),
    Constant(Vec<T>)
}

#[derive(Debug)]
pub struct Label<T: Copy + Debug> 
{
    content: LabelContents<T>,
    // Alignment, Public, etc. Directives.
    /* directives: something similar to std::bitset< DirectivesMaxSize > from C++ */
}

#[derive(Debug)]
pub struct Parser<T: Copy + Debug> 
{
    // Parsed labels, e.x. Constants or functions w/ directives.
    pub labels: Vec<Label<T>>
}

impl<T: Copy + Debug> Parser<T> 
{


    // Parse incoming tokens to functional/constant data labels with directives. 
    pub fn parse(tokens: Vec<Token<T>>) 
    {

    }
}