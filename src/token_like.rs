pub trait TokenLike {
    fn kind(&self) -> &String;
    // TODO maybe lexeme needs to be a generic trait (string, num, etc)
    fn lexeme(&self) -> &String;
}

impl TokenLike for (String, String) {
    fn kind(&self) -> &String {
        &self.0
    }

    fn lexeme(&self) -> &String {
        &self.1
    }
}
