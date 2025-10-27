pub fn add_padded_numbering(pad: usize, n: usize) -> String {
	format!("{n:0pad$}")
}

pub fn print_header() {
	println!("\n| {:^40} | {:^40} |", "CURRENT", "NEW");
	let separator = "=".repeat(40);
	println!("| {separator:<40} | {separator:<40} |");
}

#[cfg(test)]
mod tests {
	use crate::helpers::add_padded_numbering;
	use test_case::test_case;

	#[test_case(3, 42, "042"; "3-padding for 42")]
	#[test_case(2, 42, "42"; "2-padding for 42")]
	#[test_case(6, 4, "000004"; "6-padding for 4")]
	fn test_add_padded_numbering(pad: usize, n: usize, expected: &str) {
		let result = add_padded_numbering(pad, n);
		assert_eq!(result, expected);
		assert_ne!(result, String::new(), "Function returned an empty string!");
	}
}
