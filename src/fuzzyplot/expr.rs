use anyhow::{anyhow, Result};
use num_complex::Complex64 as C64;
use std::f64::consts::{E, PI, TAU};
use std::ops::Range;

pub const LANES: usize = 64;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Op {
    Pow,
    Mul,
    Div,
    Add,
    Sub,
    PlusMinus,
    Neg,
    Pos,
    PosNeg,
    Fact,
    Percent,
    Eq,
}

impl Op {
    fn precedence(self) -> u8 {
        match self {
            Op::Mul | Op::Div => 3,
            Op::Add | Op::Sub | Op::PlusMinus => 2,
            Op::Eq => 1,
            _ => 4,
        }
    }

    fn is_left_associative(self) -> bool {
        !matches!(self, Op::Pow | Op::Neg | Op::Pos | Op::PosNeg)
    }

    fn should_shunt(self, other: Op) -> bool {
        other.precedence() > self.precedence()
            || (other.precedence() == self.precedence()
                && other.is_left_associative())
    }

    fn is_postfix(self) -> bool {
        matches!(self, Op::Fact | Op::Percent)
    }
}

#[derive(Debug, Clone)]
enum Token {
    Open,
    Close,
    Op(Op),
    Name(String),
    Num(f64),
    Comma,
}

#[derive(Debug, Clone)]
enum ParenToken {
    Op(Op),
    Num(f64),
    Name(String),
    Sub(Vec<ParenToken>),
    Comma,
}

#[derive(Debug, Clone)]
enum Expr {
    Num(f64),
    Op(Op),
    Sub(Vec<Expr>),
    Var(String),
    Func(String, Vec<Vec<Expr>>),
}

impl Expr {
    fn is_operand(&self) -> bool {
        !matches!(self, Expr::Op(_))
    }
}

const FUNCS: [&str; 16] = [
    "sin", "cos", "max", "min", "sqrt", "nrt", "tan", "abs", "asin", "acos",
    "atan", "atant", "floor", "round", "log", "gamma",
];

const UNARY_FUNCS: [&str; 11] = [
    "sin", "cos", "tan", "asin", "acos", "atan", "sqrt", "abs", "floor",
    "round", "gamma",
];

fn is_func(token: &ParenToken) -> bool {
    match token {
        ParenToken::Name(name) => FUNCS.contains(&name.as_str()),
        _ => false,
    }
}

fn is_unary_func(token: &ParenToken) -> bool {
    match token {
        ParenToken::Name(name) => UNARY_FUNCS.contains(&name.as_str()),
        _ => false,
    }
}

fn missing() -> anyhow::Error {
    anyhow!("Expected something that wasn't found: Expected another expression")
}

fn unexpected(token: &str) -> anyhow::Error {
    anyhow!("Got unexpected token: '{}'", token)
}

fn tokenize(raw: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut rest = raw.trim();
    while !rest.is_empty() {
        rest = rest.trim_start();
        let c = rest.chars().next().unwrap();
        let expects_operand = match tokens.last() {
            None | Some(Token::Open) | Some(Token::Comma) => true,
            Some(Token::Op(op)) => !op.is_postfix(),
            Some(Token::Name(name)) => UNARY_FUNCS.contains(&name.as_str()),
            _ => false,
        };
        let after_num = matches!(tokens.last(), Some(Token::Num(_)));
        let (token, len) = match c {
            '(' => (Token::Open, 1),
            ')' => (Token::Close, 1),
            ',' if !expects_operand => (Token::Comma, 1),
            '-' if expects_operand => (Token::Op(Op::Neg), 1),
            '+' if expects_operand => (Token::Op(Op::Pos), 1),
            '±' if expects_operand => (Token::Op(Op::PosNeg), c.len_utf8()),
            '-' => (Token::Op(Op::Sub), 1),
            '+' => (Token::Op(Op::Add), 1),
            '±' => (Token::Op(Op::PlusMinus), c.len_utf8()),
            '*' | '×' if !expects_operand => {
                (Token::Op(Op::Mul), c.len_utf8())
            }
            '/' | '÷' if !expects_operand => {
                (Token::Op(Op::Div), c.len_utf8())
            }
            '^' if !expects_operand => (Token::Op(Op::Pow), 1),
            '=' if !expects_operand => (Token::Op(Op::Eq), 1),
            '!' if !expects_operand => (Token::Op(Op::Fact), 1),
            '%' if !expects_operand => (Token::Op(Op::Percent), 1),
            c if c.is_alphabetic() || c == '_' => {
                let len = rest
                    .find(|c: char| !(c.is_alphabetic() || c == '_'))
                    .unwrap_or(rest.len());
                (Token::Name(rest[..len].to_string()), len)
            }
            c if (c.is_ascii_digit() || c == '.') && !after_num => {
                let len = rest
                    .find(|c: char| !(c.is_ascii_digit() || c == '.'))
                    .unwrap_or(rest.len());
                let num = &rest[..len];
                if num.matches('.').count() > 1 {
                    return Err(unexpected(&c.to_string()));
                }
                let num =
                    num.parse().map_err(|_| unexpected(&c.to_string()))?;
                (Token::Num(num), len)
            }
            c => return Err(unexpected(&c.to_string())),
        };
        tokens.push(token);
        rest = &rest[len..];
    }
    Ok(tokens)
}

