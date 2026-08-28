//! Reproduces Servo page-load timeout behavior against a hanging local page and Reuters.
//!
//! Servo cannot shut down cleanly in-process. Worker tests print `INTEGRATION_TEST_PASSED`
//! and exit before teardown; wrapper tests run them in isolated subprocesses and accept
//! either exit code 0 or a pass marker (Servo may SIGSEGV during process exit).

use std::{
    io::{Read, Write},
    net::{Shutdown, TcpListener},
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

use nodescraper_rs::messages::{
    response::ResponseType, response_error::ResponseError, website_scraper_error::WebsiteError,
    WebsiteScraperArgs,
};
use nodescraper_rs::scrapers::Scraper;
use nodescraper_rs::scrapers::website::WebsiteScraper;

const PASS_MARKER: &str = "INTEGRATION_TEST_PASSED";

fn scrape_error_message(resp: &nodescraper_rs::messages::Response) -> Option<String> {
    match &resp.response_type {
        Some(ResponseType::Error(err)) => match &err.response_error {
            Some(ResponseError::WebsiteScraperError(w)) => match &w.website_error {
                Some(WebsiteError::ScrapeFailed(f)) => Some(f.message.clone()),
                _ => None,
            },
            _ => None,
        },
        _ => None,
    }
}

fn is_ok(resp: &nodescraper_rs::messages::Response) -> bool {
    matches!(resp.response_type, Some(ResponseType::Ok(_)))
}

fn finish_passed_test(scraper: WebsiteScraper) -> ! {
    scraper.shutdown();
    eprintln!("{PASS_MARKER}");
    std::process::exit(0);
}

/// Run an ignored worker test in a fresh process.
fn run_isolated(test_name: &str) {
    let exe = std::env::current_exe().expect("current_exe");
    let output = Command::new(exe)
        .args(["--nocapture", "--exact", test_name, "--ignored"])
        .output()
        .unwrap_or_else(|err| panic!("spawn isolated test subprocess {test_name}: {err}"));

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stdout.is_empty() {
        eprint!("{stdout}");
    }
    if !stderr.is_empty() {
        eprint!("{stderr}");
    }

    let passed = output.status.success() || stderr.contains(PASS_MARKER) || stdout.contains(PASS_MARKER);
    assert!(
        passed,
        "isolated test {test_name} failed: {} (no pass marker)",
        output.status
    );
}

#[test]
fn scrape_hanging_script_times_out_waiting_for_page_load() {
    run_isolated("scrape_hanging_script_times_out_waiting_for_page_load__worker");
}

#[test]
fn scrape_reuters_ipsos_polls_page_load() {
    run_isolated("scrape_reuters_ipsos_polls_page_load__worker");
}

/// Local HTML page whose script never finishes loading — exercises LoadStatus never Complete.
#[tokio::test(flavor = "multi_thread")]
#[ignore = "isolated subprocess worker"]
async fn scrape_hanging_script_times_out_waiting_for_page_load__worker() {
    let _ = env_logger::try_init();
    unsafe {
        std::env::set_var("NODESCRAPER_PAGE_LOAD_TIMEOUT_SECS", "20");
    }

    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let addr = listener.local_addr().expect("addr");
    let stop = Arc::new(AtomicBool::new(false));
    let stop_bg = Arc::clone(&stop);
    let server = thread::spawn(move || {
        listener.set_nonblocking(true).ok();
        let mut hanging: Vec<std::net::TcpStream> = Vec::new();
        while !stop_bg.load(Ordering::Relaxed) {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    let mut buf = [0u8; 1024];
                    let n = stream.read(&mut buf).unwrap_or(0);
                    let req = String::from_utf8_lossy(&buf[..n]);
                    if req.contains("GET /hang") {
                        hanging.push(stream);
                    } else {
                        let body = r#"<!doctype html><html><head>
<script src="/hang"></script>
</head><body><h1>hanging page</h1><p>poll content here</p></body></html>"#;
                        let resp = format!(
                            "HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                            body.len(),
                            body
                        );
                        let _ = stream.write_all(resp.as_bytes());
                        let _ = stream.shutdown(Shutdown::Both);
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(20));
                }
                Err(_) => break,
            }
        }
        hanging.clear();
    });

    let url = format!("http://{addr}/");
    let scraper = WebsiteScraper::init().expect("init scraper");
    let start = Instant::now();
    let resp = scraper
        .scrape(
            1,
            WebsiteScraperArgs {
                source_url: url.clone(),
            },
        )
        .await;
    let elapsed = start.elapsed();

    stop.store(true, Ordering::Relaxed);
    let _ = server.join();

    eprintln!("hanging-script scrape response: {resp:#?}");
    eprintln!("elapsed: {elapsed:?}");

    let msg = scrape_error_message(&resp).expect("expected scrape error for hanging page");
    assert!(
        msg.contains("timed out waiting for page load"),
        "expected page-load timeout; got: {msg} / {resp:#?}"
    );
    assert!(
        elapsed >= Duration::from_secs(15),
        "timeout returned too early: {elapsed:?}"
    );

    finish_passed_test(scraper);
}

/// Live Reuters URL from the reported failure (network + Servo).
#[tokio::test(flavor = "multi_thread")]
#[ignore = "isolated subprocess worker"]
async fn scrape_reuters_ipsos_polls_page_load__worker() {
    let _ = env_logger::try_init();
    unsafe {
        std::env::remove_var("NODESCRAPER_PAGE_LOAD_TIMEOUT_SECS");
    }

    let scraper = WebsiteScraper::init().expect("init scraper");
    let start = Instant::now();
    let resp = scraper
        .scrape(
            127,
            WebsiteScraperArgs {
                source_url: "https://www.reuters.com/reuters-ipsos-polls/".into(),
            },
        )
        .await;
    let elapsed = start.elapsed();

    eprintln!("reuters scrape response: {resp:#?}");
    eprintln!("elapsed: {elapsed:?}");

    if let Some(msg) = scrape_error_message(&resp) {
        eprintln!("reuters scrape error message: {msg}");
        assert!(
            !msg.contains("timed out waiting for page load"),
            "Reuters should not time out once HeadParsed is accepted; got: {msg}"
        );
    } else {
        assert!(is_ok(&resp), "unexpected response shape: {resp:#?}");
    }

    finish_passed_test(scraper);
}
