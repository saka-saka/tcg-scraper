use axum::{
    extract::{Query, State},
    response::IntoResponse,
};
use maud::{html, Markup, DOCTYPE};
use meilisearch_sdk::Client;
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

pub async fn pokemon(state: State<MyState>) -> Result<Markup, Error> {
    let exps =
        sqlx::query!("SELECT code, name FROM pokemon_trainer_expansion ORDER BY release_date DESC")
            .fetch_all(&state.pool)
            .await?;
    Ok(html! {
        (DOCTYPE)
        script src="https://unpkg.com/htmx.org@1.9.10" {}
        script src="https://unpkg.com/hyperscript.org@0.9.12" {}
        link rel="stylesheet" href="/stylesheets.css" {}
        form hx-get="/list" hx-trigger="change,load" hx-target="#list" {
            select name="code" {
                @for exp in exps {
                    option value={(exp.code)} {
                        (exp.name) "(" (exp.code) ")"
                    }
                }
            }
        }
        #list {}
    })
}

#[derive(Deserialize)]
pub struct ListQuery {
    code: String,
}
pub async fn list(query: Query<ListQuery>, state: State<MyState>) -> Result<Markup, Error> {
    let cards = sqlx::query!(
        "SELECT name, number, rarity::text, exp_code
        FROM pokewiki
        WHERE exp_code = LOWER($1)
        ORDER BY number",
        query.code
    )
    .fetch_all(&state.pool)
    .await?;
    let markup = html! {
        table #list {
            @for card in cards {
                tr hx-get={ (format!("/modal?code={}", card.name)) } hx-target="body" hx-swap="beforeend" {
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

#[derive(Deserialize)]
pub struct ModalQuery {
    code: String,
}

pub async fn modal(state: State<MyState>, query: Query<ModalQuery>) -> Result<Markup, Error> {
    let card = sqlx::query!(
        "SELECT code, name, kind, number, rarity, expansion_code from pokemon_trainer_printing WHERE code = $1", query.code
    )
    .fetch_one(&state.pool)
    .await?;
    let (n, setsize) = card.number.split_once('/').unwrap();
    Ok(html! {
        #modal _="on closeModal add .closing then wait for animationend then remove me" {
            .modal-underlay _="on click trigger closeModal" {}
            .modal-content {
                h1 { "hihi" }
                div { (card.code) }
                div { (card.name) }
                div { (card.rarity.unwrap()) }
                div {
                    input value={(n)};
                    "/"
                    (setsize)
                }
                div { (card.expansion_code) }
                button _="on click trigger closeModal" { "close" }
                button _="on click trigger closeModal" { "duplicate" }
            }
        }
    })
}

pub async fn stylesheets() -> impl IntoResponse {
    let css = r#"
table {
    font-family: 'Oswald', sans-serif;
    border-collapse: collapse;
}
td {
    padding-right: 10px;
    padding-left: 10px;
    padding-top: 5px;
    padding-bottom: 5px;
}
tr:nth-of-type(even) td {
    background-color: #f3f3f3;
}
/***** MODAL DIALOG ****/
#modal {
	/* Underlay covers entire screen. */
	position: fixed;
	top:0px;
	bottom: 0px;
	left:0px;
	right:0px;
	background-color:rgba(0,0,0,0.5);
	z-index:1000;

	/* Flexbox centers the .modal-content vertically and horizontally */
	display:flex;
	flex-direction:column;
	align-items:center;

	/* Animate when opening */
	animation-name: fadeIn;
	animation-duration:150ms;
	animation-timing-function: ease;
}

#modal > .modal-underlay {
	/* underlay takes up the entire viewport. This is only
	required if you want to click to dismiss the popup */
	position: absolute;
	z-index: -1;
	top:0px;
	bottom:0px;
	left: 0px;
	right: 0px;
}

#modal > .modal-content {
	/* Position visible dialog near the top of the window */
	margin-top:10vh;

	/* Sizing for visible dialog */
	width:80%;
	max-width:600px;

	/* Display properties for visible dialog*/
	border:solid 1px #999;
	border-radius:8px;
	box-shadow: 0px 0px 20px 0px rgba(0,0,0,0.3);
	background-color:white;
	padding:20px;

	/* Animate when opening */
	animation-name:zoomIn;
	animation-duration:150ms;
	animation-timing-function: ease;
}

#modal.closing {
	/* Animate when closing */
	animation-name: fadeOut;
	animation-duration:150ms;
	animation-timing-function: ease;
}

#modal.closing > .modal-content {
	/* Animate when closing */
	animation-name: zoomOut;
	animation-duration:150ms;
	animation-timing-function: ease;
}

@keyframes fadeIn {
	0% {opacity: 0;}
	100% {opacity: 1;}
} 

@keyframes fadeOut {
	0% {opacity: 1;}
	100% {opacity: 0;}
} 

@keyframes zoomIn {
	0% {transform: scale(0.9);}
	100% {transform: scale(1);}
} 

@keyframes zoomOut {
	0% {transform: scale(1);}
	100% {transform: scale(0.9);}
} 
        "#
    .to_string();
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
    Meilisearch(#[from] meilisearch_sdk::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        (StatusCode::INTERNAL_SERVER_ERROR, self.to_string()).into_response()
    }
}