fn to_paren_tokens(tokens: &[Token]) -> Result<Vec<ParenToken>> {
    let mut paren_tokens = Vec::new();
    let mut start = 0;
    let mut depth = 0;
    for (i, token) in tokens.iter().enumerate() {
        match token {
            Token::Open => {
                if depth == 0 {
                    start = i;
                }
                depth += 1;
            }
            Token::Close => {
                if depth == 0 {
                    return Err(anyhow!("Parentheses didn't match"));
                }
                depth -= 1;
                if depth == 0 {
                    paren_tokens.push(ParenToken::Sub(to_paren_tokens(
                        &tokens[start + 1..i],
                    )?));
                }
            }
            _ if depth > 0 => {}
            Token::Op(op) => paren_tokens.push(ParenToken::Op(*op)),
            Token::Name(name) => {
                paren_tokens.push(ParenToken::Name(name.clone()))
            }
            Token::Num(num) => paren_tokens.push(ParenToken::Num(*num)),
            Token::Comma => paren_tokens.push(ParenToken::Comma),
        }
    }
    if depth != 0 {
        return Err(anyhow!("Parentheses didn't match"));
    }
    Ok(paren_tokens)
}

fn paren_to_exprs(raw: Vec<ParenToken>) -> Result<Vec<Expr>> {
    let mut exprs = Vec::new();
    let mut tokens = raw.into_iter().peekable();
    while let Some(token) = tokens.next() {
        match token {
            ParenToken::Name(name) if FUNCS.contains(&name.as_str()) => {
                if let Some(ParenToken::Sub(_)) = tokens.peek() {
                    if let Some(ParenToken::Sub(sub)) = tokens.next() {
                        exprs.push(Expr::Func(name, tokens_to_args(sub)?));
                    }
                } else if UNARY_FUNCS.contains(&name.as_str()) {
                    let arg = take_argument(&mut tokens);
                    exprs.push(Expr::Func(name, vec![paren_to_exprs(arg)?]));
                } else {
                    exprs.push(Expr::Var(name));
                }
            }
            ParenToken::Name(name) => exprs.push(Expr::Var(name)),
            ParenToken::Sub(sub) => exprs.push(Expr::Sub(paren_to_exprs(sub)?)),
            ParenToken::Num(num) => exprs.push(Expr::Num(num)),
            ParenToken::Op(op) => exprs.push(Expr::Op(op)),
            ParenToken::Comma => return Err(unexpected(",")),
        }
    }
    Ok(exprs)
}

type Tokens = std::iter::Peekable<std::vec::IntoIter<ParenToken>>;

