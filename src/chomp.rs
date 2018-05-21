use std::iter::Peekable;
use std::vec::IntoIter;

pub struct Chomp {
    bytes: Peekable<IntoIter<u8>>,
    bits_left: usize,
    current_byte: Option<u8>,
    bits_left_in_byte: u8,
}

impl Chomp {
    pub fn new(bytes: Vec<u8>) -> Chomp {
        let bits_left = bytes.len() * 8;
        let mut bytes = bytes.into_iter().peekable();
        let current_byte = bytes.next();
        let bits_left_in_byte = if current_byte.is_some() { 8 } else { 0 };

        Chomp {
            bytes,
            bits_left,
            current_byte,
            bits_left_in_byte,
        }
    }

    pub fn chomp_or<E>(&mut self, nr_bits: u8, err: E) -> Result<u8, E> {
        self.chomp(nr_bits).ok_or(err)
    }

    pub fn chomp_or_u16<E: Clone>(&mut self, nr_bits: u8, err: E) -> Result<u16, E> {
        let mut bits = nr_bits;

        let mut result: u16 = 0;
        while bits > 8 {
            result = (self.chomp(8).ok_or(err.clone())? as u16) << (bits - 8);
            bits -= 8;
        }

        result += self.chomp(bits).ok_or(err.clone())? as u16;

        Ok(result)
    }

    pub fn chomp(&mut self, nr_bits: u8) -> Option<u8> {
        if nr_bits < 1 || nr_bits > 8 || nr_bits as usize > self.bits_left {
            return None;
        }

        if nr_bits < self.bits_left_in_byte {
            self.nibble(nr_bits)
        } else if nr_bits == self.bits_left_in_byte {
            let mut result = 0;

            if let Some(ref mut byte) = self.current_byte {
                result = *byte >> (8 - self.bits_left_in_byte);
            }

            self.bits_left -= self.bits_left_in_byte as usize;
            self.current_byte = self.bytes.next();
            self.bits_left_in_byte = if self.current_byte.is_some() { 8 } else { 0 };

            Some(result)
        } else {
            let mut result = 0;
            let bits_to_go = nr_bits - self.bits_left_in_byte;

            if let Some(ref mut byte) = self.current_byte {
                result = (*byte >> (8 - self.bits_left_in_byte)) << bits_to_go;
            }

            self.bits_left -= self.bits_left_in_byte as usize;

            if let None = self.bytes.peek() {
                // calculations were off?
                return None;
            }

            self.current_byte = self.bytes.next();
            self.bits_left_in_byte = 8;

            let nibble = self.nibble(bits_to_go).unwrap(); // we just peeked

            Some(result + nibble)
        }
    }

    fn nibble(&mut self, nr_bits: u8) -> Option<u8> {
        if let Some(ref mut byte) = self.current_byte {
            let result = *byte >> (8 - nr_bits);
            *byte <<= nr_bits;

            self.bits_left_in_byte -= nr_bits;

            Some(result)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    pub fn empty() {
        let mut chomp = Chomp::new(vec![]);

        assert_eq!(None, chomp.chomp(4));
    }

    #[test]
    pub fn nibbles() {
        let mut chomp = Chomp::new(vec![0b11000100]);

        assert_eq!(Some(0b11), chomp.chomp(2));
        assert_eq!(Some(0b0001), chomp.chomp(4));
        assert_eq!(Some(0b0), chomp.chomp(1));

        assert_eq!(None, chomp.chomp(2));
    }

    #[test]
    pub fn chomp() {
        let mut chomp = Chomp::new(vec![0b11000100, 0b10101010]);

        assert_eq!(Some(0b110001), chomp.chomp(6));
        assert_eq!(Some(0b001010), chomp.chomp(6));
        assert_eq!(None, chomp.chomp(6));
        assert_eq!(Some(0b1010), chomp.chomp(4));
        assert_eq!(None, chomp.chomp(4));
    }

    #[test]
    pub fn chomp_on_border() {
        let mut chomp = Chomp::new(vec![0b11000100, 0b10101010]);

        assert_eq!(Some(0b110001), chomp.chomp(6));
        assert_eq!(Some(0b00), chomp.chomp(2));
        assert_eq!(Some(0b1010), chomp.chomp(4));
        assert_eq!(Some(0b1010), chomp.chomp(4));
        assert_eq!(None, chomp.chomp(4));
    }

    #[test]
    pub fn chomp_u16() {
        let mut chomp = Chomp::new(vec![0b11000100, 0b10101010]);

        assert_eq!(Ok(0b110001001010), chomp.chomp_or_u16(12, ()));
        assert_eq!(Some(0b1010), chomp.chomp(4));
        assert_eq!(None, chomp.chomp(4));
    }

    #[test]
    pub fn chomp_a_lot() {
        let mut chomp = Chomp::new(vec![
            0b11000100, 0b10101010, 0b11000100, 0b10101010, 0b11000100, 0b10101010,
        ]);

        assert_eq!(Some(0b110001), chomp.chomp(6));
        assert_eq!(Some(0b00101010), chomp.chomp(8));
        assert_eq!(Some(0b10110001), chomp.chomp(8));
        assert_eq!(Some(0b00101010), chomp.chomp(8));
        assert_eq!(Some(0b10110001), chomp.chomp(8));
        assert_eq!(Some(0b00101010), chomp.chomp(8));
        assert_eq!(None, chomp.chomp(8));
    }
}
