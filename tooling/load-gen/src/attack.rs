use indicatif::{ProgressBar, ProgressStyle};
use rand::seq::SliceRandom;
use reqwest::Client;
use serde_json::Value;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::time::{self, Instant};

pub async fn run(
    client: &Client,
    gateway_url: &str,
    admin_url: &str,
    duration: u64,
    rate: u64,
    _workers: usize,
) {
    println!("⚔️  Starting attack: {} rps for {}s", rate, duration);

    // 1. Fetch Snapshot to get Targets (Keys and Routes)
    println!("📥 Fetching targets from Admin API...");
    let snapshot_res = client
        .get(format!("{}/snapshot", admin_url))
        .send()
        .await
        .unwrap();
    let snapshot: Value = snapshot_res.json().await.unwrap();

    let keys: Vec<String> = snapshot["api_keys"]
        .as_object()
        .unwrap()
        .keys()
        .cloned()
        .collect();

    let paths: Vec<String> = snapshot["routes"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| {
            let mp = &r["match_path"];
            if mp["type"] == "Prefix" {
                mp["value"].as_str().map(|s| s.to_string())
            } else {
                None
            }
        })
        .collect();

    if keys.is_empty() {
        panic!("No API keys found! Provision first.");
    }
    if paths.is_empty() {
        panic!("No routes found! Provision first.");
    }

    println!(
        "🎯 Targets loaded: {} keys, {} routes",
        keys.len(),
        paths.len()
    );

    let success_count = Arc::new(AtomicU64::new(0));
    let limit_count = Arc::new(AtomicU64::new(0));
    let upstream_fail_count = Arc::new(AtomicU64::new(0));
    let error_count = Arc::new(AtomicU64::new(0));

    let keys = Arc::new(keys);
    let paths = Arc::new(paths);
    let gateway_url = Arc::new(gateway_url.to_string());

    let pb = ProgressBar::new(duration);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner} {msg}")
            .unwrap(),
    );

    // Loop task
    let client = client.clone();

    // Clone ARCs for the task
    let keys_task = keys.clone();
    let paths_task = paths.clone();
    let gateway_task = gateway_url.clone();

    let success_task = success_count.clone();
    let limit_task = limit_count.clone();
    let upstream_task = upstream_fail_count.clone();
    let error_task = error_count.clone();

    let interval_micros = 1_000_000 / rate;

    let request_task = tokio::spawn(async move {
        let mut interval = time::interval(time::Duration::from_micros(interval_micros));
        let end_time = Instant::now() + time::Duration::from_secs(duration);

        while Instant::now() < end_time {
            interval.tick().await;

            let client = client.clone();
            let keys = keys_task.clone();
            let paths = paths_task.clone();
            let gateway_url = gateway_task.clone();

            let success_cnt = success_task.clone();
            let limit_cnt = limit_task.clone();
            let upstream_cnt = upstream_task.clone();
            let error_cnt = error_task.clone();

            tokio::spawn(async move {
                let (key, url) = {
                    let mut rng = rand::thread_rng();
                    let key = keys.choose(&mut rng).unwrap().clone();
                    let path = paths.choose(&mut rng).unwrap();
                    (key, format!("{}{}", gateway_url, path))
                };

                let res = client.get(&url).header("x-api-key", key).send().await;

                match res {
                    Ok(r) => {
                        let status = r.status();
                        if status.is_success() {
                            success_cnt.fetch_add(1, Ordering::Relaxed);
                        } else if status == 429 {
                            limit_cnt.fetch_add(1, Ordering::Relaxed);
                        } else if status.is_server_error() {
                            upstream_cnt.fetch_add(1, Ordering::Relaxed);
                        } else {
                            error_cnt.fetch_add(1, Ordering::Relaxed);
                        }
                    }
                    Err(_) => {
                        error_cnt.fetch_add(1, Ordering::Relaxed);
                    }
                }
            });
        }
    });

    // Progress loop
    let monitor_success = success_count.clone();
    let monitor_limit = limit_count.clone();
    let monitor_upstream = upstream_fail_count.clone();

    for _ in 0..duration {
        time::sleep(time::Duration::from_secs(1)).await;
        pb.inc(1);
        let s = monitor_success.load(Ordering::Relaxed);
        let l = monitor_limit.load(Ordering::Relaxed);
        let u = monitor_upstream.load(Ordering::Relaxed);
        pb.set_message(format!("OK: {}, 429: {}, 5xx: {}", s, l, u));
    }

    let _ = request_task.await;
    pb.finish();

    println!("🏁 Attack complete.");
    println!("--------------------------------------------------");
    println!(
        "✅ 2xx OK:         {}",
        success_count.load(Ordering::Relaxed)
    );
    println!("⛔ 429 Limited:    {}", limit_count.load(Ordering::Relaxed));
    println!(
        "🔥 5xx Upstream:   {}",
        upstream_fail_count.load(Ordering::Relaxed)
    );
    println!("❌ Errors:         {}", error_count.load(Ordering::Relaxed));
    println!("--------------------------------------------------");
    println!(
        "Total Requests: {}",
        success_count.load(Ordering::Relaxed)
            + limit_count.load(Ordering::Relaxed)
            + upstream_fail_count.load(Ordering::Relaxed)
            + error_count.load(Ordering::Relaxed)
    );
}
