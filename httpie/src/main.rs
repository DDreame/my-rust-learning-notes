extern crate clap;
use clap::Parser;
use colored::*;
use reqwest::{header, Client, Response, Url};
use anyhow::{anyhow,Result, Ok};
use mime::Mime;
use std::{collections::HashMap, str::FromStr};

// 定义 HTTPie 的 CLI 的主入口，它包含若干个子命令
// 下面 /// 的注释是文档，clap 会将其作为 CLI 的帮助

/// A naive httpie implementation with Rust, can you imagine how easy it is?
#[derive(Parser, Debug)]
struct Opts{
    #[clap(subcommand)]
    subcmd: SubCommand,
}

/// 目前只支持 get、 post 
#[derive(Parser, Debug)]
enum SubCommand {
    Get(Get),
    Post(Post)

}

/// get 请求 使用 url
#[derive(Parser, Debug)]
struct Get {
    #[arg(value_parser = parse_url)]
    url: String,
}

/// post 请求
#[derive(Parser, Debug)]
struct Post {
    #[arg(value_parser = parse_url)]
    url: String,
    #[arg(value_parser = parse_kv_pair)]
    body: Vec<KvPair>,
}
#[derive(Debug, Clone, PartialEq)]
struct KvPair {
    k: String,
    v: String,
}

impl FromStr for KvPair {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut iter = s.split('=');
        let err = || anyhow!(format!("Failed to parse: {}", s));
        Ok(Self { 
            k: (iter.next().ok_or_else(err)?).to_string(),
            v: (iter.next().ok_or_else(err)?).to_string(), 
        }
        )
    }
}

fn parse_kv_pair(s: &str) -> Result<KvPair> {
    s.parse()
}

fn parse_url(s: &str) -> Result<String> {
    let _url: Url= s.parse()?;
    Ok(s.into())
}

async fn get(client: Client, args: &Get) -> Result<()> {
    let resp = client.get(&args.url).send().await?; 
    Ok(print_resp(resp).await?)
}

async fn post(client: Client, args: &Post) -> Result<()> {
    let mut body = HashMap::new();
    for kv in &args.body {
        body.insert(&kv.k, &kv.v);
    }
    let resp = client.post(&args.url).json(&body).send().await?;
    Ok(print_resp(resp).await?)
}

// 打印服务器版本号 + 状态码
fn print_status(resp: &Response) {
    let status = format!("{:?} {}", resp.version(), resp.status()).blue();
    println!("{}\n", status);
}
// 打印服务器返回的 HTTP header
fn print_headers(resp: &Response) { 
    for (name, value) in resp.headers() {
        println!("{}: {:?}", name.to_string().green(), value);   
    }   
    print!("\n");
}
/// 打印服务器返回的 HTTP body
fn print_body(m: Option<Mime>, body: &String) {
    match m { 
        // 对于 "application/json" 我们 pretty print
        Some(v) if v == mime::APPLICATION_JSON => {   
            println!("{}", jsonxf::pretty_print(body).unwrap().cyan()) 
        }       
        // 其它 mime type，我们就直接输出 
        _ => println!("{}", body),   
    }
}
/// 打印整个响应
async fn print_resp(resp: Response) -> Result<()> { 
    print_status(&resp);  
    print_headers(&resp);
    let mime = get_content_type(&resp);
    let body = resp.text().await?;   
    print_body(mime, &body);
    Ok(())
}
/// 将服务器返回的 content-type 解析成 Mime 类型
fn get_content_type(resp: &Response) -> Option<Mime> { 
    resp.headers().get(header::CONTENT_TYPE)
        .map(|v| v.to_str().unwrap().parse().unwrap())
}

#[tokio::main]
async fn main() -> Result<()>{
    let opts = Opts::parse();
    println!("{:?}", opts);
    let client = Client::new();
    let result = match opts.subcmd{
        SubCommand::Get(ref args) => get(client, args).await?,
        SubCommand::Post(ref args)  => post(client, args).await?,
    };
    
    Ok(result)
}
