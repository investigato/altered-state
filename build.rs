use omnicopy_to_output::copy_to_output;

fn main() {
	copy_to_output("src/wwwroot").expect("Could not copy wwwroot");
	copy_to_output("config.json").expect("Could not copy config.json");
}