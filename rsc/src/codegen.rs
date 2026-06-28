use std::collections::{HashMap, HashSet};
use crate::ast::*;
use crate::error::{CompileError, Result};

// ── register aliases ──────────────────────────────────────────────────────────
const R0:  u8 = 0;
const R1:  u8 = 1;
const R2:  u8 = 2;
const R3:  u8 = 3;
const R11: u8 = 11; // address scratch
const R13: u8 = 13; // frame pointer
const SP:  u8 = 14;

// ── abstract instruction list ─────────────────────────────────────────────────

#[derive(Clone, Debug)]
enum Ins {
    Nop,  Hlt,
    Mov(u8, u8),  Ldi(u8, u32),
    Ld(u8, u8),   St(u8, u8),
    Ldb(u8, u8),  Stb(u8, u8),
    Add(u8, u8),  Sub(u8, u8),
    Mul(u8, u8),  Div(u8, u8),  Mod(u8, u8),
    And(u8, u8),  Or(u8, u8),   Xor(u8, u8),  Not(u8),
    Shl(u8, u8),  Shr(u8, u8),
    Addi(u8, u32), Subi(u8, u32),
    Andi(u8, u32), Ori(u8, u32), Xori(u8, u32),
    Cmp(u8, u8),  Cmpi(u8, u32),
    Push(u8), Pop(u8),
    Call(u32),  CallL(String),
    Ret,
    Jmp(u32),  JmpL(String),
    Jz(u32),   JzL(String),
    Jnz(u32),  JnzL(String),
    Jlt(u32),  JltL(String),
    Jgt(u32),  JgtL(String),
    Jle(u32),  JleL(String),
    Jge(u32),  JgeL(String),
    Out(u8, u32),
    Label(String),
}

impl Ins {
    fn byte_len(&self) -> u32 {
        use Ins::*;
        match self {
            Nop | Hlt | Not(_) | Ret => 1,
            Mov(_, _) | Ld(_, _) | St(_, _) | Ldb(_, _) | Stb(_, _)
            | Add(_, _) | Sub(_, _) | Mul(_, _) | Div(_, _) | Mod(_, _)
            | And(_, _) | Or(_, _) | Xor(_, _) | Shl(_, _) | Shr(_, _)
            | Cmp(_, _) | Push(_) | Pop(_) => 2,
            Ldi(_, _) | Addi(_, _) | Subi(_, _) | Andi(_, _) | Ori(_, _)
            | Xori(_, _) | Cmpi(_, _) | Out(_, _) => 6,
            Call(_) | CallL(_) | Jmp(_) | JmpL(_)
            | Jz(_) | JzL(_) | Jnz(_) | JnzL(_)
            | Jlt(_) | JltL(_) | Jgt(_) | JgtL(_)
            | Jle(_) | JleL(_) | Jge(_) | JgeL(_) => 5,
            Label(_) => 0,
        }
    }

