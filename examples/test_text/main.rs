use app::{App, AppContext};
use edtui::{EditorEventHandler, EditorState, Lines};
use std::error::Error;
use term::Term;
mod app;
mod term;
mod theme;

pub type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> Result<()> {
    let mut term = Term::new()?;
    let mut app = App {
        context: AppContext::new(),
        should_quit: false,
    };
    app.run(&mut term)
}

impl AppContext {
    pub fn new() -> Self {
        Self {
            state: EditorState::new(Lines::from(
                "English: The quick brown fox jumps over the lazy dog.

Español: El rápido zorro marrón salta sobre el perro perezoso.

Français: Le vif renard brun saute par-dessus le chien paresseux.

Deutsch: Der flinke braune Fuchs springt über den faulen Hund.

Русский: Быстрая коричневая лиса перепрыгивает через ленивую собаку.

中文 (简体): 快速的棕色狐狸跳过了懒狗。

日本語: 素早い茶色の狐が怠け者の犬を飛び越えます。

العربية: الثعلب البني السريع يقفز فوق الكلب الكسول.

한국어: 빠른 갈색 여우가 게으른 개를 뛰어넘는다.

Türkçe: Hızlı kahverengi tilki tembel köpeğin üzerinden atlar.

🌍 Unicode Mix: The quīck bröwn fôx jumps 🦊 över thę lazy 🐶 døg. 👾

Greek: Η γρήγορη καφέ αλεπού πηδάει πάνω από το τεμπέλικο σκυλί.

Hebrew: השועל החום המהיר קופץ מעל הכלב העצלן.

Polski: Szybki brązowy lis skacze nad leniwym psem.

ไทย: สุนัขจิ้งจอกสีน้ำตาลกระโดดข้ามสุนัขขี้เกียจ.

🌈 Emoji: The 🦊quick brown f🐕x jumps over 🛌lazy animals 🎉 in different languages! 🌟

Esperanto: La rapida bruna vulpo saltas super la laca hundo.

Italiano: La veloce volpe marrone salta sopra il cane pigro.

עִברִית (Hebrew): השועל החום המהיר קופץ מעל הכלב העצלן.

📜 Unicode Text: T͞ḩe ҉qu̸ic͠ķ b̴row͠n ͘f̴ox̡ ju̡mp͞s o̸ver͟ t̴he l̛az̴y d҉ơg.",
            )),
            event_handler: EditorEventHandler::default(),
        }
    }
}