fn take_argument(tokens: &mut Tokens) -> Vec<ParenToken> {
    let mut arg = Vec::new();
    if !take_factor(tokens, &mut arg) {
        return arg;
    }
    loop {
        match tokens.peek() {
            Some(ParenToken::Op(Op::Pow)) => {
                arg.extend(tokens.next());
                if !take_factor(tokens, &mut arg) {
                    break;
                }
            }
            Some(ParenToken::Op(op)) if op.is_postfix() => {
                arg.extend(tokens.next())
            }
            Some(t) if is_func(t) => break,
            Some(ParenToken::Num(_))
            | Some(ParenToken::Name(_))
            | Some(ParenToken::Sub(_)) => arg.extend(tokens.next()),
            _ => break,
        }
    }
    arg
}

fn take_factor(tokens: &mut Tokens, arg: &mut Vec<ParenToken>) -> bool {
    while let Some(ParenToken::Op(Op::Neg | Op::Pos | Op::PosNeg)) =
        tokens.peek()
    {
        arg.extend(tokens.next());
    }
    match tokens.peek() {
        Some(t) if is_func(t) => {
            let unary = tokens.peek().map_or(false, is_unary_func);
            arg.extend(tokens.next());
            if let Some(ParenToken::Sub(_)) = tokens.peek() {
                arg.extend(tokens.next());
            } else if unary {
                arg.push(ParenToken::Sub(take_argument(tokens)));
            }
            true
        }
        Some(ParenToken::Num(_))
        | Some(ParenToken::Name(_))
        | Some(ParenToken::Sub(_)) => {
            arg.extend(tokens.next());
            true
        }
        _ => false,
    }
}

fn tokens_to_args(raw: Vec<ParenToken>) -> Result<Vec<Vec<Expr>>> {
    raw.split(|t| matches!(t, ParenToken::Comma))
        .filter(|arg| !arg.is_empty())
        .map(|arg| paren_to_exprs(arg.to_vec()))
        .collect()
}

fn insert_operators(mut raw: Vec<Expr>) -> Vec<Expr> {
    let mut i = 0;
    while i + 1 < raw.len() {
        if raw[i].is_operand() && raw[i + 1].is_operand() {
            raw.insert(i + 1, Expr::Op(Op::Mul));
        } else {
            if matches!(raw[i], Expr::Op(op) if op.is_postfix())
                && raw[i + 1].is_operand()
            {
                raw.insert(i + 1, Expr::Op(Op::Mul));
            }
            i += 1;
        }
    }
    raw.into_iter()
        .map(|expr| match expr {
            Expr::Sub(sub) => Expr::Sub(insert_operators(sub)),
            Expr::Func(name, args) => Expr::Func(
                name,
                args.into_iter().map(insert_operators).collect(),
            ),
            expr => expr,
        })
        .collect()
}

fn to_postfix(raw: Vec<Expr>) -> Vec<Expr> {
    let mut stack = Vec::new();
    let mut ops: Vec<Op> = Vec::new();
    for expr in raw {
        match expr {
            Expr::Op(op) => {
                while let Some(&top) = ops.last() {
                    if !op.should_shunt(top) {
                        break;
                    }
                    stack.push(Expr::Op(top));
                    ops.pop();
                }
                ops.push(op);
            }
            Expr::Sub(sub) => stack.push(Expr::Sub(to_postfix(sub))),
            Expr::Func(name, args) => stack.push(Expr::Func(
                name,
                args.into_iter().map(to_postfix).collect(),
            )),
            expr => stack.push(expr),
        }
    }
    while let Some(op) = ops.pop() {
        stack.push(Expr::Op(op));
    }
    stack
}

#[derive(Debug, Clone, Copy)]
enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    Log,
}

#[derive(Debug, Clone, Copy)]
enum UnOp {
    Neg,
    Percent,
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Abs,
    Floor,
    Round,
    Gamma,
}

