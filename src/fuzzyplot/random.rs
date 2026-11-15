use fastrand::Rng;

enum Operation {
    Binary(&'static str),
    Unary(&'static str),
}

use Operation::{Binary, Unary};

const CLASSES: [&[Operation]; 7] = [
    &[Binary("=")],
    &[Binary("+"), Binary("-")],
    &[Binary("*"), Binary("/")],
    &[Binary("^"), Unary("sqrt")],
    &[Unary("abs")],
    &[Unary("sin"), Unary("cos"), Unary("tan")],
    &[Unary("asin"), Unary("acos"), Unary("atan"), Unary("gamma")],
];

const VARIABLES: [&str; 4] = ["x", "y", "r", "t"];

const CONSTANTS: [&str; 8] = ["1", "2", "3", "4", "5", "pi", "e", "i"];

enum Node {
    Leaf(&'static str),
    Unary(&'static str, Box<Node>),
    Binary(&'static str, Box<Node>, Box<Node>),
}

pub fn equation(rng: &mut Rng, mean: f64) -> String {
    loop {
        let size = poisson(rng, mean);
        let (node, vars) = expression(rng, size);
        if vars {
            return print(&node);
        }
    }
}

fn poisson(rng: &mut Rng, mean: f64) -> usize {
    let limit = (-mean).exp();
    let mut product = rng.f64();
    let mut count = 0;
    while product > limit {
        product *= rng.f64();
        count += 1;
    }
    count
}

fn expression(rng: &mut Rng, size: usize) -> (Node, bool) {
    if size == 0 {
        return if rng.bool() {
            (Node::Leaf(VARIABLES[rng.usize(..VARIABLES.len())]), true)
        } else {
            (Node::Leaf(CONSTANTS[rng.usize(..CONSTANTS.len())]), false)
        };
    }
    let class = CLASSES[rng.usize(..CLASSES.len())];
    match class[rng.usize(..class.len())] {
        Unary(name) => {
            let (arg, vars) = expression(rng, size - 1);
            (Node::Unary(name, Box::new(arg)), vars)
        }
        Binary(op) => {
            let left_size = rng.usize(0..size);
            let (a, a_vars) = expression(rng, left_size);
            let (b, b_vars) = expression(rng, size - 1 - left_size);
            (Node::Binary(op, Box::new(a), Box::new(b)), a_vars || b_vars)
        }
    }
}

fn binding(op: &str) -> u8 {
    match op {
        "=" => 1,
        "+" | "-" => 2,
        "*" | "/" => 3,
        _ => 4,
    }
}

fn print(node: &Node) -> String {
    match node {
        Node::Leaf(name) => name.to_string(),
        Node::Unary(name, arg) => {
            let printed = print(arg);
            match **arg {
                Node::Leaf(_) | Node::Unary(..) | Node::Binary("^", ..)
                    if !printed.starts_with('(') =>
                {
                    format!("{} {}", name, printed)
                }
                _ => format!("{}({})", name, printed),
            }
        }
        Node::Binary(op, a, b) => {
            let parent = binding(op);
            let left = match &**a {
                Node::Binary(child, ..)
                    if binding(child) < parent
                        || (binding(child) == parent
                            && (*op == "^" || *op == "=")) =>
                {
                    format!("({})", print(a))
                }
                Node::Unary(name, arg) if *op == "^" => {
                    format!("{}({})", name, print(arg))
                }
                _ => print(a),
            };
            let right = match &**b {
                Node::Binary(child, ..)
                    if binding(child) < parent
                        || (binding(child) == parent && *op != "^") =>
                {
                    format!("({})", print(b))
                }
                _ => print(b),
            };
            format!("{} {} {}", left, op, right)
        }
    }
}
