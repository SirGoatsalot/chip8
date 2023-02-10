use chip8_core::Emu;

fn main() {
    println!("--- TESTING CHIP8_CORE ---");
    let mut emu = Emu::new();
    let program_draw_420 = vec![
        0x6101, // store 1 in reg 1
        0x6201, // store 1 in reg 2
        0x6304, // Store 4 in reg 3
        0x6402, // Store 2 in reg 4
        0x6500, // Store 0 in reg 5
        0xF329, // retrieve font of 4
        0xD125, // draw sprite in i
        0x6106, // update x coordinate
        0xF429, // update sprite
        0xD125, // draw
        0x610B, // update x coordinate
        0xF529, // update sprite
        0xD125, // draw
    ];

    let program_test_bcd = vec![
        0x61FF,
        0x6210,
        0x63AC,
        0xA690,
        0xF133,
        0xF233,
        0xF333
    ];

    let program = vec![
        // store values in v_regs
        0x60FF,
        0x6110,
        0x62AC,
        0xA300,
        0xF355
    ];
    for op in program {
        emu.test_execute(op);
    }
}
