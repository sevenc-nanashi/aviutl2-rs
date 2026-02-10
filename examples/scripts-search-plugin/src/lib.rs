use aviutl2::AnyResult;

mod gui;

#[aviutl2::plugin(GenericPlugin)]
pub struct ScriptsSearchPlugin {
    window: aviutl2_eframe::EframeWindow,
}
#[derive(Debug, Clone, PartialEq)]
pub struct EffectData {
    effect: aviutl2::generic::Effect,
    search_name: nucleo_matcher::Utf32String,
    search_label: nucleo_matcher::Utf32String,
    name: String,
    label: String,
}
pub struct EffectDb {
    effects: Vec<EffectData>,
}
pub static EFFECTS: std::sync::OnceLock<EffectDb> = std::sync::OnceLock::new();

pub static EDIT_HANDLE: aviutl2::generic::GlobalEditHandle =
    aviutl2::generic::GlobalEditHandle::new();

impl aviutl2::generic::GenericPlugin for ScriptsSearchPlugin {
    fn new(_info: aviutl2::AviUtl2Info) -> AnyResult<Self> {
        Self::init_logging();
        log::info!("Initializing Rusty Scripts Search Plugin...");
        let window =
            aviutl2_eframe::EframeWindow::new("RustyScriptsSearchPlugin", move |cc, handle| {
                Ok(Box::new(gui::ScriptsSearchApp::new(cc, handle)))
            })?;

        Ok(ScriptsSearchPlugin { window })
    }

    fn register(&mut self, registry: &mut aviutl2::generic::HostAppHandle) {
        registry.set_plugin_information(&format!(
            "Scripts Search for AviUtl2, written in Rust / v{version} / https://github.com/sevenc-nanashi/aviutl2-rs/tree/main/examples/scripts-search-plugin",
            version = env!("CARGO_PKG_VERSION")
        ));
        registry
            .register_window_client("Rusty Scripts Search Plugin", &self.window)
            .unwrap();
        let edit_handle = registry.create_edit_handle();
        EDIT_HANDLE.init(edit_handle);
    }

    fn on_project_load(&mut self, _project: &mut aviutl2::generic::ProjectFile) {
        let maybe_config = ScriptsSearchPlugin::load_aviutl2_ini();
        let config = match maybe_config {
            Ok(cfg) => cfg,
            Err(e) => {
                log::error!("Failed to load aviutl2.ini: {}", e);
                return;
            }
        };
        let Some(effects_table) = config.get_table("Effect") else {
            log::error!("Effect section not found in aviutl2.ini");
            return;
        };
        EFFECTS.get_or_init(|| {
            let effects = EDIT_HANDLE.get_effects();
            let mut has_missing_label = false;
            let effects = effects
                .into_iter()
                .map(|effect| {
                    let section_name =
                        aviutl2::config::get_language_text(&effect.name, &effect.name)
                            .expect("effect name contains null byte");
                    let effects_name = aviutl2::config::get_language_text("Effect", &effect.name)
                        .expect("Effect.name contains null byte");
                    let name = if effect.name != section_name {
                        section_name
                    } else if effect.name != effects_name {
                        effects_name
                    } else {
                        effect.name.clone()
                    };
                    let label = effects_table
                        .get_table(&effect.name)
                        .and_then(|t| t.get_value("label"));
                    let label = match label {
                        Some(l) => aviutl2::config::get_language_text("Effect", l)
                            .expect("effect label contains null byte"),
                        None => {
                            has_missing_label = true;
                            "？？？".to_string()
                        }
                    };
                    EffectData {
                        effect,
                        search_name: nucleo_matcher::Utf32String::from(
                            normalize_kana_for_search(&name).as_str(),
                        ),
                        search_label: nucleo_matcher::Utf32String::from(
                            normalize_kana_for_search(&label).as_str(),
                        ),
                        name,
                        label,
                    }
                })
                .collect();
            EffectDb { effects }
        });
    }
}

impl ScriptsSearchPlugin {
    fn init_logging() {
        aviutl2::logger::LogBuilder::new()
            .filter_level(if cfg!(debug_assertions) {
                log::LevelFilter::Debug
            } else {
                log::LevelFilter::Info
            })
            .init();
    }

    fn load_aviutl2_ini() -> AnyResult<aviutl2::alias::Table> {
        let main_config_path = aviutl2::config::app_data_path().join("aviutl2.ini");
        let content = std::fs::read_to_string(&main_config_path)?;
        Ok(content.parse()?)
    }
}

pub fn normalize_kana_for_search(input: &str) -> String {
    let (normalized, _) = normalize_kana_for_search_with_map(input);
    normalized
}

pub fn normalize_kana_for_search_with_map(input: &str) -> (String, Vec<(usize, usize)>) {
    if input.is_empty() {
        return (String::new(), Vec::new());
    }
    let chars: Vec<char> = input.chars().collect();
    let mut output = String::with_capacity(chars.len());
    let mut map = Vec::with_capacity(chars.len());
    let mut index = 0;
    while index < chars.len() {
        let start = index;
        let c = chars[index];
        if is_hiragana(c) {
            output.push(c);
            map.push((start, start));
            index += 1;
            continue;
        }
        if is_katakana(c) {
            output.push(katakana_to_hiragana(c));
            map.push((start, start));
            index += 1;
            continue;
        }
        if let Some(mut kata) = halfwidth_to_fullwidth_katakana(c) {
            let mut end = start;
            if index + 1 < chars.len() {
                let next = chars[index + 1];
                if (next == 'ﾞ' || next == 'ﾟ')
                    && let Some(combined) = apply_diacritic(kata, next)
                {
                    kata = combined;
                    end = index + 1;
                    index += 1;
                }
            }
            if is_katakana(kata) {
                output.push(katakana_to_hiragana(kata));
            } else {
                output.push(kata);
            }
            map.push((start, end));
            index += 1;
            continue;
        }
        output.push(c);
        map.push((start, start));
        index += 1;
    }
    (output, map)
}

