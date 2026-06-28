pub fn decode_subject(raw: &str) -> String {
    let out = if let Some(current) = raw.strip_prefix("=?UTF-8?Q?")
        && let Some(current) = current.strip_suffix("?=")
    {
        QEqParser::parse(current)
    } else if let Some(current) = raw.strip_prefix("=?UTF-8?B?")
        && let Some(current) = current.strip_suffix("?=")
    {
        decode_base64(current)
    } else {
        raw.to_owned()
    };
    out.trim().to_owned()
}

fn decode_base64(raw: &str) -> String {
    let mut bytes = Vec::new();
    let mut buffer = 0u32;
    let mut bits_collected = 0;

    for byte in raw.bytes() {
        if byte == b'=' {
            break;
        }

        let value = match byte {
            b'A'..=b'Z' => byte - b'A',
            b'a'..=b'z' => byte - b'a' + 26,
            b'0'..=b'9' => byte - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            _ => unreachable!(),
        };

        buffer = (buffer << 6) | value as u32;
        bits_collected += 6;

        if bits_collected >= 8 {
            bits_collected -= 8;
            bytes.push((buffer >> bits_collected) as u8);
        }
    }

    String::from_utf8_lossy(&bytes).to_string()
}

#[derive(Default)]
struct QEqParser {
    state: QEqState,
    buf: String,
    bytes: Vec<u8>,
}

#[derive(Default)]
enum QEqState {
    #[default]
    None,
    Eq,
    One(char),
}

impl QEqParser {
    fn add_char(&mut self, ch: char) {
        match self.state {
            QEqState::None if ch == '=' => self.state = QEqState::Eq,
            QEqState::None => {
                self.clear_bytes();
                self.buf.push(ch);
            }
            QEqState::Eq => self.state = QEqState::One(ch),
            QEqState::One(prev) if ch == '=' => {
                self.clear_bytes();
                self.buf.extend(['=', prev]);
                self.state = QEqState::Eq
            }
            QEqState::One(prev) => {
                if let Some(prev_digit) = hex_char_to_int(prev)
                    && let Some(next_digit) = hex_char_to_int(ch)
                {
                    self.bytes.push(prev_digit * 16 + next_digit);
                } else {
                    self.clear_bytes();
                    self.buf.extend(['=', prev, ch]);
                }
                self.state = QEqState::None;
            }
        }
    }

    fn clear_bytes(&mut self) {
        self.buf.push_str(&String::from_utf8_lossy(&self.bytes));
        self.bytes.clear();
    }

    fn clear(&mut self) {
        self.clear_bytes();
        match self.state {
            QEqState::None => (),
            QEqState::Eq => self.buf.push('='),
            QEqState::One(ch) => self.buf.extend(['=', ch]),
        }
    }

    fn parse(raw: &str) -> String {
        let mut this = Self::default();
        for ch in raw.replace('_', " ").replace("?= =?UTF-8?Q?", "").chars() {
            this.add_char(ch);
        }
        this.clear();
        this.buf
    }
}

fn hex_char_to_int(ch: char) -> Option<u8> {
    Some(match ch {
        '0'..='9' => (ch as u8) - b'0',
        'a'..='f' => (ch as u8) - b'a' + 10,
        'A'..='F' => (ch as u8) - b'A' + 10,
        _ => return None,
    })
}
