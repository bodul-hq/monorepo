//! Read-only coverage of the canonical offer facts on saved storefront pages.
use bodul_shared::prelude::RetailerCode;
use retailer_data_extractor::{extract_for, normalize_for, structuralize_for};
use retailer_sauce::{
    availability::Availability, item_condition::ItemCondition, structuralized::content::Content,
};
use serde_json::Value;
use std::{
    path::PathBuf,
    sync::atomic::{AtomicUsize, Ordering},
};

#[test]
#[ignore = "opt-in read-only survey of the external multi-GB retailer corpus"]
fn canonical_offer_facts_survive_the_facade_pipeline() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../data");
    if !root.is_dir() {
        eprintln!("saved source corpus unavailable; portable fixtures still run");
        return;
    }
    let limit = std::env::var("OFFER_FACTS_MAX_PAGES_PER_STOREFRONT")
        .ok()
        .map(|value| {
            let count = value
                .parse::<usize>()
                .expect("page limit must be a positive integer");
            assert!(count > 0, "page limit must be a positive integer");
            count
        });
    let folders: Vec<_> = std::fs::read_dir(root).unwrap().flatten().collect();
    let next = AtomicUsize::new(0);
    // Bound concurrent HTML trees while independent storefronts share the work.
    std::thread::scope(|scope| {
        let workers: Vec<_> = (0..4)
            .map(|_| {
                scope.spawn(|| {
                    while let Some(folder) = folders.get(next.fetch_add(1, Ordering::Relaxed)) {
                        survey_storefront(folder, limit);
                    }
                })
            })
            .collect();
        for worker in workers {
            worker.join().unwrap();
        }
    });
}

fn survey_storefront(folder: &std::fs::DirEntry, limit: Option<usize>) {
    let slug = folder.file_name().to_string_lossy().into_owned();
    if !(slug.starts_with("anker") || slug.starts_with("minisforum") || slug.starts_with("ugreen"))
        || slug == "ankerkr"
    {
        return;
    }
    let Ok(code) = RetailerCode::try_from(slug.as_str()) else {
        return;
    };
    let Ok(files) = std::fs::read_dir(folder.path().join("offer_details/sources")) else {
        return;
    };
    // Pages, successful pages, variations, comparison prices, availability, condition,
    // descriptions, failed pipelines. Failures remain visible in the coverage report.
    let mut counts = [0usize; 8];
    let mut files: Vec<_> = files
        .flatten()
        .filter(|file| file.path().extension().is_some_and(|ext| ext == "html"))
        .collect();
    files.sort_by_key(|file| file.file_name());
    for file in files.into_iter().take(limit.unwrap_or(usize::MAX)) {
        counts[0] += 1;
        let html = std::fs::read_to_string(file.path()).unwrap();
        let result = (|| {
            let extracted = extract_for(code, &html)?;
            let normalized = normalize_for(code, &extracted)
                .ok_or("no normalizer")??
                .document_json
                .ok_or("no normalized document")?;
            let output = structuralize_for(code, &normalized)
                .ok_or("no structuralizer")??
                .document_json
                .ok_or("no structuralized document")?;
            Ok::<_, String>((
                serde_json::from_str::<Value>(&normalized).unwrap(),
                serde_json::from_str::<Value>(&output).unwrap(),
            ))
        })();
        let (normalized, output) = match result {
            Ok(result) => result,
            Err(error) => {
                counts[7] += 1;
                eprintln!("OFFER_FACTS_ERROR|{}|{error}", file.path().display());
                continue;
            }
        };
        counts[1] += 1;
        let description = normalized["product"]["description"].as_str();
        for variation in output["variations"].as_array().unwrap() {
            counts[2] += 1;
            for key in ["was_price", "availability", "item_condition", "content"] {
                assert!(variation.get(key).is_some(), "{slug}: missing {key}");
            }
            assert_eq!(
                variation["content"],
                serde_json::json!({"description":description}),
                "{slug}: description must come only from the primary normalized product"
            );
            if !variation["was_price"].is_null() {
                counts[3] += 1;
                assert_eq!(
                    variation["was_price"]["currency"], variation["price"]["currency"],
                    "{slug}"
                );
                assert!(
                    variation["was_price"]["amount"].as_i64().unwrap()
                        > variation["price"]["amount"].as_i64().unwrap(),
                    "{slug}"
                );
            }
            if !variation["availability"].is_null() {
                counts[4] += 1;
                serde_json::from_value::<Availability>(variation["availability"].clone())
                    .unwrap()
                    .guard()
                    .unwrap();
            }
            if !variation["item_condition"].is_null() {
                counts[5] += 1;
                serde_json::from_value::<ItemCondition>(variation["item_condition"].clone())
                    .unwrap()
                    .guard()
                    .unwrap();
            }
            serde_json::from_value::<Content>(variation["content"].clone())
                .unwrap()
                .guard()
                .unwrap();
            counts[6] += usize::from(description.is_some());
        }
    }
    println!(
        "OFFER_FACTS|{slug}|{}|{}|{}|{}|{}|{}|{}|{}",
        counts[0], counts[1], counts[2], counts[3], counts[4], counts[5], counts[6], counts[7]
    );
}
