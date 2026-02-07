            Err(e) => {
                logging::log(
                    "EXEC",
                    &format!("Failed to start interactive session: {}", e),
                );
                self.last_output = Some(SharedString::from(format!("✗ Error: {}", e)));
                cx.notify();
            }
        }
    }
