// Fibonacci numbers in the custom PC Rust subset.
//
// Compile:  cargo run -p rsc --bin rsc -- examples/fibonacci.rs roms/fibonacci_rs.bin
// Run:      cargo run --bin custom_pc -- roms/fibonacci_rs.bin
//
// Built-in functions available:
//   serial_write(byte: u32)              — emit byte to host stdout
//   fb_write(addr: u32, ch: u32, attr: u32) — write char+attr to text FB
//   mem_read_byte(addr: u32) -> u32      — read one byte from memory
//   mem_write_byte(addr: u32, byte: u32) — write one byte to memory
//   hlt()                               — halt the CPU

fn print_decimal(mut n: u32, fb_ptr: u32) -> u32 {
    // Prints n as decimal to FB at fb_ptr and serial.
    // Returns updated fb_ptr (advanced by number of chars * 2).
    let attr: u32 = 0x0F;
    let mut tmp: u32 = n;
    let mut digits: u32 = 0;

    // Count digits
    if tmp == 0 {
        digits = 1;
    } else {
        while tmp > 0 {
            digits = digits + 1;
            tmp = tmp / 10;
        }
    }

    // Write each digit (most significant first) by extracting via divisor
    let mut div: u32 = 1;
    let mut i: u32 = 1;
    while i < digits {
        div = div * 10;
        i = i + 1;
    }

    let mut ptr: u32 = fb_ptr;
    while div > 0 {
        let digit: u32 = n / div;
        let ch: u32 = digit + 48;   // '0' = 48
        fb_write(ptr, ch, attr);
        serial_write(ch);
        ptr = ptr + 2;
        n = n - digit * div;
        div = div / 10;
    }

    return ptr;
}

fn main() {
    let fb_base: u32 = 0xC000;
    let mut ptr: u32 = fb_base;
    let attr: u32 = 0x0F;

    let mut a: u32 = 0;
    let mut b: u32 = 1;
    let mut count: u32 = 16;

    while count > 0 {
        ptr = print_decimal(a, ptr);

        // Write space separator
        fb_write(ptr, 32, attr);
        serial_write(32);
        ptr = ptr + 2;

        // Advance: a, b = b, a + b
        let tmp: u32 = b;
        b = a + b;
        a = tmp;

        count = count - 1;
    }

    hlt();
}
