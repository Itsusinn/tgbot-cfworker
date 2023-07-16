#![feature(let_chains)]

use telegram_bot_api::{bot, methods, types};
use worker::RouteContext;

type WorkerRequest = worker::Request;
type WorkerEnv = worker::Env;
type WorkerResponse = worker::Response;

#[worker::event(fetch)]
pub async fn main(
  req: WorkerRequest,
  env: WorkerEnv,
  _: worker::Context,
) -> worker::Result<WorkerResponse> {
  let router = worker::Router::new();
  router
    .post_async("/webhook", |mut req, ctx| async move {
      let update: types::Update = serde_json::from_value(req.json().await?)?;
      convert_err(handle_update(ctx, update).await)
    })
    .get_async("/get_me", |_req, ctx| async move {
      convert_err(get_me(ctx).await)
    })
    .get_async("/init_webhook", |_req, ctx| async move {
      convert_err(init_webhook(ctx).await)
    })
    .run(req, env)
    .await
}

async fn get_me(ctx: RouteContext<()>) -> Result<WorkerResponse, AppError> {
  let token = ctx.secret("BOT_TOKEN")?.to_string();
  let bot = bot::BotApi::new(token, None).await?;
  let me = bot.get_me().await?;
  Ok(WorkerResponse::from_html(format!("{:#?}", me)).unwrap())
}

async fn init_webhook(ctx: RouteContext<()>) -> Result<WorkerResponse, AppError> {
  let token = ctx.secret("BOT_TOKEN")?.to_string();
  let webhook_url = ctx.secret("WEBHOOK_URL")?.to_string();
  let bot = bot::BotApi::new(token, None).await?;
  bot
    .set_webhook(methods::SetWebhook::new(webhook_url.clone()))
    .await?;
  Ok(WorkerResponse::from_html(format!("Set webhook to {webhook_url}",)).unwrap())
}

async fn handle_update(
  ctx: RouteContext<()>,
  update: types::Update,
) -> Result<WorkerResponse, AppError> {
  let token = ctx.secret("BOT_TOKEN")?.to_string();
  let bot = bot::BotApi::new(token.clone(), None).await?;
  // if let Some(message) = &update.message {
  //   let chat = &message.chat;
  //   bot
  //     .send_message(methods::SendMessage::new(
  //       chat.id.into(),
  //       "I'm a ... crab!".to_string(),
  //     ))
  //     .await?;
  // }
  // if let Some(message) = update.message
  // && let Some(sticker) = message.sticker
  // && let Some(file_path) = bot.get_file(methods::GetFile::new(sticker.file_id)).await?.file_path {
  //   let chat = message.chat;
  //   let file_url = format!("https://api.telegram.org/file/bot{token}/{file_path}");
  //   // let bytes = reqwest::get(file_url).await?.bytes().await?;
  //   let photo = types::InputFile::FileURL(file_url);
  //   bot.send_photo(methods::SendPhoto::new(chat.id.into(), photo)).await?;
  // }
  if let Some(message) = update.message
  && let Some(sticker) = message.sticker {
    let chat = message.chat;
    // let bytes = reqwest::get(file_url).await?.bytes().await?;
    let photo = types::InputFile::FileID(sticker.file_id);
    bot.send_photo(methods::SendPhoto::new(chat.id.into(), photo)).await?;
  }
  Ok(WorkerResponse::from_html("True").unwrap())
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
  #[error(transparent)]
  WorkerError(#[from] worker::Error),
  #[error(transparent)]
  ReqwestError(#[from] reqwest::Error),
  #[error(transparent)]
  BoxError(#[from] Box<dyn std::error::Error>),
}

fn convert_err(former: Result<WorkerResponse, AppError>) -> worker::Result<WorkerResponse> {
  match former {
    Ok(v) => Ok(v),
    Err(err) => {
      worker::console_error!("{}", err);
      // Err(worker::Error::RustError(format!("{}", err)))
      WorkerResponse::from_html("True")
    }
  }
}
