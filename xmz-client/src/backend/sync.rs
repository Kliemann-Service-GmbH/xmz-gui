use backend::types::Backend;
use backend::types::BKResponse;
use error::Error;
use std::thread;
use utils::json_q;


pub fn sync(bk: &Backend) -> Result<(), Error> {

    let since = bk.data.lock().unwrap().since.clone();

    let mut params: Vec<(&str, String)> = vec![];

    let timeout;

    if since.is_empty() {
        timeout = 0;
    } else {
        timeout = 30;
    }

    let baseurl = bk.get_base_url()?;
    let url = bk.url("", params)?;

    let tx = bk.tx.clone();
    let data = bk.data.clone();

    let attrs = json!(null);

    thread::spawn(move || {
        match json_q("get", &url, &attrs, timeout) {
            Ok(r) => {},
            Err(err) => { tx.send(BKResponse::SyncError(err)).unwrap() }
        };
    });

    Ok(())
}
