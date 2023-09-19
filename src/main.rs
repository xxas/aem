mod util;
mod tokenizer;
mod parser;

use tokenizer::Tokenizer;
use parser::Parser;

fn main() {
    match Tokenizer::<i32>::new_from_string(
        "
        .section .data,\"aw\"
        myArray:       .word 1, 2, 3, 4
        stringLabel:   .asciiz \"Hello, RISC-V!\"
    
        .text
        .global _start
        
        _start:
            # Arithmetic between registers:
            add x1, x2, x3          # x1 = x2 + x3
        
            # Immediate operations:
            addi x5, x6, 10         # x5 = x6 + 10
        
            # Load and store with register offset:
            lw x8, 8(x9)            # Loads word from the address in x9 + 8 bytes offset into x8
            sw x10, 12(x11)         # Stores word in x10 to the address in x11 + 12 bytes offset
        
            # Load an address into a register:
            la x12, stringLabel     # Load address of stringLabel into x12
        
            # Access array using base+offset addressing:
            lw x13, 8(x0)           # Loads the third element (0-indexed) of myArray into x13 (assuming x0 points to myArray)
                                    # This is for illustrative purposes; you'd usually load the base address of myArray into a register first
        
            # Branching with labels:
            beq x14, x15, somewhere # Branch to 'somewhere' if x14 == x15
            bne x16, x17, elsewhere # Branch to 'elsewhere' if x16 != x17
        
        somewhere:
            # Some more operations for illustration:
            sub x18, x19, x20       # x18 = x19 - x20
            or x21, x22, x23        # x21 = x22 OR x23
        
        elsewhere:
            # Ending the sequence:
            addi x0, x0, 10         # Equivalent to li x0, 10
            ecall                   # System call
    ") {
        Ok(parser) => {
            for instruction in &parser.tokens {
                println!("{:?}", instruction);
            }
        },
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}