#[derive(Debug, Clone)]
enum Inst {
    Bin {
        op: BinOp,
        a: Range<usize>,
        b: Range<usize>,
        dst: usize,
    },
    PowI {
        a: Range<usize>,
        n: i32,
        dst: usize,
    },
    PlusMinus {
        a: Range<usize>,
        b: Range<usize>,
        dst: usize,
    },
    PosNeg {
        a: Range<usize>,
        dst: usize,
    },
    Un {
        op: UnOp,
        a: Range<usize>,
        dst: usize,
    },
    Sqrt {
        a: Range<usize>,
        dst: usize,
    },
    MinMax {
        max: bool,
        args: Vec<Range<usize>>,
        dst: usize,
    },
    Eq {
        args: Vec<Range<usize>>,
        sharpness: f64,
        plain: bool,
        dst: usize,
    },
}

impl Inst {
    fn exec(&self, buf: &mut [C64], lanes: usize) {
        match self {
            Inst::Bin { op, a, b, dst } => {
                let (src, out) = buf.split_at_mut(dst * lanes);
                let (a, b) = (slots(src, a, lanes), slots(src, b, lanes));
                match op {
                    BinOp::Add => cartesian(a, b, out, lanes, |a, b| a + b),
                    BinOp::Sub => cartesian(a, b, out, lanes, |a, b| a - b),
                    BinOp::Mul => cartesian(a, b, out, lanes, |a, b| a * b),
                    BinOp::Div => cartesian(a, b, out, lanes, div),
                    BinOp::Pow => cartesian(a, b, out, lanes, pow),
                    BinOp::Log => cartesian(a, b, out, lanes, |a, b| {
                        div(a.log10(), b.log10())
                    }),
                }
            }
            Inst::PowI { a, n, dst } => {
                let (src, out) = buf.split_at_mut(dst * lanes);
                map(slots(src, a, lanes), out, |a| powi(a, *n));
            }
            Inst::PlusMinus { a, b, dst } => {
                let (src, out) = buf.split_at_mut(dst * lanes);
                let (a, b) = (slots(src, a, lanes), slots(src, b, lanes));
                let (adds, subs) = out.split_at_mut(a.len() * b.len() / lanes);
                cartesian(a, b, adds, lanes, |a, b| a + b);
                cartesian(a, b, subs, lanes, |a, b| a - b);
            }
            Inst::PosNeg { a, dst } => {
                let (src, out) = buf.split_at_mut(dst * lanes);
                let mut out = out.chunks_exact_mut(lanes);
                for a in slots(src, a, lanes).chunks_exact(lanes) {
                    map(a, out.next().unwrap(), |a| -a);
                    out.next().unwrap().copy_from_slice(a);
                }
            }
            Inst::Un { op, a, dst } => {
                let (src, out) = buf.split_at_mut(dst * lanes);
                let a = slots(src, a, lanes);
                match op {
                    UnOp::Neg => map(a, out, |a| -a),
                    UnOp::Percent => map(a, out, |a| a * 0.01),
                    UnOp::Sin => map(a, out, C64::sin),
                    UnOp::Cos => map(a, out, C64::cos),
                    UnOp::Tan => map(a, out, C64::tan),
                    UnOp::Asin => map(a, out, C64::asin),
                    UnOp::Acos => map(a, out, C64::acos),
                    UnOp::Atan => map(a, out, C64::atan),
                    UnOp::Abs => map(a, out, |a| C64::new(a.norm(), 0.0)),
                    UnOp::Floor => {
                        map(a, out, |a| C64::new(a.re.floor(), a.im.floor()))
                    }
                    UnOp::Round => {
                        map(a, out, |a| C64::new(a.re.round(), a.im.round()))
                    }
                    UnOp::Gamma => map(a, out, gamma),
                }
            }
            Inst::Sqrt { a, dst } => {
                let (src, out) = buf.split_at_mut(dst * lanes);
                let mut out = out.chunks_exact_mut(lanes);
                for a in slots(src, a, lanes).chunks_exact(lanes) {
                    let pos = out.next().unwrap();
                    map(a, pos, C64::sqrt);
                    map(pos, out.next().unwrap(), |r| -r);
                }
            }
            Inst::MinMax { max, args, dst } => {
                let (src, out) = buf.split_at_mut(dst * lanes);
                for lane in 0..lanes {
                    let value = |slot: usize| src[slot * lanes + lane];
                    let first = args[0].clone();
                    let mut best = value(first.end - 1);
                    let candidates = args
                        .iter()
                        .flat_map(|arg| arg.clone())
                        .skip(1)
                        .chain(first.start..first.end - 1);
                    for c in candidates.map(value) {
                        if (*max && c.re > best.re) || (!*max && c.re < best.re)
                        {
                            best = c;
                        }
                    }
                    out[lane] = best;
                }
            }
            Inst::Eq {
                args,
                sharpness,
                plain,
                dst,
            } => {
                let (src, out) = buf.split_at_mut(dst * lanes);
                let src = &*src;
                let n = args.len() as f64;
                let combos: usize = args.iter().map(|arg| arg.len()).product();
                for lane in 0..lanes {
                    let mut min_d = f64::INFINITY;
                    for combo in 0..combos {
                        let values = || {
                            let mut k = combo;
                            args.iter().map(move |arg| {
                                let slot = arg.start + k % arg.len();
                                k /= arg.len();
                                src[slot * lanes + lane]
                            })
                        };
                        let mean = values().sum::<C64>() / n;
                        let spread: f64 =
                            values().map(|z| (z - mean).norm_sqr()).sum();
                        let d = if *plain {
                            2.0 * spread / (n - 1.0)
                        } else {
                            let total: f64 =
                                values().map(|z| z.norm_sqr()).sum();
                            if total == 0.0 {
                                0.0
                            } else {
                                spread / total
                            }
                        };
                        min_d = min_d.min(d);
                    }
                    let closeness =
                        ((min_d * sharpness).powi(2) + 1.0).powi(-1);
                    out[lane] = C64::new(closeness, 0.0);
                }
            }
        }
    }
}

