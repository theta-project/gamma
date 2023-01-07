use bytes::{BufMut, BytesMut, Buf, Bytes};
use std::error::Error;

pub trait BytesMutExt<B: ByteSlice + GrowableByteSlice> {
    fn put_header(&mut self, id: i16);
    fn fix_header(&mut self, start: usize);
    fn len(&self) -> usize;

    fn put_bool(&mut self, val: bool);
    fn put_uleb(&mut self, len: usize);
    fn put_string(&mut self, string: &str);

    fn get_bool(&mut self) -> Result<bool, Box<dyn Error>>;
    fn get_uleb(&mut self) -> Result<usize, Box<dyn Error>>;
    fn get_string(&mut self) -> Result<String, Box<dyn Error>>;

    fn with_header<F, R>(&mut self, id: i16, f: F) -> Result<R, Box<dyn Error>>
    where
        F: FnOnce(&mut Self) -> Result<R, Box<dyn Error>>,
    {
        // record start and put header
        let start = self.len();
        self.put_header(id);

        // run caller provided code
        let result = f(self)?;

        // cleanup
        self.fix_header(start);

        Ok(result)
    }
}

pub trait BytesExt {
    fn take_while(&mut self, f: impl FnMut(u8) -> bool) -> Result<Bytes, Box<dyn Error>>;
}

impl BytesMutExt for BytesMut {
    fn len(&self) -> usize {
        // call BytesMut::len() to get the length of the buffer
        BytesMut::len(self)
    }

    fn put_header(&mut self, id: i16) -> Result<(), Box<dyn Error>> {
        self.put_i16_le(id)?;
        self.put_bool(false)?;
        self.put_u32_le(0)?;

        Ok(())
    }

    fn fix_header(&mut self, start: usize) -> Result<(), Box<dyn Error>> {
        if start >= self.len() {
            return Err(Box::new(Error::InvalidInput));
        }

        let length = self.len() - start - 7;
        self[start + 3] = length as u8;

        Ok(())
    }

    fn put_bool(&mut self, val: bool) -> Result<(), Box<dyn Error>> {
        self.put_u8(val as u8)?;
        Ok(())
    }

    fn put_uleb(&mut self, mut len: usize) -> Result<(), Box<dyn Error>> {
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

        self.put(uleb.as_slice())?;

        Ok(())
    }
    
    fn put_string(&mut self, string: &str) -> Result<(), Box<dyn Error>> {
        let length = string.len();

        self.put_u8(0xb)?;
        self.put_uleb(length)?;
        self.put(string.as_bytes())?;

        Ok(())
    }

    fn get_bool(&mut self) -> Result<bool, Box<dyn Error>> {
        let byte = self.get_u8()?;
        Ok(byte != 0)
    }

    fn get_uleb(&mut self) -> Result<usize, Box<dyn Error>> {
        let mut result = 0;
        let mut shift = 0;

        // read the first byte
        let mut byte = self.get_u8()?;

        // check if the highest bit is set
        if (byte & 0x80) == 0 {
            // the value is one byte long, return it
            return Ok((byte & 0x7f) as usize);
        }

        // the value is longer than one byte, read the remaining bytes
        loop {
            result |= (byte & 0x7f) << shift;
            shift += 7;

            byte = self.get_u8()?;
            if (byte & 0x80) == 0 {
                // the highest bit is not set, the value has ended
                break;
            }
        }

        result |= (byte & 0x7f) << shift;

        Ok(result as usize)
    }

    fn get_string(&mut self) -> Result<String, Box<dyn Error>> {
        // read the length of the string
        let _ = self.get_u8()?;
        let length = self.get_uleb()?;

        // check if the length is non-zero
        if length == 0 {
            return Ok(String::new());
        }

        // read the characters of the string
        let mut string = String::with_capacity(length);
        for _ in 0..length {
            let current_char = self.get(0)?;
            string.push(*current_char as char);
            self.advance(1);
        }

        // remove any null characters at the end of the string
        string.retain(|x| x != '\0' && x != '\u{b}');

        Ok(string)
    }

    
}

impl BytesExt for Bytes {
    /// returns a slice of this `Bytes` value that contains the elements 
    /// for which the given predicate returns `true`.
    fn take_while(&mut self, mut f: impl FnMut(u8) -> bool) -> &[u8] {
        // return early if the buffer is empty
        if self.is_empty() {
            return &[];
        }

        let mut len = 0;
        while let Some(b) = self.get(len) {
            if !f(*b) {
                break;
            }
            len += 1;
        }

        // return a slice of the original buffer
        let (left, _right) = self.split_to(len);
        left
    }
}

pub type Buffer = BytesMut;
