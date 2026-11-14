use anyhow::{bail, Context, Result};
use libtea_core::{model::ProductReleaseWithCollection, ArtifactFormat};
use std::env;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() {
        eprintln!("Usage: tea-check <url> [url ...]");
        eprintln!();
        eprintln!("Fetch TEA product release JSON from one or more URLs and print a report.");
        std::process::exit(1);
    }

    let rt = tokio::runtime::Runtime::new()
        .context("Failed to create async runtime")
        .expect("tokio runtime must initialize");

    let results: Vec<(String, Result<()>)> = rt.block_on(async {
        futures::future::join_all(
            args.iter().map(|url| fetch_and_report(url.clone()))
        ).await
        .into_iter()
        .zip(args.into_iter())
        .map(|(result, url)| (url, result))
        .collect()
    });

    let mut exit_code = 0;
    let total = results.len();
    for (url, result) in &results {
        if let Err(e) = result {
            if total > 1 {
                eprintln!("Error processing {url}: {e:?}");
            } else {
                eprintln!("Error: {e:?}");
            }
            exit_code = 1;
        }
    }

    std::process::exit(exit_code);
}

async fn fetch_and_report(url: String) -> Result<()> {
    let response = reqwest::get(&url).await.context("HTTP request failed")?;

    if !response.status().is_success() {
        bail!("API returned status {}", response.status());
    }

    let bytes = response.bytes().await.context("Failed to read response body")?;
    let value = libtea_core::model::parse_product_release(&bytes)
        .context("Failed to parse TEA product release JSON")?;

    print_report(&value);
    Ok(())
}

fn print_report(value: &ProductReleaseWithCollection) {
    let release = &value.release;

    println!("=== Product Release ===");
    println!("UUID:          {}", release.uuid);
    println!("Product UUID:  {}", release.product);
    if let Some(name) = &release.product_name {
        println!("Product Name:  {name}");
    }
    println!("Version:       {}", release.version);
    println!("Created:       {}", release.created_date);
    if let Some(rd) = &release.release_date {
        println!("Released:      {rd}");
    }
    if let Some(pr) = &release.pre_release {
        println!("Pre-release:   {pr}");
    }

    if let Some(ids) = &release.identifiers {
        if !ids.is_empty() {
            println!("\n=== Identifiers ({}) ===", ids.len());
            for id in ids {
                println!("  {0}: {1}", id.id_type, id.id_value);
            }
        }
    }

    println!("\n=== Components ({}) ===", release.components.len());
    for comp in &release.components {
        println!("  UUID:    {}", comp.uuid);
        if let Some(rel) = &comp.release {
            println!("  Release: {rel}");
        }
    }

    let collection = &value.latest_collection;
    println!("\n=== Collection ===");
    println!("UUID:          {}", collection.uuid);
    println!("Version:       {}", collection.version);
    println!("Created:       {}", collection.created_date);
    println!("Belongs to:    {}", collection.belongs_to);
    println!("Update reason: {}", collection.update_reason.reason_type);
    if let Some(comment) = &collection.update_reason.comment {
        println!("Comment:       {comment}");
    }

    if !collection.artifacts.is_empty() {
        println!("\n=== Artifacts ({}) ===", collection.artifacts.len());
        for art in &collection.artifacts {
            println!("  UUID:      {}", art.uuid);
            println!("  Version:   {}", art.version);
            if let Some(name) = &art.name {
                println!("  Name:      {name}");
            }
            println!("  Type:      {}", art.artifact_type);
            println!("  Created:   {}", art.created_date);
            if !art.formats.is_empty() {
                println!("  Formats:   {}", art.formats.len());
                for fmt in &art.formats {
                    print_format(fmt);
                }
            }
        }
    }
}

fn print_format(f: &ArtifactFormat) {
    let mut parts = vec![format!("    Media Type: {}", f.media_type)];
    if let Some(desc) = &f.description {
        parts.push(format!("Description: {desc}"));
    }
    if let Some(url) = &f.url {
        parts.push(format!("URL: {url}"));
    }
    if let Some(surl) = &f.signature_url {
        parts.push(format!("Signature: {surl}"));
    }
    if let Some(checksums) = &f.checksums {
        parts.push(format!("Checksums: {}", checksums.len()));
        for cs in checksums {
            parts.push(format!("      {0}: {1}", cs.algorithm(), cs.hex_value()));
        }
    }
    for p in &parts {
        println!("{p}");
    }
}