fn slots<'a>(buf: &'a [C64], range: &Range<usize>, lanes: usize) -> &'a [C64] {
    &buf[range.start * lanes..range.end * lanes]
}

fn map(a: &[C64], out: &mut [C64], f: impl Fn(C64) -> C64) {
    for (out, &a) in out.iter_mut().zip(a) {
        *out = f(a);
    }
}

fn cartesian(
    a: &[C64],
    b: &[C64],
    out: &mut [C64],
    lanes: usize,
    f: impl Fn(C64, C64) -> C64,
) {
    let mut out = out.chunks_exact_mut(lanes);
    for a in a.chunks_exact(lanes) {
        for b in b.chunks_exact(lanes) {
            for (out, (&a, &b)) in
                out.next().unwrap().iter_mut().zip(a.iter().zip(b))
            {
                *out = f(a, b);
            }
        }
    }
}

fn div(a: C64, b: C64) -> C64 {
    let (ar, ai, br, bi) = (a.re, a.im, b.re, b.im);
    let (mut x, mut y) = if br.abs() < bi.abs() {
        let r = br / bi;
        let d = br * r + bi;
        ((ar * r + ai) / d, (ai * r - ar) / d)
    } else {
        let r = bi / br;
        let d = br + bi * r;
        ((ar + ai * r) / d, (ai - ar * r) / d)
    };
    if x.is_nan() && y.is_nan() {
        let inf =
            |v: f64| (if v.is_infinite() { 1.0_f64 } else { 0.0 }).copysign(v);
        if br == 0.0 && bi == 0.0 && (!ar.is_nan() || !ai.is_nan()) {
            let s = f64::INFINITY.copysign(br);
            x = s * ar;
            y = s * ai;
        } else if (ar.is_infinite() || ai.is_infinite())
            && br.is_finite()
            && bi.is_finite()
        {
            let (ar, ai) = (inf(ar), inf(ai));
            x = f64::INFINITY * (ar * br + ai * bi);
            y = f64::INFINITY * (ai * br - ar * bi);
        } else if (br.is_infinite() || bi.is_infinite())
            && ar.is_finite()
            && ai.is_finite()
        {
            let (br, bi) = (inf(br), inf(bi));
            x = 0.0 * (ar * br + ai * bi);
            y = 0.0 * (ai * br - ar * bi);
        }
    }
    C64::new(x, y)
}

