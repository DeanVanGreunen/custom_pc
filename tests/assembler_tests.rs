//! Smoke tests that verify the assembler binary produces correct output.
//!
//! These tests run `cargo run --bin asm` as a subprocess so they exercise
//! the full assembler pipeline end-to-end.

use std::fs;
use std::process::Command;

/// Write a temporary .asm file, assemble it, and return the binary bytes.
fn assemble(name: &str, src: &str) -> Vec<u8> {
    let dir = std::env::temp_dir().join("custom_pc_asm_tests");
    fs::create_dir_all(&dir).unwrap();
    let asm_path = dir.join(format!("{name}.asm"));
    let bin_path = dir.join(format!("{name}.bin"));
    fs::write(&asm_path, src).unwrap();

    let status = Command::new(env!("CARGO"))
        .args(["run", "--bin", "asm", "--manifest-path"])
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/assembler/Cargo.toml"))
        .arg("--")
        .arg(&asm_path)
        .arg(&bin_path)
        .status()
        .expect("failed to run assembler");

    assert!(status.success(), "assembler failed for source:\n{src}");
    fs::read(&bin_path).expect("assembler did not produce output")
}

#[test]
fn nop_hlt() {
    let bytes = assemble("nop_hlt", "nop\nhlt");
    assert_eq!(bytes, &[0x00, 0x01]);
}

#[test]
fn ldi_and_hlt() {
    let bytes = assemble("ldi_and_hlt", "ldi r0, 0x1234\nhlt");
    assert_eq!(bytes, &[0x03, 0x00, 0x34, 0x12, 0x01]);
}

#[test]
fn forward_label() {
    let bytes = assemble("forward_label", "jmp end\nnop\nend: hlt");
    assert_eq!(&bytes[0..3], &[0x30, 0x04, 0x00]);
    assert_eq!(bytes[3], 0x00);
    assert_eq!(bytes[4], 0x01);
}

#[test]
fn data_directives() {
    let bytes = assemble("data_directives", ".byte 1, 2\n.word 0xBEEF\n.string \"AB\"");
    assert_eq!(bytes, &[1, 2, 0xEF, 0xBE, b'A', b'B']);
}

#[test]
fn org_pads_with_zeros() {
    let bytes = assemble("org_pads", "nop\n.org 0x04\nhlt");
    assert_eq!(bytes, &[0x00, 0x00, 0x00, 0x00, 0x01]);
}

#[test]
fn equ_constant() {
    let bytes = assemble("equ_constant", ".equ MAGIC, 0x42\nldi r0, MAGIC\nhlt");
    // ldi r0, 0x0042 → 0x03 0x00 0x42 0x00
    assert_eq!(bytes, &[0x03, 0x00, 0x42, 0x00, 0x01]);
}