    fn emit(&self, out: &mut Vec<u8>) {
        fn pack(d: u8, s: u8) -> u8 { (d << 4) | (s & 0x0F) }
        fn i32(out: &mut Vec<u8>, v: u32) { out.extend_from_slice(&v.to_le_bytes()); }
        use Ins::*;
        match self {
            Nop => out.push(0x00),
            Hlt => out.push(0x01),
            Mov(d, s)  => { out.push(0x02); out.push(pack(*d, *s)); }
            Ldi(d, v)  => { out.push(0x03); out.push(pack(*d, 0));  i32(out, *v); }
            Ld(d, s)   => { out.push(0x04); out.push(pack(*d, *s)); }
            St(d, s)   => { out.push(0x05); out.push(pack(*d, *s)); }
            Ldb(d, s)  => { out.push(0x06); out.push(pack(*d, *s)); }
            Stb(d, s)  => { out.push(0x07); out.push(pack(*d, *s)); }
            Add(d, s)  => { out.push(0x08); out.push(pack(*d, *s)); }
            Sub(d, s)  => { out.push(0x09); out.push(pack(*d, *s)); }
            And(d, s)  => { out.push(0x0A); out.push(pack(*d, *s)); }
            Or(d, s)   => { out.push(0x0B); out.push(pack(*d, *s)); }
            Xor(d, s)  => { out.push(0x0C); out.push(pack(*d, *s)); }
            Not(d)     => { out.push(0x0D); out.push(pack(*d, 0)); }
            Shl(d, s)  => { out.push(0x0E); out.push(pack(*d, *s)); }
            Shr(d, s)  => { out.push(0x0F); out.push(pack(*d, *s)); }
            Addi(d, v) => { out.push(0x10); out.push(pack(*d, 0));  i32(out, *v); }
            Subi(d, v) => { out.push(0x11); out.push(pack(*d, 0));  i32(out, *v); }
            Andi(d, v) => { out.push(0x12); out.push(pack(*d, 0));  i32(out, *v); }
            Ori(d, v)  => { out.push(0x13); out.push(pack(*d, 0));  i32(out, *v); }
            Xori(d, v) => { out.push(0x14); out.push(pack(*d, 0));  i32(out, *v); }
            Cmp(d, s)  => { out.push(0x15); out.push(pack(*d, *s)); }
            Cmpi(d, v) => { out.push(0x16); out.push(pack(*d, 0));  i32(out, *v); }
            Mul(d, s)  => { out.push(0x17); out.push(pack(*d, *s)); }
            Div(d, s)  => { out.push(0x18); out.push(pack(*d, *s)); }
            Mod(d, s)  => { out.push(0x19); out.push(pack(*d, *s)); }
            Push(s)    => { out.push(0x20); out.push(pack(0, *s)); }
            Pop(d)     => { out.push(0x21); out.push(pack(*d, 0)); }
            Call(a)    => { out.push(0x22); i32(out, *a); }
            Ret        =>   out.push(0x23),
            Jmp(a)     => { out.push(0x30); i32(out, *a); }
            Jz(a)      => { out.push(0x31); i32(out, *a); }
            Jnz(a)     => { out.push(0x32); i32(out, *a); }
            Jlt(a)     => { out.push(0x3A); i32(out, *a); }
            Jgt(a)     => { out.push(0x39); i32(out, *a); }
            Jle(a)     => { out.push(0x3C); i32(out, *a); }
            Jge(a)     => { out.push(0x3B); i32(out, *a); }
            Out(s, p)  => { out.push(0x41); out.push(pack(*s, 0)); i32(out, *p); }
            CallL(_) | JmpL(_) | JzL(_) | JnzL(_) | JltL(_)
            | JgtL(_) | JleL(_) | JgeL(_) | Label(_) => {}
        }
    }
}

// ── per-function compile context ──────────────────────────────────────────────

struct Ctx {
    // stack of (name→fp_offset, fp_offset at scope entry)
    scopes: Vec<(HashMap<String, i32>, i32)>,
    // current FP offset (0 at function entry, negative as locals are pushed)
    fp_offset: i32,
}

impl Ctx {
    fn new() -> Self { Ctx { scopes: vec![(HashMap::new(), 0)], fp_offset: 0 } }

    fn push_scope(&mut self) {
        self.scopes.push((HashMap::new(), self.fp_offset));
    }

    // Returns how many bytes to pop (addi SP, N) to undo scope's locals.
    fn pop_scope(&mut self) -> u32 {
        let (_, entry) = self.scopes.pop().unwrap();
        let bytes = (entry - self.fp_offset) as u32;
        self.fp_offset = entry;
        bytes
    }

    fn alloc_local(&mut self, name: &str) -> i32 {
        self.fp_offset -= 4;
        self.scopes.last_mut().unwrap().0.insert(name.to_string(), self.fp_offset);
        self.fp_offset
    }

    fn lookup(&self, name: &str) -> Option<i32> {
        for (scope, _) in self.scopes.iter().rev() {
            if let Some(&off) = scope.get(name) { return Some(off); }
        }
        None
    }
}

// ── code generator ────────────────────────────────────────────────────────────

pub struct Codegen {
    code:          Vec<Ins>,
    label_counter: u32,
    known_fns:     HashSet<String>,
}

impl Codegen {
    pub fn new() -> Self {
        Codegen { code: Vec::new(), label_counter: 0, known_fns: HashSet::new() }
    }

    pub fn compile(&mut self, prog: &Program) -> Result<Vec<u8>> {
        for f in &prog.functions { self.known_fns.insert(f.name.clone()); }

        // Entry stub: call main then halt
        self.code.push(Ins::CallL("main".into()));
        self.code.push(Ins::Hlt);

        for f in &prog.functions { self.compile_fn(f)?; }

        let code = std::mem::take(&mut self.code);
        Self::link_code(code)
    }

