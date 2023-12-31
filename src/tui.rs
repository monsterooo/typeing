use std::{
    fmt::Display,
    io::{stdout, Stdout, Write},
};
use termion::{
    clear,
    color::{self, Color},
    cursor::{self, DetectCursorPos},
    raw::{IntoRawMode, RawTerminal},
    style, terminal_size,
};

use crate::TypeingError;

const MIN_LINE_WIDTH: usize = 50;

/// 描述具有可打印长度的内容
///
/// 例如，包含颜色字符的字符串在打印时的长度与其中的字节数或字符数不同
pub trait HasLength {
    fn length(&self) -> usize;
}

/// 保存要在终端上打印的一些文本
///
/// 这提供了一个抽象
/// - 通过['HasLength']特性在终端上打印时检索实际字符的数量
/// - 用于通过各种' with_* '方法格式化文本
///
/// 通常，这在切片形式中使用为'&[Text]'，
/// 因为单个['Text']只保存一个字符串，
/// 并且所有字符串都以相同的方式格式化。例如，
/// 您不能将['Text']的一部分格式为绿色，
/// 而其余部分为红色。你应该使用一段['Text']，
/// 每段都以不同的方式格式化
#[derive(Debug, Clone)]
pub struct Text {
    /// 原始文本
    raw_text: String,
    /// 没有格式的文本
    text: String,
    /// 在终端上打印时所取的字符宽度的实际数目
    length: usize,
}

impl Text {
    /// 从原始字符串构造一个新的Text
    /// 提示：确保此字符串本身没有格式化字符、零宽度字符或多宽度字符
    pub fn new(text: String) -> Self {
        let length = text.len();
        Self {
            raw_text: text.clone(),
            text,
            length,
        }
    }

    /// 具有格式化原始文本
    pub fn raw_text(&self) -> &String {
        &self.raw_text
    }

    /// 没有格式化的实际打印文本
    pub fn text(&self) -> &String {
        &self.text
    }

    /// 为文本添加模糊样式
    pub fn with_faint(mut self) -> Self {
        self.raw_text = format!("{}{}{}", style::Faint, self.raw_text, style::NoFaint);
        self
    }

    /// 给文本添加下划线
    pub fn with_underline(mut self) -> Self {
        self.raw_text = format!("{}{}{}", style::Underline, self.raw_text, style::Reset);
        self
    }

    /// 为文本添加指定的颜色
    pub fn with_color<C>(mut self, color: C) -> Self
    where
        C: Color,
    {
        self.raw_text = format!(
            "{}{}{}",
            color::Fg(color),
            self.raw_text,
            color::Fg(color::Reset)
        );
        self
    }
}

impl HasLength for Text {
    fn length(&self) -> usize {
        self.length
    }
}

impl HasLength for [Text] {
    fn length(&self) -> usize {
        self.iter().map(|t| t.length()).sum()
    }
}

impl From<String> for Text {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for Text {
    fn from(value: &str) -> Self {
        Self::new(value.to_string())
    }
}

impl From<char> for Text {
    fn from(value: char) -> Self {
        Self::new(value.to_string())
    }
}

impl Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.raw_text)
    }
}

/// 一行字的位置
#[derive(Clone, Copy)]
struct LinePos {
    /// 终端窗口中该行的 y 位置
    pub y: u16,
    /// 行中第一个字符的 x 位置
    pub x: u16,
    /// 该行的长度（字符数）
    pub length: u16,
}

/// 光标位置
struct CursorPos {
    pub lines: Vec<LinePos>,
    pub cur_line: usize,
    pub cur_char_in_line: u16,
}

impl CursorPos {
    pub fn new() -> Self {
        Self {
            lines: Vec::new(),
            cur_line: 0,
            cur_char_in_line: 0,
        }
    }

    pub fn next(&mut self) -> (u16, u16) {
        let line = self.lines[self.cur_line];
        let max_chars_index = line.length - 1;

        if self.cur_char_in_line < max_chars_index {
            // 如果未超过最大字符，则当前字符位置+1
            self.cur_char_in_line += 1;
        } else {
            if self.cur_line + 1 < self.lines.len() {
                // 如果字符位置达到当前行最大位置，则向下移动一行
                self.cur_line += 1;
                self.cur_char_in_line = 0;
            }
        }

        self.cur_pos()
    }

