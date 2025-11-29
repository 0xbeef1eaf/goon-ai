#[cfg(test)]
mod tests {
    use super::super::window_manager::{WindowHandle, WindowManager, WindowMessage, WindowOptions};
    use std::time::Duration;
    use uuid::Uuid;

    #[test]
    fn test_window_options_default() {
        let options = WindowOptions::default();
        assert_eq!(options.title, None);
        assert_eq!(options.opacity, 1.0);
        assert_eq!(options.always_on_top, false);
        assert_eq!(options.decorations, true);
        assert_eq!(options.visible, true);
        assert_eq!(options.timeout, None);
    }

    #[test]
    fn test_window_handle_equality() {
        let id = Uuid::new_v4();
        let h1 = WindowHandle(id);
        let h2 = WindowHandle(id);
        let h3 = WindowHandle(Uuid::new_v4());

        assert_eq!(h1, h2);
        assert_ne!(h1, h3);
    }

    #[test]
    fn test_timeout_logic() {
        let mut wm = WindowManager::new();
        let handle = WindowHandle(Uuid::new_v4());

        // Add a short timeout
        wm.add_test_deadline(handle, Duration::from_millis(50));

        // Initially, it shouldn't be expired
        let next = wm.check_timeouts();
        // check_timeouts returns the *next* deadline if any, but also closes expired ones.
        // Since we just added it, it shouldn't be expired yet (unless system is super slow).
        // But check_timeouts returns Option<Instant>.

        assert!(next.is_some());

        // Wait for timeout
        std::thread::sleep(Duration::from_millis(100));

        // Now check again
        let next_after = wm.check_timeouts();

        // The window should have been "closed" (removed from deadlines)
        // So next_after should be None (no more deadlines)
        assert!(next_after.is_none());
    }

    #[test]
    fn test_message_queue() {
        let mut wm = WindowManager::new();
        let handle = WindowHandle(Uuid::new_v4());

        wm.push_message(WindowMessage::CloseRequested(handle));

        let messages = wm.poll_messages();
        assert_eq!(messages.len(), 1);
        match messages[0] {
            WindowMessage::CloseRequested(h) => assert_eq!(h, handle),
            _ => panic!("Wrong message type"),
        }

        // Test input message
        wm.push_message(WindowMessage::CursorMoved(handle, 100.0, 200.0));
        let messages = wm.poll_messages();
        assert_eq!(messages.len(), 1);
        match messages[0] {
            WindowMessage::CursorMoved(h, x, y) => {
                assert_eq!(h, handle);
                assert_eq!(x, 100.0);
                assert_eq!(y, 200.0);
            }
            _ => panic!("Wrong message type"),
        }

        let messages_empty = wm.poll_messages();
        assert_eq!(messages_empty.len(), 0);
    }
}
