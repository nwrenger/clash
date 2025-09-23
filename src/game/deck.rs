use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{
    error::{Error, Result},
    game::Settings,
};
use rand::{rng, seq::IteratorRandom};
use reqwest::Client;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json;
use tokio::{fs, io::AsyncWriteExt};

const API_BASE: &str = "https://api.crcast.cc/v1";

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeckInfo {
    pub meta: DeckMeta,
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Deck {
    pub meta: DeckMeta,
    pub blacks: Vec<BlackCard>,
    pub whites: Vec<WhiteCard>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeckMeta {
    pub name: String,
    pub deckcode: String,
    pub language: String,
    pub nsfw: bool,
    pub blacks_count: usize,
    pub whites_count: usize,
    #[serde(default = "empty_timestamp")]
    pub fetched_at: u64,
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

const fn empty_timestamp() -> u64 {
    0
}

impl Deck {
    /// Where to store cached decks
    fn cache_file_path(cache: &Path, code: &str) -> PathBuf {
        let mut cache = cache.to_path_buf();
        cache.push(format!("{}.json", code));
        cache
    }

    /// Try load a cached deck from disk.
    async fn load_cache(cache: &Path, code: &str) -> Result<Self> {
        let path = Self::cache_file_path(cache, code);
        let data = fs::read_to_string(&path).await?;
        let deck = serde_json::from_str(&data)?;
        Ok(deck)
    }

    /// Save a deck to disk cache.
    pub async fn save(&self, cache: &Path) -> Result<()> {
        let path = Self::cache_file_path(cache, &self.meta.deckcode);
        let mut f = fs::File::create(&path).await?;
        let data = serde_json::to_string_pretty(self)?;
        f.write_all(data.as_bytes()).await?;
        Ok(())
    }

    /// Fetch from crcast.cc and convert into our `Deck`
    pub async fn fetch(code: &str) -> Result<Deck> {
        let url = format!("{API_BASE}/decks/{code}");
        let client = Client::new();
        let resp = client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<CrCastApiResponse>()
            .await?;

        let mut deck: Deck = resp.into();
        deck.meta.fetched_at = now();

        Ok(deck)
    }

    /// To format decks into the DeckInfo used in Settings
    fn into_infos(decks: Vec<Deck>, last_info: Option<Vec<DeckInfo>>) -> Vec<DeckInfo> {
        let mut infos: Vec<DeckInfo> = decks
            .into_iter()
            .map(|d| DeckInfo {
                meta: d.meta.clone(),
                enabled: false,
            })
            .collect();

        // Apply enabled from last_info
        if let Some(last_info) = last_info {
            for before in last_info {
                if let Some((i, _)) = infos
                    .iter()
                    .enumerate()
                    .find(|(_i, d)| d.meta.deckcode == before.meta.deckcode)
                {
                    infos[i].enabled = before.enabled;
                }
            }
        }

        infos
    }

    /// Lists all downloaded deck infos (simply loading from disk)
    pub async fn get_all_cached_info(
        cache: &PathBuf,
        last_info: Option<Vec<DeckInfo>>,
    ) -> Result<Vec<DeckInfo>> {
        let all = Deck::get_all_cached(cache).await?;
        Ok(Self::into_infos(all, last_info))
    }

    /// Lists all downloaded deck infos (first updating, then loading)
    pub async fn update_all_cached_info(
        cache: &PathBuf,
        last_info: Option<Vec<DeckInfo>>,
    ) -> Result<Vec<DeckInfo>> {
        let all = Deck::update_all_cached(cache).await?;
        Ok(Self::into_infos(all, last_info))
    }

    /// Helper for reading cached folder and returning decks
    async fn all_cached(cache: &PathBuf) -> Result<Vec<Self>> {
        let mut decks = Vec::new();

        if cache.exists() {
            let mut entries = fs::read_dir(cache).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();

                if path.extension().and_then(|e| e.to_str()) == Some("json") {
                    if let Ok(data) = fs::read_to_string(&path).await {
                        if let Ok(deck) = serde_json::from_str::<Self>(&data) {
                            decks.push(deck);
                        }
                    }
                }
            }
        }
        Ok(decks)
    }

    /// Lists all downloaded decks (simply loading from disk)
    pub async fn get_all_cached(cache: &PathBuf) -> Result<Vec<Self>> {
        let decks = Self::all_cached(cache).await?;
        Ok(decks)
    }

    /// Lists all downloaded decks (first updating, then loading)
    pub async fn update_all_cached(cache: &PathBuf) -> Result<Vec<Self>> {
        let mut decks = Vec::new();

        for mut deck in Self::all_cached(cache).await? {
            if let Ok(fetched) = Self::fetch(&deck.meta.deckcode).await {
                fetched.save(cache).await?;
                deck = fetched;
            }
            decks.push(deck);
        }

        Ok(decks)
    }

    /// Get all decks which are enabled in the `settings`
    pub async fn get_enabled(cache: &Path, settings: &Settings) -> Result<Vec<Deck>> {
        let codes: Vec<&str> = settings
            .decks
            .iter()
            .filter(|di| di.enabled)
            .map(|di| di.meta.deckcode.as_str())
            .collect();

        let mut enabled = Vec::new();
        for code in codes {
            if let Ok(deck) = Self::load_cache(cache, code).await {
                enabled.push(deck);
            }
        }
        Ok(enabled)
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct WhiteCard {
    pub text: String,
}

impl WhiteCard {
    /// Pick multiple random white cards (up to `count`) from all cached decks
    pub async fn choose_random(
        cache: &Path,
        count: usize,
        settings: &Settings,
    ) -> Result<Vec<WhiteCard>> {
        let decks = Deck::get_enabled(cache, settings).await?;

        if !decks.is_empty() {
            let mut rng = rng();
            let whites: Vec<WhiteCard> = decks
                .iter()
                .flat_map(|d| d.whites.iter())
                .choose_multiple(&mut rng, count)
                .into_iter()
                .cloned()
                .collect();

            if whites.is_empty() {
                Err(Error::Deck(String::from("No white cards available")))
            } else {
                Ok(whites)
            }
        } else {
            Err(Error::Deck(String::from("No white cards available")))
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct BlackCard {
    pub text: String,
    pub fields: usize,
}

impl BlackCard {
    /// Pick a single random black card from all cached decks
    pub async fn choose_random(cache: &Path, settings: &Settings) -> Result<BlackCard> {
        let decks = Deck::get_enabled(cache, settings).await?;

        if !decks.is_empty() {
            let mut rng = rng();
            let black = decks
                .iter()
                .flat_map(|d| d.blacks.iter())
                .choose(&mut rng)
                .cloned()
                .ok_or_else(|| Error::Deck(String::from("No black cards available")))?;

            Ok(black)
        } else {
            Err(Error::Deck(String::from("No black cards available")))
        }
    }
}

#[derive(Deserialize, Debug)]
struct CrCastApiResponse {
    deck: CrCastResponse,
}

/// Matches the crcast JSON exactly
#[derive(Deserialize, Debug)]
struct CrCastResponse {
    name: String,
    deckcode: String,
    language: String,
    #[serde(deserialize_with = "bool_from_int")]
    nsfw: bool,
    #[serde(rename = "blacks")]
    raw_blacks: Vec<RawCard>,
    #[serde(rename = "whites")]
    raw_whites: Vec<RawCard>,
    #[serde(rename = "blackCount")]
    raw_blacks_count: usize,
    #[serde(rename = "whiteCount")]
    raw_whites_count: usize,
}

fn bool_from_int<'de, D>(deserializer: D) -> std::result::Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let v = u8::deserialize(deserializer)?;
    Ok(v == 1)
}

#[derive(Deserialize, Debug)]
struct RawCard {
    pub text: String,
}

/// Turn a CrCastApiResponse into your Deck type
impl From<CrCastApiResponse> for Deck {
    fn from(api: CrCastApiResponse) -> Self {
        let deck = api.deck;

        let blacks = deck
            .raw_blacks
            .into_iter()
            .map(|rc| {
                let (mut text, mut fields) = normalize_placeholders(&rc.text);
                // Making sure that blacks without placeholders still have a field and a placeholder at the end
                if fields == 0 {
                    fields += 1;
                    text.push_str(" _");
                }
                BlackCard { text, fields }
            })
            .collect();

        let whites = deck
            .raw_whites
            .into_iter()
            .map(|rc| {
                let (text, _) = normalize_placeholders(&rc.text);
                WhiteCard { text }
            })
            .collect();

        Deck {
            meta: DeckMeta {
                name: deck.name,
                deckcode: deck.deckcode,
                language: deck.language,
                nsfw: deck.nsfw,
                blacks_count: deck.raw_blacks_count,
                whites_count: deck.raw_whites_count,
                fetched_at: empty_timestamp(),
            },
            blacks,
            whites,
        }
    }
}

fn normalize_placeholders(text: &str) -> (String, usize) {
    // If no placeholders, skip the upcomming loop
    if !text.contains('_') {
        return (text.to_owned(), 0);
    }

    let mut normalized = String::new();
    let mut count = 0;
    let mut in_blank = false;

    for ch in text.chars() {
        if ch == '_' {
            if !in_blank {
                // start of a new blank → insert one '_'
                normalized.push('_');
                count += 1;
                in_blank = true;
            }
            // skip extra underscores
        } else {
            in_blank = false;
            normalized.push(ch);
        }
    }

    (normalized, count)
}