    pub fn prev(&mut self) -> (u16, u16) {
        if self.cur_char_in_line > 0 {
            // 当前行可以向前移动字符
            self.cur_char_in_line -= 1;
        } else {
            // 当前行不能向前移动字符
            if self.cur_line > 0 {
                // 并且不是在第一行，则代表可以继续向上移动行
                self.cur_line -= 1;
                self.cur_char_in_line = self.lines[self.cur_line].length - 1;
            }
        }

        self.cur_pos()
    }

    pub fn cur_pos(&self) -> (u16, u16) {
        let line = self.lines[self.cur_line];
        (line.x + self.cur_char_in_line, line.y)
    }
}

/// 终端UI
pub struct TypeingTui {
    stdout: RawTerminal<Stdout>,
    cursor_pos: CursorPos,
    track_lines: bool,
    bottom_lines_len: usize,
}

type MaybeError<T = ()> = Result<T, TypeingError>;

impl TypeingTui {
    /// 为TUI初始化原始模式的标准输出
    pub fn new() -> Self {
        Self {
            stdout: stdout().into_raw_mode().unwrap(),
            cursor_pos: CursorPos::new(),
            track_lines: false,
            bottom_lines_len: 0,
        }
    }

    // 重置光标
    pub fn reset(&mut self) {
        self.cursor_pos = CursorPos::new()
    }

    // 刷新终端
    pub fn flush(&mut self) -> MaybeError {
        self.stdout.flush()?;
        Ok(())
    }

    /// 重置Tui
    pub fn reset_screen(&mut self) -> MaybeError {
        let (sizex, sizey) = terminal_size()?;

        write!(
            self.stdout,
            "{}{}{}",
            clear::All,
            cursor::Goto(sizex / 2, sizey / 2),
            cursor::BlinkingBar
        )?;
        self.flush()?;
        Ok(())
    }

    /// 显示单行文本
    pub fn display_a_line(&mut self, text: &[Text]) -> MaybeError {
        self.display_a_line_raw(text)?;
        self.flush()?;

        Ok(())
    }

    /// 与['display_a_line']相同，但没有刷新
    fn display_a_line_raw<T, U>(&mut self, text: U) -> MaybeError
    where
        U: AsRef<[T]>,
        [T]: HasLength,
        T: Display,
    {
        let len = text.as_ref().len() as u16;
        write!(self.stdout, "{}", cursor::Left(len / 2))?;

        if self.track_lines {
            let (x, y) = self.stdout.cursor_pos()?;
            self.cursor_pos.lines.push(LinePos { x, y, length: len })
        }

        for t in text.as_ref() {
            self.display_raw_text(t)?;
        }

        write!(self.stdout, "{}", cursor::Left(len))?;

        Ok(())
    }

    /// 显示多行文本
    ///
    /// - 一行文本由一段 [`Text`] 描述，它们连接并显示在同一行上
    /// 这些线垂直居中，每条线本身水平居中
    pub fn display_lines<T, U>(&mut self, lines: &[T]) -> MaybeError
    where
        T: AsRef<[U]>,
        [U]: HasLength,
        U: Display,
    {
        let (sizex, sizey) = terminal_size()?;
        let line_offset = lines.len() as u16 / 2;

        for (line_no, line) in lines.iter().enumerate() {
            write!(
                self.stdout,
                "{}",
                cursor::Goto(sizex / 2, sizey / 2 + (line_no as u16) - line_offset)
            )?;
            self.display_a_line_raw(line.as_ref())?;
        }
        self.flush()?;

        Ok(())
    }

    /// 在屏幕底部显示多行文本
    pub fn display_lines_bottom<T, U>(&mut self, lines: &[T]) -> MaybeError
    where
        T: AsRef<[U]>,
        [U]: HasLength,
        U: Display,
    {
        let (sizex, sizey) = terminal_size()?;
        let line_offset = lines.len() as u16;

        self.bottom_lines_len = lines.len();

        for (line_no, line) in lines.iter().enumerate() {
            write!(
                self.stdout,
                "{}",
                cursor::Goto(sizex / 2, sizey - 1 + (line_no as u16) - line_offset)
            )?;
            self.display_a_line_raw(line.as_ref())?;
        }
        self.flush()?;

        Ok(())
    }