    fn new_label(&mut self) -> String {
        self.label_counter += 1;
        format!("__L{}", self.label_counter)
    }

    // ── function ──────────────────────────────────────────────────────────────

    fn compile_fn(&mut self, func: &Function) -> Result<()> {
        self.code.push(Ins::Label(func.name.clone()));

        // Prologue: save FP and set FP = SP
        self.code.push(Ins::Push(R13));
        self.code.push(Ins::Mov(R13, SP));

        let mut ctx = Ctx::new();

        // Spill register args into stack locals
        let arg_regs = [R0, R1, R2, R3];
        let n_reg_params = func.params.len().min(4);
        for (i, param) in func.params.iter().enumerate().take(n_reg_params) {
            // push rN (which currently holds the arg)
            // but we can't push r0 then try to push r1 after potentially clobbering r0...
            // We push in order R0, R1, R2, R3 — safe since each push uses the reg value
            // before any subsequent push touches it.
            self.code.push(Ins::Push(arg_regs[i]));
            ctx.alloc_local(&param.name);
        }
        if func.params.len() > 4 {
            return Err(CompileError::Codegen("functions with >4 parameters not supported".into()));
        }

        self.compile_stmts(&func.body, &mut ctx, None, None)?;

        // Default epilogue (unreachable if all paths return, but emitted for safety)
        self.emit_epilogue();
        Ok(())
    }

    fn emit_epilogue(&mut self) {
        self.code.push(Ins::Mov(SP, R13)); // SP = FP
        self.code.push(Ins::Pop(R13));      // restore saved FP
        self.code.push(Ins::Ret);
    }

    // ── statements ───────────────────────────────────────────────────────────

    fn compile_stmts(
        &mut self, stmts: &[Stmt], ctx: &mut Ctx,
        loop_start: Option<&str>, loop_end: Option<&str>,
    ) -> Result<()> {
        for s in stmts { self.compile_stmt(s, ctx, loop_start, loop_end)?; }
        Ok(())
    }

    fn compile_stmt(
        &mut self, stmt: &Stmt, ctx: &mut Ctx,
        loop_start: Option<&str>, loop_end: Option<&str>,
    ) -> Result<()> {
        match stmt {
            Stmt::Let { name, init, .. } => {
                if let Some(expr) = init {
                    self.compile_expr(expr, ctx)?;
                } else {
                    self.code.push(Ins::Ldi(R0, 0));
                }
                ctx.alloc_local(name);
                self.code.push(Ins::Push(R0));
            }

            Stmt::Expr(expr) => { self.compile_expr(expr, ctx)?; }

            Stmt::If { cond, then_body, else_body } => {
                let else_lbl = self.new_label();
                let end_lbl  = self.new_label();

                self.compile_cond_jump(cond, &else_lbl.clone(), ctx)?;

                ctx.push_scope();
                self.compile_stmts(then_body, ctx, loop_start, loop_end)?;
                let then_bytes = ctx.pop_scope();
                if then_bytes > 0 { self.code.push(Ins::Addi(SP, then_bytes)); }

                if !else_body.is_empty() {
                    self.code.push(Ins::JmpL(end_lbl.clone()));
                }
                self.code.push(Ins::Label(else_lbl));

                if !else_body.is_empty() {
                    ctx.push_scope();
                    self.compile_stmts(else_body, ctx, loop_start, loop_end)?;
                    let else_bytes = ctx.pop_scope();
                    if else_bytes > 0 { self.code.push(Ins::Addi(SP, else_bytes)); }
                    self.code.push(Ins::Label(end_lbl));
                }
            }

            Stmt::While { cond, body } => {
                let loop_lbl = self.new_label();
                let end_lbl  = self.new_label();

                self.code.push(Ins::Label(loop_lbl.clone()));
                self.compile_cond_jump(cond, &end_lbl.clone(), ctx)?;

                ctx.push_scope();
                self.compile_stmts(body, ctx, Some(&loop_lbl.clone()), Some(&end_lbl.clone()))?;
                let body_bytes = ctx.pop_scope();
                if body_bytes > 0 { self.code.push(Ins::Addi(SP, body_bytes)); }

                self.code.push(Ins::JmpL(loop_lbl));
                self.code.push(Ins::Label(end_lbl));
            }

            Stmt::Loop { body } => {
                let loop_lbl = self.new_label();
                let end_lbl  = self.new_label();

                self.code.push(Ins::Label(loop_lbl.clone()));

                ctx.push_scope();
                self.compile_stmts(body, ctx, Some(&loop_lbl.clone()), Some(&end_lbl.clone()))?;
                let body_bytes = ctx.pop_scope();
                if body_bytes > 0 { self.code.push(Ins::Addi(SP, body_bytes)); }

                self.code.push(Ins::JmpL(loop_lbl));
                self.code.push(Ins::Label(end_lbl));
            }

            Stmt::Return(val) => {
                if let Some(expr) = val { self.compile_expr(expr, ctx)?; }
                self.emit_epilogue(); // mov SP,FP; pop FP; ret
            }
        }
        Ok(())
    }

