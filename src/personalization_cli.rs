//! Offline API for the selected Kit workspace. One bounded JSON request, no GUI or provider.
pub fn run_if_requested() -> bool {
    let mut args = std::env::args_os().skip(1);
    if args.next().as_deref() != Some(std::ffi::OsStr::new("--personalize")) {
        return false;
    }
    use std::io::Read;
    let result = (|| {
        let body = match (args.next(), args.next()) {
            (Some(s), None) if s == "describe" => br#"{"op":"describe"}"#.to_vec(),
            (Some(s), None) if s == "inspect" => br#"{"op":"inspect"}"#.to_vec(),
            (None, None) => {
                let mut b = Vec::new();
                std::io::stdin()
                    .lock()
                    .take(65537)
                    .read_to_end(&mut b)
                    .map_err(|_| sk_personal::Error::InvalidRequest)?;
                b
            }
            _ => return Err(sk_personal::Error::InvalidRequest),
        };
        sk_personal::api::answer(
            &crate::setup::get_kit_path(),
            &body,
            sk_personal::api::safe_mode(),
        )
    })();
    match result {
        Ok(data) => println!("{}", serde_json::json!({"ok":true,"data":data})),
        Err(error) => {
            println!("{}", serde_json::json!({"ok":false,"error":error}));
            std::process::exit(1);
        }
    }
    true
}
