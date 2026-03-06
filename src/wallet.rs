use borsh::{BorshSerialize, BorshDeserialize};
use std::time::{SystemTime, UNIX_EPOCH};

// ===== EXERCISE 2: WALLET STRUCT =====

/// Represents the direction of a transaction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone, PartialEq)]
enum TransactionDirection {
    In,   // Money received/deposited
    Out,  // Money sent/withdrawn
}

/// Represents a single transaction
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct Transaction {
    amount: u64,
    direction: TransactionDirection,
    timestamp: u64,
}

/// Represents a wallet with address, balance, and transaction history
#[derive(BorshSerialize, BorshDeserialize, Debug, Clone)]
struct Wallet {
    address: String,
    balance: u64,
    transactions: Vec<Transaction>,
}

/// Custom error types for wallet operations
#[derive(Debug)]
enum WalletError {
    InsufficientFunds { required: u64, available: u64 },
}

impl std::fmt::Display for WalletError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            WalletError::InsufficientFunds { required, available } => {
                write!(f, "Insufficient funds: required {}, available {}", required, available)
            }
        }
    }
}

impl Wallet {
    /// Creates a new wallet with the given address and initial balance
    fn new(address: String, initial_balance: u64) -> Self {
        Wallet {
            address,
            balance: initial_balance,
            transactions: Vec::new(),
        }
    }

    /// Deposits the given amount into the wallet
    fn deposit(&mut self, amount: u64) -> Result<(), WalletError> {
        self.balance = self.balance.checked_add(amount)
            .expect("Balance overflow");
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time error")
            .as_secs();

        self.transactions.push(Transaction {
            amount,
            direction: TransactionDirection::In,
            timestamp,
        });

        println!("✓ Deposited: {} lamports (new balance: {})", amount, self.balance);
        Ok(())
    }

    /// Withdraws the given amount from the wallet
    fn withdraw(&mut self, amount: u64) -> Result<(), WalletError> {
        if amount > self.balance {
            return Err(WalletError::InsufficientFunds {
                required: amount,
                available: self.balance,
            });
        }

        self.balance -= amount;
        
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("Time error")
            .as_secs();

        self.transactions.push(Transaction {
            amount,
            direction: TransactionDirection::Out,
            timestamp,
        });

        println!("✓ Withdrew: {} lamports (new balance: {})", amount, self.balance);
        Ok(())
    }

    /// Returns the current balance
    fn get_balance(&self) -> u64 {
        self.balance
    }

    /// Returns the transaction history
    fn get_transactions(&self) -> &[Transaction] {
        &self.transactions
    }
}