    // ── condition → conditional jump to false_label ───────────────────────────

    fn compile_cond_jump(&mut self, expr: &Expr, false_lbl: &str, ctx: &mut Ctx) -> Result<()> {
        match expr {
            Expr::BinOp(left, op @ (BinOp::Lt | BinOp::Gt | BinOp::Le | BinOp::Ge | BinOp::Eq | BinOp::Ne), right) => {
                self.compile_expr(left, ctx)?;
                self.code.push(Ins::Push(R0));
                self.compile_expr(right, ctx)?;
                self.code.push(Ins::Pop(R1));
                // R1 = left, R0 = right; cmp R1, R0 → flags on (left - right)
                self.code.push(Ins::Cmp(R1, R0));
                let lbl = false_lbl.to_string();
                let jump = match op {
                    BinOp::Lt => Ins::JgeL(lbl),
                    BinOp::Gt => Ins::JleL(lbl),
                    BinOp::Le => Ins::JgtL(lbl),
                    BinOp::Ge => Ins::JltL(lbl),
                    BinOp::Eq => Ins::JnzL(lbl),
                    BinOp::Ne => Ins::JzL(lbl),
                    _ => unreachable!(),
                };
                self.code.push(jump);
            }
            // For non-comparison expressions, materialise the bool then branch
            _ => {
                self.compile_expr(expr, ctx)?;
                self.code.push(Ins::Cmpi(R0, 0));
                self.code.push(Ins::JzL(false_lbl.to_string()));
            }
        }
        Ok(())
    }

    // ── expressions (result in R0) ────────────────────────────────────────────

    fn compile_expr(&mut self, expr: &Expr, ctx: &mut Ctx) -> Result<()> {
        match expr {
            Expr::Lit(n) => { self.code.push(Ins::Ldi(R0, *n)); }

            Expr::Var(name) => {
                let off = ctx.lookup(name).ok_or_else(||
                    CompileError::Codegen(format!("undefined variable '{name}'")))?;
                self.emit_load(off, R0);
            }

            Expr::Assign(name, rhs) => {
                self.compile_expr(rhs, ctx)?;
                let off = ctx.lookup(name).ok_or_else(||
                    CompileError::Codegen(format!("undefined variable '{name}'")))?;
                self.emit_store(off, R0);
            }

            Expr::UnOp(op, inner) => {
                self.compile_expr(inner, ctx)?;
                match op {
                    UnOp::Neg => {
                        // neg r0: r0 = 0 - r0
                        self.code.push(Ins::Ldi(R1, 0));
                        self.code.push(Ins::Sub(R1, R0));
                        self.code.push(Ins::Mov(R0, R1));
                    }
                    UnOp::Not => {
                        // logical not: r0 = (r0 == 0) ? 1 : 0
                        let t = self.new_label();
                        let e = self.new_label();
                        self.code.push(Ins::Cmpi(R0, 0));
                        self.code.push(Ins::JnzL(t.clone()));
                        self.code.push(Ins::Ldi(R0, 1));
                        self.code.push(Ins::JmpL(e.clone()));
                        self.code.push(Ins::Label(t));
                        self.code.push(Ins::Ldi(R0, 0));
                        self.code.push(Ins::Label(e));
                    }
                }
            }

            Expr::BinOp(left, op, right) => {
                self.compile_binop(left, op, right, ctx)?;
            }

            Expr::Call(name, args) => {
                self.compile_call(name, args, ctx)?;
            }
        }
        Ok(())
    }

