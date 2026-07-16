#[path = "cpu/cpu.rs"]
mod cpu;

use cpu::Cpu;

fn main() {
	let cpu = Cpu::new;

	println!("Hello, world!");
}
