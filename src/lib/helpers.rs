pub fn add_padded_numbering(pad: usize, n: usize) -> String {
	format!("{n:0pad$}")
}

pub fn print_header() {
	println!("\n| {:^40} | {:^40} |", "CURRENT", "NEW");
	let separator = "=".repeat(40);
	println!("| {separator:<40} | {separator:<40} |");
}
