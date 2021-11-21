pub struct DescriptorParser<'a> {
    bytes: &'a [u8],
    off: usize,
}

#[derive(Debug, Default)]
pub struct MethodDescriptor {
    pub params: Vec<JType>,
    pub ret: JType,
    pub arg_cells: usize,
}

#[derive(Debug)]
pub enum JType {
    FI(char),
    DJ(char),
    A(String),
    // void
    V,
}

impl JType {
    pub fn class(&self) -> String {
        match self {
            Self::FI(x) => x.to_string(),
            Self::DJ(x) => x.to_string(),
            Self::A(x) => x.to_string(),
            Self::V => "V".to_string(),
        }
    }
}

impl Default for JType {
    fn default() -> Self {
        Self::V
    }
}

pub trait JTypeDescriptor {
    // -1 for reference
    // 1 for bool byte char short int float
    // 2 for double long
    fn slots(&self) -> i32;
}

impl JTypeDescriptor for str {
    fn slots(&self) -> i32 {
        match self.as_bytes()[0] {
            b'Z' | b'B' | b'C' | b'S' | b'I' | b'F' => 1,
            b'D' | b'J' => 2,
            _ => -1,
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
        self.u8();

        let ret = self.parse_param();

        let mut r = MethodDescriptor {
            params,
            ret,
            arg_cells: 0,
        };

        for t in r.params.iter() {
            match t {
                JType::A(_) | JType::FI(_) => r.arg_cells += 1,
                _ => r.arg_cells += 2,
            }
        }

        r
    }

    fn parse_params(&mut self) -> Vec<JType> {
        let mut v: Vec<JType> = Vec::new();
        while self.peek() != b')' {
            v.push(self.parse_param());
        }
        v
    }

    fn parse_param(&mut self) -> JType {
        if self.peek() == b'[' {
            return self.parse_arr().1;
        }
        return self.parse_no_array();
    }

    // dim, full descriptor, element descriptor
    pub fn parse_arr(&mut self) -> (u8, JType, JType) {
        let mut dims: u8 = 0;
        let mut s = String::new();

        while self.peek() == b'[' {
            s.push(self.u8() as char);
            dims += 1;
        }

        let now = self.off;
        let element = self.parse_no_array();

        s.push_str(&String::from_utf8_lossy(&self.bytes[now..self.off]));
        (dims, JType::A(s), element)
    }

    pub fn parse_no_array(&mut self) -> JType {
        let cur = self.u8();
        match cur {
            b'B' | b'Z' | b'I' | b'F' | b'S' | b'C' => {
                return JType::FI(cur as char);
            }
            b'J' | b'D' => {
                return JType::DJ(cur as char);
            }
            b'V' => JType::V,
            b'L' => {
                let mut s = String::new();
                while self.peek() != b';' {
                    s.push(self.u8() as char);
                }
                self.u8();
                return JType::A(s);
            }
            _ => panic!("parse no array failed {}", unsafe {
                String::from_utf8_unchecked(self.bytes.to_vec())
            }),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {
        let desc = "([[Ljava/lang/Object;IIJ)V".as_bytes().to_vec();

        let mut p = super::DescriptorParser::new(&desc);

        println!("{:#?}", p.parse_method())
    }
}
