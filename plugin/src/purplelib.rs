//! Purple-lib: the settings under Options > Mod Options > Skill Simulator.
//!
//! Optional. Purple-lib calls `purplelib_on_register` (found by name in the exported symbols) once from its first
//! update; without Purple-lib nothing here runs and the Forge menu stays the only settings window. Changes apply at
//! once and are saved to skill_simulator.ini (the same file and writer as the Forge menu) when the Options window
//! closes. Records and API follow Purple-lib's purplelib.h, API version 4 (Switch layout: 4-byte pointers), as ported
//! in Better Skillinfo's src/purplelib.rs (works there). Purple-lib is someone else's mod: public API only.
use core::ffi::{c_char, c_void};
use core::sync::atomic::{AtomicU32, Ordering};

const API_VERSION: u32 = 4;
const ROW_LABELS: u32 = 0;

#[repr(C)]
pub struct Row {
    name: *const c_char,
    help: *const c_char,
    labels: *const *const c_char,
    label_count: u32,
    kind: u32,
    value_count: u32,
    first: i32,
    step: i32,
    suffix: *const c_char,
    flags: u32,
}
#[repr(C)]
pub struct Options {
    size: u32,
    context: *mut c_void,
    title: Option<unsafe extern "C" fn(*mut c_void, u32) -> *const c_char>,
    row_count: u32,
    row: Option<unsafe extern "C" fn(*mut c_void, u32, u32, *mut Row) -> bool>,
    get: Option<unsafe extern "C" fn(*mut c_void, u32) -> u32>,
    set: Option<unsafe extern "C" fn(*mut c_void, u32, u32)>,
    closed: Option<unsafe extern "C" fn(*mut c_void)>,
    value_text: Option<unsafe extern "C" fn(*mut c_void, u32, u32, u32) -> *const c_char>,
    action: Option<unsafe extern "C" fn(*mut c_void, u32)>,
}
#[repr(C)]
pub struct Help {
    size: u32,
    context: *mut c_void,
    category: u32,
    page_count: u32,
    title: Option<unsafe extern "C" fn(*mut c_void, u32) -> *const c_char>,
    page: Option<unsafe extern "C" fn(*mut c_void, u32, u32) -> *const c_char>,
    open: Option<unsafe extern "C" fn(*mut c_void)>,
}
#[repr(C)]
pub struct Api {
    version: u32,
    add_options: Option<unsafe extern "C" fn(*const Options) -> bool>,
    add_help: Option<unsafe extern "C" fn(*const Help) -> bool>,
    hold_input: Option<unsafe extern "C" fn()>,
    options_shown: Option<unsafe extern "C" fn(*const Options) -> bool>,
    options_row: Option<unsafe extern "C" fn(*const Options) -> i32>,
    add_set_help: Option<unsafe extern "C" fn(*const Options, *const Help) -> bool>,
    help_shown: Option<unsafe extern "C" fn(*const Options) -> bool>,
}
#[cfg(target_arch = "arm")]
const _: () = {
    assert!(core::mem::size_of::<Options>() == 40);
    assert!(core::mem::size_of::<Row>() == 40);
    assert!(core::mem::size_of::<Help>() == 28);
    assert!(core::mem::size_of::<Api>() == 32);
};

/// Strings Purple-lib (and the game) may keep a pointer to: they must live forever, so only static NUL-terminated bytes.
struct Labels<const N: usize>([*const c_char; N]);
unsafe impl<const N: usize> Sync for Labels<N> {}
macro_rules! labels {
    ($($t:literal),*) => { Labels([$(concat!($t, "\0").as_ptr() as *const c_char),*]) };
}

/// Rows: 0 language, 1 button layout, 2 logging
const ROWS: u32 = 3;
const PAGES: u32 = 2;

