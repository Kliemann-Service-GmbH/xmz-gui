// In diesem Modul sollte später die Authentication gegen den Server erfolgen

use error::Error;
use backend::types::{Backend, BKResponse};
use globals;
use utils::json_q;



pub fn login(bk: &Backend, url: String) -> Result<(), Error> {
    bk.data.lock().unwrap().server_url = url;
    let url = bk.url("", vec![])?;

    let tx = bk.tx.clone();
    let attrs = json!(null);

    match json_q("get", &url, &attrs, globals::TIMEOUT) {
            Ok(_js) => {
                tx.send(BKResponse::ConnectSuccessfull).unwrap();
            },
            Err(err) => {
                tx.send(BKResponse::LoginError(err)).unwrap();
            }
        }

    Ok(())
}