fn pow(a: C64, b: C64) -> C64 {
    if b.im == 0.0 {
        if let Some(n) = as_i32(b.re) {
            return powi(a, n);
        }
        if a.im == 0.0 && a.re >= 0.0 {
            return C64::new(a.re.powf(b.re), 0.0);
        }
    }
    if a.re == 0.0 && a.im == 0.0 && b.re > 0.0 {
        return C64::new(0.0, 0.0);
    }
    (b * a.ln()).exp()
}

const LANCZOS: [f64; 9] = [
    0.99999999999980993,
    676.5203681218851,
    -1259.1392167224028,
    771.32342877765313,
    -176.61502916214059,
    12.507343278686905,
    -0.13857109526572012,
    9.9843695780195716e-6,
    1.5056327351493116e-7,
];

fn gamma(z: C64) -> C64 {
    if z.re < 0.5 {
        let reflected = (z * PI).sin() * gamma(C64::new(1.0, 0.0) - z);
        return div(C64::new(PI, 0.0), reflected);
    }
    let z = z - 1.0;
    let mut sum = C64::new(LANCZOS[0], 0.0);
    for (i, &c) in LANCZOS.iter().enumerate().skip(1) {
        sum += div(C64::new(c, 0.0), z + i as f64);
    }
    let t = z + 7.5;
    (2.0 * PI).sqrt() * ((z + 0.5) * t.ln() - t).exp() * sum
}

fn as_i32(v: f64) -> Option<i32> {
    if v.fract() == 0.0 && v.abs() <= i32::MAX as f64 {
        Some(v as i32)
    } else {
        None
    }
}

