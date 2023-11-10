use aem::{ 
    mem::*, 
    lexer::*, lex
};

// Produces various instructions and operand types.
#[test]
fn parse_instructions() 
{
    const CODE_STR: &'static str = 
    r#"
        # x5 = x6 + 255
        addi x5, x6, 0xff

        lw   a2, -8()        # Produces an expected error.
        lw   a2, -8(sp)      # Load word from memory address pointed by (sp + 8) into a2.
        beq  a0, zero, label # Branch to 'label' if a0 is equal to zero.

        auipc t0, %hi(function_addr)        # Load the upper 20 bits of function_addr into t0.
        jalr  ra, t0, %lo(function_addr)    # Jump to the function using the lower 12 bits of function_addr.

        ret
    "#;

    match lex!(CODE_STR)
    {
        Ok(tokens) =>
        {   // addi x5, x6, 0xff
            assert_eq!(tokens[0],
                Emittable::Instruction("addi".into(),
                    vec![
                        RValue::Register('x', 5).into(),
                        RValue::Register('x', 6).into(),
                        RValue::Immediate(255).into()
                    ]
                ).into()
            );

            // lw   a2, -8(sp)
            assert_eq!(tokens[1],
                Emittable::Instruction("lw".into(),
                    vec![
                        RValue::Register('x', 12).into(),
                        Operand::Address(RValue::Register('x', 2), RValue::Immediate(-8)).into()
                    ]
                ).into()
            );

            // beq  a0, zero, label
            assert_eq!(tokens[2],
                Emittable::Instruction("beq".into(),
                    vec![
                        RValue::Register('x', 10).into(),
                        RValue::Register('x', 0).into(),
                        RValue::Identifier("label".into()).into()
                    ]
                ).into()
            );

            //  auipc t0, %hi(function_addr)
            assert_eq!(tokens[3],
                Emittable::Instruction("auipc".into(),
                    vec![
                        RValue::Register('x', 5).into(),
                        Operand::RelocationFn("%hi".into(), RValue::Identifier("function_addr".into()))
                    ]
                ).into()
            );

            // jalr  ra, t0, %lo(function_addr)
            assert_eq!(tokens[4],
                Emittable::Instruction("jalr".into(),
                    vec![
                        RValue::Register('x', 1).into(),
                        RValue::Register('x', 5).into(),
                        Operand::RelocationFn("%lo".into(), RValue::Identifier("function_addr".into()))
                    ]
                ).into()
            );

            // ret
            assert_eq!(tokens[5],
                Emittable::Instruction("ret".into(), vec![]).into()
            );
        },
        Err(lexer_err) =>
        { // Propagate error produced by 
            match lexer_err 
            {
                LexerErr::Syntax(ref message) => 
                { // Produced by incomplete relocation function at "lw   a2, -8()".
                    assert!(message.contains(r#"Unexpected instruction operand: -8()"#))
                },
                LexerErr::Parsing(ref message) => 
                {
                    panic!(r#"Error while parsing: "{}""#, message)
                }
            }
        }
    }
}

#[test]
fn parse_directives()
{
    const CODE_STR: &'static str = 
            r#" # alignment directives
                .p2align 0x4, 0xff, 0
                .align 4
            .rodata     # rodata section
                .macro macro zval
                    .byte 0x08, 0x7f, 126, 125, 0, -125, -126
                    .half 0x7fff, 0x7ffe, 32763, 32764,  -32763, -32764, zval
                    .word 0x7fffffff, 0x7fffffe, 2147483645, 2147483644, -2147483644, -2147483645, zval
                    .dword 0x7fffffffffffffff, 0x7ffffffffffffffe, 9223372036854775805, 9223372036854775804, -9223372036854775804, -9223372036854775805, zval
                    .string "hello world!"
                .endm
                macro 0x001be64a
            .bss
                .zero 24
            .text
                
            "#;

    match lex!(CODE_STR)
    {
        Ok(tokens) =>
        {   // .p2align 0x4, 0xff, 0
            assert_eq!(tokens[0],
                Token::Directive(Align::AsPow(4, 0xff, 0).into())
            );

            // .align 4
            assert_eq!(tokens[1],
                Token::Directive(Align::AsPow(4, 0, 0).into())
            );

            // .rodata
            assert_eq!(tokens[2],
                Token::Directive(Directive::Section("rodata".into(), SectionFlags::ALLOCATE, 2))
            );

            // .macro macro zval
            assert_eq!(tokens[3],
                Token::Directive(Directive::Macro("macro".into(), vec!["zval".into()]))
            );

            // .byte 0x08, 0x7f, 126, 125, 0, -125, -126
            assert_eq!(tokens[4],
                Token::Emittable(Emittable::Byte(vec![
                    RValue::Immediate(0x08).into(),
                    RValue::Immediate(0x7f).into(),
                    RValue::Immediate(126).into(),
                    RValue::Immediate(125).into(),
                    RValue::Immediate(0).into(),
                    RValue::Immediate(-125).into(),
                    RValue::Immediate(-126).into()
                ]).into())
            );

            // .half 0x7fff, 0x7ffe, 32763, 32764,  -32763, -32764, zval
            assert_eq!(tokens[5],
                Token::Emittable(Emittable::Half(vec![
                    RValue::Immediate(0x7fff).into(),
                    RValue::Immediate(0x7ffe).into(),
                    RValue::Immediate(32763).into(),
                    RValue::Immediate(32764).into(),
                    RValue::Immediate(-32763).into(),
                    RValue::Immediate(-32764).into(),
                    RValue::Identifier("zval".into()).into()
                ]).into())
            );

            // .word 0x7fffffff, 0x7fffffe, 2147483645, 2147483644, -2147483644, -2147483645, zval
            assert_eq!(tokens[6],
                Token::Emittable(Emittable::Word(vec![
                    RValue::Immediate(0x7fffffff).into(),
                    RValue::Immediate(0x7fffffe).into(),
                    RValue::Immediate(2147483645).into(),
                    RValue::Immediate(2147483644).into(),
                    RValue::Immediate(-2147483644).into(),
                    RValue::Immediate(-2147483645).into(),
                    RValue::Identifier("zval".into()).into()
                ]).into())
            );

            // .dword 0x7fffffffffffffff, 0x7ffffffffffffffe, 9223372036854775805, 9223372036854775804, -9223372036854775804, -9223372036854775805, zval
            assert_eq!(tokens[7],
                Token::Emittable(Emittable::Dword(vec![
                    RValue::Immediate(0x7fffffffffffffff).into(),
                    RValue::Immediate(0x7ffffffffffffffe).into(),
                    RValue::Immediate(9223372036854775805).into(),
                    RValue::Immediate(9223372036854775804).into(),
                    RValue::Immediate(-9223372036854775804).into(),
                    RValue::Immediate(-9223372036854775805).into(),
                    RValue::Identifier("zval".into()).into()
                ]).into())
            );

            // .string "hello world!"
            assert_eq!(tokens[8],
                Token::Emittable(Emittable::String("hello world!".into()).into())
            );

            // .endm
            assert_eq!(tokens[9],
                Token::Directive(Directive::Marker("endm".into()))
            );

            // macro 0x001be64a
            assert_eq!(tokens[10],
                Token::Emittable(Emittable::Instruction("macro".into(), vec![RValue::Immediate(0x001be64a).into()]).into())
            );

            // .bss
            assert_eq!(tokens[11],
                Token::Directive(Directive::Section("bss".into(), SectionFlags::ALLOCATE, 2))
            );
            
            // .zero 24
            assert_eq!(tokens[12],
                Token::Emittable(Emittable::Byte(vec![RValue::Immediate(0); 24]).into())
            );
        },
        Err(lex_err) =>
        {
            panic!("failed {:?}", lex_err)
        }
    }
}