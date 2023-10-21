use xemu::lexer::*;

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

    match Lexer::new(CODE_STR)
    {
        Ok(lexer) =>
        {
            // addi x5, x6, 0xff
            assert_eq!(lexer.tokens[0],
                Emittable::Instruction("addi".into(),
                    vec![
                        RValue::Register('x', 5).into(),
                        RValue::Register('x', 6).into(),
                        RValue::Immediate(255).into()
                    ]
                ).into()
            );

            // lw   a2, -8(sp)
            assert_eq!(lexer.tokens[1],
                Emittable::Instruction("lw".into(),
                    vec![
                        RValue::Register('x', 12).into(),
                        Operand::Address(RValue::Register('x', 2), RValue::Immediate(-8)).into()
                    ]
                ).into()
            );

            // beq  a0, zero, label
            assert_eq!(lexer.tokens[2],
                Emittable::Instruction("beq".into(),
                    vec![
                        RValue::Register('x', 10).into(),
                        RValue::Register('x', 0).into(),
                        RValue::Identifier("label".into()).into()
                    ]
                ).into()
            );

            //  auipc t0, %hi(function_addr)
            assert_eq!(lexer.tokens[3],
                Emittable::Instruction("auipc".into(),
                    vec![
                        RValue::Register('x', 5).into(),
                        Operand::RelocationFn("%hi".into(), RValue::Identifier("function_addr".into()))
                    ]
                ).into()
            );

            // jalr  ra, t0, %lo(function_addr)
            assert_eq!(lexer.tokens[4],
                Emittable::Instruction("jalr".into(),
                    vec![
                        RValue::Register('x', 1).into(),
                        RValue::Register('x', 5).into(),
                        Operand::RelocationFn("%lo".into(), RValue::Identifier("function_addr".into()))
                    ]
                ).into()
            );

            // ret
            assert_eq!(lexer.tokens[3],
                Emittable::Instruction("ret".into(), vec![]).into()
            );
        },
        Err(lexer_err) =>
        { // Propagate error produced by lexer.
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