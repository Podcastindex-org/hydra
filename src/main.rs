use std::error::Error;
use std::fmt;
use std::time::SystemTime;
use rusqlite::{ Connection };
use reqwest::header;

//##: Global definitions
static USERAGENT: &str = "Hydra (PodcastIndex)/v0.1";
struct Podcast {
    id: u64,
    url: String,
    title: String
}
#[derive(Debug)]
struct HydraError(String);

//##: Implement
impl fmt::Display for HydraError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fatal error: {}", self.0)
    }
}
impl Error for HydraError {}


//##: -------------------- Main() -----------------------
//##: ---------------------------------------------------
fn main() {
    //Globals
    //let pi_database_url: &str = "https://cloudflare-ipfs.com/ipns/k51qzi5uqu5dkde1r01kchnaieukg7xy9i6eu78kk3mm3vaa690oaotk1px6wo/podcastindex_feeds.db.tgz";
    let sqlite_file: &str = "podcastindex_feeds.db";

    //Fetch urls
    let podcasts = get_feeds_from_sql(sqlite_file);
    match podcasts {
        Ok(podcasts) => {
            for podcast in podcasts {
                println!("{:#?}|{:#?}|{:#?}", podcast.id, podcast.url, podcast.title);
                check_feed_is_updated(podcast.url.as_str());
            }
        },
        Err(e) => println!("{}", e),
    }
}
//##: ---------------------------------------------------



//##: Get a list of podcasts from the downloaded sqlite db
fn get_feeds_from_sql(sqlite_file: &str) -> Result<Vec<Podcast>, Box<dyn Error>> {
    //Locals
    let mut podcasts: Vec<Podcast> = Vec::new();

    //Restrict to feeds that have updated in a reasonable amount of time
    let since_time: u64 = match SystemTime::now().duration_since(SystemTime::UNIX_EPOCH) {
        Ok(n) => n.as_secs() - (86400 * 90),
        Err(_) => panic!("SystemTime before UNIX EPOCH!"),
    };

    //Connect to the PI sqlite database file
    let sql = Connection::open(sqlite_file);
    match sql {
        Ok(sql) => {
            println!("Got some podcasts.");

            //Run the query and store the result
            let sql_text: String = format!("SELECT id, url, title FROM podcasts WHERE newestItemPubdate > {} LIMIT 1", since_time);
            let stmt = sql.prepare(sql_text.as_str());
            match stmt {
                Ok(mut dbresults) => {
                    let podcast_iter = dbresults.query_map([], |row| {
                        Ok(Podcast {
                            id: row.get(0).unwrap(),
                            url: row.get(1).unwrap(),
                            title: row.get(2).unwrap()
                        })
                    }).unwrap();

                    //Iterate the list and store
                    for podcast in podcast_iter {
                        let pod: Podcast = podcast.unwrap();
                        podcasts.push(pod);
                    }
                },
                Err(e) => return Err(Box::new(HydraError(format!("Error running SQL query: [{}]", e).into())))
            }

            //sql.close();

            Ok(podcasts)
        },
        Err(e) => return Err(Box::new(HydraError(format!("Error running SQL query: [{}]", e).into())))
    }


    


}

//##: Fetch the content of a url
fn fetch_feed(url: &str) -> Result<bool, Box<dyn Error>> {
    let feed_url: &str = url;

    //##: Build the query with the required headers
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static(USERAGENT));
    let client = reqwest::blocking::Client::builder().default_headers(headers).build().unwrap();

    //##: Send the request and display the results or the error
    let res = client.get(feed_url).send();
    match res {
        Ok(res) => {
            println!("Response Status: [{}]", res.status());
            println!("Response Body: {}", res.text().unwrap());
            return Ok(true);
        },
        Err(e) => {
            eprintln!("Error: [{}]", e);
            return Err(Box::new(HydraError(format!("Error running SQL query: [{}]", e).into())));
        }
    }

}


//##: Do a head check on a url to see if it's been modified
fn check_feed_is_updated(url: &str) -> Result<bool, Box<dyn Error>> {

    //##: Build the query with the required headers
    let mut headers = header::HeaderMap::new();
    headers.insert("User-Agent", header::HeaderValue::from_static(USERAGENT));
    //headers.insert("Last-Modified", header::HeaderValue::from_static());
    let client = reqwest::blocking::Client::builder().default_headers(headers).build().unwrap();

    //##: Send the request and display the results or the error
    let res = client.get(url).send();
    match res {
        Ok(res) => {
            println!("Response Status: [{}]", res.status());
            for h in res.headers().into_iter() {
                println!("Response Headers: {:?}", h);
            }

            return Ok(true);
        },
        Err(e) => {
            eprintln!("Error: [{}]", e);
            return Err(Box::new(HydraError(format!("Error running SQL query: [{}]", e).into())));
        }
    }
}






//----------Scratchpad-------------

// use std::io::prelude::*;
// use std::fs::File;
// use std::io::BufReader;
// use futures::stream::StreamExt;

// fn read_lines(path: &str) -> std::io::Result<Vec<String>> {
//     let file = File::open(path)?;
//     let reader = BufReader::new(file);
//     Ok(
//         reader.lines().filter_map(Result::ok).collect()
//     )
// }

// #[tokio::main]
// async fn fetch_feeds(urls_file: &str) -> Result<(), Box<dyn std::error::Error>> {
//     let paths: Vec<String> = read_lines(urls_file)?;
//     let fetches = futures::stream::iter(
//         paths.into_iter().map(|path| {
//             async move {
//                 match reqwest::get(&path).await {
//                     Ok(resp) => {
//                         match resp.text().await {
//                             Ok(text) => {
//                                 println!("RESPONSE: {} bytes from {}", text.len(), path);
//                             }
//                             Err(_) => println!("ERROR reading {}", path),
//                         }
//                     }
//                     Err(_) => println!("ERROR downloading {}", path),
//                 }
//             }
//         })
//     ).buffer_unordered(100).collect::<Vec<()>>();
//     fetches.await;
//     Ok(())
// }