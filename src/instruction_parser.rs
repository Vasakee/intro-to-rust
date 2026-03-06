use borsh::{BorshSerialize, BorshDeserialize};

// ===== EXERCISE 3: INSTRUCTION PARSER =====

/// Represents different Solana program instructions
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
enum ProgramInstruction {
    /// Create a new account with specified space
    CreateAccount { space: u64 },
    /// Transfer lamports between accounts
    Transfer { lamports: u64 },
    /// Close an account
    CloseAccount,
}

/// Custom error for instruction processing
#[derive(Debug)]
enum InstructionError {
    DeserializationError(String),
    // InvalidInstruction(String),  // REMOVED: never used
}

impl std::fmt::Display for InstructionError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            InstructionError::DeserializationError(msg) => write!(f, "Deserialization error: {}", msg),
            // InstructionError::InvalidInstruction(msg) => write!(f, "Invalid instruction: {}", msg),  // REMOVED
        }
    }
}

/// Processes a byte slice by deserializing it into a ProgramInstruction and printing what it would do
fn process_instruction(data: &[u8]) -> Result<(), InstructionError> {
    // Deserialize the instruction from bytes
    let instruction = ProgramInstruction::try_from_slice(data)
        .map_err(|e| InstructionError::DeserializationError(e.to_string()))?;

    // Process the instruction based on its type
    match instruction {
        ProgramInstruction::CreateAccount { space } => {
            println!("CreateAccount Instruction:");
            println!("   - Action: Create a new account");
            println!("   - Space allocated: {} bytes", space);
            println!("   - This would create an account with {} bytes of data space", space);
        }
        ProgramInstruction::Transfer { lamports } => {
            println!("Transfer Instruction:");
            println!("   - Action: Transfer lamports between accounts");
            println!("   - Amount: {} lamports", lamports);
            println!("   - This would transfer {} lamports from sender to receiver", lamports);
        }
        ProgramInstruction::CloseAccount => {
            println!(" CloseAccount Instruction:");
            println!("   - Action: Close an account");
            println!("   - This would close the account and return rent to the owner");
        }
    }

    Ok(())
}

/// Helper function to serialize an instruction and return the bytes
fn serialize_instruction(instruction: &ProgramInstruction) -> Result<Vec<u8>, InstructionError> {
    borsh::to_vec(instruction)
        .map_err(|e| InstructionError::DeserializationError(e.to_string()))
}

/// Demonstrates Exercise 3: Instruction Parser functionality
pub fn run_exercise_3() {
    println!("\n===== EXERCISE 3: INSTRUCTION PARSER =====\n");

    // Create sample instructions
    let instructions = vec![
        ProgramInstruction::CreateAccount { space: 1024 },
        ProgramInstruction::Transfer { lamports: 1000000 }, // 1 SOL in lamports
        ProgramInstruction::CloseAccount,
        ProgramInstruction::CreateAccount { space: 2048 },
        ProgramInstruction::Transfer { lamports: 500000 }, // 0.5 SOL in lamports
    ];

    println!("--- Creating and Processing Instructions ---\n");

    // Process each instruction: serialize -> deserialize -> print
    for (i, instruction) in instructions.iter().enumerate() {
        println!("Instruction {}: {:?}", i + 1, instruction);

        // Serialize the instruction
        match serialize_instruction(instruction) {
            Ok(bytes) => {
                println!("✓ Serialized to {} bytes: {:?}", bytes.len(), &bytes[..bytes.len().min(16)]);

                // Process (deserialize and print what it does)
                match process_instruction(&bytes) {
                    Ok(_) => println!("✓ Processed successfully\n"),
                    Err(e) => println!("Processing failed: {}\n", e),
                }
            }
            Err(e) => println!("Serialization failed: {}\n", e),
        }
    }

    // Demonstrate error handling with invalid data
    println!("--- Error Handling Test ---");
    let invalid_data = vec![255, 255, 255, 255]; // Invalid instruction data
    println!("Testing with invalid data: {:?}", invalid_data);
    match process_instruction(&invalid_data) {
        Ok(_) => println!("Unexpectedly succeeded"),
        Err(e) => println!(" Expected error: {}\n", e),
    }

    println!("===== Exercise 3 Complete =====\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_account_instruction() {
        let instruction = ProgramInstruction::CreateAccount { space: 1024 };
        let bytes = serialize_instruction(&instruction).unwrap();
        let processed = process_instruction(&bytes);
        assert!(processed.is_ok());
    }

    #[test]
    fn test_transfer_instruction() {
        let instruction = ProgramInstruction::Transfer { lamports: 1000000 };
        let bytes = serialize_instruction(&instruction).unwrap();
        let processed = process_instruction(&bytes);
        assert!(processed.is_ok());
    }

    #[test]
    fn test_close_account_instruction() {
        let instruction = ProgramInstruction::CloseAccount;
        let bytes = serialize_instruction(&instruction).unwrap();
        let processed = process_instruction(&bytes);
        assert!(processed.is_ok());
    }

    #[test]
    fn test_serialization_round_trip() {
        let original = ProgramInstruction::Transfer { lamports: 500000 };
        let bytes = serialize_instruction(&original).unwrap();
        let deserialized = ProgramInstruction::try_from_slice(&bytes).unwrap();
        assert_eq!(original, deserialized);
    }

    #[test]
    fn test_invalid_data_deserialization() {
        let invalid_data = vec![99, 99, 99]; // Invalid enum variant
        let result = process_instruction(&invalid_data);
        assert!(result.is_err());
    }

    #[test]
    fn test_instruction_sizes() {
        // CreateAccount should be larger due to u64 space field
        let create = serialize_instruction(&ProgramInstruction::CreateAccount { space: 1024 }).unwrap();
        // CloseAccount should be smallest (just enum discriminant)
        let close = serialize_instruction(&ProgramInstruction::CloseAccount).unwrap();
        // Transfer should be medium size
        let transfer = serialize_instruction(&ProgramInstruction::Transfer { lamports: 1000000 }).unwrap();

        assert!(create.len() > close.len());
        assert!(transfer.len() > close.len());
    }
}
