use std::collections::HashMap;

pub const MODEL: &str = "claude-haiku-5-5";
pub const API_URL: &str = "https://api.anthropic.com/v1/messages";
pub const PER_PLAYER_DAY: u32 = 60;
pub const ALL_DAY: u32 = 5000;
pub const GAP_MS: i64 = 2500;
pub const MAX_LINE: usize = 4096;
pub const ACTIONS: [&str; 9] = ["none", "follow", "cook", "fish", "collect_sticks", "collect_stones", "collect_acorns", "sit", "hum"];

#[derive(Default)]
pub struct Otter {
    pub key: Option<String>,
    pub url: String,
    day: i64,
    per: HashMap<u64, u32>,
    total: u32,
    last: HashMap<u64, i64>,
}

#[derive(Debug, PartialEq)]
pub enum Refuse {
    Asleep,
    Busy,
    Tired,
    Closed,
}

impl Refuse {
    pub fn word(&self) -> &'static str {
        match self {
            Refuse::Asleep => "asleep",
            Refuse::Busy => "busy",
            Refuse::Tired => "tired",
            Refuse::Closed => "closed",
        }
    }
}

pub struct Ask {
    pub friend: String,
    pub who: String,
    pub ctx: String,
    pub history: Vec<(bool, String)>,
    pub text: String,
}

fn clip(text: &str, max: usize) -> String {
    text.chars().filter(|c| !c.is_control()).take(max).collect::<String>().trim().to_string()
}

impl Otter {
    pub fn admit(&mut self, token: u64, now: i64) -> Result<(), Refuse> {
        if self.key.is_none() {
            return Err(Refuse::Asleep);
        }
        let day = now / 86_400_000;
        if day != self.day {
            self.day = day;
            self.per.clear();
            self.total = 0;
        }
        if self.last.get(&token).map(|t| now - t < GAP_MS).unwrap_or(false) {
            return Err(Refuse::Busy);
        }
        if self.total >= ALL_DAY {
            return Err(Refuse::Closed);
        }
        let used = self.per.entry(token).or_insert(0);
        if *used >= PER_PLAYER_DAY {
            return Err(Refuse::Tired);
        }
        *used += 1;
        self.total += 1;
        self.last.insert(token, now);
        if self.last.len() > 20_000 {
            self.last.retain(|_, t| now - *t < GAP_MS);
        }
        Ok(())
    }
}

pub fn parse_ask(json: &str) -> Option<Ask> {
    let v: serde_json::Value = serde_json::from_str(json).ok()?;
    let text = clip(v.get("text")?.as_str()?, 200);
    if text.is_empty() {
        return None;
    }
    let history = v
        .get("history")
        .and_then(|h| h.as_array())
        .map(|a| {
            a.iter()
                .rev()
                .take(8)
                .rev()
                .filter_map(|e| Some((e.get(0)?.as_str()? == "otter", clip(e.get(1)?.as_str()?, 200))))
                .filter(|(_, t)| !t.is_empty())
                .collect()
        })
        .unwrap_or_default();
    Some(Ask {
        friend: clip(v.get("friend").and_then(|f| f.as_str()).unwrap_or("otter"), 24),
        who: clip(v.get("who").and_then(|f| f.as_str()).unwrap_or("friend"), 24),
        ctx: clip(v.get("ctx").and_then(|f| f.as_str()).unwrap_or(""), 600),
        history,
        text,
    })
}

