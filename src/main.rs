use aem::{ 
    asm::*
};

fn main() {
    match Assembler::new(
        r#"
        .macro mult_add x, y, z
        mul x, y, z
        add x, x, z
    .endm
        mult_add x0, x1, x2
        nop
        "#)
    {
        Ok(assembler) => 
        {
            // print hex view of generated assembly.
            for (i, chunk) in assembler.object.binary.chunks(4).enumerate() 
            {
                // Ensure that we have 4 bytes to form a complete u32, otherwise pad with zeros.
                let mut bytes = [0u8; 4];
                for (j, &byte) in chunk.iter().enumerate() 
                {
                    bytes[j] = byte;
                }
            
                // Convert bytes to u32; using little-endian format here.
                let value = u32::from_le_bytes(bytes);
            
                // Print the u32 value in hexadecimal format.
                println!("{:04x}: 0x{:08x}", i * 4, value);
            }
        },
        Err(e) => 
        {
            println!("{:?}", e);
        }
    }
}