use custom_pc::memory::Memory;

#[test]
fn byte_round_trip() {
    let mut mem = Memory::new();
    mem.write_byte(0x1234, 0xAB);
    assert_eq!(mem.read_byte(0x1234), 0xAB);
}

#[test]
fn word_round_trip_little_endian() {
    let mut mem = Memory::new();
    mem.write_word(0x0100, 0xBEEF).unwrap();
    assert_eq!(mem.read_byte(0x0100), 0xEF);
    assert_eq!(mem.read_byte(0x0101), 0xBE);
    assert_eq!(mem.read_word(0x0100).unwrap(), 0xBEEF);
}

#[test]
fn word_at_ffff_is_error() {
    let mut mem = Memory::new();
    assert!(mem.write_word(0xFFFF, 0x1234).is_err());
    assert!(mem.read_word(0xFFFF).is_err());
}

#[test]
fn load_rom() {
    let mut mem = Memory::new();
    mem.load(&[0x01, 0x02, 0x03]);
    assert_eq!(mem.read_byte(0), 0x01);
    assert_eq!(mem.read_byte(1), 0x02);
    assert_eq!(mem.read_byte(2), 0x03);
    assert_eq!(mem.read_byte(3), 0x00); // uninitialized
}
