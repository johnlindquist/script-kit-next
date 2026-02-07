                std::thread::spawn(move || {
                    use std::io::Write;
                    use std::os::unix::io::AsRawFd;

                    // Log the stdin file descriptor for debugging
                    let fd = stdin.as_raw_fd();
                    logging::log("EXEC", &format!("Writer thread started, stdin fd={}", fd));

                    // Check if fd is a valid pipe
                    #[cfg(unix)]
                    {
                        let stat_result = unsafe {
                            let mut stat: libc::stat = std::mem::zeroed();
                            libc::fstat(fd, &mut stat)
                        };
                        if stat_result == 0 {
                            logging::log("EXEC", &format!("fd={} fstat succeeded", fd));
                        } else {
                            logging::log(
                                "EXEC",
                                &format!(
                                    "fd={} fstat FAILED: errno={}",
                                    fd,
                                    std::io::Error::last_os_error()
                                ),
                            );
                        }
                    }

                    loop {
                        match response_rx.recv() {
                            Ok(response) => {
                                let json = match protocol::serialize_message(&response) {
                                    Ok(j) => j,
                                    Err(e) => {
                                        logging::log(
                                            "EXEC",
                                            &format!("Failed to serialize: {}", e),
                                        );
                                        continue;
                                    }
                                };
                                // Use truncated logging to avoid full payload in logs
                                logging::log_protocol_send(fd, &json);
                                let bytes = format!("{}\n", json);
                                let bytes_len = bytes.len();

                                // Check fd validity before write
                                let fcntl_result = unsafe { libc::fcntl(fd, libc::F_GETFD) };
                                logging::log(
                                    "EXEC",
                                    &format!(
                                        "Pre-write fcntl(F_GETFD) on fd={}: {}",
                                        fd, fcntl_result
                                    ),
                                );

                                match stdin.write_all(bytes.as_bytes()) {
                                    Ok(()) => {
                                        logging::log(
                                            "EXEC",
                                            &format!(
                                                "Write succeeded: {} bytes to fd={}",
                                                bytes_len, fd
                                            ),
                                        );
                                    }
                                    Err(e) => {
                                        logging::log(
                                            "EXEC",
                                            &format!(
                                                "Failed to write {} bytes: {} (kind={:?})",
                                                bytes_len,
                                                e,
                                                e.kind()
                                            ),
                                        );
                                        break;
                                    }
                                }
                                if let Err(e) = stdin.flush() {
                                    logging::log(
                                        "EXEC",
                                        &format!("Failed to flush fd={}: {}", fd, e),
                                    );
                                    break;
                                }
                                logging::log("EXEC", &format!("Flush succeeded for fd={}", fd));
                            }
                            Err(_) => {
                                logging::log("EXEC", "Response channel closed, writer exiting");
                                break;
                            }
                        }
                    }
                    logging::log("EXEC", "Writer thread exiting");
                });

                // Reader thread - handles receiving messages from script (blocking is OK here)
                // CRITICAL: Move _process_handle and _child into this thread to keep them alive!
                // When the reader thread exits, they'll be dropped and the process killed.
