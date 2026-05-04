//! WASM-side system info — pulls from the browser via `web_sys`.
//!
//! Mirrors the native `details` module's `format_os()` -> String entry
//! point. The browser exposes a different (smaller, privacy-restricted)
//! info set than `sysinfo`: no real CPU brand or kernel, no IP, no
//! hostname. What we *can* get: userAgent (browser + OS family),
//! navigator.platform, hardwareConcurrency (logical CPU thread count),
//! screen size, viewport size, language, online state.

#[derive(Default, Clone)]
pub struct Details {
    pub user_agent: String,
    pub platform: String,
    pub language: String,
    pub threaded_cores: String,
    pub screen: String,
    pub viewport: String,
    pub online: String,
}

impl Details {
    pub fn new() -> Details {
        Details::default()
    }

    pub fn gather(&mut self) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let nav = window.navigator();

        self.user_agent = nav.user_agent().unwrap_or_default();
        self.platform = nav.platform().unwrap_or_default();
        self.language = nav.language().unwrap_or_default();
        self.threaded_cores = format!("{}", nav.hardware_concurrency());
        self.online = format!("{}", nav.on_line());

        if let Ok(screen) = window.screen() {
            let w = screen.width().unwrap_or(0);
            let h = screen.height().unwrap_or(0);
            let depth = screen.color_depth().unwrap_or(0);
            self.screen = format!("{}x{} ({} bpp)", w, h, depth);
        }

        let vw = window.inner_width().ok().and_then(|v| v.as_f64()).unwrap_or(0.0);
        let vh = window.inner_height().ok().and_then(|v| v.as_f64()).unwrap_or(0.0);
        self.viewport = format!("{}x{}", vw as i32, vh as i32);
    }

    pub fn format_os(&mut self) -> String {
        self.gather();

        let mut output = String::new();
        output.push_str("SYSTEM DETAILS (browser)\n");

        output.push_str("\nBROWSER\n");
        output.push_str(&format!("User Agent       : {}\n", self.user_agent));
        output.push_str(&format!("Platform         : {}\n", self.platform));
        output.push_str(&format!("Language         : {}\n", self.language));
        output.push_str(&format!("Online           : {}\n", self.online));

        output.push_str("\nCPU\n");
        output.push_str(&format!(
            "Threaded Cores   : {} (navigator.hardwareConcurrency)\n",
            self.threaded_cores
        ));

        output.push_str("\nDISPLAY\n");
        output.push_str(&format!("Screen           : {}\n", self.screen));
        output.push_str(&format!("Viewport         : {}\n", self.viewport));

        output.push_str(
            "\nNote: native-only info (kernel, RAM, IP, CPU brand, hostname) \
             is restricted by the browser sandbox.\n",
        );

        output
    }
}