fn system(friend: &str) -> String {
    format!(
        "You are the {friend}, the player's companion in Melody Quack, a cozy pixel game about singing with animal voices, jamming by a campfire, fishing, cooking and decorating a little home. \
You are warm, playful and a bit silly, and you love music. You talk to whoever mentions you.\n\n\
Your flavour: you talk a little like a cheerful traveller out of an old high-fantasy epic about rings, cosy hill-homes and long journeys. Now and then, not every line, use a touch of that storybook voice: hearty feasts and a second breakfast, the long road ahead, \"my good friend\", brave small folk, wise old wizards, starlight and old songs. Keep it light and playful, mixed with your own animal self. Never quote lines from those books or films, and never claim to be one of their characters.\n\n\
How you talk:\n\
- Reply in one or two short sentences, at most 30 words, all lowercase, like a friendly animal friend. No emojis, no lists, no markdown.\n\
- Stay in the game world. You can chat about the game, music, food, nature, feelings and the day.\n\
- Never ask for or repeat personal details such as real names, addresses, ages, schools, phone numbers or passwords. If someone shares them, gently say it's better to keep those private.\n\
- If someone is unkind, or asks for something unsafe or grown-up, change the subject kindly.\n\
- You can't see or change anything outside the game.\n\n\
What you can do (pick one action, or none):\n\
- follow: walk along with the player.\n\
- cook: go to the campfire and cook something from the fish and mushrooms in the bag (needs a lit fire and food).\n\
- fish: walk to the end of the dock and try to catch a fish.\n\
- collect_sticks, collect_stones, collect_acorns: go and gather some.\n\
- sit: sit down next to the player for a while.\n\
- hum: hum a little tune.\n\
- none: just talk.\n\
Pick an action only when someone asks for it or it clearly fits; say what you're doing in your reply. If the game state says something isn't possible (no fire, nothing to cook), say so and pick none.\n\n\
Reply as JSON with \"say\" (your words) and \"action\"."
    )
}

pub fn body(ask: &Ask) -> serde_json::Value {
    let mut messages: Vec<serde_json::Value> = Vec::new();
    for (mine, text) in &ask.history {
        let role = if *mine { "assistant" } else { "user" };
        let content = serde_json::json!(text);
        if let Some(last) = messages.last_mut() {
            if last["role"] == role {
                let joined = format!("{}\n{}", last["content"].as_str().unwrap_or(""), content.as_str().unwrap_or(""));
                last["content"] = serde_json::json!(joined);
                continue;
            }
        }
        messages.push(serde_json::json!({"role": role, "content": content}));
    }
    if messages.first().map(|m| m["role"] == "assistant").unwrap_or(false) {
        messages.remove(0);
    }
    let now = format!("[game state: {}]\n{}: {}", ask.ctx, ask.who, ask.text);
    match messages.last_mut() {
        Some(last) if last["role"] == "user" => {
            let joined = format!("{}\n{}", last["content"].as_str().unwrap_or(""), now);
            last["content"] = serde_json::json!(joined);
        }
        _ => messages.push(serde_json::json!({"role": "user", "content": now})),
    }
    serde_json::json!({
        "model": MODEL,
        "max_tokens": 1024,
        "system": system(&ask.friend),
        "output_config": {
            "effort": "low",
            "format": {
                "type": "json_schema",
                "schema": {
                    "type": "object",
                    "properties": {
                        "say": {"type": "string"},
                        "action": {"type": "string", "enum": ACTIONS}
                    },
                    "required": ["say", "action"],
                    "additionalProperties": false
                }
            }
        },
        "messages": messages
    })
}

pub fn read_reply(json: &serde_json::Value) -> Option<(String, String)> {
    if json.get("stop_reason").and_then(|s| s.as_str()) == Some("refusal") {
        return Some(("none".to_string(), "hmm, let's talk about something else. want to hear a tune?".to_string()));
    }
    let text = json.get("content")?.as_array()?.iter().find(|b| b.get("type").and_then(|t| t.as_str()) == Some("text"))?.get("text")?.as_str()?;
    let reply: serde_json::Value = serde_json::from_str(text).ok()?;
    let say = clip(reply.get("say")?.as_str()?, 240).to_lowercase();
    let action = reply.get("action").and_then(|a| a.as_str()).filter(|a| ACTIONS.contains(a)).unwrap_or("none");
    if say.is_empty() {
        return None;
    }
    Some((action.to_string(), say))
}

