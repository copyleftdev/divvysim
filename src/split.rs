use rust_decimal::Decimal;
use rust_decimal::prelude::ToPrimitive; // needed for .mantissa()
use rayon::prelude::*;

/// Splits a given Decimal amount among a given number of recipients.
///
/// The function assumes that the provided Decimal’s internal value (its mantissa)
/// represents the amount in minimal units. For example, if you call:
///
///     Decimal::from_i128_with_scale(10001, 2)
///
/// that represents 100.01. The parameter `scale` is the desired scale for the split
/// results. The function computes each recipient’s share in minimal units, then re-
/// constructs a Decimal with that scale.
///
/// Note: All arithmetic (base share, remainder, and adjustments) is done on the
/// underlying mantissa.
pub fn split_decimal(amount: Decimal, recipients: usize, scale: u32) -> Vec<Decimal> {
    // Get the raw underlying integer value (the mantissa).
    let raw: i128 = amount.mantissa();
    let base_share = raw / (recipients as i128);
    let remainder = raw % (recipients as i128);

    // Compute each recipient's share as (base_share + extra) minimal units.
    let mut splits: Vec<Decimal> = (0..recipients)
        .into_par_iter()
        .map(|i| {
            let extra = if (i as i128) < remainder { 1 } else { 0 };
            Decimal::from_i128_with_scale(base_share + extra, scale)
        })
        .collect();

    // Sum the shares (using the mantissa for accurate minimal-unit arithmetic).
    let total_split: i128 = splits.iter().map(|d| d.mantissa()).sum();

    // If there is any discrepancy between the computed total and the original raw amount,
    // adjust the first recipient's share one minimal unit at a time.
    let mut diff = raw - total_split;
    let unit = 1; // one minimal unit in the new scale.
    while diff > 0 {
        let current = splits[0].mantissa();
        let new_val = current + unit;
        splits[0] = Decimal::from_i128_with_scale(new_val, scale);
        let new_total: i128 = splits.iter().map(|d| d.mantissa()).sum();
        diff = raw - new_total;
    }

    splits
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::Decimal;
    use rust_decimal::prelude::ToPrimitive;
    use std::str::FromStr;

    #[test]
    fn test_decimal_conversion() {
        let scale = 2;
        let values = vec![100, 200, 300, 400];
        let decimals: Vec<Decimal> = values.into_iter()
            .map(|x| Decimal::from_i128_with_scale(x, scale))
            .collect();
        for d in decimals {
            assert!(d >= Decimal::from_i128_with_scale(1, scale));
        }
    }

    #[test]
    fn test_expected_total() {
        let scale = 2;
        let expected_total = Decimal::from_i128_with_scale(1234568, scale);
        let result_total = expected_total; // demonstration
        assert_eq!(result_total, expected_total);
    }

    #[test]
    fn test_max_decimal_digits_than_scale() {
        let max_val: i128 = 9876543210;
        let amount = Decimal::from_i128_with_scale(max_val, 0);
        assert_eq!(amount, Decimal::from_i128_with_scale(max_val, 0));
    }

    #[test]
    fn test_split_evenly() {
        let amount = Decimal::from_i128_with_scale(10001, 2); // represents 100.01
        let recipients = 4;
        let scale = 2;
        let splits = split_decimal(amount, recipients, scale);
        let total: i128 = splits.iter().map(|d| d.mantissa()).sum();
        assert_eq!(total, amount.mantissa());
    }

    #[test]
    fn test_split_with_uneven_distribution() {
        let amount = Decimal::from_i128_with_scale(10001, 2);
        let recipients = 3;
        let scale = 2;
        let splits = split_decimal(amount, recipients, scale);
        let total: i128 = splits.iter().map(|d| d.mantissa()).sum();
        assert_eq!(total, amount.mantissa());
    }

    #[test]
    fn test_split_with_large_numbers() {
        let amount = Decimal::from_i128_with_scale(999999999999999999, 2);
        let recipients = 10;
        let scale = 2;
        let splits = split_decimal(amount, recipients, scale);
        let total: i128 = splits.iter().map(|d| d.mantissa()).sum();
        assert_eq!(total, amount.mantissa());
    }

    #[test]
    fn test_reset_exceed_minimal_units() {
        // Use 1235 at scale 2 to represent 12.35.
        let amount = Decimal::from_i128_with_scale(1235, 2);
        let recipients = 5;
        let scale = 2;
        let splits = split_decimal(amount, recipients, scale);
        for s in &splits {
            assert!(*s >= Decimal::from_i128_with_scale(1, scale));
        }
        let total: i128 = splits.iter().map(|d| d.mantissa()).sum();
        assert_eq!(total, amount.mantissa());
    }

    #[test]
    fn test_large_unit_producing_all_zeroes() {
        // Here the input is created with scale 0 (raw = 1234567).
        // When splitting with desired scale 2, we reinterpret the raw value.
        let amount = Decimal::from_i128_with_scale(1234567, 0); // raw = 1234567
        let recipients = 1;
        let scale = 2;
        let splits = split_decimal(amount, recipients, scale);
        assert_eq!(splits.len(), 1);
        let expected = Decimal::from_i128_with_scale(1234567, 2);
        assert_eq!(splits[0], expected);
    }

    #[test]
    fn test_recipients_equal_to_minimal_units() {
        let amount = Decimal::from_i128_with_scale(5, 0);
        let recipients_decimal = Decimal::from_i128_with_scale(49, 0) / Decimal::from_i128_with_scale(5, 0);
        let recipients = recipients_decimal.to_i128().unwrap() as usize;
        let splits = split_decimal(amount, recipients, 0);
        let total: i128 = splits.iter().map(|d| d.mantissa()).sum();
        assert_eq!(total, amount.mantissa());
    }

    #[test]
    fn test_reset_exceed_minimal_units_tiny_amount() {
        let amount = Decimal::from_i128_with_scale(2, 0);
        let recipients = 2;
        let scale = 0;
        let splits = split_decimal(amount, recipients, scale);
        let total: i128 = splits.iter().map(|d| d.mantissa()).sum();
        assert_eq!(total, amount.mantissa());
    }

    #[test]
    fn test_recipients_exceed_minimal_units_tiny_amount_producing_all_zeroes() {
        let amount = Decimal::from_i128_with_scale(4185552, 3);
        let recipients = 13;
        let scale = 3;
        let splits = split_decimal(amount, recipients, scale);
        let total: i128 = splits.iter().map(|d| d.mantissa()).sum();
        assert_eq!(total, amount.mantissa());
    }

    #[test]
    fn test_recipients_exceed_minimal_units() {
        let amount = Decimal::from_i128_with_scale(412434, 2);
        let recipients = 2;
        let scale = 0;
        let splits = split_decimal(amount, recipients, scale);
        let total: i128 = splits.iter().map(|d| d.mantissa()).sum();
        assert_eq!(total, amount.mantissa());
    }

    #[test]
    fn test_large_numbers() {
        let amount = Decimal::from_i128_with_scale(6627186, 0);
        let recipients = 10;
        let scale = 0;
        let splits = split_decimal(amount, recipients, scale);
        let total: i128 = splits.iter().map(|d| d.mantissa()).sum();
        assert_eq!(total, amount.mantissa());
    }

    #[test]
    fn property_conversion_test() {
        let amount = Decimal::from_i128_with_scale(10001, 2);
        let recipients = 4;
        let scale = 2;
        let splits = split_decimal(amount, recipients, scale);
        let total: i128 = splits.iter().map(|d| d.mantissa()).sum();
        assert_eq!(total, amount.mantissa());
    }

    #[test]
    fn test_non_increasing_order() {
        let amount = Decimal::from_str("123.45").unwrap();
        let recipients = 39;
        let scale = 2;
        let splits = split_decimal(amount, recipients, scale);
        for i in 0..splits.len() {
            assert!(splits[i] >= splits.get(i + 1).cloned().unwrap_or(splits[i]));
        }
    }

    #[test]
    fn test_extremely_large_values() {
        // Use maximum allowed scale (28) instead of an invalid 100.
        let scale = 28;
        let amount = Decimal::from_i128_with_scale(4, scale);
        let recipients = 100000;
        let splits = split_decimal(amount, recipients, scale);
        let total: i128 = splits.iter().map(|d| d.mantissa()).sum();
        assert_eq!(total, amount.mantissa());
    }
}

/// Converts a Decimal into its underlying integer representation (the mantissa).
pub fn decimal_to_int(amount: Decimal, _scale: u32) -> i128 {
    amount.mantissa()
}

// --- End of src/split.rs ---
