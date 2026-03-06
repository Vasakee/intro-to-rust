use borsh::{BorshSerialize, BorshDeserialize};
use std::io::{self, Write};

mod wallet;  // Exercise 2: Wallet struct
mod instruction_parser;  // Exercise 3: Instruction parser

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
    // ParseError(String),  // REMOVED: never used
    SerializationError(String),
    NegativeOperand(String),  // NEW: for input validation
}

impl std::fmt::Display for CalcError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            CalcError::DivisionByZero => write!(f, "Cannot divide by zero"),
            CalcError::InvalidOperation(op) => write!(f, "Unknown operation: {}", op),
            // CalcError::ParseError(msg) => write!(f, "Parse error: {}", msg),  // REMOVED
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

// EXERCISE 1: INPUT VALIDATION - Validates operands based on operation type
// Rejects negative numbers for operations that don't support them
fn validate_operands(op: &Operation, a: f64, b: f64) -> Result<(), CalcError> {
    match op {
        // Modulo is undefined for negative operands - both must be non-negative
        Operation::Modulo => {
            if a < 0.0 {
                return Err(CalcError::NegativeOperand(
                    "Modulo operation: dividend (first number) must be non-negative (a >= 0)".to_string()
                ));
            }
            if b < 0.0 {
                return Err(CalcError::NegativeOperand(
                    "Modulo operation: divisor (second number) must be non-negative (b >= 0)".to_string()
                ));
            }
        }
        // Power operation: negative base with fractional exponent is undefined
        Operation::Power => {
            if a < 0.0 {
                return Err(CalcError::NegativeOperand(
                    "Power operation: base (first number) must be non-negative (a >= 0). \
                    Negative bases with fractional exponents produce undefined results (NaN)".to_string()
                ));
            }
        }
        // Add, Subtract, Multiply, Divide accept all real numbers (including negative)
        _ => {}
    }
    Ok(())
}

// EXERCISE 1: Additional validation function for user input
// Checks if a number is valid for the chosen operation
fn validate_input_for_operation(op: &Operation, operand_num: u32, value: f64) -> Result<(), CalcError> {
    let operand_name = if operand_num == 1 { "First number" } else { "Second number" };
    
    match op {
        Operation::Modulo => {
            if value < 0.0 {
                return Err(CalcError::NegativeOperand(
                    format!("{} in modulo operation must be non-negative (>= 0)", operand_name)
                ));
            }
        }
        Operation::Power => {
            if operand_num == 1 && value < 0.0 {
                return Err(CalcError::NegativeOperand(
                    format!("{} (base) in power operation must be non-negative (>= 0)", operand_name)
                ));
            }
        }
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
        Operation::Modulo => {  
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
    println!("=== Solana CLI Tool - Exercises ===\n");
    println!("Choose an exercise:");
    println!("  1. Calculator with Input Validation (Exercise 1)");
    println!("  2. Wallet Struct with Transactions (Exercise 2)");
    println!("  3. Instruction Parser (Exercise 3)\n");

    print!("> Enter exercise number (1-3): ");
    io::stdout().flush().unwrap();
    
    let mut choice = String::new();
    io::stdin().read_line(&mut choice).unwrap();
    
    match choice.trim() {
        "1" => run_calculator(),
        "2" => wallet::run_exercise_2(),
        "3" => instruction_parser::run_exercise_3(),
        _ => println!("Invalid choice"),
    }
}

fn run_calculator() {
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

                // EXERCISE 1: Validate first input for this operation
                if let Err(e) = validate_input_for_operation(&op, 1, a) {
                    println!("  ❌ {}", e);
                    continue;
                }

                let b_str = read_input("  Second number: ");
                let b: f64 = match b_str.parse() {
                    Ok(v) => v,
                    Err(_) => { println!("Error: invalid number"); continue; }
                };

                // EXERCISE 1: Validate second input for this operation
                if let Err(e) = validate_input_for_operation(&op, 2, b) {
                    println!("  ❌ {}", e);
                    continue;
                }

                // NEW: validate before executing
                if let Err(e) = validate_operands(&op, a, b) {
                    println!("  ❌ Error: {}", e);
                    continue;
                }

                match execute(&op, a, b) {
                    Ok(result) => {
                        println!("  ✓ Result: {}", result);
                        history.entries.push(Calculation {
                            operation: format!("{:?}", op),
                            operand_a: a,
                            operand_b: b,
                            result,
                        });
                    }
                    Err(e) => println!("  ❌ Error: {}", e),
                }
            }
        }
    }
}