    pub fn display_words(&mut self, words: &[String]) -> MaybeError<Vec<Text>> {
        self.reset();
        // 当前行的单词长度
        let mut current_len = 0;
        let mut max_word_len = 0;
        let mut line = Vec::new();
        let mut lines = Vec::new();
        let (terminal_width, terminal_height) = terminal_size()?;
        // 控制台40%宽
        let max_width = terminal_width * 2 / 5;
        const MAX_WORDS_PER_LINE: usize = 10;

        for word in words {
            // +1 是因为行尾有一个额外的空格
            max_word_len = std::cmp::max(max_word_len, word.len() + 1);
            let new_len = current_len + word.len() as u16 + 1;
            // 行字长小于总宽40%，并且下一次增加的单词不超过总宽40%。那么才追加单词到当前行
            if line.len() < MAX_WORDS_PER_LINE && new_len <= max_width {
                line.push(word.clone());
                current_len += word.len() as u16 + 1
            } else {
                // 在每行的末尾添加一个额外的空格，因为用户会本能地在每个单词后面键入一个空格(至少我是这样做的)
                // 追加一行
                lines.push(Text::from(line.join(" ") + " ").with_faint());

                // 新行的第一个单词
                line = vec![word.clone()];
                current_len = word.len() as u16 + 1;
            }
        }

        lines.push(Text::from(line.join(" ")).with_faint());
        max_word_len = std::cmp::max(max_word_len + 1, MIN_LINE_WIDTH);
        if lines.len() + self.bottom_lines_len + 2 > terminal_height as usize {
            return Err(TypeingError::from(format!(
                "终端高度太短! Typeing 至少需要 {} 行，得到 {} 行",
                lines.len() + self.bottom_lines_len + 2,
                terminal_height
            )));
        } else if max_word_len > terminal_width as usize {
            return Err(TypeingError::from(format!(
                "终端宽度太低! Typeing 至少需要 {} 列，得到 {} 列",
                max_word_len, terminal_width
            )));
        }
        self.track_lines = true;
        self.display_lines(
            lines
                .iter()
                .cloned()
                .map(|line| [line])
                .collect::<Vec<[Text; 1]>>()
                .as_slice(),
        )?;
        self.track_lines = false;
        self.move_to_cur_pos()?;
        self.flush()?;

        Ok(lines)
    }

    /// 显示一个原始文本
    pub fn display_raw_text<T>(&mut self, text: &T) -> MaybeError
    where
        T: Display,
    {
        write!(self.stdout, "{}", text)?;

        Ok(())
    }

    /// 隐藏光标
    pub fn hide_cursor(&mut self) -> MaybeError {
        write!(self.stdout, "{}", cursor::Hide)?;
        self.flush()?;
        Ok(())
    }

    /// 显示光标
    pub fn show_cursor(&mut self) -> MaybeError {
        write!(self.stdout, "{}", cursor::Show)?;
        self.flush()?;
        Ok(())
    }

    /// 将光标前面的文本替换为给定文本
    pub fn replace_text<T>(&mut self, text: T) -> MaybeError
    where
        T: Display,
    {
        self.move_to_prev_char()?;
        self.display_raw_text(&text)?;
        self.move_to_cur_pos()?;

        Ok(())
    }

    pub fn move_to_next_char(&mut self) -> MaybeError {
        let (x, y) = self.cursor_pos.next();
        write!(self.stdout, "{}", cursor::Goto(x, y));

        Ok(())
    }

    pub fn move_to_prev_char(&mut self) -> MaybeError {
        let (x, y) = self.cursor_pos.prev();
        write!(self.stdout, "{}", cursor::Goto(x, y));

        Ok(())
    }

    pub fn move_to_cur_pos(&mut self) -> MaybeError {
        let (x, y) = self.cursor_pos.cur_pos();
        write!(self.stdout, "{}", cursor::Goto(x, y));

        Ok(())
    }

    pub fn current_line(&self) -> usize {
        self.cursor_pos.cur_line
    }
}

impl Default for TypeingTui {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TypeingTui {
    /// 重置终端
    /// 清空终端，将光标设置为不闪烁的块
    fn drop(&mut self) {
        write!(self.stdout, "{}{}{}", clear::All, cursor::SteadyBlock, cursor::Goto(1,1))
            .expect("Could not reset terminal while exiting");
        self.flush().expect("Could not flush stdout while exiting");
    }
}