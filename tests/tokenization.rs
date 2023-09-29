use xsint::tokenizer::*;

#[test]
fn section_parsing() 
{
    let input = ".rodata\n.data\n.text\n.section .custom aw";
    let tokenizer = Tokenizer::<i32>::new_from_string(input);
    match tokenizer 
    {
        Ok(parser) => 
        {
            assert_eq!(parser.tokens[0], Token::Section("rodata".to_string(), SectionFlags::ALLOCATE, vec![]));
            assert_eq!(parser.tokens[1], Token::Section("data".to_string(), SectionFlags::ALLOCATE | SectionFlags::WRITE, vec![]));
            assert_eq!(parser.tokens[2], Token::Section("text".to_string(), SectionFlags::EXECUTE, vec![]));
            assert_eq!(parser.tokens[3], Token::Section("custom".to_string(), SectionFlags::ALLOCATE | SectionFlags::WRITE, vec![]));
        },
        Err(_) =>
        {
             panic!("Failed on tokenizer test 'section_parsing'")
        },
    }
}

#[test]
fn instruction_parsing() 
{
    let input = ".text\naddi x5, x6, 0xff";
    let tokenizer = Tokenizer::<i32>::new_from_string(input);
    match tokenizer {
        Ok(parser) => 
        {
            assert_eq!(parser.tokens[0], Token::Section("text".to_string(), SectionFlags::EXECUTE, vec![
                    Token::Instruction("addi".to_string(), vec![Token::Register('x', 5), Token::Register('x', 6), Token::Immediate(255)])
                ])
            );
        },
        Err(_) => panic!("Failed on tokenizer test 'instruction_parsing'!"),
    }
}

#[test]
fn data_parsing() {
    let input = "
    # Some data to parse...
    .rodata # <<< Read only data section.
        # vvv Data label.
        data_label:
            .asciz \"Hello, World!\"
            .word 100 , 0x100,235
    .bss";

    let tokenizer = Tokenizer::<i32>::new_from_string(input);

    match tokenizer 
    {
        Ok(parser) => 
        {
            assert_eq!(parser.tokens[0], Token::Section("rodata".to_string(), SectionFlags::ALLOCATE, vec![
                    Token::Label("data_label".to_string(), vec![
                        Token::Data(DataType::String("\"Hello, World!\"".to_string())),
                        Token::Data(DataType::Word(vec![100, 0x100, 235]))
                    ])
                ])
            );

            assert_eq!(parser.tokens[1], Token::Section("bss".to_string(), SectionFlags::ALLOCATE, vec![]));
        },
        Err(_) => panic!("Failed on tokenizer test 'data_parsing'!"),
    }
}