mod tests {
    use super::*;

    fn text(request: &OneshotRequest) -> &str {
        request.prompt.as_str()
    }

    #[test]
    fn a_single_user_message_is_sent_verbatim_with_system_instructions() {
        let request = GooseProvider::request(
            &[
                ProviderMessage::system("Be terse."),
                ProviderMessage::user("What is 2+2?"),
            ],
            "ollama/qwen3:4b",
        );
        assert_eq!(request.system.as_deref(), Some("Be terse."));
        assert_eq!(text(&request), "What is 2+2?");
        assert_eq!(request.model.as_deref(), Some("ollama/qwen3:4b"));
    }

    #[test]
    fn earlier_turns_travel_as_a_transcript_before_the_latest_message() {
        let request = GooseProvider::request(
            &[
                ProviderMessage::user("My name is Ada."),
                ProviderMessage::assistant("Hi Ada."),
                ProviderMessage::user("What is my name?"),
            ],
            "",
        );
        let prompt = text(&request);
        assert!(prompt.starts_with("Conversation so far:"), "{prompt}");
        assert!(prompt.contains("User: My name is Ada."));
        assert!(prompt.contains("Assistant: Hi Ada."));
        assert!(prompt.ends_with("What is my name?"));
        assert_eq!(request.model, None);
    }

    #[test]
    fn images_come_from_the_latest_user_message() {
        let request = GooseProvider::request(
            &[ProviderMessage::user_with_images(
                "Describe this",
                vec![ProviderImage::png("AAAA".to_string())],
            )],
            "",
        );
        assert_eq!(request.images.len(), 1);
        assert_eq!(request.images[0].mime_type, "image/png");
    }

    #[test]
    fn the_registry_finds_the_provider_that_owns_a_model() {
        struct Fixture;
        impl AiProvider for Fixture {
            fn provider_id(&self) -> &str {
                "fixture"
            }
            fn display_name(&self) -> &str {
                "Fixture"
            }
            fn available_models(&self) -> Vec<ModelInfo> {
                vec![ModelInfo::new("fixture/one", "One", "fixture", true, 0)]
            }
            fn send_message(&self, _: &[ProviderMessage], _: &str) -> Result<String> {
                Ok(String::new())
            }
            fn stream_message(
                &self,
                _: &[ProviderMessage],
                _: &str,
                _: StreamCallback,
                _: Option<&str>,
            ) -> Result<()> {
                Ok(())
            }
        }
        let mut registry = ProviderRegistry::new();
        assert!(!registry.has_any_provider());
        registry.register(Arc::new(Fixture));
        assert_eq!(
            registry
                .find_provider_for_model("fixture/one")
                .map(|provider| provider.provider_id()),
            Some("fixture")
        );
        assert!(registry.find_provider_for_model("missing").is_none());
    }
}
