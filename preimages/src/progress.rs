use alloy_primitives::Address;
use indicatif::{ProgressBar, ProgressStyle};
pub struct AddressProgressBar {
    inner: ProgressBar,
}

impl AddressProgressBar {
    pub fn new() -> Self {
        let inner = ProgressBar::new(0x10000);
        inner.set_style(
            ProgressStyle::with_template("{bar:50.cyan/blue} {percent}% [eta: {eta}] {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        Self { inner }
    }

    pub fn finish(self) {
        self.inner.finish();
    }

    pub fn progress(&mut self, addr: Address) {
        self.inner
            .set_position(u64::from(addr[0]) << 8 | u64::from(addr[1]));
        self.inner.set_message(addr.to_string().to_lowercase());
    }
}
