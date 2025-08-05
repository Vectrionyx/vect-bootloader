use core::fmt;
use log::{Level, Record};
use x86_64::instructions::port::{Port, PortGeneric, ReadWriteAccess};

pub struct SerialLogger;

const COM1: u16 = 0x3F8;

impl SerialLogger {
    pub fn init() {
        unsafe {
            let mut port = Port::new(COM1 + 1);
            port.write(0x00u8);
            let mut port = Port::new(COM1 + 3);
            port.write(0x80u8); // Enable DLAB (set baud rate divisor)

            let mut port = Port::new(COM1 + 0);
            port.write(0x03u8); // Divisor low byte (38400 baud)

            let mut port = Port::new(COM1 + 1);
            port.write(0x00u8); // Divisor high byte

            let mut port = Port::new(COM1 + 3);
            port.write(0x03u8); // 8 bits, no parity, one stop bit

            let mut port = Port::new(COM1 + 2);
            port.write(0xC7u8); // Enable FIFO, clear them, 14-byte threshold

            let mut port = Port::new(COM1 + 4);
            port.write(0x0Bu8); // IRQs enabled, RTS/DSR set
        }
    }

    fn write_byte(byte: u8) {
        unsafe {
            let mut line_status: PortGeneric<u8, ReadWriteAccess> = Port::new(COM1 + 5);
            let mut data: PortGeneric<u8, ReadWriteAccess> = Port::new(COM1);

            while (line_status.read() & 0x20) == 0 {}
            data.write(byte);
        }
    }
}

struct SerialWriter;

impl fmt::Write for SerialWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for &b in s.as_bytes() {
            if b == b'\n' {
                SerialLogger::write_byte(b'\n');
            }
            SerialLogger::write_byte(b);
        }

        Ok(())
    }
}

impl log::Log for SerialLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= Level::Trace
    }

    fn log(&self, record: &Record) {
        use core::fmt::Write;
        if self.enabled(record.metadata()) {
            let mut writer = SerialWriter;
            let _ = write!(writer, "{} - {}\n", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

pub static LOGGER: SerialLogger = SerialLogger;