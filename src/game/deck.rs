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
use serde::{Deserialize, Serialize};
use serde_json;
use tokio::{fs, io::AsyncWriteExt};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DeckInfo {
    pub name: String,
    pub deckcode: String,
    pub blacks_count: usize,
    pub whites_count: usize,
    pub enabled: bool,
    pub fetched_at: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Deck {
    pub name: String,
    pub deckcode: String,
    pub blacks: Vec<BlackCard>,
    pub whites: Vec<WhiteCard>,
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

    /// Try load a cached deck from disk. Maybe useful sometime.
    async fn load_cache(cache: &Path, code: &str) -> Result<Self> {
        let path = Self::cache_file_path(cache, code);
        let data = fs::read_to_string(&path).await?;
        let deck = serde_json::from_str(&data)?;
        Ok(deck)
    }

    /// Save a deck to disk cache.
    pub async fn save(&self, cache: &Path) -> Result<()> {
        let path = Self::cache_file_path(cache, &self.deckcode);
        let mut f = fs::File::create(&path).await?;
        let data = serde_json::to_string_pretty(self)?;
        f.write_all(data.as_bytes()).await?;
        Ok(())
    }

    /// Fetch from crcast.cc and convert into our `Deck`
    pub async fn fetch(code: &str) -> Result<Deck> {
        let url = format!("https://api.crcast.cc/v1/cc/decks/{}/all", code);
        let client = Client::new();
        let resp = client
            .get(&url)
            .send()
            .await?
            .error_for_status()?
            .json::<CrCastResponse>()
            .await?;

        let mut deck: Deck = resp.into();
        deck.fetched_at = now();

        Ok(deck)
    }

    /// To format decks into the DeckInfo used in Settings
    fn into_info(decks: Vec<Deck>, last_info: Option<Vec<DeckInfo>>) -> Vec<DeckInfo> {
        let mut infos: Vec<DeckInfo> = decks
            .into_iter()
            .map(|d| DeckInfo {
                name: d.name.clone(),
                deckcode: d.deckcode.clone(),
                blacks_count: d.blacks.len(),
                whites_count: d.whites.len(),
                enabled: false,
                fetched_at: d.fetched_at,
            })
            .collect();

        // Apply enabled from last_info
        if let Some(last_info) = last_info {
            for before in last_info {
                if let Some((i, _)) = infos
                    .iter()
                    .enumerate()
                    .find(|(_i, d)| d.deckcode == before.deckcode)
                {
                    infos[i].enabled = before.enabled;
                }
            }
        }

        infos
    }

    /// Lists all downloaded deck infos
    ///
    /// Simply loading them from disk
    pub async fn get_all_cached_info(
        cache: &PathBuf,
        last_info: Option<Vec<DeckInfo>>,
    ) -> Result<Vec<DeckInfo>> {
        let all = Deck::get_all_cached(cache).await?;
        Ok(Self::into_info(all, last_info))
    }

    /// Lists all downloaded decks
    ///
    /// First updating and then loading them from disk
    pub async fn update_all_cached_info(
        cache: &PathBuf,
        last_info: Option<Vec<DeckInfo>>,
    ) -> Result<Vec<DeckInfo>> {
        let all = Deck::update_all_cached(cache).await?;
        Ok(Self::into_info(all, last_info))
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

    /// Lists all downloaded decks
    ///
    /// Simply loading them from disk
    pub async fn get_all_cached(cache: &PathBuf) -> Result<Vec<Self>> {
        let decks = Self::all_cached(cache).await?;
        Ok(decks)
    }

    /// Lists all downloaded decks
    ///
    /// First updating and then loading them from disk
    pub async fn update_all_cached(cache: &PathBuf) -> Result<Vec<Self>> {
        let mut decks = Vec::new();

        for mut deck in Self::all_cached(cache).await? {
            if let Ok(fetched) = Self::fetch(&deck.deckcode).await {
                fetched.save(cache).await?;
                deck = fetched;
            }
            decks.push(deck);
        }

        Ok(decks)
    }

    /// Get all extensions which are enabled in the `settings`
    pub async fn get_enabled(cache: &Path, settings: &Settings) -> Result<Vec<Deck>> {
        let codes: Vec<&str> = settings
            .decks
            .iter()
            .filter(|ds| ds.enabled)
            .map(|ds| ds.deckcode.as_str())
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

/// Matches the crcast JSON exactly
#[derive(Deserialize)]
struct CrCastResponse {
    name: String,
    watermark: String,
    #[serde(rename = "calls")]
    raw_blacks: Vec<RawBlackCard>,
    #[serde(rename = "responses")]
    raw_whites: Vec<RawWhiteCard>,
}

#[derive(Deserialize)]
struct RawBlackCard {
    pub text: Vec<String>,
}

#[derive(Deserialize)]
struct RawWhiteCard {
    pub text: Vec<String>,
}

/// Turn a CrCastResponse into your Deck type
impl From<CrCastResponse> for Deck {
    fn from(api: CrCastResponse) -> Self {
        let name = api.name;
        let code = api.watermark;
        let blacks = api
            .raw_blacks
            .into_iter()
            .map(|rb| {
                let text = rb.text.join(" _ ");
                let fields = rb.text.len().saturating_sub(1);
                BlackCard { text, fields }
            })
            .collect();

        let whites = api
            .raw_whites
            .into_iter()
            .filter_map(|rw| rw.text.into_iter().next())
            .map(|t| WhiteCard { text: t })
            .collect();

        Deck {
            name,
            deckcode: code,
            blacks,
            whites,
            fetched_at: empty_timestamp(),
        }
    }
}
