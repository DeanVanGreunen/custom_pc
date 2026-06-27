use custom_pc::Machine;

fn machine_with(rom: &[u8]) -> Machine {
    let mut m = Machine::new();
    m.load_rom(rom);
    m
}

fn run(rom: &[u8]) -> Machine {
    let mut m = machine_with(rom);
    m.run(Some(100_000)).unwrap();
    m
}

#[test]
fn nop_then_hlt() {
    // nop (0x00) then hlt (0x01)
    let m = run(&[0x00, 0x01]);
    assert!(m.is_halted());
    assert_eq!(m.cpu.cycles, 2);
}

#[test]
fn ldi_loads_immediate() {
    // ldi r3, 0xABCD → 0x03 0x30 0xCD 0xAB ; hlt
    let m = run(&[0x03, 0x30, 0xCD, 0xAB, 0x01]);
    assert_eq!(m.cpu.regs.get(3), 0xABCD);
}

#[test]
fn mov_copies_register() {
    // ldi r0, 42 ; mov r1, r0 ; hlt
    let m = run(&[
        0x03, 0x00, 42, 0x00,  // ldi r0, 42
        0x02, 0x10,             // mov r1, r0
        0x01,                   // hlt
    ]);
    assert_eq!(m.cpu.regs.get(1), 42);
}

#[test]
fn add_sets_zero_flag() {
    // ldi r0, 1 ; ldi r1, 0xFFFF ; add r0, r1 ; hlt
    let m = run(&[
        0x03, 0x00, 0x01, 0x00, // ldi r0, 1
        0x03, 0x10, 0xFF, 0xFF, // ldi r1, 0xFFFF
        0x08, 0x01,             // add r0, r1  → 0 (carry)
        0x01,                   // hlt
    ]);
    assert_eq!(m.cpu.regs.get(0), 0);
    assert!(m.cpu.flags.zero);
    assert!(m.cpu.flags.carry);
}

#[test]
fn sub_and_cmp_set_flags() {
    // ldi r0, 5 ; ldi r1, 5 ; sub r0, r1 ; hlt
    let m = run(&[
        0x03, 0x00, 5, 0x00,
        0x03, 0x10, 5, 0x00,
        0x09, 0x01, // sub r0, r1
        0x01,
    ]);
    assert_eq!(m.cpu.regs.get(0), 0);
    assert!(m.cpu.flags.zero);
    assert!(m.cpu.flags.carry); // a >= b ⟹ carry set (no borrow)
}

#[test]
fn jmp_branches() {
    // jmp 0x0005 ; nop ; hlt @ 5
    let m = run(&[0x30, 0x05, 0x00, 0x00, 0x00, 0x01]);
    assert!(m.is_halted());
    assert_eq!(m.cpu.pc, 6);
}

#[test]
fn jz_not_taken_when_not_zero() {
    // ldi r0, 1 ; cmpi r0, 0 ; jz 0xFFFF ; hlt
    let m = run(&[
        0x03, 0x00, 1, 0x00,         // ldi r0, 1
        0x16, 0x00, 0x00, 0x00,      // cmpi r0, 0  → not zero
        0x31, 0xFF, 0xFF,            // jz  0xFFFF  (should not jump)
        0x01,                        // hlt
    ]);
    assert!(m.is_halted());
}

#[test]
fn memory_load_store() {
    // ldi r0, 0x0100 ; ldi r1, 0xBEEF ; st [r0], r1 ; ld r2, [r0] ; hlt
    let m = run(&[
        0x03, 0x00, 0x00, 0x01, // ldi r0, 0x0100
        0x03, 0x10, 0xEF, 0xBE, // ldi r1, 0xBEEF
        0x05, 0x01,             // st [r0], r1
        0x04, 0x20,             // ld r2, [r0]
        0x01,                   // hlt
    ]);
    assert_eq!(m.cpu.regs.get(2), 0xBEEF);
}

#[test]
fn push_pop_round_trip() {
    // ldi r0, 0x1234 ; push r0 ; ldi r0, 0 ; pop r1 ; hlt
    let m = run(&[
        0x03, 0x00, 0x34, 0x12, // ldi r0, 0x1234
        0x20, 0x00,             // push r0
        0x03, 0x00, 0x00, 0x00, // ldi r0, 0
        0x21, 0x10,             // pop r1
        0x01,
    ]);
    assert_eq!(m.cpu.regs.get(1), 0x1234);
}

#[test]
fn call_ret() {
    // 0x0000: call 0x0006
    // 0x0003: ldi r0, 99
    // 0x0007: hlt
    // 0x0006: ret
    let mut rom = vec![0u8; 16];
    // call 0x0006
    rom[0] = 0x22; rom[1] = 0x06; rom[2] = 0x00;
    // ldi r0, 99 (executed after ret)
    rom[3] = 0x03; rom[4] = 0x00; rom[5] = 99; rom[6] = 0x00;
    // wait, let's rearrange so call is at 0, subroutine at 7, hlt after ldi
    // 0: call 7   (3 bytes)
    // 3: ldi r0,99 (4 bytes)
    // 7: ret  (1 byte)
    // 8: hlt
    let mut rom = vec![0u8; 16];
    rom[0] = 0x22; rom[1] = 0x07; rom[2] = 0x00; // call 0x0007
    rom[3] = 0x03; rom[4] = 0x00; rom[5] = 99; rom[6] = 0x00; // ldi r0, 99
    rom[7] = 0x23; // ret  — should go back to 0x0003 and run ldi
    rom[8] = 0x01; // unreachable hlt (we hit hlt at different address)

    // After call 7: PC=7, stack has 3.
    // ret → PC=3, executes ldi r0,99, then PC=7 again → infinite loop.
    // Better layout: call the subroutine, it does work, rets, then we hlt.
    let mut rom = vec![0u8; 16];
    rom[0] = 0x22; rom[1] = 0x05; rom[2] = 0x00; // call 0x0005
    // 3: hlt
    rom[3] = 0x01;
    // 4: padding nop (unreachable)
    rom[4] = 0x00;
    // 5: ldi r0, 0x42
    rom[5] = 0x03; rom[6] = 0x00; rom[7] = 0x42; rom[8] = 0x00;
    // 9: ret
    rom[9] = 0x23;

    let m = run(&rom);
    assert_eq!(m.cpu.regs.get(0), 0x42);
    assert!(m.is_halted());
}
