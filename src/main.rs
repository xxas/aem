mod tokenizer;

use tokenizer::Tokenizer;

fn main() {
    match Tokenizer::<i32>::new_from_string(
    "
    .section .hello aw
        our_string: .asiiz \"Null terminating string\"
    .text
        .globl label_0
        label_0:
        addi x5, x6, 0xff
        label:
        addi x5, x6, 10
        sub x6, x18, x4
        jal x0, label_0
        jal x0, 10(x5)
        jal x0, 10(label_0)
        jalr x20, x21
        mul x7, x8, x9") {
        Ok(parser) => 
        {
            for token in &parser.tokens 
            {
                println!("{:?}", token);
            }
        },
        Err(e) => 
        {
            println!("{}", e);
        }
    }
}

/*
    Section("hello", ALLOCATE | WRITE, [
        Label("Text", [Debug(".asiiz \"Null terminating string\"")])])
    Section("text", EXECUTE, [
        Label("label_0", [
            Instruction("addi", [Register('x', 5), Register('x', 6), Immediate(255)])]), 
        Label("label", [
            Instruction("addi", [Register('x', 5), Register('x', 6), Immediate(10)]), 
            Instruction("sub", [Register('x', 6), Register('x', 18), Register('x', 4)]), 
            Instruction("jal", [Register('x', 0), Destination("label_0")]), 
            Instruction("jal", [Register('x', 0), Offset { base: Register('x', 5), offset: 10 }]), 
            Instruction("jal", [Register('x', 0), Offset { base: Label("label_0"), offset: 10 }]), 
            Instruction("jalr", [Register('x', 20), Register('x', 21)]), 
            Instruction("mul", [Register('x', 7), Register('x', 8), Register('x', 9)])
        ])
    ])
*/