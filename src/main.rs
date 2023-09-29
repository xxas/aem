mod tokenizer;
use tokenizer::*;

#[cfg(test)]
mod tests {
    mod tokenizer;
}

fn main() {
    match Tokenizer::<i32>::new_from_string(
    r#"
    # vvv try
    .rodata # << Yoo!!
        data_label:
            .asciz "Hello, World!"
            .word 100 , 0x100,235
    .bss"#) 
    {
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