/// One language's text for the page. The language row lists the mod's languages in App::language order (the Forge
/// menu's), named in the page's language: the page is drawn with the game font, which may lack the native names.
/// Quotes are ASCII only; help text breaks lines with \r\n.
struct Text {
    names: Labels<3>,
    helps: Labels<3>,
    on_off: Labels<2>,
    languages: Labels<9>,
    pages: Labels<2>,
}
/// Button layouts are product names, the same in every language (App::layout order)
static LAYOUTS: Labels<3> = labels!("Nintendo", "Xbox", "PlayStation");
/// The page's text in the mod's language order: en, zh-Hans, zh-Hant, ja, ko, de, fr, it, es
static TEXT: [Text; 9] = [
    // en
    Text {
        names: labels!("Language", "Button layout", "Enable logging"),
        helps: labels!(
            "The language of the simulator window.",
            "The button icons shown in the window.",
            "Writes search conditions and frame timing to log.txt (for bug reports)."
        ),
        on_off: labels!("Off", "On"),
        languages: labels!("English", "Chinese (S)", "Chinese (T)", "Japanese", "Korean", "German", "French", "Italian", "Spanish"),
        pages: labels!(
            "In a village or the Gathering Hub, open\r\nthe simulator from page 3 of the + menu.\r\nDuring quests, use the Open button in\r\nthe Forge settings.",
            "Changes show at once. They are saved\r\nto skill_simulator.ini in your save\r\nfolder when you close the window."
        ),
    },
    // zh-Hans
    Text {
        names: labels!("语言", "按键布局", "启用日志记录"),
        helps: labels!("模拟器窗口使用的语言。", "窗口中显示的按键图标。", "将搜索条件和帧时间写入 log.txt（用于报告问题）。"),
        on_off: labels!("关", "开"),
        languages: labels!("英语", "简体中文", "繁体中文", "日语", "韩语", "德语", "法语", "意大利语", "西班牙语"),
        pages: labels!(
            "在村庄和集会所，从 + 菜单的第3页\r\n打开模拟器。任务中请使用 Forge\r\n设置中的打开按钮。",
            "更改会立即生效。关闭窗口时保存到\r\n存档文件夹中的 skill_simulator.ini。"
        ),
    },
    // zh-Hant
    Text {
        names: labels!("語言", "按鍵配置", "啟用日誌記錄"),
        helps: labels!("模擬器視窗使用的語言。", "視窗中顯示的按鍵圖示。", "將搜尋條件和影格時間寫入 log.txt（用於回報問題）。"),
        on_off: labels!("關", "開"),
        languages: labels!("英文", "簡體中文", "繁體中文", "日文", "韓文", "德文", "法文", "義大利文", "西班牙文"),
        pages: labels!(
            "在村莊和集會所，從 + 選單的第3頁\r\n開啟模擬器。任務中請使用 Forge\r\n設定中的開啟按鈕。",
            "變更會立即生效。關閉視窗時儲存到\r\n存檔資料夾中的 skill_simulator.ini。"
        ),
    },
    // ja
    Text {
        names: labels!("言語", "ボタン配置", "ログ記録を有効にする"),
        helps: labels!(
            "シミュレーター画面の言語です。",
            "画面に表示するボタンアイコンです。",
            "検索条件とフレーム時間を log.txt に記録します（不具合報告用）。"
        ),
        on_off: labels!("オフ", "オン"),
        languages: labels!("英語", "中国語（簡体）", "中国語（繁体）", "日本語", "韓国語", "ドイツ語", "フランス語", "イタリア語", "スペイン語"),
        pages: labels!(
            "村と集会所では + メニューの3ページ目\r\nから開きます。クエスト中は Forge\r\n設定の開くボタンを使います。",
            "変更はすぐに反映されます。ウィンドウを\r\n閉じるとセーブフォルダの\r\nskill_simulator.ini に保存されます。"
        ),
    },
    // ko
    Text {
        names: labels!("언어", "버튼 배치", "로그 기록 활성화"),
        helps: labels!(
            "시뮬레이터 창에 쓸 언어입니다.",
            "창에 표시할 버튼 아이콘입니다.",
            "검색 조건과 프레임 시간을 log.txt에 남깁니다(문제 보고용)."
        ),
        on_off: labels!("끄기", "켜기"),
        languages: labels!("영어", "중국어(간체)", "중국어(번체)", "일본어", "한국어", "독일어", "프랑스어", "이탈리아어", "스페인어"),
        pages: labels!(
            "마을과 집회소에서는 + 메뉴 3페이지에서\r\n시뮬레이터를 엽니다. 퀘스트 중에는\r\nForge 설정의 열기 버튼을 쓰세요.",
            "변경 내용은 바로 반영됩니다. 창을 닫으면\r\n세이브 폴더의 skill_simulator.ini에\r\n저장됩니다."
        ),
    },
    // de
    Text {
        names: labels!("Sprache", "Tastenlayout", "Protokollierung aktivieren"),
        helps: labels!(
            "Sprache des Simulator-Fensters.",
            "Die im Fenster gezeigten Tastensymbole.",
            "Schreibt Suchbedingungen und Frame-Zeiten in log.txt (für Fehlerberichte)."
        ),
        on_off: labels!("Aus", "An"),
        languages: labels!("Englisch", "Chinesisch (V)", "Chinesisch (T)", "Japanisch", "Koreanisch", "Deutsch", "Französisch", "Italienisch", "Spanisch"),
        pages: labels!(
            "Im Dorf und in der Versammlungshalle:\r\nSeite 3 des +-Menüs. In Quests: die\r\nSchaltfläche Öffnen im Forge-Menü.",
            "Änderungen gelten sofort. Beim Schließen\r\nwerden sie in skill_simulator.ini im\r\nSpielstand-Ordner gespeichert."
        ),
    },
    // fr
    Text {
        names: labels!("Langue", "Disposition des boutons", "Activer la journalisation"),
        helps: labels!(
            "Langue de la fenêtre du simulateur.",
            "Icônes de boutons affichées dans la fenêtre.",
            "Écrit les conditions de recherche et le temps des images dans log.txt (pour signaler un bug)."
        ),
        on_off: labels!("Non", "Oui"),
        languages: labels!("Anglais", "Chinois (S)", "Chinois (T)", "Japonais", "Coréen", "Allemand", "Français", "Italien", "Espagnol"),
        pages: labels!(
            "Au village et au rassemblement : page 3\r\ndu menu +. En quête : bouton Ouvrir\r\ndes réglages Forge.",
            "Les changements s'appliquent aussitôt et\r\nsont enregistrés dans skill_simulator.ini\r\nà la fermeture de la fenêtre."
        ),
    },
    // it
    Text {
        names: labels!("Lingua", "Layout pulsanti", "Attiva il registro"),
        helps: labels!(
            "Lingua della finestra del simulatore.",
            "Icone dei pulsanti mostrate nella finestra.",
            "Scrive condizioni di ricerca e tempi dei frame in log.txt (per segnalare problemi)."
        ),
        on_off: labels!("No", "Sì"),
        languages: labels!("Inglese", "Cinese (S)", "Cinese (T)", "Giapponese", "Coreano", "Tedesco", "Francese", "Italiano", "Spagnolo"),
        pages: labels!(
            "Nel villaggio e nella sala raduni: pagina\r\n3 del menu +. In missione: pulsante Apri\r\ndelle impostazioni Forge.",
            "Le modifiche valgono subito e vengono\r\nsalvate in skill_simulator.ini alla\r\nchiusura della finestra."
        ),
    },
    // es
    Text {
        names: labels!("Idioma", "Disposición de botones", "Activar el registro"),
        helps: labels!(
            "Idioma de la ventana del simulador.",
            "Iconos de botones que se muestran.",
            "Escribe las condiciones de búsqueda y el tiempo de fotogramas en log.txt (para informar de errores)."
        ),
        on_off: labels!("No", "Sí"),
        languages: labels!("Inglés", "Chino (S)", "Chino (T)", "Japonés", "Coreano", "Alemán", "Francés", "Italiano", "Español"),
        pages: labels!(
            "En la aldea y el salón: página 3 del\r\nmenú +. En misiones: botón Abrir de\r\nlos ajustes de Forge.",
            "Los cambios se aplican al instante y se\r\nguardan en skill_simulator.ini al\r\ncerrar la ventana."
        ),
    },
];
/// Purple-lib passes the game's language (MT Framework order: 0 Japanese, 1 English, 2 French, 3 Spanish, 4 German,
/// 5 Italian, 6 Korean, 7 Chinese (T), 8 Chinese (S); 1 at first and 6 under the Korean patch, confirmed in Better
/// Skillinfo's log). Chosen again on every call. Unknown numbers are English.
fn text(language: u32) -> &'static Text {
    const MOD_LANGUAGE: [usize; 9] = [3, 0, 6, 8, 5, 7, 4, 2, 1];
    &TEXT[MOD_LANGUAGE.get(language as usize).copied().unwrap_or(0)]
}

