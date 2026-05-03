use sysinfo::System;
use local_ip_address::local_ip;

#[derive(Default, Clone)]
pub struct Details {
    pub name: String,
    pub kernel: String,
    pub version: String,
    pub host_name: String,
    pub physical_cores: String,
    pub threaded_cores: String,
    pub mem_used: String,
    pub mem_avail: String,
    pub mem_total: String,
    pub cpu_brand: String,
    pub cpu_freq: String,
    pub ip_addr: String,
}

impl Details {
    pub fn new() -> Details {
        Details::default()
    }

    pub fn get_ip(&mut self) {
        match local_ip() {
            Ok(ip) => self.ip_addr = format!("{}", ip),
            Err(err) => self.ip_addr = format!("Failed to get ip address {}", err),
        };
    }

    pub fn get_os(&mut self) {
        let mut sys = System::new_all();
        sys.refresh_all();

        match local_ip() {
            Ok(ip) => self.ip_addr = format!("{}", ip),
            Err(err) => self.ip_addr = format!("Failed to get ip address {}", err),
        };

        // Detect the actual distro instead of using the generic system name
        // which usually just reports "Linux".
        let mut detected_os = String::new();

        if let Ok(content) = std::fs::read_to_string("/etc/os-release") {
            let mut found = false;

            for line in content.lines() {
                if line.starts_with("PRETTY_NAME=") {
                    let name = line
                        .trim_start_matches("PRETTY_NAME=")
                        .trim_matches('"')
                        .trim();

                    if name.contains("Mint") {
                        detected_os = name.to_string();
                        found = true;
                        break;
                    }

                    detected_os = name.to_string();
                }
            }

            if !found && detected_os.is_empty() {
                for line in content.lines() {
                    if line.starts_with("NAME=") {
                        let name = line
                            .trim_start_matches("NAME=")
                            .trim_matches('"')
                            .trim();
                        detected_os = name.to_string();
                        break;
                    }
                }
            }
        }

        if detected_os.is_empty() {
            if let Ok(content) = std::fs::read_to_string("/etc/lsb-release") {
                for line in content.lines() {
                    if line.starts_with("DISTRIB_DESCRIPTION=") {
                        let name = line
                            .trim_start_matches("DISTRIB_DESCRIPTION=")
                            .trim_matches('"')
                            .trim();
                        detected_os = name.to_string();
                        break;
                    }
                }
            }
        }

        if !detected_os.is_empty() {
            if detected_os.contains("Mint") {
                let mint_parts: Vec<&str> = detected_os.split_whitespace().collect();
                if mint_parts.len() >= 3 {
                    self.name = format!("{} {}", mint_parts[0], mint_parts[1]);
                } else {
                    self.name = detected_os;
                }
            } else {
                self.name = detected_os;
            }
        } else if let Some(alpha) = System::name() {
            self.name = alpha;
        }

        if let Some(beta) = System::kernel_version() {
            self.kernel = beta;
        }

        if let Some(gamma) = System::os_version() {
            self.version = gamma;
        }

        if let Some(delta) = System::host_name() {
            self.host_name = delta;
        }

        self.physical_cores = match System::physical_core_count() {
            Some(count) => format!("{}", count),
            None => "Unknown".to_string(),
        };
        self.threaded_cores = format!("{}", sys.cpus().len());
        self.mem_total = format!("{:.2} GB", sys.total_memory() as f64 / 1024.0 / 1024.0);
        self.mem_avail = format!("{:.2} GB", sys.available_memory() as f64 / 1024.0 / 1024.0);
        self.mem_used = format!("{:.2} GB", sys.used_memory() as f64 / 1024.0 / 1024.0);
        if let Some(cpu) = sys.cpus().first() {
            self.cpu_freq = format!("{:.2} GHz", cpu.frequency() as f64 / 1000.0);
            self.cpu_brand = cpu.brand().to_string();
        }
    }

    #[allow(dead_code)]
    pub fn print_os(&mut self) {
        println!("System Name      = {}", self.name);
        println!("System Kernel    = {}", self.kernel);
        println!("Version          = {}", self.version);
        println!("Host Name        = {}", self.host_name);
        println!("Physical Cores   = {}", self.physical_cores);
        println!("Threaded Cores   = {}", self.threaded_cores);
        println!("Total memory     = {}", self.mem_total);
        println!("Available Memory = {}", self.mem_avail);
        println!("Used Memory      = {}", self.mem_used);
        println!("CPU Frequency    = {}", self.cpu_freq);
        println!("CPU Vendor       = {}", self.cpu_brand);
    }

    pub fn format_os(&mut self) -> String {
        self.get_os();
        self.get_ip();

        let mut output = String::new();

        output.push_str("SYSTEM DETAILS\n");

        output.push_str("\nSYSTEM\n");
        output.push_str(&format!("System Name      : {}\n", self.name));
        output.push_str(&format!("System Kernel    : {}\n", self.kernel));
        output.push_str(&format!("Version          : {}\n", self.version));
        output.push_str(&format!("Host Name        : {}\n", self.host_name));

        output.push_str("\nCPU\n");
        output.push_str(&format!("Physical Cores   : {}\n", self.physical_cores));
        output.push_str(&format!("Threaded Cores   : {}\n", self.threaded_cores));
        output.push_str(&format!("CPU Frequency    : {}\n", self.cpu_freq));
        output.push_str(&format!("CPU Vendor       : {}\n", self.cpu_brand));

        output.push_str("\nMEMORY\n");
        output.push_str(&format!("Total Memory     : {}\n", self.mem_total));
        output.push_str(&format!("Available Memory : {}\n", self.mem_avail));
        output.push_str(&format!("Used Memory      : {}\n", self.mem_used));

        output.push_str("\nNETWORK\n");
        output.push_str(&format!("IP Address       : {}\n", self.ip_addr));

        output
    }
}
