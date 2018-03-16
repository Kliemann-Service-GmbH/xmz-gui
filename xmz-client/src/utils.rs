use error::Error;
use serde_json::Value as JsonValue;
use std::time::Duration as StdDuration;
use url::Url;
use reqwest;


// from https://stackoverflow.com/a/43992218/1592377
#[macro_export]
macro_rules! clone {
    (@param _) => ( _ );
    (@param $x:ident) => ( $x );
    ($($n:ident),+ => move || $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move || $body
        }
    );
    ($($n:ident),+ => move |$($p:tt),+| $body:expr) => (
        {
            $( let $n = $n.clone(); )+
            move |$(clone!(@param $p),)+| $body
        }
    );
}

#[macro_export]
macro_rules! client_url {
    ($b:expr, $path:expr, $params:expr) => (
        build_url($b, &format!("/{}", $path), $params)
    )
}



pub fn build_url(base: &Url, path: &str, params: Vec<(&str, String)>) -> Result<Url, Error> {
    let mut url = base.join(path)?;

    {
        let mut query = url.query_pairs_mut();
        query.clear();
        for (k, v) in params {
            query.append_pair(k, &v);
        }
    }

    Ok(url)
}


pub fn json_q(method: &str, url: &Url, attrs: &JsonValue, timeout: u64) -> Result<JsonValue, Error> {
    let mut clientbuilder = reqwest::ClientBuilder::new();
    let client = match timeout {
        0 => clientbuilder.build()?,
        n => clientbuilder.timeout(StdDuration::from_secs(n)).build()?
    };

    println!("url: {}", &url);
    let mut conn = match method {
        "post" => client.post(url.as_str()),
        "put" => client.put(url.as_str()),
        "delete" => client.delete(url.as_str()),
        _ => client.get(url.as_str()),
    };

    let conn2 = conn.json(attrs);
    let mut res = conn2.send()?;

    if !res.status().is_success() {
        return match res.json() {
            Ok(js) => Err(Error::XMZError(js)),
            Err(err) => Err(Error::ReqwestError(err))
        }
    }

    let json: Result<JsonValue, reqwest::Error> = res.json();
    match json {
        Ok(js) => {
            // Evtl. mehr Fehler auslesen
            Ok(js)
        },
        Err(_) => Err(Error::BackendError),
    }
}
