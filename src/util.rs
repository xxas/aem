use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq)]
pub enum AddressingMode<T: Copy + Debug>
{
    PCRelative(T),     // via the auipc, jal and br* instructions.
    RegisterOffset(T), // via the jalr, addi and all memory instructions.
    Absolute(T)        // via the lui instruction.
}