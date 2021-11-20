pub struct DescriptorParser<'a> {
    bytes: &'a [u8],
    off: usize,
}

#[derive(Debug, Default)]
pub struct MethodDescriptor {
    pub params: Vec<String>,
    pub jtypes: Vec<JType>,
    pub ret: String,
    pub arg_cells: usize,
}

#[derive(Debug)]
pub enum JType {
    IF,
    DJ,
    A,
}

pub trait JTypeDescriptor {
    fn jtype(&self) -> JType;
}

impl JTypeDescriptor for str {
    fn jtype(&self) -> JType {
        match self.as_bytes()[0] {
            b'Z' | b'B' | b'C' | b'S' | b'I' | b'F' => JType::IF,
            b'D' | b'J' => JType::DJ,
            _ => JType::A,
        }
    }
}

impl DescriptorParser<'_> {
    pub fn new(bytes: &[u8]) -> DescriptorParser {
        DescriptorParser { bytes, off: 0 }
    }

    fn u8(&mut self) -> u8 {
        let u = self.bytes[self.off];
        self.off += 1;
        u
    }

    fn peek(&self) -> u8 {
        self.bytes[self.off]
    }

    pub fn parse_method(&mut self) -> MethodDescriptor {
        if self.peek() != b'(' {
            panic!("not a method descriptor");
        }

        self.u8();

        let params = self.parse_params();
        let jtypes: Vec<JType> = params.iter().map(|x| x.jtype()).collect();
        self.u8();

        let ret = self.parse_param();

        let mut r = MethodDescriptor {
            params,
            ret,
            jtypes,
            arg_cells: 0,
        };

        for t in r.jtypes.iter() {
            match t {
                JType::A | JType::IF => r.arg_cells += 1,
                _ => r.arg_cells += 2,
            }
        }

        r
    }

    fn parse_params(&mut self) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        while self.peek() != b')' {
            v.push(self.parse_param());
        }
        v
    }

    fn parse_param(&mut self) -> String {
        if self.peek() == b'[' {
            return self.parse_arr().1;
        }
        return self.parse_no_array();
    }

    // dim, full descriptor, element descriptor
    pub fn parse_arr(&mut self) -> (u8, String, String) {
        let mut dims: u8 = 0;
        let mut s = String::new();

        while self.peek() == b'[' {
            s.push(self.u8() as char);
            dims += 1;
        }

        let element = self.parse_no_array();
        s.push_str(&element);
        (dims, s, element)
    }

    pub fn parse_no_array(&mut self) -> String {
        let cur = self.u8();
        match cur {
            b'B' | b'Z' | b'J' | b'I' | b'D' | b'F' | b'S' | b'V' | b'C' => {
                return unsafe {
                    String::from_utf8_unchecked(self.bytes[self.off - 1..self.off].to_vec())
                };
            }
            b'L' => {
                let mut s = String::new();

                while self.peek() != b';' {
                    s.push(self.u8() as char);
                }

                self.u8();
                return s;
            }
            _ => panic!("parse no array failed {}", unsafe {
                String::from_utf8_unchecked(self.bytes.to_vec())
            }),
        }
    }
}
