mod plurk;
mod comet;

use plurk::{Plurk, PlurkError};
use comet::{PlurkComet};
use std::collections::BTreeSet;

fn main() -> Result<(), PlurkError> {
    let file_name = "keys.toml";
    let plurk = Plurk::from_file(file_name)?;

    let plurk = plurk.auth()?;
    plurk.write_in_file(file_name)?;

    plurk.print();

    let res = plurk.request("/APP/Users/me", None, None)?;

    plurk::print_user(res);

    let res = plurk.request("/APP/Realtime/getUserChannel", None, None)?;

    let mut comet = PlurkComet::new(res["comet_server"].as_str().unwrap())?;
    comet.print();

    let res = comet.call_once_mut()?;
    println!("{:?}", res);


    // let res = plurk.request("/APP/Profile/getOwnProfile", None, None)?;
    // println!("{:?}", res);

    // let mut data: BTreeSet<(&str, &str)> = BTreeSet::new();
    // data.insert(("content", "This plurk is send from rust 🦀."));
    // data.insert(("qualifier", "says"));
    // let res = plurk.request("/APP/Timeline/plurkAdd", Some(data), None)?;
    // println!("{:?}", res);

    // let mut file: BTreeSet<(&str, &str)> = BTreeSet::new();
    // file.insert(("image", "/Users/dephilia/Pictures/datas/vTuber/Ee2tEXoU8AABOjk.jpeg"));
    // let res = plurk.request("/APP/Timeline/uploadPicture", None, Some(file))?;
    // println!("{:?}", res);

    Ok(())
}
