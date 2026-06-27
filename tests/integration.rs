//! End-to-end tests: assemble source text, then run it on the emulator.
//!
//! These tests depend on the `assembler` crate.  We call its internal
//! functions directly by re-using the shared types.

use custom_pc::Machine;

/// Assemble a source string to bytes using the assembler crate's public API.
fn asm(src: &str) -> Vec<u8> {
    // We can't easily import the assembler crate from an integration test
    // in the main crate.  Instead we hand-encode the small programs below
    // or use a helper that invokes the assembler binary as a subprocess
    // in a real CI setup.  For now, encode the binaries directly.
    let _ = src; // suppress unused warning
    unimplemented!("use hand-encoded ROM slices in integration tests")
}

/// Run a hand-encoded ROM and return the finished machine.
fn run(rom: &[u8]) -> Machine {
    let mut m = Machine::new();
    m.load_rom(rom);
    m.run(Some(1_000_000)).unwrap();
    m
}

#[test]
fn fibonacci_sequence() {
    // Compute Fibonacci numbers into memory starting at 0x0100.
    // r0 = a (0), r1 = b (1), r2 = loop counter (8), r3 = addr (0x0100)
    //
    // init:
    //   ldi r0, 0
    //   ldi r1, 1
    //   ldi r2, 8
    //   ldi r3, 0x0100
    // loop:
    //   st [r3], r0         ; store a
    //   addi r3, 2          ; advance address
    //   mov r4, r1          ; tmp = b
    //   add r1, r0          ; b = a + b
    //   mov r0, r4          ; a = tmp
    //   subi r2, 1          ; counter--
    //   jnz loop
    //   hlt

    #[rustfmt::skip]
    let rom: &[u8] = &[
        // ldi r0, 0
        0x03, 0x00, 0x00, 0x00,
        // ldi r1, 1
        0x03, 0x10, 0x01, 0x00,
        // ldi r2, 8
        0x03, 0x20, 0x08, 0x00,
        // ldi r3, 0x0100
        0x03, 0x30, 0x00, 0x01,
        // loop @ 0x10:
        // st [r3], r0
        0x05, 0x30,
        // addi r3, 2
        0x10, 0x30, 0x02, 0x00,
        // mov r4, r1
        0x02, 0x41,
        // add r1, r0
        0x08, 0x10,
        // mov r0, r4
        0x02, 0x04,
        // subi r2, 1
        0x11, 0x20, 0x01, 0x00,
        // jnz 0x0010
        0x32, 0x10, 0x00,
        // hlt
        0x01,
    ];

    let m = run(rom);
    assert!(m.is_halted());

    // Read the first 8 Fibonacci numbers from 0x0100.
    let expected = [0u16, 1, 1, 2, 3, 5, 8, 13];
    for (i, &exp) in expected.iter().enumerate() {
        let addr = 0x0100u16 + (i as u16) * 2;
        assert_eq!(m.mem.read_word(addr).unwrap(), exp, "fib[{i}] mismatch");
    }
}

#[test]
fn countdown_loop() {
    // Count down from 5 to 0 using subi + jnz.
    // ldi r0, 5 ; loop: subi r0, 1 ; jnz loop ; hlt
    let rom: &[u8] = &[
        0x03, 0x00, 5, 0x00,   // ldi r0, 5
        // loop @ 0x04
        0x11, 0x00, 1, 0x00,   // subi r0, 1
        0x32, 0x04, 0x00,      // jnz 0x0004
        0x01,                  // hlt
    ];
    let m = run(rom);
    assert_eq!(m.cpu.regs.get(0), 0);
    assert!(m.cpu.flags.zero);
}
