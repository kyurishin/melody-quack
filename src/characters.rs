#[allow(unused_imports)]
use crate::*;
#[allow(unused_imports)]
use macroquad::prelude::*;

#[derive(Clone, Copy)]
pub(crate) struct CatLook {
    pub(crate) fur: Color,
    pub(crate) stripes: Option<Color>,
    pub(crate) eye: Option<Color>,
}

#[derive(Clone, Copy)]
pub(crate) struct DogLook {
    pub(crate) fur: Color,
    pub(crate) ear: Color,
    pub(crate) muzzle: Color,
    pub(crate) tail: Option<bool>,
    pub(crate) collar: Option<Color>,
}

#[derive(Clone, Copy)]
pub(crate) enum Body {
    Cat(CatLook),
    Dog(DogLook),
    Duck,
    Owl,
    Fox,
    Otter,
}

#[derive(Clone, Copy)]
pub(crate) enum VoiceKind {
    Meow { shift: i32, bright: f32, length: f32 },
    Woof,
    Yap,
    Howl,
    Quack,
    Hoot,
    Bellow,
    Shell,
}

pub(crate) fn voice_word(kind: VoiceKind) -> &'static str {
    match kind {
        VoiceKind::Meow { .. } => "meow",
        VoiceKind::Woof => "woof",
        VoiceKind::Yap => "yap",
        VoiceKind::Howl => "awoo",
        VoiceKind::Quack => "quack",
        VoiceKind::Hoot => "hoo",
        VoiceKind::Bellow => "yowl",
        VoiceKind::Shell => "clack",
    }
}

pub(crate) struct Character {
    pub(crate) name: &'static str,
    pub(crate) noun: &'static str,
    pub(crate) body: Body,
    pub(crate) voice: VoiceKind,
}

pub(crate) fn roster() -> Vec<Character> {
    vec![
        Character {
            name: "cream cat",
            noun: "cat",
            body: Body::Cat(CatLook { fur: rgba(250, 226, 192), stripes: None, eye: None }),
            voice: VoiceKind::Meow { shift: 0, bright: 1.0, length: 0.55 },
        },
        Character {
            name: "russian blue",
            noun: "cat",
            body: Body::Cat(CatLook { fur: rgba(98, 102, 118), stripes: None, eye: Some(rgba(186, 214, 72)) }),
            voice: VoiceKind::Meow { shift: 0, bright: 0.55, length: 0.6 },
        },
        Character {
            name: "ginger tabby",
            noun: "cat",
            body: Body::Cat(CatLook { fur: rgba(240, 160, 84), stripes: Some(rgba(198, 112, 52)), eye: Some(rgba(120, 200, 96)) }),
            voice: VoiceKind::Meow { shift: 0, bright: 1.45, length: 0.45 },
        },
        Character {
            name: "black cat",
            noun: "cat",
            body: Body::Cat(CatLook { fur: rgba(54, 52, 64), stripes: None, eye: Some(rgba(244, 204, 60)) }),
            voice: VoiceKind::Meow { shift: -12, bright: 0.9, length: 0.62 },
        },
        Character {
            name: "labrador",
            noun: "dog",
            body: Body::Dog(DogLook { fur: rgba(232, 196, 128), ear: rgba(200, 156, 92), muzzle: rgba(232, 196, 128), tail: Some(false), collar: None }),
            voice: VoiceKind::Woof,
        },
        Character {
            name: "cream pup",
            noun: "dog",
            body: Body::Dog(DogLook { fur: rgba(248, 230, 194), ear: rgba(214, 164, 104), muzzle: rgba(254, 244, 222), tail: None, collar: Some(rgba(52, 178, 168)) }),
            voice: VoiceKind::Yap,
        },
        Character {
            name: "shiba",
            noun: "dog",
            body: Body::Dog(DogLook { fur: rgba(226, 140, 72), ear: rgba(214, 126, 60), muzzle: rgba(250, 240, 226), tail: Some(true), collar: Some(rgba(200, 50, 60)) }),
            voice: VoiceKind::Howl,
        },
        Character { name: "duck", noun: "duck", body: Body::Duck, voice: VoiceKind::Quack },
        Character { name: "owl", noun: "owl", body: Body::Owl, voice: VoiceKind::Hoot },
        Character { name: "fox", noun: "fox", body: Body::Fox, voice: VoiceKind::Bellow },
        Character { name: "sea otter", noun: "otter", body: Body::Otter, voice: VoiceKind::Shell },
    ]
}

pub(crate) fn render_voice(kind: VoiceKind, note: u8) -> Vec<f32> {
    let f = |shift: i32| midi_to_freq((note as i32 + shift) as u8);
    match kind {
        VoiceKind::Meow { shift, bright, length } => render_meow_style(f(shift), length, bright),
        VoiceKind::Woof => render_woof(f(-12)),
        VoiceKind::Yap => render_yap(f(-5)),
        VoiceKind::Howl => render_howl(f(-12)),
        VoiceKind::Quack => render_quack(f(-12)),
        VoiceKind::Hoot => render_hoot_machine(f(-12)),
        VoiceKind::Bellow => render_fox(f(0)),
        VoiceKind::Shell => render_shell(f(0)),
    }
}
