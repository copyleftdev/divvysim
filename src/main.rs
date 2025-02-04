// src/main.rs

// Import the split module (ensures that src/split.rs is compiled)
mod split;

use rust_decimal::Decimal;

fn main() {
    // Initialize logging if desired (optional)
    env_logger::init();

    // Example: split 100.01 among 4 recipients using scale 2.
    // Here, 10001 minimal units at scale 2 represent 100.01.
    let amount = Decimal::from_i128_with_scale(10001, 2);
    let recipients = 4;
    let scale = 2;

    // Call the split function from the split module.
    let splits = split::split_decimal(amount, recipients, scale);

    // Print the split results.
    println!(
        "Splitting {} among {} recipients at scale {} yields: {:?}",
        amount, recipients, scale, splits
    );

    // Print each recipient’s share.
    for (i, share) in splits.iter().enumerate() {
        println!("Recipient {}: {}", i + 1, share);
    }
}
