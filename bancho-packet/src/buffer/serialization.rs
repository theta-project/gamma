use bytes::{Buf, BufMut, Bytes, BytesMut};

pub trait BytesMutExt {
    fn put_header(&mut self, id: i16);
    fn fix_header(&mut self, start: usize);

    fn put_bool(&mut self, val: bool);
    fn put_uleb(&mut self, len: usize);
    fn put_string(&mut self, string: &str);

    fn get_bool(&mut self) -> bool;
    fn get_uleb(&mut self) -> usize;
    fn get_string(&mut self) -> String;

    fn with_header(&mut self, id: i16, f: impl FnOnce(&mut Self));
}

pub trait BytesExt {
    fn take_while(&mut self, f: impl FnMut(u8) -> bool) -> Bytes;
}

impl BytesMutExt for BytesMut {
    fn put_header(&mut self, id: i16) {
        self.put_i16_le(id);
        self.put_bool(false);
        self.put_u32_le(0);
    }

    fn fix_header(&mut self, start: usize) {
        let length = self.len() - start - 7;
        (self[start + 3]) = length as u8;
    }

    fn put_bool(&mut self, val: bool) {
        self.put_u8(val as u8);
    }

    fn put_uleb(&mut self, mut len: usize) {
        let mut uleb: Vec<u8> = vec![0; 32];

        let mut uleb_len: usize = 0;

        while len > 0 {
            uleb[uleb_len] = (len as u8) & 0x7f;

            len >>= 7;
            if len != 0 {
                uleb[uleb_len] |= 0x80;
            }

            uleb_len += 1;
        }

        uleb.retain(|&x| x != 0);

        self.put(uleb.as_slice());
    }
    fn put_string(&mut self, string: &str) {
        let length = string.len();

        self.put_u8(0xb);
        self.put_uleb(length);
        self.put(string.as_bytes());
    }

    fn get_bool(&mut self) -> bool {
        self.get_u8() != 0
    }

    fn get_uleb(&mut self) -> usize {
        let mut result = 0;
        let mut shift = 0;

        let mut byte = self.get_u8();

        if (byte & 0x80) == 0 {
            result |= (byte & 0x7f) << shift;
        } else {
            let mut end = false;

            while !end {
                if shift > 0 {
                    byte = self.get_u8();
                }

                result |= (byte & 0x7f) << shift;
                if (byte & 0x80) == 0 {
                    end = true;
                }
                shift += 7;
            }
        }

        result as usize
    }

    fn get_string(&mut self) -> String {
        let _ = self.get_u8();
        let length = self.get_uleb();
        let mut string = "".to_string();

        if length > 0 {
            let mut i = 0;

            while i < length + 1 {
                let current_char = self.get(i);
                let chr = match current_char {
                    Some(x) => *x as char,
                    None => '\0',
                };

                string = format!("{}{}", string, chr); //;
                i += 1;
            }

            self.advance(length);
        }
        string.retain(|x| x != '\0' && x != '\u{b}');
        string
    }

    fn with_header(&mut self, id: i16, f: impl FnOnce(&mut Self)) {
        // record start and put header
        let start = self.len();
        self.put_header(id);

        // run caller provided code
        f(self);

        // cleanup
        self.fix_header(start);
    }
}

impl BytesExt for Bytes {
    // TODO: This doesnt need to copy, could just slice into the buf
    fn take_while(&mut self, mut f: impl FnMut(u8) -> bool) -> Bytes {
        let mut len = 0;
        while let Some(b) = self.get(len) {
            if !f(*b) {
                break;
            }
            len += 1;
        }

        self.copy_to_bytes(len)
    }
}

pub type Buffer = BytesMut;
