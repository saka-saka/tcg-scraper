use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use maud::{html, Markup, DOCTYPE};
use meilisearch_sdk::client::Client;
use reqwest::{header::CONTENT_TYPE, StatusCode};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Postgres};

use crate::application::ptcg::Ptcg;

#[derive(Clone)]
pub struct MyState {
    pub pool: Pool<Postgres>,
    pub client: Client,
    pub ptcg: Ptcg,
}

pub async fn root() -> Markup {
    html! {
        script src="https://unpkg.com/htmx.org@1.9.10" {}
        title { "hello world" }
        h1 { "hello world" }
        input class="search"
            type="search"
            name="query"
            hx-get="/search"
            hx-trigger="input changed delay:500ms, load"
            placeholder="type something here";
        #spent {}
        #hits {}
        div.table {
            div.thead {
                div.tr {
                    span.th { "name" }
                }
            }
            div.tbody #search-result {}
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct SearchParam {
    query: String,
}

#[derive(Deserialize, Serialize, Debug)]
struct Card {
    list_item: ListItem,
}

#[derive(Deserialize, Serialize, Debug)]
struct ListItem {
    name: String,
}

pub async fn search(param: Query<SearchParam>, state: State<MyState>) -> Result<Markup, Error> {
    let cards = state.client.index("cards");
    let result = cards
        .search()
        .with_query(&param.query)
        .execute::<Card>()
        .await?;
    Ok(html! {
        #spent hx-swap-oob="true" { (result.processing_time_ms) }
        #hits hx-swap-oob="true" {
            @if let Some(total_hits) = result.estimated_total_hits {
                (total_hits)
            } @else {
                "N/A"
            }
        }
        div.tbody #search-result hx-swap-oob="true" {
            @for card in result.hits {
                div.tr {
                    span.td { (card.result.list_item.name) }
                }
            }
        }
    })
}

pub async fn pokemon() -> Result<Markup, Error> {
    Ok(html! {
        (DOCTYPE)
        script src="https://unpkg.com/htmx.org@1.9.10" {}
        script src="https://unpkg.com/hyperscript.org@0.9.12" {}
        link rel="stylesheet" href="/stylesheets.css" {}
        #sapper {
            .flex {
                section.flex.row.wrap.quater {}
                section.flex.row.wrap.half.screen-v-scroll.py {
                    .large.title {}
                    #list {}
                }
                section.flex.row.wrap.quater.screen-v-scroll.noselect hx-get="/explist" hx-trigger="load" hx-target="#explist" {
                    #explist {}
                }
            }
        }
    })
}

#[derive(Deserialize)]
pub struct ListQuery {
    code: String,
}
pub async fn list(query: Query<ListQuery>, state: State<MyState>) -> Result<Markup, Error> {
    let cards = sqlx::query!(
        r#"
        SELECT
            COALESCE(ptp.name, wiki.name) AS "name!",
            COALESCE(ptp.number, wiki.number) AS "number!",
            COALESCE(ptp.expansion_code, wiki.exp_code) "exp_code!",
            COALESCE(ptp.rarity, wiki.rarity::TEXT) rarity,
            ptp.code as "code?"
        FROM pokemon_trainer_printing ptp
        FULL JOIN pokewiki wiki
            ON LOWER(wiki.exp_code) = LOWER(ptp.expansion_code)
            AND wiki.name = ptp.name
            AND wiki.number = ptp.number
        WHERE
            LOWER(ptp.expansion_code) = LOWER($1) OR LOWER(wiki.exp_code) = LOWER($1)
        ORDER BY "number!"
        "#,
        query.code
    )
    .fetch_all(&state.pool)
    .await?;
    let markup = html! {
        h1 { (query.code) }
        table #list {
            @for card in cards {
                tr hx-get={ (format!("/modal?name={}&number={}&exp_code={}", card.name, card.number, card.exp_code)) } hx-target="body" hx-swap="beforeend" {
                    td { img.table_img src={(format!("https://asia.pokemon-card.com/tw/card-img/tw{:08}.png", card.code.unwrap_or("0".to_string()).parse::<i32>().unwrap()))}; }
                    td { (card.name) }
                    td { (card.number) }
                    td { (card.rarity.unwrap_or("Unknown".to_string())) }
                    td { (card.exp_code) }
                }
            }
        }
    };
    Ok(markup)
}
pub async fn exp_list(state: State<MyState>) -> Result<Markup, Error> {
    let exps =
        sqlx::query!("SELECT code, name FROM pokemon_trainer_expansion ORDER BY release_date DESC")
            .fetch_all(&state.pool)
            .await?;
    let markup = html! {
        .pad {
            @for exp in exps {
                a.green href="#" _="on click take .selected from a.green for the event's target" hx-get={ (format!("/list?code={}", exp.code)) } hx-target="#list" {
                    (format!("{:<5}:{}",exp.code, exp.name))
                }
                br;
            }
        }
    };
    Ok(markup)
}

#[derive(Deserialize)]
pub struct ModalQuery {
    name: String,
    number: String,
    exp_code: String,
}

pub async fn modal(state: State<MyState>, query: Query<ModalQuery>) -> Result<Markup, Error> {
    let card = sqlx::query!(
        r#"
        SELECT
            COALESCE(ptp.name, wiki.name) AS "name!",
            COALESCE(ptp.number, wiki.number) AS "number!",
            COALESCE(ptp.expansion_code, wiki.exp_code) "exp_code!",
            COALESCE(ptp.rarity, wiki.rarity::TEXT) rarity
        FROM pokemon_trainer_printing ptp
        FULL JOIN pokewiki wiki
            ON LOWER(wiki.exp_code) = LOWER(ptp.expansion_code)
            AND wiki.name = ptp.name
            AND wiki.number = ptp.number
        WHERE
            (LOWER(ptp.expansion_code) = LOWER($1) OR LOWER(wiki.exp_code) = LOWER($1))
            AND
            (ptp.name = $2 OR wiki.name = $2)
            AND
            (ptp.number = $3 OR wiki.number = $3)
        "#,
        query.exp_code,
        query.name,
        query.number
    )
    .fetch_one(&state.pool)
    .await?;

    let (n, setsize) = card.number.split_once('/').unwrap();
    Ok(html! {
        #modal _="on closeModal add .closing then wait for animationend then remove me" {
            .modal-underlay _="on click trigger closeModal" {}
            .modal-content {
                h1 { "hihi" }
                div { (card.name) }
                div { (card.rarity.unwrap_or("Unknown".to_string())) }
                div {
                    input value={(n)};
                    "/"
                    (setsize)
                }
                div { (card.exp_code) }
                button _="on click trigger closeModal" { "close" }
                button _="on click trigger closeModal" { "duplicate" }
            }
        }
    })
}

pub async fn stylesheets() -> impl IntoResponse {
    let css = include_str!("stylesheets.css");
    ([(CONTENT_TYPE, "text/css")], css)
}

pub async fn prepare(state: State<MyState>) -> Result<(), Error> {
    state.ptcg.prepare_ptcg_expansions().await.unwrap();
    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("sqlx error {0}")]
    SQLx(#[from] sqlx::Error),
    #[error("meilisearch error {0}")]
    Meilisearch(#[from] meilisearch_sdk::errors::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