/// Demonstrates Exercise 2: Wallet functionality with serialization
pub fn run_exercise_2() {
    println!("\n===== EXERCISE 2: WALLET STRUCT =====\n");

    // Create a new wallet
    let mut wallet = Wallet::new(
        "Dn4noZ5jgGH4wLZtD3LQ5DEi8DRubNqewP2QRfsNEqJ".to_string(),
        1000,  // Initial balance: 1000 lamports
    );

    println!(" Created wallet: {}", wallet.address);
    println!(" Initial balance: {} lamports\n", wallet.get_balance());

    // Perform deposits
    println!("--- Performing Transactions ---");
    if let Err(e) = wallet.deposit(500) {
        println!("Deposit failed: {}", e);
    }

    if let Err(e) = wallet.deposit(250) {
        println!("Deposit failed: {}", e);
    }

    // Perform a withdrawal
    if let Err(e) = wallet.withdraw(300) {
        println!("Withdrawal failed: {}", e);
    }

    // Try to withdraw more than we have (should fail)
    println!("\n--- Attempting to withdraw more than available ---");
    match wallet.withdraw(2000) {
        Ok(_) => println!("Withdrawal succeeded"),
        Err(e) => println!("{}", e),
    }

    // Display final state
    println!("\n--- Final Wallet State ---");
    println!("Final balance: {} lamports", wallet.get_balance());
    println!("Total transactions: {}", wallet.get_transactions().len());
    for (i, tx) in wallet.get_transactions().iter().enumerate() {
        let direction = match tx.direction {
            TransactionDirection::In => "IN ",
            TransactionDirection::Out => "OUT",
        };
        println!("  [{}] {} {} lamports (timestamp: {})", i + 1, direction, tx.amount, tx.timestamp);
    }

    // Serialize the wallet
    println!("\n--- Serialization Test ---");
    match borsh::to_vec(&wallet) {
        Ok(serialized) => {
            println!("✓ Serialized wallet to {} bytes", serialized.len());
            println!("  First 32 bytes: {:?}...", &serialized[..serialized.len().min(32)]);

            // Deserialize the wallet
            match Wallet::try_from_slice(&serialized) {
                Ok(deserialized) => {
                    println!("✓ Deserialized wallet successfully");
                    println!("  Address: {}", deserialized.address);
                    println!("  Balance: {} lamports", deserialized.get_balance());
                    println!("  Transactions: {}", deserialized.get_transactions().len());

                    // Verify the data is identical
                    if deserialized.balance == wallet.balance
                        && deserialized.address == wallet.address
                        && deserialized.transactions.len() == wallet.transactions.len()
                    {
                        println!("✓ Serialization round-trip successful - data intact!");
                    }
                }
                Err(e) => println!("Deserialization failed: {}", e),
            }
        }
        Err(e) => println!("Serialization failed: {}", e),
    }

    println!("\n===== Exercise 2 Complete =====\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wallet_creation() {
        let wallet = Wallet::new("test_address".to_string(), 1000);
        assert_eq!(wallet.address, "test_address");
        assert_eq!(wallet.get_balance(), 1000);
        assert_eq!(wallet.get_transactions().len(), 0);
    }

    #[test]
    fn test_deposit() {
        let mut wallet = Wallet::new("test_address".to_string(), 1000);
        let result = wallet.deposit(500);
        assert!(result.is_ok());
        assert_eq!(wallet.get_balance(), 1500);
        assert_eq!(wallet.get_transactions().len(), 1);
    }

    #[test]
    fn test_withdraw() {
        let mut wallet = Wallet::new("test_address".to_string(), 1000);
        let result = wallet.withdraw(300);
        assert!(result.is_ok());
        assert_eq!(wallet.get_balance(), 700);
        assert_eq!(wallet.get_transactions().len(), 1);
    }

    #[test]
    fn test_withdraw_insufficient_funds() {
        let mut wallet = Wallet::new("test_address".to_string(), 100);
        let result = wallet.withdraw(500);
        assert!(result.is_err());
        assert_eq!(wallet.get_balance(), 100);  // Balance unchanged
        assert_eq!(wallet.get_transactions().len(), 0);  // No transaction recorded
    }

    #[test]
    fn test_multiple_transactions() {
        let mut wallet = Wallet::new("test_address".to_string(), 1000);
        wallet.deposit(500).unwrap();
        wallet.withdraw(200).unwrap();
        wallet.deposit(100).unwrap();
        
        assert_eq!(wallet.get_balance(), 1400);
        assert_eq!(wallet.get_transactions().len(), 3);
    }

    #[test]
    fn test_serialization() {
        let mut wallet = Wallet::new("test_address".to_string(), 1000);
        wallet.deposit(500).unwrap();
        wallet.withdraw(200).unwrap();

        let serialized = borsh::to_vec(&wallet).unwrap();
        let deserialized = Wallet::try_from_slice(&serialized).unwrap();

        assert_eq!(deserialized.address, wallet.address);
        assert_eq!(deserialized.balance, wallet.balance);
        assert_eq!(deserialized.transactions.len(), wallet.transactions.len());
    }

    #[test]
    fn test_transaction_direction() {
        let mut wallet = Wallet::new("test_address".to_string(), 1000);
        wallet.deposit(500).unwrap();
        wallet.withdraw(200).unwrap();

        let txs = wallet.get_transactions();
        assert_eq!(txs[0].direction, TransactionDirection::In);
        assert_eq!(txs[1].direction, TransactionDirection::Out);
    }
}
