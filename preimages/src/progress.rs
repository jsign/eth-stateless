use alloy_primitives::{keccak256, Address};
use indicatif::{ProgressBar, ProgressStyle};
pub struct AddressProgressBar {
    inner: ProgressBar,
    hash_on_progress: bool,
}

impl AddressProgressBar {
    pub fn new(hash_on_progress: bool) -> Self {
        let inner = ProgressBar::new(0x10000);
        inner.set_style(
            ProgressStyle::with_template("{bar:50.cyan/blue} {percent}% [eta: {eta}] {msg}")
                .expect("Failed to set progress bar style template")
                .progress_chars("#>-"),
        );
        Self {
            inner,
            hash_on_progress,
        }
    }

    pub fn progress(&mut self, addr: Address) {
        let hashed_addr = keccak256(addr);
        let progress_val = if self.hash_on_progress {
            hashed_addr.as_slice()
        } else {
            addr.as_slice()
        };
        self.inner
            .set_position(u64::from(progress_val[0]) << 8 | u64::from(progress_val[1]));
        self.inner.set_message(hex::encode(progress_val));
    }
}