    fn compile_binop(&mut self, left: &Expr, op: &BinOp, right: &Expr, ctx: &mut Ctx) -> Result<()> {
        // Short-circuit for && and ||
        match op {
            BinOp::And => {
                let false_lbl = self.new_label();
                let end_lbl   = self.new_label();
                self.compile_expr(left, ctx)?;
                self.code.push(Ins::Cmpi(R0, 0));
                self.code.push(Ins::JzL(false_lbl.clone()));
                self.compile_expr(right, ctx)?;
                self.code.push(Ins::Cmpi(R0, 0));
                self.code.push(Ins::JzL(false_lbl.clone()));
                self.code.push(Ins::Ldi(R0, 1));
                self.code.push(Ins::JmpL(end_lbl.clone()));
                self.code.push(Ins::Label(false_lbl));
                self.code.push(Ins::Ldi(R0, 0));
                self.code.push(Ins::Label(end_lbl));
                return Ok(());
            }
            BinOp::Or => {
                let true_lbl = self.new_label();
                let end_lbl  = self.new_label();
                self.compile_expr(left, ctx)?;
                self.code.push(Ins::Cmpi(R0, 0));
                self.code.push(Ins::JnzL(true_lbl.clone()));
                self.compile_expr(right, ctx)?;
                self.code.push(Ins::Cmpi(R0, 0));
                self.code.push(Ins::JnzL(true_lbl.clone()));
                self.code.push(Ins::Ldi(R0, 0));
                self.code.push(Ins::JmpL(end_lbl.clone()));
                self.code.push(Ins::Label(true_lbl));
                self.code.push(Ins::Ldi(R0, 1));
                self.code.push(Ins::Label(end_lbl));
                return Ok(());
            }
            _ => {}
        }

        // General case: eval left → push; eval right → R0; pop → R1
        self.compile_expr(left, ctx)?;
        self.code.push(Ins::Push(R0));
        self.compile_expr(right, ctx)?;
        self.code.push(Ins::Pop(R1));
        // R1 = left, R0 = right

        match op {
            BinOp::Add    => { self.code.push(Ins::Add(R1, R0)); self.code.push(Ins::Mov(R0, R1)); }
            BinOp::Sub    => { self.code.push(Ins::Sub(R1, R0)); self.code.push(Ins::Mov(R0, R1)); }
            BinOp::Mul    => { self.code.push(Ins::Mul(R1, R0)); self.code.push(Ins::Mov(R0, R1)); }
            BinOp::Div    => { self.code.push(Ins::Div(R1, R0)); self.code.push(Ins::Mov(R0, R1)); }
            BinOp::Mod    => { self.code.push(Ins::Mod(R1, R0)); self.code.push(Ins::Mov(R0, R1)); }
            BinOp::BitAnd => { self.code.push(Ins::And(R1, R0)); self.code.push(Ins::Mov(R0, R1)); }
            BinOp::BitOr  => { self.code.push(Ins::Or(R1, R0));  self.code.push(Ins::Mov(R0, R1)); }
            BinOp::BitXor => { self.code.push(Ins::Xor(R1, R0)); self.code.push(Ins::Mov(R0, R1)); }
            // Comparisons → materialise 0 or 1 in R0
            cmp_op => {
                self.code.push(Ins::Cmp(R1, R0));
                let true_lbl = self.new_label();
                let end_lbl  = self.new_label();
                let jump = match cmp_op {
                    BinOp::Eq => Ins::JzL(true_lbl.clone()),
                    BinOp::Ne => Ins::JnzL(true_lbl.clone()),
                    BinOp::Lt => Ins::JltL(true_lbl.clone()),
                    BinOp::Gt => Ins::JgtL(true_lbl.clone()),
                    BinOp::Le => Ins::JleL(true_lbl.clone()),
                    BinOp::Ge => Ins::JgeL(true_lbl.clone()),
                    _ => unreachable!(),
                };
                self.code.push(jump);
                self.code.push(Ins::Ldi(R0, 0));
                self.code.push(Ins::JmpL(end_lbl.clone()));
                self.code.push(Ins::Label(true_lbl));
                self.code.push(Ins::Ldi(R0, 1));
                self.code.push(Ins::Label(end_lbl));
            }
        }
        Ok(())
    }

