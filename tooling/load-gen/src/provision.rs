use indicatif::{ProgressBar, ProgressStyle};
use reqwest::Client;
use serde_json::json;
use uuid::Uuid;

pub async fn run(
    client: &Client,
    admin_url: &str,
    num_tenants: usize,
    services_per_tenant: usize,
    keys_per_tenant: usize,
) {
    println!("🚀 Provisioning {} tenants...", num_tenants);

    let pb = ProgressBar::new(num_tenants as u64);
    pb.set_style(
        ProgressStyle::default_bar()
            .template(
                "{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("#>-"),
    );

    for i in 0..num_tenants {
        let tenant_slug = format!(
            "tenant-{}",
            Uuid::new_v4().to_string().split('-').next().unwrap()
        );

        // 1. Create Tenant
        let tenant_res = client
            .post(format!("{}/tenants", admin_url))
            .json(&json!({
                "slug": tenant_slug,
                "display_name": format!("Load Test Tenant {}", i)
            }))
            .send()
            .await
            .unwrap();

        if !tenant_res.status().is_success() {
            eprintln!(
                "Failed to create tenant: {:?}",
                tenant_res.text().await.unwrap()
            );
            continue;
        }
        let tenant: serde_json::Value = tenant_res.json().await.unwrap();
        let tenant_id = tenant["id"].as_str().unwrap();

        // 2. Create Services & Routes
        for j in 0..services_per_tenant {
            let service_res = client
                .post(format!("{}/services", admin_url))
                .json(&json!({
                    "tenant_id": tenant_id,
                    "name": format!("service-{}-{}", i, j),
                    "host": format!("service-{}.internal", j)
                }))
                .send()
                .await
                .unwrap();

            let service: serde_json::Value = service_res.json().await.unwrap();
            let service_id = service["id"].as_str().unwrap();

            // Route
            client
                .post(format!("{}/routes", admin_url))
                .json(&json!({
                    "service_id": service_id,
                    "name": "default-route",
                    "path": format!("/api/v1/service-{}", j)
                }))
                .send()
                .await
                .unwrap();
        }

        // 3. Create Keys
        for _ in 0..keys_per_tenant {
            client
                .post(format!("{}/api-keys", admin_url))
                .json(&json!({
                    "tenant_id": tenant_id,
                    "key": format!("sk_load_{}_{}", i, Uuid::new_v4().to_string().replace("-", ""))
                }))
                .send()
                .await
                .unwrap();
        }

        pb.inc(1);
    }

    pb.finish_with_message("Done");
    println!("✅ Provisioning complete.");
}