fn powi(a: C64, n: i32) -> C64 {
    let mut result = C64::new(1.0, 0.0);
    let mut base = a;
    let mut e = n.unsigned_abs();
    while e > 0 {
        if e & 1 == 1 {
            result *= base;
        }
        base *= base;
        e >>= 1;
    }
    if n < 0 {
        div(C64::new(1.0, 0.0), result)
    } else {
        result
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Var {
    X,
    Y,
    R,
    T,
}

pub struct Compiler {
    init: Vec<C64>,
    insts: Vec<Inst>,
    uses_r: bool,
    uses_t: bool,
    sharpness: f64,
    plain: bool,
}

#[derive(Debug)]
pub struct Program {
    init: Vec<C64>,
    insts: Vec<Inst>,
    pub uses_r: bool,
    pub uses_t: bool,
}

struct Operand {
    range: Range<usize>,
    is_const: bool,
}

enum Item {
    Value(Operand),
    Chain(Vec<Operand>),
}

impl Compiler {
    pub fn new(sharpness: f64, plain: bool) -> Compiler {
        Compiler {
            init: vec![C64::new(0.0, 0.0); 4],
            insts: Vec::new(),
            uses_r: false,
            uses_t: false,
            sharpness,
            plain,
        }
    }

    pub fn expression(&mut self, raw: &str) -> Result<Range<usize>> {
        let tokens = to_paren_tokens(&tokenize(raw)?)?;
        let postfix = to_postfix(insert_operators(paren_to_exprs(tokens)?));
        Ok(self.compile(postfix)?.range)
    }

    pub fn finish(self) -> Program {
        Program {
            init: self.init,
            insts: self.insts,
            uses_r: self.uses_r,
            uses_t: self.uses_t,
        }
    }

    fn integer(&self, operand: &Operand) -> Option<i32> {
        let value = self.init[operand.range.start];
        if operand.is_const && operand.range.len() == 1 && value.im == 0.0 {
            as_i32(value.re)
        } else {
            None
        }
    }

    fn alloc(&mut self, len: usize) -> usize {
        let start = self.init.len();
        self.init.resize(start + len, C64::new(0.0, 0.0));
        start
    }

    fn constant(&mut self, value: C64) -> Operand {
        let start = self.alloc(1);
        self.init[start] = value;
        Operand {
            range: start..start + 1,
            is_const: true,
        }
    }

    fn emit(
        &mut self,
        len: usize,
        inputs: &[&Operand],
        inst: impl FnOnce(usize) -> Inst,
    ) -> Operand {
        let dst = self.alloc(len);
        let inst = inst(dst);
        let is_const = inputs.iter().all(|i| i.is_const);
        if is_const {
            inst.exec(&mut self.init, 1);
        } else {
            self.insts.push(inst);
        }
        Operand {
            range: dst..dst + len,
            is_const,
        }
    }

    fn value(&mut self, item: Item) -> Operand {
        match item {
            Item::Value(operand) => operand,
            Item::Chain(args) => {
                let ranges = args.iter().map(|a| a.range.clone()).collect();
                let inputs: Vec<&Operand> = args.iter().collect();
                let (sharpness, plain) = (self.sharpness, self.plain);
                self.emit(1, &inputs, |dst| Inst::Eq {
                    args: ranges,
                    sharpness,
                    plain,
                    dst,
                })
            }
        }
    }

    fn pop(&mut self, stack: &mut Vec<Item>) -> Result<Operand> {
        let item = stack.pop().ok_or_else(missing)?;
        Ok(self.value(item))
    }

    fn compile(&mut self, postfix: Vec<Expr>) -> Result<Operand> {
        let mut stack: Vec<Item> = Vec::new();
        for expr in postfix {
            if let Expr::Op(Op::Eq) = expr {
                let b = self.pop(&mut stack)?;
                let chain = match stack.pop().ok_or_else(missing)? {
                    Item::Chain(mut args) => {
                        args.push(b);
                        args
                    }
                    Item::Value(a) => vec![a, b],
                };
                stack.push(Item::Chain(chain));
                continue;
            }
            let operand = match expr {
                Expr::Num(num) => self.constant(C64::new(num, 0.0)),
                Expr::Var(name) => self.variable(&name)?,
                Expr::Sub(sub) => self.compile(sub)?,
                Expr::Func(name, args) => {
                    let args = args
                        .into_iter()
                        .map(|arg| self.compile(arg))
                        .collect::<Result<Vec<_>>>()?;
                    self.function(&name, args)?
                }
                Expr::Op(op) if op.precedence() == 4 && op != Op::Pow => {
                    let a = self.pop(&mut stack)?;
                    let n = a.range.len();
                    let r = a.range.clone();
                    match op {
                        Op::Pos => a,
                        Op::Neg => self.emit(n, &[&a], |dst| Inst::Un {
                            op: UnOp::Neg,
                            a: r,
                            dst,
                        }),
                        Op::Percent => self.emit(n, &[&a], |dst| Inst::Un {
                            op: UnOp::Percent,
                            a: r,
                            dst,
                        }),
                        Op::PosNeg => self.emit(2 * n, &[&a], |dst| {
                            Inst::PosNeg { a: r, dst }
                        }),
                        _ => {
                            return Err(anyhow!(
                                "The operation 'Factorial' is not \
                                supported for the type Any"
                            ))
                        }
                    }
                }
                Expr::Op(op) => {
                    let b = self.pop(&mut stack)?;
                    let a = self.pop(&mut stack)?;
                    let n = a.range.len() * b.range.len();
                    let (ra, rb) = (a.range.clone(), b.range.clone());
                    if op == Op::PlusMinus {
                        self.emit(2 * n, &[&a, &b], |dst| Inst::PlusMinus {
                            a: ra,
                            b: rb,
                            dst,
                        })
                    } else if let Some(n) =
                        self.integer(&b).filter(|_| op == Op::Pow)
                    {
                        self.emit(ra.len(), &[&a], |dst| Inst::PowI {
                            a: ra,
                            n,
                            dst,
                        })
                    } else {
                        let op = match op {
                            Op::Add => BinOp::Add,
                            Op::Sub => BinOp::Sub,
                            Op::Mul => BinOp::Mul,
                            Op::Div => BinOp::Div,
                            _ => BinOp::Pow,
                        };
                        self.emit(n, &[&a, &b], |dst| Inst::Bin {
                            op,
                            a: ra,
                            b: rb,
                            dst,
                        })
                    }
                }
            };
            stack.push(Item::Value(operand));
        }
        if stack.len() > 1 {
            return Err(anyhow!(
                "Expected something that wasn't found: \
                Expected another operator"
            ));
        }
        self.pop(&mut stack)
    }

    fn variable(&mut self, name: &str) -> Result<Operand> {
        let var = |v: Var| Operand {
            range: v as usize..v as usize + 1,
            is_const: false,
        };
        Ok(match name {
            "x" => var(Var::X),
            "y" => var(Var::Y),
            "r" => {
                self.uses_r = true;
                var(Var::R)
            }
            "t" => {
                self.uses_t = true;
                var(Var::T)
            }
            "pi" => self.constant(C64::new(PI, 0.0)),
            "e" => self.constant(C64::new(E, 0.0)),
            "i" => self.constant(C64::new(0.0, 1.0)),
            "tau" => self.constant(C64::new(TAU, 0.0)),
            _ => return Err(anyhow!("Variable '{}' is not defined", name)),
        })
    }

    fn function(&mut self, name: &str, args: Vec<Operand>) -> Result<Operand> {
        let incorrect = || anyhow!("A function was passed incorrect arguments");
        let un = match name {
            "sin" => Some(UnOp::Sin),
            "cos" => Some(UnOp::Cos),
            "tan" => Some(UnOp::Tan),
            "asin" => Some(UnOp::Asin),
            "acos" => Some(UnOp::Acos),
            "atan" => Some(UnOp::Atan),
            "abs" => Some(UnOp::Abs),
            "floor" => Some(UnOp::Floor),
            "round" => Some(UnOp::Round),
            "gamma" => Some(UnOp::Gamma),
            _ => None,
        };
        match name {
            _ if un.is_some() || name == "sqrt" => {
                if args.len() != 1 {
                    return Err(incorrect());
                }
                let a = &args[0];
                let (n, r) = (a.range.len(), a.range.clone());
                Ok(match un {
                    Some(op) => {
                        self.emit(n, &[a], |dst| Inst::Un { op, a: r, dst })
                    }
                    None => {
                        self.emit(2 * n, &[a], |dst| Inst::Sqrt { a: r, dst })
                    }
                })
            }
            "log" => {
                if args.len() != 2 {
                    return Err(incorrect());
                }
                let (a, b) = (&args[0], &args[1]);
                let n = a.range.len() * b.range.len();
                let (ra, rb) = (a.range.clone(), b.range.clone());
                Ok(self.emit(n, &[a, b], |dst| Inst::Bin {
                    op: BinOp::Log,
                    a: ra,
                    b: rb,
                    dst,
                }))
            }
            "max" | "min" => {
                if args.is_empty() {
                    return Err(incorrect());
                }
                let ranges = args.iter().map(|a| a.range.clone()).collect();
                let inputs: Vec<&Operand> = args.iter().collect();
                Ok(self.emit(1, &inputs, |dst| Inst::MinMax {
                    max: name == "max",
                    args: ranges,
                    dst,
                }))
            }
            "nrt" => Err(anyhow!(
                "The operation 'Nth Root' is not supported for the type Complex"
            )),
            "atant" => Err(anyhow!(
                "The operation 'Atan2' is not supported for the type Complex"
            )),
            _ => Err(anyhow!("Function '{}' is not defined", name)),
        }
    }
}

impl Program {
    pub fn buffer(&self) -> Vec<C64> {
        self.init
            .iter()
            .flat_map(|&v| std::iter::repeat(v).take(LANES))
            .collect()
    }

    pub fn set_vars(&self, buf: &mut [C64], lane: usize, vars: [C64; 4]) {
        for (slot, &var) in vars.iter().enumerate() {
            buf[slot * LANES + lane] = var;
        }
    }

    pub fn eval(&self, buf: &mut [C64]) {
        for inst in &self.insts {
            inst.exec(buf, LANES);
        }
    }

    pub fn results<'a>(
        &self,
        buf: &'a [C64],
        range: Range<usize>,
        lane: usize,
    ) -> impl Iterator<Item = C64> + 'a {
        range.map(move |slot| buf[slot * LANES + lane])
    }
}
