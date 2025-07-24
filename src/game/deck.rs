use std::path::PathBuf;

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
    pub enabled: bool,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Deck {
    pub name: String,
    pub deckcode: String,
    pub blacks: Vec<BlackCard>,
    pub whites: Vec<WhiteCard>,
}

impl Deck {
    /// Where to store cached decks
    fn cache_file_path(mut cache: PathBuf, code: &str) -> PathBuf {
        cache.push(format!("{}.json", code));
        cache
    }

    /// Try load a cached deck from disk. Maybe useful sometime.
    async fn load_cache(cache: PathBuf, code: &str) -> Result<Self> {
        let path = Self::cache_file_path(cache, code);
        let data = fs::read_to_string(&path).await?;
        let deck = serde_json::from_str(&data)?;
        Ok(deck)
    }

    /// Save a deck to disk cache.
    pub async fn save(&self, cache: PathBuf) -> Result<()> {
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

        Ok(resp.into())
    }

    pub async fn get_all_cached_info(
        cache: PathBuf,
        decks_before: Option<Vec<DeckInfo>>,
    ) -> Result<Vec<DeckInfo>> {
        let all = Deck::get_all_cached(cache).await?;

        let mut infos: Vec<DeckInfo> = all
            .into_iter()
            .map(|d| DeckInfo {
                name: d.name.clone(),
                deckcode: d.deckcode.clone(),
                enabled: false,
            })
            .collect();

        // Apply enabled from decks_before
        if let Some(decks_before) = decks_before {
            for before in decks_before {
                if let Some((i, _)) = infos
                    .iter()
                    .enumerate()
                    .find(|(_i, d)| d.deckcode == before.deckcode)
                {
                    infos[i].enabled = before.enabled;
                }
            }
        }

        Ok(infos)
    }

    /// List all downloaded decks by code, loading them from disk
    pub async fn get_all_cached(cache: PathBuf) -> Result<Vec<Self>> {
        let mut decks = Vec::new();
        if cache.exists() {
            let mut rd = fs::read_dir(cache).await?;
            while let Some(entry) = rd.next_entry().await? {
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

    pub async fn get_enabled(cache: PathBuf, settings: &Settings) -> Result<Vec<Deck>> {
        let codes: Vec<&str> = settings
            .decks
            .iter()
            .filter(|ds| ds.enabled)
            .map(|ds| ds.deckcode.as_str())
            .collect();

        let mut enabled = Vec::new();
        for code in codes {
            if let Ok(deck) = Self::load_cache(cache.clone(), code).await {
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
        cache: PathBuf,
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
    pub async fn choose_random(cache: PathBuf, settings: &Settings) -> Result<BlackCard> {
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
        }
    }
}
