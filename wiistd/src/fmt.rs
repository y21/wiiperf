use core::fmt::{self, Write};

pub struct FmtBuffer {
    pub buf: [u8; 256],
    pub len: usize,
}

impl Write for FmtBuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let bytes = s.as_bytes();
        if self.len + bytes.len() > self.buf.len() {
            return Err(fmt::Error);
        }
        self.buf[self.len..self.len + bytes.len()].copy_from_slice(bytes);
        self.len += bytes.len();
        Ok(())
    }
}

pub fn report_arguments(args: fmt::Arguments<'_>) {
    let mut buf = FmtBuffer {
        buf: [0u8; 256],
        len: 0,
    };
    buf.write_fmt(args).unwrap();
    crate::osreport::report_bytes(&buf.buf[..buf.len]);
}

#[macro_export]
macro_rules! println {
    ($($arg:tt)*) => {
        $crate::fmt::report_arguments(format_args!($($arg)*));
    };
}
