use crate::models::laserv::Completion;

pub struct ChatCompletionText<I>
where
    I: Iterator<Item = Completion>,
{
    source: I,
}

impl<I> ChatCompletionText<I>
where
    I: Iterator<Item = Completion>,
{
    pub fn new(source: I) -> Self {
        Self { source }
    }
}

impl<I> Iterator for ChatCompletionText<I>
where
    I: Iterator<Item = Completion>,
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.source
            .next()
            .map(|c| c.first_content().unwrap_or("").into())
    }
}

pub struct Lines<I>
where
    I: Iterator<Item = String>,
{
    source: I,
    buffer: String,
}

impl<I> Lines<I>
where
    I: Iterator<Item = String>,
{
    pub fn new(source: I) -> Self {
        Self {
            source,
            buffer: String::new(),
        }
    }
}

impl<I> Iterator for Lines<I>
where
    I: Iterator<Item = String>,
{
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some(pos) = self.buffer.find('\n') {
                let mut line = self.buffer.drain(..=pos).collect::<String>();
                line.pop();
                return Some(line);
            }

            match self.source.next() {
                Some(chunk) => self.buffer.push_str(&chunk),
                None => {
                    if self.buffer.is_empty() {
                        return None;
                    }
                    return Some(std::mem::take(&mut self.buffer));
                }
            }
        }
    }
}

pub struct CodeBlocks<I>
where
    I: Iterator<Item = String>,
{
    source: I,
    inside_code_block: bool,
    first: bool,
    block_num: usize,
}

impl<I> CodeBlocks<I>
where
    I: Iterator<Item = String>,
{
    pub fn new(source: I) -> Self {
        Self {
            source,
            inside_code_block: false,
            first: false,
            block_num: 0,
        }
    }

    fn is_fence(line: &str) -> bool {
        line.trim_start().starts_with("```")
    }
}

impl<I> Iterator for CodeBlocks<I>
where
    I: Iterator<Item = String>,
{
    type Item = (String, usize);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(line) = self.source.next() {
            if Self::is_fence(&line) {
                self.inside_code_block = !self.inside_code_block;
                if self.inside_code_block {
                    self.block_num += 1;
                }
                continue;
            }

            if self.inside_code_block {
                if !self.first {
                    self.first = true;
                    self.block_num = 0;
                    return Some((line, self.block_num));
                }
                return Some((format!("\n{line}"), self.block_num));
            }
        }

        None
    }
}

pub fn chat_completions_code<I>(completions: I) -> CodeBlocks<Lines<ChatCompletionText<I>>>
where
    I: Iterator<Item = Completion>,
{
    CodeBlocks::new(Lines::new(ChatCompletionText::new(completions)))
}
