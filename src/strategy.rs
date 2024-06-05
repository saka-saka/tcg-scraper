use std::{collections::HashMap, ops::Range};

use serde::Deserialize;

#[derive(Deserialize, Debug)]
#[serde(tag = "source", rename_all = "snake_case")]
pub enum Source {
    Manual(ManualStrategy),
    Ptcg(PtcgStrategy),
    Wiki(WikiStrategy),
    TcgCollector(TcgCollectorStrategy),
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ManualStrategy {
    Data(Data),
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct Data {
    card_data: Vec<CardData>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub struct CardData {
    number: String,
    name: String,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PtcgStrategy {
    All,
    Pic,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WikiStrategy {
    Data(WikiData),
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum TcgCollectorStrategy {
    Pic(TcgCollectorPic),
    PicByName(PicByName),
    PicMappings(PicMappings),
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub struct PicByName {
    exps: Vec<String>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub struct TcgCollectorPic {
    range: Range<i32>,
}

#[derive(Deserialize, Debug)]
#[serde(tag = "type", rename_all = "snake_case")]
pub struct PicMappings {
    mappings: HashMap<String, String>,
}

#[derive(Deserialize, Debug)]
pub struct WikiData {
    url: url::Url,
    range: Option<Range<i32>>,
}

impl WikiData {
    pub fn url(&self) -> url::Url {
        self.url.clone()
    }
    pub fn range(&self) -> Option<Range<i32>> {
        self.range.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ptcg_all() {
        let json = r#"
  {
    "source": "ptcg",
    "type": "all"
  }
        "#;
        let _source: Source = serde_json::from_str(json).unwrap();
    }
    #[test]
    fn test_ptcg_pic() {
        let json = r#"
  {
    "source": "ptcg",
    "type": "pic"
  }
        "#;
        let _source: Source = serde_json::from_str(json).unwrap();
    }
    #[test]
    fn test_wiki_data() {
        let json = r#"
  {
    "source": "wiki",
    "url": "https://wiki.52poke.com/wiki/%E8%BF%9E%E5%87%BB%E5%A4%A7%E5%B8%88%EF%BC%88TCG%EF%BC%89",
    "type": "data",
    "range": [1, 100]
  }
        "#;
        let _source: Source = serde_json::from_str(json).unwrap();
    }
    #[test]
    fn test_tcg_collector_by_name() {
        let json = r#"
  {
    "source": "tcg_collector",
    "type": "pic_by_name",
    "exps": ["sm8b", "sm4+"]
  }
        "#;
        let _source: Source = serde_json::from_str(json).unwrap();
    }
    #[test]
    fn test_tcg_collector_pic_mappings() {
        let json = r#"
  {
    "source": "tcg_collector",
    "type": "pic_mappings",
    "mappings": { "185": "sm9|96" }
  }
        "#;
        let _source: Source = serde_json::from_str(json).unwrap();
    }
    #[test]
    fn test_vec_sources1() {
        let json = r#"[
  {
    "source": "ptcg",
    "type": "all"
  },
  {
    "source": "wiki",
    "url": "https://wiki.52poke.com/wiki/%E8%BF%9E%E5%87%BB%E5%A4%A7%E5%B8%88%EF%BC%88TCG%EF%BC%89",
    "type": "data",
    "range": [71, 91]
  },
  {
    "source": "tcg_collector",
    "type": "pic",
    "range": [71, 91]
  }
]"#;
        let _source: Vec<Source> = serde_json::from_str(json).unwrap();
    }
    #[test]
    fn test_vec_sources2() {
        let json = r#"[
  {
    "source": "wiki",
    "url": "https://wiki.52poke.com/wiki/%E4%BC%97%E6%98%9F%E4%BA%91%E9%9B%86%E7%BB%84%E5%90%88%E7%AF%87_SET_B%EF%BC%88TCG%EF%BC%89",
    "type": "data"
  },
  {
    "source": "ptcg",
    "type": "pic"
  },
  {
    "source": "tcg_collector",
    "type": "pic_by_name",
    "exps": ["sm8b", "sm4+"]
  }
]"#;
        let _source: Vec<Source> = serde_json::from_str(json).unwrap();
    }
    #[test]
    fn test_vec_sources3() {
        let json = r#"[
  {
    "source": "wiki",
    "url": "https://wiki.52poke.com/wiki/%E5%8F%8C%E5%80%8D%E7%88%86%E5%87%BB_SET_A%EF%BC%88TCG%EF%BC%89",
    "type": "data"
  },
  {
    "source": "ptcg",
    "type": "pic"
  },
  {
    "source": "tcg_collector",
    "type": "pic_by_name",
    "exps": ["sm10", "sm10a", "sm10b", "sm9", "sm9a", "sm9b"]
  },
  {
    "source": "tcg_collector",
    "type": "pic_mappings",
    "mappings": {
      "185": "sm9|96",
      "186": "sm9b|99"
    }
  }
]"#;
        let _source: Vec<Source> = serde_json::from_str(json).unwrap();
    }
}
