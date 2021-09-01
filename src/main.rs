mod plurk;

use plurk::{Plurk, PlurkError, Value};
use std::collections::BTreeMap;

fn main() -> Result<(), PlurkError> {
    let file_name = "keys.toml";
    let plurk = Plurk::from_file(file_name)?;

    let plurk = plurk.auth()?;
    plurk.write_in_file(file_name)?;

    plurk.print();

    let res: Value = plurk.request("/APP/Users/me", None, None)?;

    plurk::print_user(res);

    // let res: Value = plurk.request("/APP/Realtime/getUserChannel", None, None)?;

    // println!("comet_server: {}", res["comet_server"]);
    // println!("channel_name: {}", res["channel_name"]);

    // let v: Value = plurk.request("/APP/Profile/getOwnProfile", None, None)?;
    // println!("{:?}", v);

    // let mut data: BTreeMap<String, String> = BTreeMap::new();
    // data.insert("content".to_string(), "This plurk is send from rust 🦀.".to_string());
    // data.insert("qualifier".to_string(), "says".to_string());
    // let v: Value = plurk.request("/APP/Timeline/plurkAdd", Some(data), None)?;
    // println!("{:?}", v);

    // let mut file: BTreeMap<String, String> = BTreeMap::new();
    // file.insert("image".to_string(), "/Users/dephilia/Pictures/datas/vTuber/Ee2tEXoU8AABOjk.jpeg".to_string());
    // let v: Value = plurk.request("/APP/Timeline/uploadPicture", None, Some(file))?;
    // println!("{:?}", v);

    Ok(())
}
