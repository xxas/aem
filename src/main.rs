use aem::{ 
    asm::*, assemble
};

fn main() {
    match assemble!(
        r#"
    .macro mult_nop_add x, y, z
        mul x, y, z
        nop
        add x, x, z
        addi x, x, 0xff
        neg x, x
        nop
    .endm
        mult_nop_add x0, x1, x2  
        # result:
        #   mul x0, x1, x2
        #   nop
        #   add x0, x0, x2
        #   addi x0, x0, 0xff
        #   neg x0, x0
        #   nop
        nop"#)
    {
        Ok(object) => 
        {   // print hex view of generated assembly.
            for (i, chunk) in object.binary.chunks(4).enumerate() 
            {
                // Initialize bytes array with zeros.
                let mut bytes = [0u8; 4];

                // Copy up to 4 bytes from the chunk into the bytes array.
                bytes[..chunk.len()].copy_from_slice(chunk);

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