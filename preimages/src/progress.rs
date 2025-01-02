use alloy_primitives::Address;
use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
pub struct PreimagesProgressBar {
    inner: ProgressBar,
}

impl PreimagesProgressBar {
    pub fn new() -> Result<Self> {
        let inner = ProgressBar::new(0x10000);
        inner.set_style(
            ProgressStyle::with_template("{bar:50.cyan/blue} {percent}% [eta: {eta}] {msg}")?
                .progress_chars("#>-"),
        );
        Ok(Self { inner })
    }

    pub fn progress(&mut self, addr: Address) {
        self.inner
            .set_position(u64::from(addr[0]) << 8 | u64::from(addr[1]));
        self.inner.set_message(addr.to_string().to_lowercase());
    }
}
