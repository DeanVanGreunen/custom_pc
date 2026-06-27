Jz   { addr: u16 },   // Jump if Zero flag set         — last result was zero
Jnz  { addr: u16 },   // Jump if Not Zero              — last result was non-zero
Jc   { addr: u16 },   // Jump if Carry set             — unsigned overflow / no borrow
Jnc  { addr: u16 },   // Jump if No Carry              — no unsigned overflow / borrow occurred
Jn   { addr: u16 },   // Jump if Negative              — result bit 15 is set (two's complement negative)
Jnn  { addr: u16 },   // Jump if Not Negative          — result bit 15 is clear
Jv   { addr: u16 },   // Jump if oVerflow set          — signed arithmetic overflowed
Jnv  { addr: u16 },   // Jump if No oVerflow           — signed arithmetic did not overflow
Jgt  { addr: u16 },   // Jump if Greater Than (signed) — !Z && (N == V)
Jlt  { addr: u16 },   // Jump if Less Than (signed)    — N != V
Jge  { addr: u16 },   // Jump if Greater or Equal (signed) — N == V
Jle  { addr: u16 },   // Jump if Less or Equal (signed)    — Z || (N != V)
That's only 12 — Jmp is the unconditional jump and not counted among the conditional ones.

The first 8 are raw flag tests — each checks exactly one flag:

jz/jnz — test the Z flag, set by any ALU op when the result is 0x0000
jc/jnc — test the C flag, set when an addition produces a carry out of bit 15, or a subtraction has no borrow (i.e. a >= b unsigned)
jn/jnn — test the N flag, which is simply a copy of bit 15 of the result
jv/jnv — test the V flag, set when the mathematical signed result doesn't fit in 16 bits (e.g. 0x7FFF + 1 overflows into negative)
The last 4 are signed comparison branches — they combine N and V to correctly handle cases where a signed overflow flipped the sign bit:

jgt — !Z && (N == V): not equal and the signed result was non-negative → strictly greater
jlt — N != V: the sign flag was "wrong" due to overflow → strictly less
jge — N == V: signed result was non-negative (possibly via overflow) → greater or equal
jle — Z || (N != V): equal, or signed result was negative → less or equal
The reason jgt/jlt/jge/jle can't just look at N alone is the overflow case. After cmp r0, r1 (which does r0 - r1), if the subtraction wraps around due to signed overflow, N gets the wrong sign. Checking N == V cancels that out: if both are set, overflow flipped the sign but the true result is still positive; if neither is set, no flip and the result is genuinely positive.