/// Last values read or set (language, layout, logging): shown when the state lock is busy for a moment
static SHOWN: [AtomicU32; 3] = [AtomicU32::new(0), AtomicU32::new(0), AtomicU32::new(0)];

unsafe extern "C" fn title(_: *mut c_void, _: u32) -> *const c_char {
    c"Skill Simulator".as_ptr()
}

unsafe extern "C" fn row(_: *mut c_void, language: u32, index: u32, out: *mut Row) -> bool {
    let Some(out) = (unsafe { out.as_mut() }) else { return false };
    if index >= ROWS {
        return false;
    }
    let t = text(language);
    let i = index as usize;
    out.name = t.names.0[i];
    out.help = t.helps.0[i];
    out.kind = ROW_LABELS;
    (out.labels, out.label_count) = match index {
        0 => (t.languages.0.as_ptr(), 9),
        1 => (LAYOUTS.0.as_ptr(), 3),
        _ => (t.on_off.0.as_ptr(), 2),
    };
    true
}

unsafe extern "C" fn get(_: *mut c_void, index: u32) -> u32 {
    let Some(cell) = SHOWN.get(index as usize) else { return 0 };
    // Called on every draw: wait only briefly for the lock (the compute thread hands it over within 0.5 ms)
    let now = crate::STATE.wait_with(5, |p| match index {
        0 => p.app.language.min(8) as u32,
        1 => p.app.layout.min(2) as u32,
        _ => p.app.logging as u32,
    });
    match now {
        Some(v) => {
            cell.store(v, Ordering::Relaxed);
            v
        }
        None => cell.load(Ordering::Relaxed),
    }
}

