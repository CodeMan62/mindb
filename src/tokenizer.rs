#[derive(Debug)]
pub enum TokenType {
    Identifier,
    Keyword,
    Number,
    Symbol,
    Unknown,
}

#[derive(Debug)]
pub struct Token {
    token_type: TokenType,
    value: String,
}

impl Token {
    pub fn new(token_type: TokenType, value: String) -> Self {
        Token { token_type, value }
    }
}

pub struct Tokenizer {
    input: String,
    position: usize,
}

impl Tokenizer {
    pub fn new(input: String) -> Self {
        Tokenizer { input, position: 0 }
    }
    pub fn get_next_token(&mut self) -> Option<Token> {
        if self.position >= self.input.len() {
            return None;
        }
        let symbols = "=!=<><=>=(),;*";
        let current_char = self.input.chars().nth(self.position).unwrap();
        if current_char.is_alphabetic() {
            return self.collect_identifier();
        } else if current_char.is_numeric() {
            return self.collect_number();
        } else if symbols.contains(current_char) {
            return self.collect_operator();
        } else {
            self.position += 1;
            return Some(Token::new(TokenType::Unknown, current_char.to_string()));
        }
    }
    pub fn collect_keywords(&mut self) -> Option<Token> {
        let current_char = self.input.chars().nth(self.position).unwrap();
        self.position += 1;
        Some(Token::new(TokenType::Keyword, current_char.to_string()))
    }
    pub fn collect_identifier(&mut self) -> Option<Token> {
        let start_pos = self.position;
        while self.position < self.input.len()
            && self
                .input
                .chars()
                .nth(self.position)
                .unwrap()
                .is_alphabetic()
        {
            self.position += 1;
        }
        let word = self.input[start_pos..self.position].to_string();
        let token_type = match word.to_uppercase().as_str() {
            "SELECT" | "INSERT" | "CREATE" | "TABLE" | "FROM" | "WHERE" | "AND" | "OR" | "INTO"
            | "VALUES" | "INT" | "TEXT" | "EXIT" | "HELP" => TokenType::Keyword,
            _ => TokenType::Identifier,
        };
        Some(Token::new(token_type, word))
    }
    pub fn collect_number(&mut self) -> Option<Token> {
        let start_pos = self.position;
        while self.position < self.input.len()
            && self.input.chars().nth(self.position).unwrap().is_numeric()
        {
            self.position += 1;
        }
        Some(Token::new(
            TokenType::Number,
            self.input[start_pos..self.position].to_string(),
        ))
    }
    pub fn collect_operator(&mut self) -> Option<Token> {
        let current_char = self.input.chars().nth(self.position).unwrap();
        self.position += 1;
        Some(Token::new(TokenType::Symbol, current_char.to_string()))
    }
    pub fn tokenize(&mut self) -> Vec<Token> {
        let mut tokens = Vec::new();
        while let Some(token) = self.get_next_token() {
            tokens.push(token);
        }
        tokens
    }
}
