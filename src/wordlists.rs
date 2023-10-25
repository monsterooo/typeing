use clap::ValueEnum;
use include_flate::flate;

flate!(static TOP_250: str          from "src/word_lists/top250");
flate!(static TOP_500: str          from "src/word_lists/top500");
flate!(static TOP_1000: str         from "src/word_lists/top1000");
flate!(static TOP_2500: str         from "src/word_lists/top2500");
flate!(static TOP_5000: str         from "src/word_lists/top5000");
flate!(static TOP_10000: str        from "src/word_lists/top10000");
flate!(static TOP_25000: str        from "src/word_lists/top25000");
flate!(static TOP_MISSPELLED: str   from "src/word_lists/commonly_misspelled");

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, ValueEnum)]
pub enum BuiltInWordlist {
    /// Source: [wordfrequency.info](https://www.wordfrequency.info/samples.asp) (top 60K lemmas sample).
    Top250,

    /// Source: [wordfrequency.info](https://www.wordfrequency.info/samples.asp) (top 60K lemmas sample).
    Top500,

    /// Source: [wordfrequency.info](https://www.wordfrequency.info/samples.asp) (top 60K lemmas sample).
    Top1000,

    /// Source: [wordfrequency.info](https://www.wordfrequency.info/samples.asp) (top 60K lemmas sample).
    Top2500,

    /// Source: [wordfrequency.info](https://www.wordfrequency.info/samples.asp) (top 60K lemmas sample).
    Top5000,

    /// Source: [Monkeytype](https://github.com/monkeytypegame/monkeytype/blob/89f160f664a9e24a6d5a99f12ce0bd5a1b093b2a/frontend/static/languages/english_10k.json)
    /// (English 10k list)
    Top10000,

    /// Source: [Monkeytype](https://github.com/monkeytypegame/monkeytype/blob/89f160f664a9e24a6d5a99f12ce0bd5a1b093b2a/frontend/static/languages/english_25k.json)
    /// (English 25k list)
    Top25000,

    /// Source: [Monkeytype](https://github.com/monkeytypegame/monkeytype/blob/89f160f664a9e24a6d5a99f12ce0bd5a1b093b2a/frontend/static/languages/english_commonly_misspelled.json)
    /// (Commonly misspelled English list)
    CommonlyMisspelled,

    /// The operating system's builtin word list.
    ///
    /// See [`OS_WORDLIST_PATH`].
    OS,
}