/// Applies at once (the window redraws in the new language / button pictures); saved in `closed`
unsafe extern "C" fn set(_: *mut c_void, index: u32, value: u32) {
    let ok = match index {
        0 => value < 9,
        1 => value < 3,
        2 => value < 2,
        _ => false,
    };
    if !ok {
        return;
    }
    crate::STATE.with(|p| {
        match index {
            0 => p.app.language = value as usize,
            1 => p.app.layout = value as usize,
            _ => p.app.logging = value != 0,
        }
        p.app.touch();
    });
    SHOWN[index as usize].store(value, Ordering::Relaxed);
}

/// The Options window closed: write skill_simulator.ini once (the update loop writes it, as for the Forge menu)
unsafe extern "C" fn closed(_: *mut c_void) {
    crate::STATE.with(|p| p.prefs_dirty = true);
}

struct Record<T>(T);
unsafe impl<T> Sync for Record<T> {}
static OPTIONS: Record<Options> = Record(Options {
    size: core::mem::size_of::<Options>() as u32,
    context: core::ptr::null_mut(),
    title: Some(title),
    row_count: ROWS,
    row: Some(row),
    get: Some(get),
    set: Some(set),
    closed: Some(closed),
    value_text: None,
    action: None,
});

unsafe extern "C" fn help_page(_: *mut c_void, language: u32, index: u32) -> *const c_char {
    text(language).pages.0.get(index as usize).copied().unwrap_or(core::ptr::null())
}
static SET_HELP: Record<Help> = Record(Help {
    size: core::mem::size_of::<Help>() as u32,
    context: core::ptr::null_mut(),
    category: 0,
    page_count: PAGES,
    title: Some(title),
    page: Some(help_page),
    open: None,
});

#[unsafe(no_mangle)]
pub unsafe extern "C" fn purplelib_on_register(api: *const Api) {
    let Some(api) = (unsafe { api.as_ref() }) else { return };
    if api.version < API_VERSION {
        warn!("[skill-sim] Purple-lib API {} is too old (needs {}): no Mod Options page", api.version, API_VERSION);
        return;
    }
    let Some(add) = api.add_options else { return };
    if unsafe { add(&OPTIONS.0) } {
        if let Some(help) = api.add_set_help {
            unsafe { help(&OPTIONS.0, &SET_HELP.0) };
        }
        // Written whether or not 'Enable logging' is on: one line that tells whether the page exists
        forge::log::prelude::info!("[skill-sim] settings under Options > Mod Options");
    } else {
        warn!("[skill-sim] Purple-lib refused the Mod Options page");
    }
}

