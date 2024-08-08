use std::{collections::HashMap, str::FromStr};

use anyhow::anyhow;
use anyhow::Result;
use colored::*;

use clap::Parser;
use mime::Mime;
use reqwest::{self, header, Client, Response, Url};

/// minihttpie
#[derive(Parser, Debug)]
#[clap(version = "1.0", author = "ncp-z@npc-z.com")]
#[clap(propagate_version = true)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

/// sub-commands, support get / post now.
#[derive(Parser, Debug)]
enum SubCommand {
    Get(Get),
    Post(Post),
}

/// feed get with an url.
#[derive(Parser, Debug)]
struct Get {
    /// get url
    #[clap(parse(try_from_str = parse_url))]
    url: String,
}

/// feed post with an url and optional key=value pairs as body.
#[derive(Parser, Debug)]
struct Post {
    /// post url
    #[clap(parse(try_from_str = parse_url))]
    url: String,
    /// post body
    #[clap(parse(try_from_str = parse_kv_pair))]
    body: Vec<KvPair>,
}

#[derive(Debug, PartialEq)]
struct KvPair {
    k: String,
    v: String,
}

impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut split = s.split("=");
        let err = || anyhow!(format!("Failed to parse {}", s));
        Ok(Self {
            k: (split.next().ok_or_else(err)?).to_string(),
            v: (split.next().ok_or_else(err)?).to_string(),
        })
    }
}

fn parse_kv_pair(s: &str) -> Result<KvPair> {
    Ok(s.parse()?)
}

fn parse_url(url: &str) -> Result<String> {
    let _url: Url = url.parse()?;
    Ok(url.into())
}

async fn get(client: Client, args: &Get) -> Result<()> {
    let resp = client.get(&args.url).send().await?;
    Ok(print_resp(resp).await?)
}

async fn post(client: Client, args: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for pair in args.body.iter() {
        body.insert(&pair.k, &pair.v);
    }
    let resp = client.post(&args.url).json(&body).send().await?;
    Ok(print_resp(resp).await?)
}

fn print_status(resp: &Response) {
    let status = format!("{:?} {}", resp.version(), resp.status()).blue();
    println!("{}\n", status);
}

fn print_headers(resp: &Response) {
    for (name, value) in resp.headers() {
        println!("{}: {:?}", name.to_string().green(), value);
    }
}

fn print_body(m: Option<Mime>, body: &String) {
    match m {
        Some(v) if v == mime::APPLICATION_JSON => {
            println!("{}", jsonxf::pretty_print(body).unwrap().cyan())
        }
        _ => println!("{}", body),
    }
}

async fn print_resp(resp: Response) -> Result<()> {
    print_status(&resp);
    print_headers(&resp);

    let mime = get_content_type(&resp);
    let body = resp.text().await?;
    println!();
    print_body(mime, &body);
    Ok(())
}

fn get_content_type(resp: &Response) -> Option<Mime> {
    resp.headers()
        .get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

#[tokio::main]
async fn main() -> Result<()> {
    let opts: Opts = Opts::parse();
    // dbg!(opts);

    let mut headers = header::HeaderMap::new();
    headers.insert("X-POWERED", "Rust".parse()?);
    headers.insert(header::USER_AGENT, "Rust Httpie".parse()?);

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let result = match opts.subcmd {
        SubCommand::Get(ref args) => get(client, args).await?,
        SubCommand::Post(ref args) => post(client, args).await?,
    };

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_url() {
        assert!(parse_url("abc").is_err());
        assert!(parse_url("http://abc.xyz").is_ok());
        assert!(parse_url("https://goog.job").is_ok());
    }
    #[test]
    fn test_parse_kv_pair() {
        assert!(parse_kv_pair("a").is_err());

        assert_eq!(
            parse_kv_pair("name=bob").unwrap(),
            KvPair {
                k: "name".into(),
                v: "bob".into()
            }
        );

        assert_eq!(
            parse_kv_pair("age=").unwrap(),
            KvPair {
                k: "age".into(),
                v: "".into()
            }
        );
    }
}
