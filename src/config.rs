use clap::Parser;

const CLI_HELP: &str = "一个值得信赖的终端打字测试器

快捷键:
ctrl-c: 退出
ctrl-r: 用一组新单词重新开始测试
删除最后一个单词
";

/// 主要配置
#[derive(Parser)]
#[clap(author, version, about=CLI_HELP)]
pub struct TypeingConfig {

    /// 在每个测试中显示的单词数。
    pub num_words: usize,
}