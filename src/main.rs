use evu::lexer::*;

fn main() {
    match Lexer::new(
        r#"
        .global _boot

        .text
        .align 4
        _boot:
            j 1f
        1:
            addi x1 , x0,   1000
            addi x2 , x1,   2000
            addi x3 , x2,  -1000
            addi x4 , x3,  -2000
            addi x5 , x4,   1000
            jalr ra, SomeSymbol
            addi x5, x3, %hi(SomeSymbol)

            la x6, variable
            addi x6, x6, 4

        .section .data "aw"
            .p2align 4, 0xff, 0
            variable:
                .word 0xdeadbeef
                .zero 0xf
                .string "Hello world!""#)
    {
        Ok(assembler) => 
        {
           println!("{:?}", assembler.tokens)

        },
        Err(e) => 
        {
            println!("{:?}", e);
        }
    }
}