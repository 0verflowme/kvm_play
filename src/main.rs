use kvm_bindings::kvm_userspace_memory_region;
use kvm_ioctls::{Kvm, VcpuExit};
use std::ptr::null_mut;
use nix::libc;

struct Device {
	mmio_start: u64,
	// mmio_end: u64,
	data: Vec<u8>,
}

impl Device {
	fn new(start: u64, size: usize) -> Self {
		Device {
			mmio_start: start,
			// mmio_end: start + size as u64,
			data: vec![0; size],
		}
	}

	fn write(&mut self, addr: u64, value: &[u8]) {
		let offset = (addr - self.mmio_start) as usize;
		for (i, v) in value.iter().enumerate() {
			if offset + i < self.data.len() {
				self.data[offset + i] = *v;
			}
		}
		println!("Device at address {:x} received value: {}", addr, self.data[offset]);
	}

	fn read(&self, addr: u64) -> u8 {
		let offset = (addr - self.mmio_start) as usize;
		self.data[offset]
	}
}

fn main() {
	let kvm = Kvm::new().expect("Failed to open KVM driver");

	let vm = kvm.create_vm().expect("Failed to create VM");
	let mem_size = 0x1000;
	let guest_addr = 0x1000;
	let load_addr: *mut u8 = unsafe {
		libc::mmap(
			null_mut(),
			mem_size,
			libc::PROT_READ | libc::PROT_WRITE,
			libc::MAP_SHARED | libc::MAP_ANONYMOUS,
			-1,
			0,
			) as *mut u8
	};
	assert!(!load_addr.is_null(), "Failed to allocate guest memory");
	let machine_code: &[u8] = include_bytes!("../guest");

	unsafe {
		libc::memcpy(load_addr as *mut libc::c_void, machine_code.as_ptr() as *const libc::c_void, machine_code.len());
	}

	let slot = 0;
	let memory_region = kvm_userspace_memory_region {
		slot,
		guest_phys_addr: guest_addr,
		memory_size: mem_size as u64,
		userspace_addr: load_addr as u64,
		flags: 0,
	};

	unsafe {
		vm.set_user_memory_region(memory_region).expect("Failed to set memory region");
	}

	let mut vcpu = vm.create_vcpu(0).expect("Failed to create VCPU");

	let mut vcpu_sregs = vcpu.get_sregs().expect("Failed to get sregs");
	
	vcpu_sregs.cs.base = 0;
	vcpu_sregs.cs.selector = 0;
	vcpu_sregs.ss.base = 0;
	vcpu_sregs.ss.selector = 0;
	vcpu_sregs.ds.base = 0;
	vcpu_sregs.ds.selector = 0;
	vcpu_sregs.es.base = 0;
	vcpu_sregs.es.selector = 0;
	vcpu_sregs.fs.base = 0;
	vcpu_sregs.fs.selector = 0;
	vcpu_sregs.gs.selector = 0;
	
	vcpu.set_sregs(&vcpu_sregs).expect("Failed to set sregs");
	let mut vcpu_regs = vcpu.get_regs().expect("Failed to get regs");
	vcpu_regs.rip = guest_addr;
	vcpu_regs.rflags = 0x0000000000000002;
	vcpu.set_regs(&vcpu_regs).expect("Failed to set registers");
	let mut device = Device::new(0x0, 0x1000);

	loop {
		match vcpu.run().expect("Failed to run VCPU") {
			VcpuExit::IoOut(port, data) => {
				println!("IO operation detected: Port: {:x}, Data: {:?}", port, data);
				if port == 0x3f8 {
					println!("Dummy device received data: {}", data[0]);
					match data[0] {
						0 ..= 127 => print!("{}", data[0] as char),
						_ => println!("\nReceived non-ASCII data: {}", data[0]),
					}
				}
				if data[0] == 0x42 {
					println!("Byte 0x42 was written to the dummy device!");
				}
			},
			VcpuExit::Hlt => {
				println!("HLT encountered, shutting down VM");
				break;
			},
			VcpuExit::MmioWrite(addr, data) => {
				println!("Handling MMIO Write to address: {:x}", addr);
				device.write(addr, data);
			},
			VcpuExit::MmioRead(addr, data_len) => {
				println!("Handling MMIO Read from address: {:x}", addr);
				let data = device.read(addr); // Perform read
				println!("Read data: {:02x} from device at address: {:x} data length: {:?}", data, addr, data_len);
				// continue;
			},
			exit_reason => panic!("Unexpected exit from VCPU: {:?}", exit_reason),
		}
	}
}
