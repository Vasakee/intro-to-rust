use borsh::{BorshSerialize, BorshDeserialize};
use std::io::{self, Write};

// ---- STRUCTS (Section 6) ----
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct Calculation {
    operation: String,
    operand_a: f64,
    operand_b: f64,
    result: f64,
}

#[derive(BorshSerialize, BorshDeserialize, Debug)]
struct CalculationHistory {
    entries: Vec<Calculation>,
}

// ---- ENUMS (Section 7) ----
#[derive(Debug)]
enum Operation {
    Add,
    Subtract,
    Multiply,
    Divide,
    Modulo,  // FIX: was incorrectly mapped to Divide before
    Power,   // NEW: raises a to the power of b
}

// ---- CUSTOM ERRORS (Section 8) ----
#[derive(Debug)]
enum CalcError {
    DivisionByZero,
    InvalidOperation(String),
    ParseError(String),
    SerializationError(String),
    NegativeOperand(String),  // NEW: for input validation
}

impl std::fmt::Display for CalcError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CalcError::DivisionByZero => write!(f, "Cannot divide by zero"),
            CalcError::InvalidOperation(op) => write!(f, "Unknown operation: {}", op),
            CalcError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            CalcError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            CalcError::NegativeOperand(msg) => write!(f, "Invalid input: {}", msg),
        }
    }
}

// ---- OWNERSHIP & BORROWING (Sections 3-4) ----
fn parse_operation(input: &str) -> Result<Operation, CalcError> {
    match input.trim().to_lowercase().as_str() {
        "add" | "+"   => Ok(Operation::Add),
        "sub" | "-"   => Ok(Operation::Subtract),
        "mul" | "*"   => Ok(Operation::Multiply),
        "div" | "/"   => Ok(Operation::Divide),
        "mod" | "%"   => Ok(Operation::Modulo),  
        "pow" | "**"  => Ok(Operation::Power),   
        other => Err(CalcError::InvalidOperation(other.to_string())),
    }
}

// NEW: validates operands based on which operation will be performed
fn validate_operands(op: &Operation, a: f64, b: f64) -> Result<(), CalcError> {
    match op {
        // Modulo is undefined for negative operands in this calculator's context
        Operation::Modulo => {
            if a < 0.0 {
                return Err(CalcError::NegativeOperand(
                    "modulo requires a non-negative dividend (a >= 0)".to_string()
                ));
            }
            if b < 0.0 {
                return Err(CalcError::NegativeOperand(
                    "modulo requires a non-negative divisor (b >= 0)".to_string()
                ));
            }
        }
        // Negative base with a fractional exponent produces NaN (e.g. (-2)^0.5)
        // Negative exponent is allowed (produces fractions), but negative base is not
        Operation::Power => {
            if a < 0.0 {
                return Err(CalcError::NegativeOperand(
                    "power requires a non-negative base (a >= 0); negative bases with fractional exponents are undefined".to_string()
                ));
            }
        }
        // Add, Subtract, Multiply, Divide accept all real numbers
        _ => {}
    }
    Ok(())
}

fn execute(op: &Operation, a: f64, b: f64) -> Result<f64, CalcError> {
    match op {
        Operation::Add      => Ok(a + b),
        Operation::Subtract => Ok(a - b),
        Operation::Multiply => Ok(a * b),
        Operation::Divide   => {
            if b == 0.0 {
                Err(CalcError::DivisionByZero)
            } else {
                Ok(a / b)
            }
        }
        Operation::Modulo => {  // FIX: now has its own real implementation
            if b == 0.0 {
                Err(CalcError::DivisionByZero)
            } else {
                Ok(a % b)
            }
        }
        Operation::Power => Ok(a.powf(b)),  // NEW
    }
}

fn read_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn serialize_history(history: &CalculationHistory) -> Result<Vec<u8>, CalcError> {
    borsh::to_vec(history)
        .map_err(|e| CalcError::SerializationError(e.to_string()))
}

fn deserialize_history(data: &[u8]) -> Result<CalculationHistory, CalcError> {
    CalculationHistory::try_from_slice(data)
        .map_err(|e| CalcError::SerializationError(e.to_string()))
}

fn main() {
    println!("=== Solana CLI Calculator & Serializer ===");
    println!("Commands: add/sub/mul/div/mod/pow, 'history', 'serialize', 'quit'\n");

    let mut history = CalculationHistory { entries: Vec::new() };

    loop {
        let cmd = read_input("> ");

        match cmd.as_str() {
            "quit" | "exit" => {
                println!("Total calculations: {}", history.entries.len());
                break;
            }
            "history" => {
                if history.entries.is_empty() {
                    println!("No calculations yet.");
                } else {
                    for (i, calc) in history.entries.iter().enumerate() {
                        println!("  [{}] {} {} {} = {}",
                            i + 1, calc.operand_a, calc.operation,
                            calc.operand_b, calc.result);
                    }
                }
            }
            "serialize" => {
                match serialize_history(&history) {
                    Ok(bytes) => {
                        println!("Serialized to {} bytes: {:?}", bytes.len(), &bytes[..bytes.len().min(20)]);
                        match deserialize_history(&bytes) {
                            Ok(decoded) => println!("Deserialized back: {} entries", decoded.entries.len()),
                            Err(e) => println!("Deserialize error: {}", e),
                        }
                    }
                    Err(e) => println!("Serialize error: {}", e),
                }
            }
            _ => {
                let op = match parse_operation(&cmd) {
                    Ok(op) => op,
                    Err(e) => { println!("Error: {}", e); continue; }
                };

                let a_str = read_input("  First number: ");
                let a: f64 = match a_str.parse() {
                    Ok(v) => v,
                    Err(_) => { println!("Error: invalid number"); continue; }
                };

                let b_str = read_input("  Second number: ");
                let b: f64 = match b_str.parse() {
                    Ok(v) => v,
                    Err(_) => { println!("Error: invalid number"); continue; }
                };

                // NEW: validate before executing
                if let Err(e) = validate_operands(&op, a, b) {
                    println!("  Error: {}", e);
                    continue;
                }

                match execute(&op, a, b) {
                    Ok(result) => {
                        println!("  Result: {}", result);
                        history.entries.push(Calculation {
                            operation: format!("{:?}", op),
                            operand_a: a,
                            operand_b: b,
                            result,
                        });
                    }
                    Err(e) => println!("  Error: {}", e),
                }
            }
        }
    }
}