    fn compile_call(&mut self, name: &str, args: &[Expr], ctx: &mut Ctx) -> Result<()> {
        // Built-in intrinsics
        match name {
            "hlt" => { self.code.push(Ins::Hlt); return Ok(()); }

            "serial_write" => {
                if args.len() != 1 { return Err(CompileError::Codegen("serial_write takes 1 arg".into())); }
                self.compile_expr(&args[0], ctx)?;
                self.code.push(Ins::Out(R0, 0x0000));
                return Ok(());
            }

            "fb_write" => {
                // fb_write(addr, ch, attr) — writes ch+attr pair at addr
                if args.len() != 3 { return Err(CompileError::Codegen("fb_write takes 3 args".into())); }
                self.compile_call_args(args, ctx, 3)?;
                // R0=addr, R1=ch, R2=attr
                self.code.push(Ins::Stb(R0, R1));   // stb [addr], ch
                self.code.push(Ins::Addi(R0, 1));
                self.code.push(Ins::Stb(R0, R2));   // stb [addr+1], attr
                return Ok(());
            }

            "mem_read_byte" => {
                if args.len() != 1 { return Err(CompileError::Codegen("mem_read_byte takes 1 arg".into())); }
                self.compile_expr(&args[0], ctx)?;
                self.code.push(Ins::Ldb(R0, R0));   // ldb r0, [r0]
                return Ok(());
            }

            "mem_write_byte" => {
                if args.len() != 2 { return Err(CompileError::Codegen("mem_write_byte takes 2 args".into())); }
                self.compile_call_args(args, ctx, 2)?;
                // R0=addr, R1=byte
                self.code.push(Ins::Stb(R0, R1));
                return Ok(());
            }

            _ => {}
        }

        // User-defined function
        if !self.known_fns.contains(name) {
            return Err(CompileError::Codegen(format!("undefined function '{name}'")));
        }
        let n = args.len().min(4);
        self.compile_call_args(args, ctx, n)?;
        self.code.push(Ins::CallL(name.to_string()));
        Ok(())
    }

    // Evaluate `n` args left-to-right, push each, then pop into r0..r(n-1).
    fn compile_call_args(&mut self, args: &[Expr], ctx: &mut Ctx, n: usize) -> Result<()> {
        for arg in &args[..n] {
            self.compile_expr(arg, ctx)?;
            self.code.push(Ins::Push(R0));
        }
        // Pop in reverse order so r0 = arg0, r1 = arg1, ...
        for i in (0..n).rev() {
            self.code.push(Ins::Pop(i as u8));
        }
        Ok(())
    }

    // ── local var access via FP-relative addressing ───────────────────────────

    fn emit_load(&mut self, fp_offset: i32, reg: u8) {
        // fp_offset is negative: e.g. -4 for first local
        let abs = (-fp_offset) as u32;
        self.code.push(Ins::Mov(R11, R13));
        self.code.push(Ins::Subi(R11, abs));
        self.code.push(Ins::Ld(reg, R11));
    }

    fn emit_store(&mut self, fp_offset: i32, reg: u8) {
        let abs = (-fp_offset) as u32;
        self.code.push(Ins::Mov(R11, R13));
        self.code.push(Ins::Subi(R11, abs));
        self.code.push(Ins::St(R11, reg));
    }

    // ── two-pass linker ───────────────────────────────────────────────────────

    fn link_code(code: Vec<Ins>) -> Result<Vec<u8>> {
        // Pass 1: collect label addresses
        let mut labels: HashMap<String, u32> = HashMap::new();
        let mut addr: u32 = 0;
        for ins in &code {
            if let Ins::Label(name) = ins {
                labels.insert(name.clone(), addr);
            } else {
                addr += ins.byte_len();
            }
        }

        let resolve = |lbl: &str| -> Result<u32> {
            labels.get(lbl).copied()
                .ok_or_else(|| CompileError::Codegen(format!("undefined label '{lbl}'")))
        };

        // Pass 2: resolve + emit
        let mut out = Vec::new();
        for ins in &code {
            let resolved: Ins = match ins {
                Ins::CallL(l) => Ins::Call(resolve(l)?),
                Ins::JmpL(l)  => Ins::Jmp(resolve(l)?),
                Ins::JzL(l)   => Ins::Jz(resolve(l)?),
                Ins::JnzL(l)  => Ins::Jnz(resolve(l)?),
                Ins::JltL(l)  => Ins::Jlt(resolve(l)?),
                Ins::JgtL(l)  => Ins::Jgt(resolve(l)?),
                Ins::JleL(l)  => Ins::Jle(resolve(l)?),
                Ins::JgeL(l)  => Ins::Jge(resolve(l)?),
                Ins::Label(_) => continue,
                other         => { other.emit(&mut out); continue; }
            };
            resolved.emit(&mut out);
        }

        Ok(out)
    }
}
