use aem::{ lexer::*, lex };

fn main() {
    match lex!("
    .equ name,0x3f
    .equ   name ,   1005
    .macro test
    .macro wtest a,  b ,c,d
    addi rd, rs, 0")
    {
        Ok(tokens) => 
        {
           println!("{:?}", tokens)

        },
        Err(e) => 
        {
            println!("{:?}", e);
        }
    }
}