fn is_hiragana(c: char) -> bool {
    ('\u{3041}'..='\u{3096}').contains(&c)
}

fn is_katakana(c: char) -> bool {
    ('\u{30A1}'..='\u{30F6}').contains(&c)
}

fn katakana_to_hiragana(c: char) -> char {
    let code = u32::from(c) - 0x60;
    char::from_u32(code).unwrap_or(c)
}

fn halfwidth_to_fullwidth_katakana(c: char) -> Option<char> {
    let mapped = match c {
        'ｦ' => 'ヲ',
        'ｧ' => 'ァ',
        'ｨ' => 'ィ',
        'ｩ' => 'ゥ',
        'ｪ' => 'ェ',
        'ｫ' => 'ォ',
        'ｬ' => 'ャ',
        'ｭ' => 'ュ',
        'ｮ' => 'ョ',
        'ｯ' => 'ッ',
        'ｰ' => 'ー',
        'ｱ' => 'ア',
        'ｲ' => 'イ',
        'ｳ' => 'ウ',
        'ｴ' => 'エ',
        'ｵ' => 'オ',
        'ｶ' => 'カ',
        'ｷ' => 'キ',
        'ｸ' => 'ク',
        'ｹ' => 'ケ',
        'ｺ' => 'コ',
        'ｻ' => 'サ',
        'ｼ' => 'シ',
        'ｽ' => 'ス',
        'ｾ' => 'セ',
        'ｿ' => 'ソ',
        'ﾀ' => 'タ',
        'ﾁ' => 'チ',
        'ﾂ' => 'ツ',
        'ﾃ' => 'テ',
        'ﾄ' => 'ト',
        'ﾅ' => 'ナ',
        'ﾆ' => 'ニ',
        'ﾇ' => 'ヌ',
        'ﾈ' => 'ネ',
        'ﾉ' => 'ノ',
        'ﾊ' => 'ハ',
        'ﾋ' => 'ヒ',
        'ﾌ' => 'フ',
        'ﾍ' => 'ヘ',
        'ﾎ' => 'ホ',
        'ﾏ' => 'マ',
        'ﾐ' => 'ミ',
        'ﾑ' => 'ム',
        'ﾒ' => 'メ',
        'ﾓ' => 'モ',
        'ﾔ' => 'ヤ',
        'ﾕ' => 'ユ',
        'ﾖ' => 'ヨ',
        'ﾗ' => 'ラ',
        'ﾘ' => 'リ',
        'ﾙ' => 'ル',
        'ﾚ' => 'レ',
        'ﾛ' => 'ロ',
        'ﾜ' => 'ワ',
        'ﾝ' => 'ン',
        _ => return None,
    };
    Some(mapped)
}

fn apply_diacritic(base: char, mark: char) -> Option<char> {
    let mapped = match (base, mark) {
        ('ウ', 'ﾞ') => 'ヴ',
        ('カ', 'ﾞ') => 'ガ',
        ('キ', 'ﾞ') => 'ギ',
        ('ク', 'ﾞ') => 'グ',
        ('ケ', 'ﾞ') => 'ゲ',
        ('コ', 'ﾞ') => 'ゴ',
        ('サ', 'ﾞ') => 'ザ',
        ('シ', 'ﾞ') => 'ジ',
        ('ス', 'ﾞ') => 'ズ',
        ('セ', 'ﾞ') => 'ゼ',
        ('ソ', 'ﾞ') => 'ゾ',
        ('タ', 'ﾞ') => 'ダ',
        ('チ', 'ﾞ') => 'ヂ',
        ('ツ', 'ﾞ') => 'ヅ',
        ('テ', 'ﾞ') => 'デ',
        ('ト', 'ﾞ') => 'ド',
        ('ハ', 'ﾞ') => 'バ',
        ('ヒ', 'ﾞ') => 'ビ',
        ('フ', 'ﾞ') => 'ブ',
        ('ヘ', 'ﾞ') => 'ベ',
        ('ホ', 'ﾞ') => 'ボ',
        ('ワ', 'ﾞ') => 'ヷ',
        ('ヰ', 'ﾞ') => 'ヸ',
        ('ヱ', 'ﾞ') => 'ヹ',
        ('ヲ', 'ﾞ') => 'ヺ',
        ('ハ', 'ﾟ') => 'パ',
        ('ヒ', 'ﾟ') => 'ピ',
        ('フ', 'ﾟ') => 'プ',
        ('ヘ', 'ﾟ') => 'ペ',
        ('ホ', 'ﾟ') => 'ポ',
        _ => return None,
    };
    Some(mapped)
}

aviutl2::register_generic_plugin!(ScriptsSearchPlugin);

#[cfg(test)]
mod tests {
    use super::normalize_kana_for_search;
    #[test]
    fn test_normalize_kana_for_search() {
        let input = "あいうえおアイウエオカキクケコｱｲｳｴｵｶﾞｷﾞｸﾞｹﾞｺﾞ";
        let expected = "あいうえおあいうえおかきくけこあいうえおがぎぐげご";
        let output = normalize_kana_for_search(input);
        assert_eq!(output, expected);
    }
}
