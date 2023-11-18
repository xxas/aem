use aem::{ 
    asm::*, assemble
};

// Expands various types of pseudo-code and generates binary.
#[test]
fn pseudocode_expansion() 
{
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
        { 
            let expected_values = [
                0x02208033,     // mul x0, x1, x2
                0x00000013,     // addi x0, x0, 0
                0x00200033,     // add x0, x0, x2
                0x0ff00013,     // addi x0, x0, 255
                0x40000033,     // sub x0, x0, x0
                0x00000013,     // addi x0, x0, 0
                0x00000013      // addi x0, x0, 0
            ];
    
            for (i, chunk) in object.binary.chunks(4).enumerate() 
            {
                let mut bytes = [0u8; 4];
                bytes[..chunk.len()].copy_from_slice(chunk);
                let value = u32::from_le_bytes(bytes);
    
                assert_eq!(value, expected_values[i], "Mismatch at address 0x{:04x}", i * 4);
            }
        },
        Err(asm_err) => 
        {
            panic!("failed {:?}", asm_err)
        }
    }
}