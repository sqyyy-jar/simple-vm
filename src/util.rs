pub trait Read {
    fn read<T: Copy>(&mut self) -> T;
}

pub trait Write {
    fn write(&mut self, bytes: &[u8]);

    fn write_i8(&mut self, num: i8) {
        self.write(&num.to_ne_bytes());
    }

    fn write_u8(&mut self, num: u8) {
        self.write(&num.to_ne_bytes());
    }

    fn write_i16(&mut self, num: i16) {
        self.write(&num.to_ne_bytes());
    }

    fn write_u16(&mut self, num: u16) {
        self.write(&num.to_ne_bytes());
    }

    fn write_i32(&mut self, num: i32) {
        self.write(&num.to_ne_bytes());
    }

    fn write_u32(&mut self, num: u32) {
        self.write(&num.to_ne_bytes());
    }

    fn write_i64(&mut self, num: i64) {
        self.write(&num.to_ne_bytes());
    }

    fn write_u64(&mut self, num: u64) {
        self.write(&num.to_ne_bytes());
    }
}

impl Write for Vec<u8> {
    fn write(&mut self, bytes: &[u8]) {
        self.extend_from_slice(bytes);
    }
}
