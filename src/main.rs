// cli
extern crate ansi_term;
#[macro_use] extern crate clap;
// validator
extern crate base58;
extern crate crypto;
// json serialization
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate serde_derive;
// https client
extern crate reqwest;

use ansi_term::Colour::{Green, Red, Yellow};
use ansi_term::Style;
use clap::{App, Arg};

use base58::FromBase58;
use crypto::digest::Digest;
use crypto::sha2::Sha256;

use std::io::Read;

#[derive(Serialize, Deserialize, Debug)]
struct Payouts {
  status: String,
  data: Vec<Data>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct Data {
  start: i64,
  end: i64,
  amount: i64,
  tx_hash: String,
  paid_on: i64,
}

fn main() {
    let matches = App::new("fbc")
                          .version(crate_version!())
                          .author(crate_authors!())
                          .about("flypool balance checker")
                          .arg(Arg::from_usage("-w, --wallet=[WALLET_ID] 'Sets wallet id for check'")
                                      .required(true)
                                      .validator(is_zcash_addr))
                          .arg(Arg::from_usage("-d, --debug 'Enable debug'"))
                          .get_matches();

    if matches.is_present("debug") {
      println!("Debugging mode is: {}", Style::new().fg(Green).paint("ON"));
    } else {
      println!("Debugging mode is: {}", Style::new().fg(Yellow).paint("OFF"));
    }

    if let Some(wallet) = matches.value_of("wallet") {
      println!("Value for wallet: {}", Style::new().fg(Green).paint(wallet));
      // begin scope issue
      let endpoint = "https://api-zcash.flypool.org";
      let url = vec!(endpoint, "miner", wallet, "payouts");
      let mut resp = reqwest::get(&url.join("/")).unwrap();
      if resp.status().is_success() {
        let mut body = String::new();
        resp.read_to_string(&mut body).unwrap();
        let p: Payouts = serde_json::from_str(&body).unwrap();
        let amount = |d: &Data| d.amount;
        let amounts: Vec<i64> = p.data.iter().map(amount).collect();
        let sum: i64 = amounts.iter().sum();
        let balance: f64 = sum as f64 / 100000000.0;
        println!("{:?}", balance);
      }
      // end scope issue
    } else {
      println!("{}", Style::new().fg(Red).paint("Set value for wallet"));
      std::process::exit(1);
    }
}

fn is_zcash_addr(val: String) -> Result<(), String> {
  let mut payload: Vec<u8> = match val.from_base58() {
      Ok(payload) => payload,
      Err(_error) => return Err(String::from("some errors.")),
    };

  if payload.len() < 5 {
    return Err(String::from("wrong payload len."))
  }

  let checksum_index = payload.len() - 4;
  let provided_checksum = payload.split_off(checksum_index);
  let checksum = double_sha256(&payload)[..4].to_vec();

  if checksum != provided_checksum {
    return Err(String::from("wrong checksum."))
  }

  Ok(())
}

fn double_sha256(payload: &[u8]) -> Vec<u8> {
    let mut hasher = Sha256::new();
    let mut hash = vec![0; hasher.output_bytes()];
    hasher.input(&payload);
    hasher.result(&mut hash);
    hasher.reset();
    hasher.input(&hash);
    hasher.result(&mut hash);
    hash.to_vec()
}
