//! All inputs are synthetic. These tests never acquire a window, AX object,
//! microphone, or clipboard. The OCR test does run Apple's actual Vision engine.
use super::*;
use std::time::Instant;
const OWNER: Owner = Owner {
    generation: 17,
    pid: 9,
    window: 12,
};
const RAW: &str = "please open the type script project and use quick look to review the mac os build before the next team meeting.";

#[test]
fn screen_spelling_ax_precedes_pixels_and_unsafe_is_not_an_ocr_fallback() {
    let text = "The QuickLook panel shows the TypeScript project.";
    let no_ocr = || panic!("must not acquire pixels");
    let result = collect(Ok(text.into()), "Acme", &[], true, no_ocr);
    let (session, publish) = Session::new(OWNER);
    publish.publish(result);
    let (final_text, receipt) = session.finish(OWNER, RAW);
    assert_eq!(receipt.source, Some("accessibility"));
    assert!(final_text
        .unwrap()
        .contains("TypeScript project and use QuickLook"));
    assert!(matches!(
        collect(Err("unsafeOrChanged"), "Acme", &[], true, no_ocr),
        Evidence::Skipped("unsafeOrChanged")
    ));
    assert!(matches!(
        collect(Ok("".into()), "", &[], false, no_ocr),
        Evidence::Skipped("noTerms")
    ));
    assert!(matches!(
        collect(Ok("".into()), "", &[], true, || Err(
            "screenCaptureUnavailable"
        )),
        Evidence::Skipped("screenCaptureUnavailable")
    ));
}

#[cfg(feature = "ocr")]
#[test]
fn screen_spelling_pixels_to_delivery() {
    let img = image::load_from_memory(include_bytes!(
        "../../tests/fixtures/dictation-screen-spelling.png"
    ))
    .unwrap()
    .into_rgba8();
    let iterations = if std::env::var_os("SK_SCREEN_SPELLING_BENCH").is_some() {
        31
    } else {
        1
    };
    let mut ms = vec![];
    for _ in 0..iterations {
        let start = Instant::now();
        let evidence = objc::rc::autoreleasepool(|| {
            collect(Ok(String::new()), "", &[], true, || {
                crate::ocr::extract_dictation_text(img.width(), img.height(), img.as_raw())
                    .map_err(|_| "ocrUnavailable")
            })
        });
        ms.push(start.elapsed().as_micros());
        let (session, publish) = Session::new(OWNER);
        publish.publish(evidence);
        let (result, receipt) = session.finish(OWNER, RAW);
        assert_eq!(receipt.source, Some("ocr"));
        let result = result.expect("synthetic pixels must change the final words");
        assert!(
            result.contains("TypeScript project and use QuickLook"),
            "{result}"
        );
    }
    if iterations > 1 {
        let first = ms.remove(0);
        ms.sort_unstable();
        println!("screen-spelling fixture: OCR + term collection, first_us={first} warm_n={} p50_us={} p95_us={}", ms.len(), ms[14], ms[28]);
    }
}

#[test]
fn screen_spelling_delivery_benchmark() {
    if std::env::var_os("SK_SCREEN_SPELLING_BENCH").is_none() {
        return;
    }
    // Bound the stock matcher with a realistic short utterance, a longer one,
    // and the largest accepted payload. Preparation is deliberately untimed:
    // the retrieval + rewrite + release of the per-recording state is timed.
    let mut terms = vec!["TypeScript".to_owned(), "QuickLook".into(), "macOS".into()];
    for i in 0..45 {
        terms.push(format!("FixtureName{i}"));
    }
    for words in [
        RAW.to_string(),
        vec![RAW; 8].join(" "),
        RAW.repeat(100)[..8192].into(),
    ] {
        for mode in ["off", "pending", "ready", "unavailable"] {
            let mut samples = Vec::new();
            for _ in 0..250 {
                let session = if mode == "off" {
                    None
                } else {
                    let (s, p) = Session::new(OWNER);
                    match mode {
                        "ready" => p.publish(Evidence::ready("accessibility", &terms)),
                        "unavailable" => p.publish(Evidence::Skipped("noTerms")),
                        _ => (),
                    }
                    Some(s)
                };
                let start = Instant::now();
                let result = std::hint::black_box(session)
                    .map(|s| s.finish(OWNER, std::hint::black_box(words.as_str())));
                std::hint::black_box(&result);
                samples.push(start.elapsed().as_nanos());
                assert_eq!(
                    result.is_some_and(|(text, _)| text.is_some()),
                    mode == "ready"
                );
            }
            samples.sort_unstable();
            println!(
                "screen-spelling delivery: {} bytes, mode={mode}, n=250 p50_us={:.1} p95_us={:.1}",
                words.len(),
                samples[124] as f64 / 1000.,
                samples[237] as f64 / 1000.
            );
        }
    }
}