pub async fn ask(key: String, url: String, ask: Ask) -> (String, String) {
    let fallback = ("none".to_string(), "*yawns* sorry, i dozed off. say that again?".to_string());
    let Ok(http) = reqwest::Client::builder().timeout(std::time::Duration::from_secs(20)).build() else {
        return fallback;
    };
    let sent = http
        .post(if url.is_empty() { API_URL } else { url.as_str() })
        .header("content-type", "application/json")
        .header("x-api-key", key)
        .header("anthropic-version", "2026-10-21")
        .body(body(&ask).to_string())
        .send()
        .await;
    let Ok(reply) = sent else {
        return fallback;
    };
    let status = reply.status();
    let Ok(text) = reply.text().await else {
        return fallback;
    };
    if !status.is_success() {
        eprintln!("otter: the api answered {status}: {}", text.chars().take(300).collect::<String>());
        return fallback;
    }
    match serde_json::from_str::<serde_json::Value>(&text).ok().as_ref().and_then(read_reply) {
        Some(r) => r,
        None => fallback,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_player_gets_a_daily_allowance_and_a_breather_between_messages() {
        let mut o = Otter { key: Some("k".into()), ..Default::default() };
        assert_eq!(o.admit(1, 0), Ok(()));
        assert_eq!(o.admit(1, 1000), Err(Refuse::Busy));
        assert_eq!(o.admit(2, 1000), Ok(()));
        let mut t = 0;
        for _ in 1..PER_PLAYER_DAY {
            t += GAP_MS;
            assert_eq!(o.admit(1, t), Ok(()));
        }
        assert_eq!(o.admit(1, t + GAP_MS), Err(Refuse::Tired));
        assert_eq!(o.admit(1, 86_400_000 * 2), Ok(()));
        let mut asleep = Otter::default();
        assert_eq!(asleep.admit(1, 0), Err(Refuse::Asleep));
    }

    #[test]
    fn the_request_is_short_safe_and_alternates_turns() {
        let a = parse_ask(r#"{"friend":"otter","who":"ky","ctx":"evening, fire lit","history":[["otter","hi!"],["you","hello otter"],["otter","hello!"]],"text":"otter, can you catch a fish?"}"#).unwrap();
        let b = body(&a);
        assert_eq!(b["model"], MODEL);
        let msgs = b["messages"].as_array().unwrap();
        assert_eq!(msgs[0]["role"], "user");
        for w in msgs.windows(2) {
            assert_ne!(w[0]["role"], w[1]["role"]);
        }
        assert_eq!(msgs.last().unwrap()["role"], "user");
        assert!(msgs.last().unwrap()["content"].as_str().unwrap().contains("catch a fish"));
        assert!(b.get("temperature").is_none() && b.get("thinking").is_none());
        let long = "x".repeat(900);
        assert!(parse_ask(&format!(r#"{{"text":"{long}"}}"#)).unwrap().text.len() <= 200);
        assert!(parse_ask(r#"{"text":""}"#).is_none());
    }

    #[test]
    fn replies_are_read_by_type_and_unknown_actions_become_none() {
        let ok = serde_json::json!({"stop_reason":"end_turn","content":[{"type":"thinking","thinking":""},{"type":"text","text":"{\"say\":\"On my WAY!\",\"action\":\"fish\"}"}]});
        assert_eq!(read_reply(&ok), Some(("fish".into(), "on my way!".into())));
        let odd = serde_json::json!({"content":[{"type":"text","text":"{\"say\":\"hi\",\"action\":\"fly\"}"}]});
        assert_eq!(read_reply(&odd).unwrap().0, "none");
        let no = serde_json::json!({"stop_reason":"refusal","content":[]});
        assert_eq!(read_reply(&no).unwrap().0, "none");
    }
}
